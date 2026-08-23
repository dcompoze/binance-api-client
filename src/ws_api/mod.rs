//! WebSocket API client.
//!
//! The WebSocket API (`wss://ws-api.binance.com:443/ws-api/v3`) is a
//! request/response interface for trading, account queries, and market data
//! over a single websocket connection.
//! It is also the only supported transport for user data streams since the
//! listenKey endpoints were removed from service on 2026-02-20.
//!
//! # Example
//!
//! ```rust,ignore
//! use binance_api_client::Binance;
//!
//! let client = Binance::new("api_key", "secret_key")?;
//! let conn = client.ws_api().connect().await?;
//!
//! // Subscribe to the user data stream (works with any API key type).
//! let subscription_id = conn.subscribe_user_data_with_signature().await?;
//!
//! // Receive user data events.
//! let mut events = conn.take_events().unwrap();
//! while let Some(event) = events.recv().await {
//!     println!("{:?}", event);
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Map, Value, json};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::config::Config;
use crate::credentials::{Credentials, get_timestamp};
use crate::error::{Error, Result};
use crate::models::websocket::WebSocketEvent;
use crate::models::{AccountInfo, CancelOrderResponse, Order, OrderFull};
use crate::rest::NewOrder;

/// Default time to wait for a response before giving up.
///
/// The server itself times out requests after 10 seconds with error -1007.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// Rate limit status returned in WebSocket API responses.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsApiRateLimit {
    pub rate_limit_type: String,
    pub interval: String,
    pub interval_num: u32,
    pub limit: u64,
    #[serde(default)]
    pub count: u64,
}

/// Session information returned by the `session.*` methods.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WsApiSession {
    /// API key the session is authenticated with, `None` when not
    /// authenticated.
    pub api_key: Option<String>,
    /// When the session was authenticated, `None` when not authenticated.
    pub authorized_since: Option<u64>,
    /// When the connection was established.
    pub connected_since: u64,
    /// Whether responses include rate limit status.
    pub return_rate_limits: bool,
    /// Current server time in milliseconds.
    pub server_time: u64,
    /// Whether a user data stream subscription is active.
    #[serde(default)]
    pub user_data_stream: bool,
}

/// A raw response frame from the WebSocket API.
#[derive(Debug, Deserialize)]
struct WsApiResponse {
    id: Value,
    status: u16,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<WsApiError>,
    #[serde(default, rename = "rateLimits")]
    rate_limits: Option<Vec<WsApiRateLimit>>,
}

#[derive(Debug, Deserialize)]
struct WsApiError {
    code: i32,
    msg: String,
}

/// Events pushed by the server outside of request/response exchanges.
#[derive(Debug, Clone)]
pub enum WsApiEvent {
    /// A user data stream event for a subscription on this connection.
    UserData {
        /// Identifies which subscription the event belongs to.
        subscription_id: Option<u64>,
        event: Box<WebSocketEvent>,
    },
    /// The server is about to shut down.
    /// Reconnect immediately, there is no grace period.
    ServerShutdown { event_time: u64 },
    /// An event that could not be parsed into a known type.
    Unknown(Value),
}

/// Entry point for the WebSocket API.
#[derive(Clone, Debug)]
pub struct WsApiClient {
    config: Config,
    credentials: Option<Credentials>,
}

impl WsApiClient {
    pub fn new(config: Config, credentials: Option<Credentials>) -> Self {
        Self {
            config,
            credentials,
        }
    }

