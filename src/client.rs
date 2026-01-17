use reqwest::StatusCode;
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use serde::de::DeserializeOwned;

use crate::config::Config;
use crate::credentials::{Credentials, build_signed_query_string};
use crate::error::{BinanceApiError, Error, Result};

/// HTTP client for Binance REST API.
#[derive(Clone)]
pub struct Client {
    http: ClientWithMiddleware,
    config: Config,
    credentials: Option<Credentials>,
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

        let reqwest_client = builder.build()?;

        // Set up retry policy for transient errors
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(3);

        let http = ClientBuilder::new(reqwest_client)
            .with(TracingMiddleware::default())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        Ok(Self {
            http,
            config,
            credentials,
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

    /// Make an unsigned GET request (for public endpoints).
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str, query: Option<&str>) -> Result<T> {
        let url = match query {
            Some(q) => format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, q),
            None => format!("{}{}", self.config.rest_api_endpoint, endpoint),
        };

        let response = self.http.get(&url).send().await?;
        self.handle_response(response).await
    }

    /// Make an unsigned GET request with query parameters as key-value pairs.
    pub async fn get_with_params<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let query = if params.is_empty() {
            None
        } else {
            Some(
                params
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&"),
            )
        };

        self.get(endpoint, query.as_deref()).await
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
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        let url = match query {
            Some(q) => format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, q),
            None => format!("{}{}", self.config.rest_api_endpoint, endpoint),
        };

        let response = self
            .http
            .get(&url)
            .headers(self.build_auth_headers(credentials)?)
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
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        let query = build_signed_query_string(
            params.iter().copied(),
            credentials,
            self.config.recv_window,
        )?;

        let url = format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, query);

        let response = self
            .http
            .get(&url)
            .headers(self.build_auth_headers(credentials)?)
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
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        let query = build_signed_query_string(
            params.iter().copied(),
            credentials,
            self.config.recv_window,
        )?;

        let url = format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, query);

        let response = self
            .http
            .post(&url)
            .headers(self.build_auth_headers_with_content_type(credentials)?)
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
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        let query = build_signed_query_string(
            params.iter().copied(),
            credentials,
            self.config.recv_window,
        )?;

        let url = format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, query);

        let response = self
            .http
            .post(&url)
            .headers(self.build_auth_headers_with_content_type(credentials)?)
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
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        let query = build_signed_query_string(
            params.iter().copied(),
            credentials,
            self.config.recv_window,
        )?;

        let url = format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, query);

        let response = self
            .http
            .delete(&url)
            .headers(self.build_auth_headers_with_content_type(credentials)?)
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
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        let query = build_signed_query_string(
            params.iter().copied(),
            credentials,
            self.config.recv_window,
        )?;

        let url = format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, query);

        let response = self
            .http
            .put(&url)
            .headers(self.build_auth_headers_with_content_type(credentials)?)
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
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        let url = if params.is_empty() {
            format!("{}{}", self.config.rest_api_endpoint, endpoint)
        } else {
            let query = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, query)
        };

        let response = self
            .http
            .post(&url)
            .headers(self.build_auth_headers(credentials)?)
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
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        let url = if params.is_empty() {
            format!("{}{}", self.config.rest_api_endpoint, endpoint)
        } else {
            let query = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, query)
        };

        let response = self
            .http
            .put(&url)
            .headers(self.build_auth_headers(credentials)?)
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
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)?;

        let url = if params.is_empty() {
            format!("{}{}", self.config.rest_api_endpoint, endpoint)
        } else {
            let query = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}{}?{}", self.config.rest_api_endpoint, endpoint, query)
        };

        let response = self
            .http
            .delete(&url)
            .headers(self.build_auth_headers(credentials)?)
            .send()
            .await?;

        self.handle_response(response).await
    }

    fn build_auth_headers(&self, credentials: &Credentials) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("binance-api-client-rs"));
        headers.insert(
            HeaderName::from_static("x-mbx-apikey"),
            HeaderValue::from_str(credentials.api_key())?,
        );
        Ok(headers)
    }

    fn build_auth_headers_with_content_type(&self, credentials: &Credentials) -> Result<HeaderMap> {
        let mut headers = self.build_auth_headers(credentials)?;
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        Ok(headers)
    }

    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            StatusCode::INTERNAL_SERVER_ERROR => Err(Error::Api {
                code: 500,
                message: "Internal server error".to_string(),
            }),
            StatusCode::SERVICE_UNAVAILABLE => Err(Error::Api {
                code: 503,
                message: "Service unavailable".to_string(),
            }),
            StatusCode::UNAUTHORIZED => Err(Error::Api {
                code: 401,
                message: "Unauthorized".to_string(),
            }),
            StatusCode::BAD_REQUEST | StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS => {
                let error: BinanceApiError = response.json().await?;
                Err(Error::from_binance_error(error))
            }
            status => Err(Error::Api {
                code: status.as_u16() as i32,
                message: format!("Unexpected status code: {}", status),
            }),
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
