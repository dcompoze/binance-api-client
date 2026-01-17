//! User data stream API endpoints.
//!
//! This module provides endpoints for managing user data streams,
//! which allow real-time account updates via WebSocket.

use serde_json::Value;

use crate::Result;
use crate::client::Client;
use crate::models::ListenKey;

// API endpoints
const API_V3_USER_DATA_STREAM: &str = "/api/v3/userDataStream";

/// User data stream API client.
///
/// Provides endpoints for managing user data streams. A listen key is required
/// to connect to the user data WebSocket stream for real-time account updates.
///
/// # Listen Key Lifecycle
///
/// 1. Start a stream with `start()` to get a listen key
/// 2. Send keepalive every 30 minutes with `keepalive()` to prevent expiration
/// 3. Close the stream with `close()` when done
///
/// Listen keys expire after 60 minutes without a keepalive.
#[derive(Clone)]
pub struct UserStream {
    client: Client,
}

impl UserStream {
    /// Create a new UserStream API client.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Start a new user data stream.
    ///
    /// Returns a listen key that can be used to connect to the user data
    /// WebSocket stream.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let listen_key = client.user_stream().start().await?;
    /// println!("Listen key: {}", listen_key);
    ///
    /// // Connect to WebSocket using: wss://stream.binance.com:9443/ws/{listen_key}
    /// ```
    pub async fn start(&self) -> Result<String> {
        let response: ListenKey = self
            .client
            .post_with_key(API_V3_USER_DATA_STREAM, &[])
            .await?;
        Ok(response.listen_key)
    }

    /// Send a keepalive for a user data stream.
    ///
    /// This should be called every 30 minutes to prevent the listen key from
    /// expiring. Listen keys expire after 60 minutes without a keepalive.
    ///
    /// # Arguments
    ///
    /// * `listen_key` - The listen key to keep alive
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let listen_key = client.user_stream().start().await?;
    ///
    /// // Every 30 minutes:
    /// client.user_stream().keepalive(&listen_key).await?;
    /// ```
    pub async fn keepalive(&self, listen_key: &str) -> Result<()> {
        let params = [("listenKey", listen_key)];
        let _: Value = self
            .client
            .put_with_key(API_V3_USER_DATA_STREAM, &params)
            .await?;
        Ok(())
    }

    /// Close a user data stream.
    ///
    /// This invalidates the listen key and closes any WebSocket connections
    /// using it.
    ///
    /// # Arguments
    ///
    /// * `listen_key` - The listen key to close
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let listen_key = client.user_stream().start().await?;
    ///
    /// // When done:
    /// client.user_stream().close(&listen_key).await?;
    /// ```
    pub async fn close(&self, listen_key: &str) -> Result<()> {
        let params = [("listenKey", listen_key)];
        let _: Value = self
            .client
            .delete_with_key(API_V3_USER_DATA_STREAM, &params)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_endpoint() {
        assert_eq!(API_V3_USER_DATA_STREAM, "/api/v3/userDataStream");
    }
}