    /// Open a connection to the WebSocket API.
    ///
    /// Connections are valid for at most 24 hours, after which the server
    /// disconnects. Callers should plan for proactive reconnection.
    pub async fn connect(&self) -> Result<WsApiConnection> {
        let (stream, _) = connect_async(&self.config.ws_api_endpoint).await?;
        let (mut sink, mut source) = stream.split();

        let pending: Arc<Mutex<HashMap<u64, oneshot::Sender<WsApiResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let (write_tx, mut write_rx) = mpsc::unbounded_channel::<Message>();
        let (event_tx, event_rx) = mpsc::unbounded_channel::<WsApiEvent>();

        // Writer task, single owner of the sink.
        tokio::spawn(async move {
            while let Some(message) = write_rx.recv().await {
                if sink.send(message).await.is_err() {
                    break;
                }
            }
        });

        // Reader task, routes responses to pending requests and pushes events.
        // It holds only a weak sender so the writer task exits when the
        // connection handle is dropped.
        let reader_pending = Arc::clone(&pending);
        let reader_write_tx = write_tx.downgrade();
        tokio::spawn(async move {
            while let Some(frame) = source.next().await {
                let frame = match frame {
                    Ok(f) => f,
                    Err(_) => break,
                };
                match frame {
                    Message::Text(text) => {
                        let value: Value = match serde_json::from_str(&text) {
                            Ok(v) => v,
                            Err(_) => continue,
                        };
                        if value.get("id").is_some() && value.get("status").is_some() {
                            if let Ok(response) =
                                serde_json::from_value::<WsApiResponse>(value.clone())
                            {
                                if let Some(id) = response.id.as_u64() {
                                    if let Some(sender) = reader_pending.lock().await.remove(&id) {
                                        let _ = sender.send(response);
                                        continue;
                                    }
                                }
                            }
                            let _ = event_tx.send(WsApiEvent::Unknown(value));
                        } else {
                            let _ = event_tx.send(parse_event(value));
                        }
                    }
                    Message::Ping(payload) => {
                        if let Some(tx) = reader_write_tx.upgrade() {
                            let _ = tx.send(Message::Pong(payload));
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }

            // Fail pending requests immediately instead of letting them
            // wait out the request timeout.
            reader_pending.lock().await.clear();
        });

        Ok(WsApiConnection {
            write_tx,
            pending,
            next_id: AtomicU64::new(1),
            events: Mutex::new(Some(event_rx)),
            last_rate_limits: Mutex::new(Vec::new()),
            credentials: self.credentials.clone(),
            recv_window: self.config.recv_window,
        })
    }
}

/// Parse a non-response frame into an event.
fn parse_event(value: Value) -> WsApiEvent {
    let subscription_id = value.get("subscriptionId").and_then(Value::as_u64);
    let Some(event) = value.get("event") else {
        return WsApiEvent::Unknown(value);
    };

    if event.get("e").and_then(Value::as_str) == Some("serverShutdown") {
        let event_time = event.get("E").and_then(Value::as_u64).unwrap_or_default();
        return WsApiEvent::ServerShutdown { event_time };
    }

    match serde_json::from_value::<WebSocketEvent>(event.clone()) {
        Ok(parsed) => WsApiEvent::UserData {
            subscription_id,
            event: Box::new(parsed),
        },
        Err(_) => WsApiEvent::Unknown(value),
    }
}

/// An open WebSocket API connection.
pub struct WsApiConnection {
    write_tx: mpsc::UnboundedSender<Message>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<WsApiResponse>>>>,
    next_id: AtomicU64,
    events: Mutex<Option<mpsc::UnboundedReceiver<WsApiEvent>>>,
    last_rate_limits: Mutex<Vec<WsApiRateLimit>>,
    credentials: Option<Credentials>,
    recv_window: u64,
}

impl WsApiConnection {
    /// Get the rate limit status reported by the most recent response.
    pub async fn rate_limits(&self) -> Vec<WsApiRateLimit> {
        self.last_rate_limits.lock().await.clone()
    }

    /// Take the event receiver for user data stream and connection events.
    ///
    /// Returns `None` if the receiver was already taken.
    pub async fn take_events(&self) -> Option<mpsc::UnboundedReceiver<WsApiEvent>> {
        self.events.lock().await.take()
    }

    /// Send a raw request and wait for the matching response.
    ///
    /// Returns the `result` field of a successful response.
    pub async fn request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let mut frame = Map::new();
        frame.insert("id".to_string(), json!(id));
        frame.insert("method".to_string(), json!(method));
        if let Some(params) = params {
            frame.insert("params".to_string(), params);
        }

        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        let payload = serde_json::to_string(&Value::Object(frame))?;
        if self.write_tx.send(Message::Text(payload.into())).is_err() {
            self.pending.lock().await.remove(&id);
            return Err(Error::InvalidConfig(
                "WebSocket API connection is closed".to_string(),
            ));
        }

        let response = match tokio::time::timeout(REQUEST_TIMEOUT, rx).await {
            Ok(Ok(response)) => response,
            Ok(Err(_)) => {
                return Err(Error::InvalidConfig(
                    "WebSocket API connection is closed".to_string(),
                ));
            }
            Err(_) => {
                self.pending.lock().await.remove(&id);
                return Err(Error::Api {
                    code: -1007,
                    message: "Timeout waiting for response, status unknown".to_string(),
                });
            }
        };

        if let Some(rate_limits) = response.rate_limits {
            *self.last_rate_limits.lock().await = rate_limits;
        }

        match response.status {
            200 => Ok(response.result.unwrap_or(Value::Null)),
            418 | 429 => {
                let (code, message) = match response.error {
                    Some(e) => (e.code, e.msg),
                    None => (response.status as i32, "Rate limited".to_string()),
                };
                Err(Error::RateLimited {
                    code,
                    message,
                    retry_after: None,
                    ip_banned: response.status == 418,
                })
            }
            status => {
                let (code, message) = match response.error {
                    Some(e) => (e.code, e.msg),
                    None => (
                        status as i32,
                        format!("Request failed with status {}", status),
                    ),
                };
                Err(Error::Api { code, message })
            }
        }
    }

    /// Send a raw request and deserialize the result.
    pub async fn request_typed<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<T> {
        let result = self.request(method, params).await?;
        Ok(serde_json::from_value(result)?)
    }

    /// Send a signed request.
    ///
    /// Adds `apiKey`, `timestamp`, `recvWindow`, and `signature` to the
    /// parameters before sending.
    pub async fn signed_request(&self, method: &str, params: Map<String, Value>) -> Result<Value> {
        let params = self.sign_params(params)?;
        self.request(method, Some(Value::Object(params))).await
    }

    /// Send a signed request and deserialize the result.
    pub async fn signed_request_typed<T: DeserializeOwned>(
        &self,
        method: &str,
        params: Map<String, Value>,
    ) -> Result<T> {
        let result = self.signed_request(method, params).await?;
        Ok(serde_json::from_value(result)?)
    }

    fn credentials(&self) -> Result<&Credentials> {
        self.credentials
            .as_ref()
            .ok_or(Error::AuthenticationRequired)
    }

    fn sign_params(&self, mut params: Map<String, Value>) -> Result<Map<String, Value>> {
        let credentials = self.credentials()?;

        params.insert("apiKey".to_string(), json!(credentials.api_key()));
        params.insert("timestamp".to_string(), json!(get_timestamp()?));
        if self.recv_window > 0 {
            params.insert("recvWindow".to_string(), json!(self.recv_window));
        }

        let payload = signature_payload(&params);
        params.insert("signature".to_string(), json!(credentials.sign(&payload)));
        Ok(params)
    }

    /// Test connectivity to the WebSocket API.
    pub async fn ping(&self) -> Result<()> {
        self.request("ping", None).await?;
        Ok(())
    }

    /// Get the current server time in milliseconds.
    pub async fn server_time(&self) -> Result<u64> {
        let result = self.request("time", None).await?;
        result
            .get("serverTime")
            .and_then(Value::as_u64)
            .ok_or_else(|| Error::Api {
                code: -1,
                message: "Missing serverTime in response".to_string(),
            })
    }

    /// Authenticate the connection with `session.logon`.
    ///
    /// Requires an Ed25519 API key.
    /// After logon, signed requests on this connection no longer need
    /// explicit `apiKey` and `signature` parameters, and
    /// `subscribe_user_data` becomes available.
    pub async fn session_logon(&self) -> Result<WsApiSession> {
        let params = self.sign_params(Map::new())?;
        self.request_typed("session.logon", Some(Value::Object(params)))
            .await
    }

    /// Query the session authentication status.
    pub async fn session_status(&self) -> Result<WsApiSession> {
        self.request_typed("session.status", None).await
    }

    /// Log out of the session.
    ///
    /// Closes subscriptions created with `subscribe_user_data` but not
    /// those created with `subscribe_user_data_with_signature`.
    pub async fn session_logout(&self) -> Result<WsApiSession> {
        self.request_typed("session.logout", None).await
    }

    /// List the active user data stream subscription ids for this session.
    pub async fn session_subscriptions(&self) -> Result<Vec<u64>> {
        let result = self.request("session.subscriptions", None).await?;
        let ids = result
            .as_array()
            .map(|entries| {
                entries
                    .iter()
                    .filter_map(|e| e.get("subscriptionId").and_then(Value::as_u64))
                    .collect()
            })
            .unwrap_or_default();
        Ok(ids)
    }

    /// Subscribe to the user data stream of the authenticated session.
    ///
    /// Requires a prior `session_logon` with an Ed25519 key.
    /// Returns the subscription id carried by matching events.
    pub async fn subscribe_user_data(&self) -> Result<u64> {
        let result = self.request("userDataStream.subscribe", None).await?;
        subscription_id_from(&result)
    }

    /// Subscribe to a user data stream using an API key signature.
    ///
    /// Works with any API key type and does not require `session_logon`.
    /// Returns the subscription id carried by matching events.
    pub async fn subscribe_user_data_with_signature(&self) -> Result<u64> {
        let params = self.sign_params(Map::new())?;
        let result = self
            .request(
                "userDataStream.subscribe.signature",
                Some(Value::Object(params)),
            )
            .await?;
        subscription_id_from(&result)
    }

    /// Unsubscribe from user data streams.
    ///
    /// Closes the given subscription, or all subscriptions when
    /// `subscription_id` is `None`.
    pub async fn unsubscribe_user_data(&self, subscription_id: Option<u64>) -> Result<()> {
        let params = subscription_id.map(|id| json!({ "subscriptionId": id }));
        self.request("userDataStream.unsubscribe", params).await?;
        Ok(())
    }

    /// Place a new order via `order.place`.
    pub async fn place_order(&self, order: &NewOrder) -> Result<OrderFull> {
        self.signed_request_typed("order.place", order_params(order.to_params()))
            .await
    }

    /// Test a new order via `order.test` without sending it to the matching engine.
    pub async fn test_order(&self, order: &NewOrder) -> Result<()> {
        let _: Value = self
            .signed_request_typed("order.test", order_params(order.to_params()))
            .await?;
        Ok(())
    }

    /// Query an order's status via `order.status`.
    pub async fn order_status(&self, symbol: &str, order_id: u64) -> Result<Order> {
        let mut params = Map::new();
        params.insert("symbol".to_string(), json!(symbol));
        params.insert("orderId".to_string(), json!(order_id));
        self.signed_request_typed("order.status", params).await
    }

    /// Cancel an order via `order.cancel`.
    pub async fn cancel_order(&self, symbol: &str, order_id: u64) -> Result<CancelOrderResponse> {
        let mut params = Map::new();
        params.insert("symbol".to_string(), json!(symbol));
        params.insert("orderId".to_string(), json!(order_id));
        self.signed_request_typed("order.cancel", params).await
    }

    /// Cancel all open orders on a symbol via `openOrders.cancelAll`.
    pub async fn cancel_all_orders(&self, symbol: &str) -> Result<Vec<CancelOrderResponse>> {
        let mut params = Map::new();
        params.insert("symbol".to_string(), json!(symbol));
        self.signed_request_typed("openOrders.cancelAll", params)
            .await
    }

    /// Query current open orders via `openOrders.status`.
    pub async fn open_orders(&self, symbol: Option<&str>) -> Result<Vec<Order>> {
        let mut params = Map::new();
        if let Some(symbol) = symbol {
            params.insert("symbol".to_string(), json!(symbol));
        }
        self.signed_request_typed("openOrders.status", params).await
    }

    /// Query account information via `account.status`.
    pub async fn account_status(&self) -> Result<AccountInfo> {
        self.signed_request_typed("account.status", Map::new())
            .await
    }

    /// Close the connection.
    ///
    /// Pending requests fail once the server confirms the close.
    pub fn close(&self) {
        let _ = self.write_tx.send(Message::Close(None));
    }
}

impl Drop for WsApiConnection {
    fn drop(&mut self) {
        // Ask the server to close so the background tasks shut down instead
        // of lingering until the 24 hour connection limit.
        let _ = self.write_tx.send(Message::Close(None));
    }
}

impl std::fmt::Debug for WsApiConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsApiConnection")
            .field("has_credentials", &self.credentials.is_some())
            .finish()
    }
}

