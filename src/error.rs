use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

use crate::models::account::{CancelReplaceErrorData, CancelReplaceErrorResponse};

/// Binance API error response structure.
#[derive(Debug, Deserialize)]
pub struct BinanceApiError {
    pub code: i32,
    pub msg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl std::fmt::Display for BinanceApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Binance API error {}: {}", self.code, self.msg)
    }
}

impl std::error::Error for BinanceApiError {}

/// Error types for the Binance client library.
#[derive(Debug, Error)]
pub enum Error {
    /// Binance API returned an error response.
    #[error("Binance API error {code}: {message}")]
    Api { code: i32, message: String },

    /// Binance API returned a cancel-replace error response.
    #[error("Binance API cancel-replace error {code}: {message}")]
    CancelReplace {
        code: i32,
        message: String,
        data: Box<CancelReplaceErrorData>,
    },

    /// HTTP request error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// HTTP middleware error.
    #[error("HTTP middleware error: {0}")]
    Middleware(#[from] reqwest_middleware::Error),

    /// WebSocket error.
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// URL parsing error.
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Authentication is required but credentials were not provided.
    #[error("Authentication required for this endpoint")]
    AuthenticationRequired,

    /// System time error (for timestamp generation).
    #[error("System time error: {0}")]
    SystemTime(#[from] std::time::SystemTimeError),

    /// Invalid header value.
    #[error("Invalid header value: {0}")]
    InvalidHeader(#[from] reqwest::header::InvalidHeaderValue),

    /// Environment variable error.
    #[error("Environment variable error: {0}")]
    EnvVar(#[from] std::env::VarError),

    /// Invalid credentials (RSA/Ed25519 key parsing error).
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),
}

impl Error {
    /// Create an API error from a Binance error response.
    pub fn from_binance_error(error: BinanceApiError) -> Self {
        Error::Api {
            code: error.code,
            message: error.msg,
        }
    }

    /// Create a cancel-replace error from a Binance error response.
    pub fn from_cancel_replace_error(error: CancelReplaceErrorResponse) -> Self {
        Error::CancelReplace {
            code: error.code,
            message: error.msg.clone(),
            data: Box::new(error.data),
        }
    }

    /// Check if this is a rate limit error (code -1003).
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, Error::Api { code: -1003, .. })
    }

    /// Check if this is an invalid signature error (code -1022).
    pub fn is_invalid_signature(&self) -> bool {
        matches!(self, Error::Api { code: -1022, .. })
    }

    /// Check if this is a timestamp out of recv_window error (code -1021).
    pub fn is_timestamp_error(&self) -> bool {
        matches!(self, Error::Api { code: -1021, .. })
    }

    /// Check if this is an unauthorized error (code -1002 or -2015).
    pub fn is_unauthorized(&self) -> bool {
        matches!(
            self,
            Error::Api {
                code: -1002 | -2015,
                ..
            }
        )
    }
}

/// Result type alias for this library.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_display() {
        let err = Error::Api {
            code: -1003,
            message: "Too many requests".to_string(),
        };
        assert_eq!(
            format!("{}", err),
            "Binance API error -1003: Too many requests"
        );
    }

    #[test]
    fn test_is_rate_limit() {
        let rate_limit_err = Error::Api {
            code: -1003,
            message: "Too many requests".to_string(),
        };
        assert!(rate_limit_err.is_rate_limit());

        let other_err = Error::Api {
            code: -1000,
            message: "Unknown error".to_string(),
        };
        assert!(!other_err.is_rate_limit());
    }

    #[test]
    fn test_is_invalid_signature() {
        let sig_err = Error::Api {
            code: -1022,
            message: "Invalid signature".to_string(),
        };
        assert!(sig_err.is_invalid_signature());
    }

    #[test]
    fn test_is_timestamp_error() {
        let ts_err = Error::Api {
            code: -1021,
            message: "Timestamp outside recv window".to_string(),
        };
        assert!(ts_err.is_timestamp_error());
    }

    #[test]
    fn test_is_unauthorized() {
        let unauth_err = Error::Api {
            code: -2015,
            message: "Invalid API key".to_string(),
        };
        assert!(unauth_err.is_unauthorized());

        let unauth_err2 = Error::Api {
            code: -1002,
            message: "Unauthorized".to_string(),
        };
        assert!(unauth_err2.is_unauthorized());
    }

    #[test]
    fn test_binance_api_error_deserialize() {
        let json = r#"{"code": -1000, "msg": "Unknown error"}"#;
        let err: BinanceApiError = serde_json::from_str(json).unwrap();
        assert_eq!(err.code, -1000);
        assert_eq!(err.msg, "Unknown error");
    }
}
