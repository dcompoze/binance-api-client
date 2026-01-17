use std::time::Duration;

/// Production REST API base URL.
pub const REST_API_ENDPOINT: &str = "https://api.binance.com";

/// Production WebSocket base URL.
pub const WS_ENDPOINT: &str = "wss://stream.binance.com:9443";

/// Testnet REST API base URL.
pub const TESTNET_REST_API_ENDPOINT: &str = "https://testnet.binance.vision";

/// Testnet WebSocket base URL.
pub const TESTNET_WS_ENDPOINT: &str = "wss://testnet.binance.vision";

/// Binance.US REST API base URL.
pub const BINANCE_US_REST_API_ENDPOINT: &str = "https://api.binance.us";

/// Binance.US WebSocket base URL.
pub const BINANCE_US_WS_ENDPOINT: &str = "wss://stream.binance.us:9443";

/// Default recv_window in milliseconds.
pub const DEFAULT_RECV_WINDOW: u64 = 5000;

/// Configuration for the Binance client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Config {
    /// REST API base URL.
    pub rest_api_endpoint: String,

    /// WebSocket base URL.
    pub ws_endpoint: String,

    /// Receive window in milliseconds.
    /// This is the number of milliseconds after the timestamp
    /// that the request is valid for.
    pub recv_window: u64,

    /// Request timeout duration.
    pub timeout: Option<Duration>,

    /// Whether this is configured for Binance.US.
    pub binance_us: bool,
}

impl Config {
    /// Create a new configuration builder.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Create a configuration for the testnet.
    pub fn testnet() -> Self {
        Config {
            rest_api_endpoint: TESTNET_REST_API_ENDPOINT.to_string(),
            ws_endpoint: TESTNET_WS_ENDPOINT.to_string(),
            recv_window: DEFAULT_RECV_WINDOW,
            timeout: None,
            binance_us: false,
        }
    }

    /// Create a configuration for Binance.US.
    pub fn binance_us() -> Self {
        Config {
            rest_api_endpoint: BINANCE_US_REST_API_ENDPOINT.to_string(),
            ws_endpoint: BINANCE_US_WS_ENDPOINT.to_string(),
            recv_window: DEFAULT_RECV_WINDOW,
            timeout: None,
            binance_us: true,
        }
    }
}

impl Default for Config {
    /// Create a configuration with production defaults.
    fn default() -> Self {
        Config {
            rest_api_endpoint: REST_API_ENDPOINT.to_string(),
            ws_endpoint: WS_ENDPOINT.to_string(),
            recv_window: DEFAULT_RECV_WINDOW,
            timeout: None,
            binance_us: false,
        }
    }
}

/// Builder for creating a custom Config.
#[derive(Clone, Debug, Default)]
pub struct ConfigBuilder {
    rest_api_endpoint: Option<String>,
    ws_endpoint: Option<String>,
    recv_window: Option<u64>,
    timeout: Option<Duration>,
    binance_us: bool,
}

impl ConfigBuilder {
    /// Set the REST API endpoint.
    pub fn rest_api_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.rest_api_endpoint = Some(endpoint.into());
        self
    }

    /// Set the WebSocket endpoint.
    pub fn ws_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.ws_endpoint = Some(endpoint.into());
        self
    }

    /// Set the receive window in milliseconds.
    pub fn recv_window(mut self, recv_window: u64) -> Self {
        self.recv_window = Some(recv_window);
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the request timeout from seconds.
    pub fn timeout_secs(self, secs: u64) -> Self {
        self.timeout(Duration::from_secs(secs))
    }

    /// Configure for Binance.US.
    pub fn binance_us(mut self, is_binance_us: bool) -> Self {
        self.binance_us = is_binance_us;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> Config {
        let (default_rest, default_ws) = if self.binance_us {
            (BINANCE_US_REST_API_ENDPOINT, BINANCE_US_WS_ENDPOINT)
        } else {
            (REST_API_ENDPOINT, WS_ENDPOINT)
        };

        Config {
            rest_api_endpoint: self
                .rest_api_endpoint
                .unwrap_or_else(|| default_rest.to_string()),
            ws_endpoint: self.ws_endpoint.unwrap_or_else(|| default_ws.to_string()),
            recv_window: self.recv_window.unwrap_or(DEFAULT_RECV_WINDOW),
            timeout: self.timeout,
            binance_us: self.binance_us,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.rest_api_endpoint, REST_API_ENDPOINT);
        assert_eq!(config.ws_endpoint, WS_ENDPOINT);
        assert_eq!(config.recv_window, DEFAULT_RECV_WINDOW);
        assert!(config.timeout.is_none());
        assert!(!config.binance_us);
    }

    #[test]
    fn test_testnet_config() {
        let config = Config::testnet();
        assert_eq!(config.rest_api_endpoint, TESTNET_REST_API_ENDPOINT);
        assert_eq!(config.ws_endpoint, TESTNET_WS_ENDPOINT);
        assert_eq!(config.recv_window, DEFAULT_RECV_WINDOW);
        assert!(!config.binance_us);
    }

    #[test]
    fn test_binance_us_config() {
        let config = Config::binance_us();
        assert_eq!(config.rest_api_endpoint, BINANCE_US_REST_API_ENDPOINT);
        assert_eq!(config.ws_endpoint, BINANCE_US_WS_ENDPOINT);
        assert!(config.binance_us);
    }

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .rest_api_endpoint("https://custom.api.com")
            .ws_endpoint("wss://custom.ws.com")
            .recv_window(3000)
            .timeout_secs(30)
            .build();

        assert_eq!(config.rest_api_endpoint, "https://custom.api.com");
        assert_eq!(config.ws_endpoint, "wss://custom.ws.com");
        assert_eq!(config.recv_window, 3000);
        assert_eq!(config.timeout, Some(Duration::from_secs(30)));
    }

    #[test]
    fn test_config_builder_binance_us_defaults() {
        let config = Config::builder().binance_us(true).build();

        assert_eq!(config.rest_api_endpoint, BINANCE_US_REST_API_ENDPOINT);
        assert_eq!(config.ws_endpoint, BINANCE_US_WS_ENDPOINT);
        assert!(config.binance_us);
    }
}