/// Extract a subscription id from a subscribe response.
fn subscription_id_from(result: &Value) -> Result<u64> {
    result
        .get("subscriptionId")
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::Api {
            code: -1,
            message: "Missing subscriptionId in response".to_string(),
        })
}

/// Parameter names with integer JSON types in the WebSocket API.
const INTEGER_PARAMS: &[&str] = &[
    "strategyId",
    "strategyType",
    "trailingDelta",
    "orderId",
    "orderListId",
    "cancelOrderId",
    "fromId",
    "startTime",
    "endTime",
    "limit",
    "subscriptionId",
];

/// Convert REST-style string parameters to a WebSocket API parameter object.
fn order_params(params: Vec<(String, String)>) -> Map<String, Value> {
    let mut map = Map::new();
    for (key, value) in params {
        let json_value = if INTEGER_PARAMS.contains(&key.as_str()) {
            match value.parse::<i64>() {
                Ok(n) => json!(n),
                Err(_) => json!(value),
            }
        } else {
            json!(value)
        };
        map.insert(key, json_value);
    }
    map
}

/// Build the signature payload for WebSocket API requests.
///
/// Parameters are sorted alphabetically by name and joined as
/// `name=value` pairs with `&`, without percent-encoding.
fn signature_payload(params: &Map<String, Value>) -> String {
    let mut keys: Vec<&String> = params.keys().filter(|k| *k != "signature").collect();
    keys.sort();
    keys.iter()
        .map(|k| format!("{}={}", k, json_value_as_plain_string(&params[*k])))
        .collect::<Vec<_>>()
        .join("&")
}

