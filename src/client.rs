use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicI64, Ordering};

use reqwest::StatusCode;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use serde::de::DeserializeOwned;

use crate::config::Config;
use crate::credentials::{Credentials, build_signed_query_string_with, get_timestamp};
use crate::error::{BinanceApiError, Error, Result};

/// HTTP client for Binance REST API.
#[derive(Clone)]
pub struct Client {
    /// Client without retries, used for state-changing requests.
    ///
    /// Retrying a POST/PUT/DELETE that timed out mid-flight can double-submit
    /// an order, so these requests are never retried automatically.
    http: ClientWithMiddleware,
    /// Client with transient-error retries, used for idempotent GET requests.
    http_retry: ClientWithMiddleware,
    config: Config,
    credentials: Option<Credentials>,
    /// Server time offset in milliseconds, applied to request timestamps.
    time_offset: Arc<AtomicI64>,
    /// Rate limit usage reported by `X-MBX-USED-WEIGHT-*` and
    /// `X-MBX-ORDER-COUNT-*` response headers, keyed by header name.
    rate_limit_usage: Arc<Mutex<HashMap<String, u64>>>,
}

impl Client {
    /// Create a new authenticated client.
    pub fn new(config: Config, credentials: Credentials) -> Result<Self> {
        Self::build(config, Some(credentials))
    }

    /// Create a new unauthenticated client for public endpoints only.
    pub fn new_unauthenticated(config: Config) -> Result<Self> {
        Self::build(config, None)
    }

    fn build(config: Config, credentials: Option<Credentials>) -> Result<Self> {
        let mut builder = reqwest::Client::builder();

        if let Some(timeout) = config.timeout {
            builder = builder.timeout(timeout);
        }

        if let Some(ref proxy) = config.proxy {
            builder = builder.proxy(reqwest::Proxy::all(proxy)?);
        }

        let reqwest_client = builder.build()?;

        let http = ClientBuilder::new(reqwest_client.clone())
            .with(TracingMiddleware::default())
            .build();

        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);
        let http_retry = ClientBuilder::new(reqwest_client)
            .with(TracingMiddleware::default())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Self {
            http,
            http_retry,
            config,
            credentials,
            time_offset: Arc::new(AtomicI64::new(0)),
            rate_limit_usage: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get the current configuration.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Check if this client has credentials.
    pub fn has_credentials(&self) -> bool {
        self.credentials.is_some()
    }

    /// Get the credentials, if configured.
    pub fn credentials(&self) -> Option<&Credentials> {
        self.credentials.as_ref()
    }

    /// Synchronize the local clock with the server.
    ///
    /// Computes the offset between server time and local time and applies
    /// it to all subsequent signed request timestamps.
    /// Prevents `-1021` errors when the local clock drifts.
    pub async fn sync_time(&self) -> Result<i64> {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ServerTimeOnly {
            server_time: i64,
        }
        let url = format!("{}/api/v3/time", self.config.rest_api_endpoint);
        let response = self.http_retry.get(&url).send().await?;
        let server: ServerTimeOnly = self.handle_response(response).await?;
        let offset = server.server_time - get_timestamp()? as i64;
        self.time_offset.store(offset, Ordering::Relaxed);
        Ok(offset)
    }

    /// Get the current server time offset in milliseconds.
    pub fn time_offset(&self) -> i64 {
        self.time_offset.load(Ordering::Relaxed)
    }

    /// Get the most recent rate limit usage reported by the server.
    ///
    /// Keys are lowercase response header names such as
    /// `x-mbx-used-weight-1m` and `x-mbx-order-count-10s`.
    pub fn rate_limit_usage(&self) -> HashMap<String, u64> {
        self.rate_limit_usage.lock().unwrap().clone()
    }

    /// SAPI endpoints do not exist on the spot testnet.
    fn check_sapi_supported(&self, endpoint: &str) -> Result<()> {
        if endpoint.starts_with("/sapi/")
            && self
                .config
                .rest_api_endpoint
                .contains("testnet.binance.vision")
        {
            return Err(Error::InvalidConfig(
                "SAPI endpoints (wallet, margin) are not available on the spot testnet".to_string(),
            ));
        }
        Ok(())
    }

    /// Build a signed request URL, checking credentials and SAPI availability.
    fn build_signed_url(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<String> {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        self.check_sapi_supported(endpoint)?;

        let query = build_signed_query_string_with(
            params.iter().copied(),
            credentials,
            self.config.recv_window,
            self.time_offset.load(Ordering::Relaxed),
            self.config.microsecond_timestamps,
        )?;

        Ok(format!(
            "{}{}?{}",
            self.config.rest_api_endpoint, endpoint, query
        ))
    }

    /// Build a full URL from an endpoint and unsigned key-value parameters.
    /// Parameter values are percent-encoded.
    fn build_url(&self, endpoint: &str, params: &[(&str, &str)]) -> String {
        if params.is_empty() {
            format!("{}{}", self.config.rest_api_endpoint, endpoint)
        } else {
            let query = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, query)
        }
    }

    pub(crate) fn record_rate_limit_headers(&self, headers: &HeaderMap) {
        let mut usage = self.rate_limit_usage.lock().unwrap();
        for (name, value) in headers {
            let name = name.as_str();
            if name.starts_with("x-mbx-used-weight") || name.starts_with("x-mbx-order-count") {
                if let Some(count) = value.to_str().ok().and_then(|v| v.parse::<u64>().ok()) {
                    usage.insert(name.to_string(), count);
                }
            }
        }
    }

    /// Make an unsigned GET request (for public endpoints).
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str, query: Option<&str>) -> Result<T> {
        let url = match query {
            Some(q) => format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, q),
            None => format!("{}{}", self.config.rest_api_endpoint, endpoint),
        };

        let response = self.http_retry.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Make an unsigned GET request with query parameters as key-value pairs.
    pub async fn get_with_params<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = self.build_url(endpoint, params);
        let response = self.http_retry.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Make a GET request with API key but no signature.
    ///
    /// Used for endpoints like historical trades that require authentication
    /// but not request signing.
    pub async fn get_with_api_key<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        query: Option<&str>,
    ) -> Result<T> {
        let url = match query {
            Some(q) => format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, q),
            None => format!("{}{}", self.config.rest_api_endpoint, endpoint),
        };

        let response = self
            .http_retry
            .get(&url)
            .headers(self.build_auth_headers()?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a signed GET request (requires credentials).
    pub async fn get_signed<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = self.build_signed_url(endpoint, params)?;

        let response = self
            .http_retry
            .get(&url)
            .headers(self.build_auth_headers()?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a signed POST request (requires credentials).
    pub async fn post_signed<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = self.build_signed_url(endpoint, params)?;

        let response = self
            .http
            .post(&url)
            .headers(self.build_auth_headers_with_content_type()?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a signed POST request and return the raw response.
    pub async fn post_signed_raw(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<reqwest::Response> {
        let url = self.build_signed_url(endpoint, params)?;

        let response = self
            .http
            .post(&url)
            .headers(self.build_auth_headers_with_content_type()?)
            .send()
            .await?;

        Ok(response)
    }

    /// Make a signed DELETE request (requires credentials).
    pub async fn delete_signed<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = self.build_signed_url(endpoint, params)?;

        let response = self
            .http
            .delete(&url)
            .headers(self.build_auth_headers_with_content_type()?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a signed PUT request (requires credentials).
    pub async fn put_signed<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = self.build_signed_url(endpoint, params)?;

        let response = self
            .http
            .put(&url)
            .headers(self.build_auth_headers_with_content_type()?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a POST request with API key but no signature (for user stream endpoints).
    pub async fn post_with_key<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = self.build_url(endpoint, params);

        let response = self
            .http
            .post(&url)
            .headers(self.build_auth_headers()?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a PUT request with API key but no signature (for user stream keepalive).
    pub async fn put_with_key<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = self.build_url(endpoint, params);

        let response = self
            .http
            .put(&url)
            .headers(self.build_auth_headers()?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Make a DELETE request with API key but no signature (for user stream close).
    pub async fn delete_with_key<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = self.build_url(endpoint, params);

        let response = self
            .http
            .delete(&url)
            .headers(self.build_auth_headers()?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    fn build_auth_headers(&self) -> Result<HeaderMap> {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("binance-api-client-rs"),
        );
        headers.insert(
            HeaderName::from_static("x-mbx-apikey"),
            HeaderValue::from_str(credentials.api_key())?,
        );
        if self.config.microsecond_timestamps {
            headers.insert(
                HeaderName::from_static("x-mbx-time-unit"),
                HeaderValue::from_static("MICROSECOND"),
            );
        }
        Ok(headers)
    }

    fn build_auth_headers_with_content_type(&self) -> Result<HeaderMap> {
        let mut headers = self.build_auth_headers()?;
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        Ok(headers)
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        if response.status() == StatusCode::OK {
            self.record_rate_limit_headers(response.headers());
            return Ok(response.json().await?);
        }
        Err(self.error_from_response(response).await)
    }

    /// Convert a non-OK response into an `Error`.
    ///
    /// Also records rate limit headers from the response.
    pub(crate) async fn error_from_response(&self, response: reqwest::Response) -> Error {
        self.record_rate_limit_headers(response.headers());
        let status = response.status();

        let retry_after = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        // Binance returns a `{code, msg}` body on most error statuses.
        // Keep the raw text as the message when the body is not in that shape.
        let body = response.text().await.unwrap_or_default();
        let api_error: Option<BinanceApiError> = serde_json::from_str(&body).ok();

        if status == StatusCode::TOO_MANY_REQUESTS || status == StatusCode::IM_A_TEAPOT {
            let (code, message) = match api_error {
                Some(e) => (e.code, e.msg),
                None => (status.as_u16() as i32, body),
            };
            return Error::RateLimited {
                code,
                message,
                retry_after,
                ip_banned: status == StatusCode::IM_A_TEAPOT,
            };
        }

        match api_error {
            Some(error) => Error::from_binance_error(error),
            None => Error::Api {
                code: status.as_u16() as i32,
                message: if body.is_empty() {
                    format!("Unexpected status code: {}", status)
                } else {
                    body
                },
            },
        }
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("config", &self.config)
            .field("has_credentials", &self.credentials.is_some())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_client_new_unauthenticated() {
        let config = Config::default();
        let client = Client::new_unauthenticated(config).unwrap();
        assert!(!client.has_credentials());
    }

    #[test]
    fn test_client_new_authenticated() {
        let config = Config::default();
        let creds = Credentials::new("api_key", "secret_key");
        let client = Client::new(config, creds).unwrap();
        assert!(client.has_credentials());
    }

    #[test]
    fn test_client_with_timeout() {
        let config = Config::builder().timeout(Duration::from_secs(30)).build();
        let client = Client::new_unauthenticated(config.clone()).unwrap();
        assert_eq!(client.config().timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_client_debug() {
        let config = Config::default();
        let creds = Credentials::new("api_key", "secret_key");
        let client = Client::new(config, creds).unwrap();
        let debug_output = format!("{:?}", client);
        assert!(debug_output.contains("has_credentials: true"));
        assert!(!debug_output.contains("secret_key"));
    }
}