/// Render a JSON value the way it appears in the signature payload.
fn json_value_as_plain_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_payload_sorted() {
        let mut params = Map::new();
        params.insert("symbol".to_string(), json!("BTCUSDT"));
        params.insert("apiKey".to_string(), json!("key"));
        params.insert("timestamp".to_string(), json!(1645423376532u64));
        let payload = signature_payload(&params);
        assert_eq!(payload, "apiKey=key&symbol=BTCUSDT&timestamp=1645423376532");
    }

    #[test]
    fn test_signature_payload_matches_documentation_vector() {
        // Test vector from the official WebSocket API documentation.
        let mut params = Map::new();
        params.insert(
            "apiKey".to_string(),
            json!("vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A"),
        );
        params.insert("symbol".to_string(), json!("BTCUSDT"));
        params.insert("side".to_string(), json!("SELL"));
        params.insert("type".to_string(), json!("LIMIT"));
        params.insert("timeInForce".to_string(), json!("GTC"));
        params.insert("quantity".to_string(), json!("0.01000000"));
        params.insert("price".to_string(), json!("52000.00"));
        params.insert("recvWindow".to_string(), json!(100));
        params.insert("timestamp".to_string(), json!(1645423376532u64));

        let payload = signature_payload(&params);
        assert_eq!(
            payload,
            "apiKey=vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A&price=52000.00&quantity=0.01000000&recvWindow=100&side=SELL&symbol=BTCUSDT&timeInForce=GTC&timestamp=1645423376532&type=LIMIT"
        );

        let credentials = Credentials::new(
            "vmPUZE6mv9SD5VNHk4HlWFsOr6aKE2zvsw0MuIgwCIPy6utIco14y7Ju91duEh8A",
            "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j",
        );
        assert_eq!(
            credentials.sign(&payload),
            "aa1b5712c094bc4e57c05a1a5c1fd8d88dcd628338ea863fec7b88e59fe2db24"
        );
    }

    #[test]
    fn test_order_params_integer_conversion() {
        let params = vec![
            ("symbol".to_string(), "BTCUSDT".to_string()),
            ("strategyId".to_string(), "42".to_string()),
            ("price".to_string(), "52000.00".to_string()),
        ];
        let map = order_params(params);
        assert_eq!(map["symbol"], json!("BTCUSDT"));
        assert_eq!(map["strategyId"], json!(42));
        assert_eq!(map["price"], json!("52000.00"));
    }

    #[test]
    fn test_parse_event_server_shutdown() {
        let value = json!({
            "event": { "e": "serverShutdown", "E": 1770123456789u64 }
        });
        match parse_event(value) {
            WsApiEvent::ServerShutdown { event_time } => {
                assert_eq!(event_time, 1770123456789);
            }
            other => panic!("Unexpected event: {:?}", other),
        }
    }

    #[test]
    fn test_parse_event_user_data() {
        let value = json!({
            "subscriptionId": 3,
            "event": {
                "e": "balanceUpdate",
                "E": 1573200697110u64,
                "a": "BTC",
                "d": "100.00000000",
                "T": 1573200697068u64
            }
        });
        match parse_event(value) {
            WsApiEvent::UserData {
                subscription_id,
                event,
            } => {
                assert_eq!(subscription_id, Some(3));
                assert!(matches!(*event, WebSocketEvent::BalanceUpdate(_)));
            }
            other => panic!("Unexpected event: {:?}", other),
        }
    }
}
