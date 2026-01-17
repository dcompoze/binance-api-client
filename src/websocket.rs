//! WebSocket client for real-time market data streaming.
//!
//! This module provides a WebSocket client for connecting to Binance's
//! real-time data streams with support for:
//!
//! - Auto-reconnection with exponential backoff
//! - Depth cache management (local order book)
//! - User data stream keep-alive
//! - Connection health monitoring
//!
//! # Example
//!
//! ```rust,ignore
//! use binance_api_client::{Binance, WebSocketClient};
//! use futures::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> binance_api_client::Result<()> {
//!     let client = Binance::new_unauthenticated()?;
//!     let ws = client.websocket();
//!
//!     // Connect to a single stream
//!     let stream = ws.agg_trade_stream("btcusdt");
//!     let mut conn = ws.connect(&stream).await?;
//!
//!     while let Some(event) = conn.next().await {
//!         match event? {
//!             WebSocketEvent::AggTrade(trade) => {
//!                 println!("{}: {} @ {}", trade.symbol, trade.quantity, trade.price);
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

use futures::{Future, SinkExt, Stream, StreamExt};
use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, RwLock, mpsc};
use tokio::time::{interval, sleep, timeout};
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream as TungsteniteStream, connect_async,
    tungstenite::{Bytes, Message},
};

use crate::config::Config;
use crate::models::OrderBook;
use crate::models::websocket::{DepthEvent, WebSocketEvent};
use crate::types::KlineInterval;
use crate::{Error, Result};

// Constants.

/// Maximum number of reconnection attempts before giving up.
const MAX_RECONNECTS: u32 = 5;

/// Maximum delay between reconnection attempts (in seconds).
const MAX_RECONNECT_DELAY_SECS: u64 = 60;

/// Base delay for exponential backoff (in milliseconds).
const BASE_RECONNECT_DELAY_MS: u64 = 100;

/// Timeout for WebSocket operations (in seconds).
const WS_TIMEOUT_SECS: u64 = 30;

/// Interval for health check pings (in seconds).
const HEALTH_CHECK_INTERVAL_SECS: u64 = 30;

/// User data stream keepalive interval (in seconds).
/// Should be less than 60 minutes (the listen key expiry time).
const USER_STREAM_KEEPALIVE_SECS: u64 = 30 * 60; // 30 minutes

// WebSocket client.

/// WebSocket client for connecting to Binance streams.
#[derive(Clone)]
pub struct WebSocketClient {
    config: Config,
}

impl WebSocketClient {
    /// Create a new WebSocket client.
    pub(crate) fn new(config: Config) -> Self {
        Self { config }
    }

    /// Get the WebSocket endpoint URL.
    pub fn endpoint(&self) -> &str {
        &self.config.ws_endpoint
    }

    /// Connect to a single stream.
    ///
    /// # Arguments
    ///
    /// * `stream` - Stream name (e.g., "btcusdt@aggTrade")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ws = client.websocket();
    /// let stream = ws.agg_trade_stream("btcusdt");
    /// let mut conn = ws.connect(&stream).await?;
    /// ```
    pub async fn connect(&self, stream: &str) -> Result<WebSocketConnection> {
        let url = format!("{}/ws/{}", self.config.ws_endpoint, stream);
        self.connect_url(&url).await
    }

    /// Connect to multiple streams (combined stream).
    ///
    /// # Arguments
    ///
    /// * `streams` - List of stream names
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ws = client.websocket();
    /// let streams = vec![
    ///     ws.agg_trade_stream("btcusdt"),
    ///     ws.agg_trade_stream("ethusdt"),
    /// ];
    /// let mut conn = ws.connect_combined(&streams).await?;
    /// ```
    pub async fn connect_combined(&self, streams: &[String]) -> Result<WebSocketConnection> {
        let streams_param = streams.join("/");
        let url = format!(
            "{}/stream?streams={}",
            self.config.ws_endpoint, streams_param
        );
        self.connect_url(&url).await
    }

    /// Connect to a user data stream.
    ///
    /// # Arguments
    ///
    /// * `listen_key` - Listen key obtained from `user_stream().start()`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let listen_key = client.user_stream().start().await?;
    /// let mut conn = client.websocket().connect_user_stream(&listen_key).await?;
    /// ```
    pub async fn connect_user_stream(&self, listen_key: &str) -> Result<WebSocketConnection> {
        let url = format!("{}/ws/{}", self.config.ws_endpoint, listen_key);
        self.connect_url(&url).await
    }

    /// Connect with auto-reconnection support.
    ///
    /// Returns a `ReconnectingWebSocket` that automatically reconnects on
    /// disconnection with exponential backoff.
    ///
    /// # Arguments
    ///
    /// * `stream` - Stream name (e.g., "btcusdt@aggTrade")
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let ws = client.websocket();
    /// let stream = ws.agg_trade_stream("btcusdt");
    /// let mut conn = ws.connect_with_reconnect(&stream).await?;
    ///
    /// // Connection will auto-reconnect on failure
    /// while let Some(event) = conn.next().await {
    ///     println!("{:?}", event?);
    /// }
    /// ```
    pub async fn connect_with_reconnect(&self, stream: &str) -> Result<ReconnectingWebSocket> {
        let url = format!("{}/ws/{}", self.config.ws_endpoint, stream);
        ReconnectingWebSocket::new(url, ReconnectConfig::default()).await
    }

    /// Connect to combined streams with auto-reconnection support.
    pub async fn connect_combined_with_reconnect(
        &self,
        streams: &[String],
    ) -> Result<ReconnectingWebSocket> {
        let streams_param = streams.join("/");
        let url = format!(
            "{}/stream?streams={}",
            self.config.ws_endpoint, streams_param
        );
        ReconnectingWebSocket::new(url, ReconnectConfig::default()).await
    }

    async fn connect_url(&self, url: &str) -> Result<WebSocketConnection> {
        let (ws_stream, _) = connect_async(url).await.map_err(Error::WebSocket)?;
        Ok(WebSocketConnection::new(ws_stream))
    }

    // Stream Name Helpers.

    /// Get the aggregate trade stream name for a symbol.
    ///
    /// Stream: `<symbol>@aggTrade`
    pub fn agg_trade_stream(&self, symbol: &str) -> String {
        format!("{}@aggTrade", symbol.to_lowercase())
    }

    /// Get the trade stream name for a symbol.
    ///
    /// Stream: `<symbol>@trade`
    pub fn trade_stream(&self, symbol: &str) -> String {
        format!("{}@trade", symbol.to_lowercase())
    }

    /// Get the kline/candlestick stream name for a symbol.
    ///
    /// Stream: `<symbol>@kline_<interval>`
    pub fn kline_stream(&self, symbol: &str, interval: KlineInterval) -> String {
        format!("{}@kline_{}", symbol.to_lowercase(), interval)
    }

    /// Get the mini ticker stream name for a symbol.
    ///
    /// Stream: `<symbol>@miniTicker`
    pub fn mini_ticker_stream(&self, symbol: &str) -> String {
        format!("{}@miniTicker", symbol.to_lowercase())
    }

    /// Get the mini ticker stream for all symbols.
    ///
    /// Stream: `!miniTicker@arr`
    pub fn all_mini_ticker_stream(&self) -> String {
        "!miniTicker@arr".to_string()
    }

    /// Get the 24hr ticker stream name for a symbol.
    ///
    /// Stream: `<symbol>@ticker`
    pub fn ticker_stream(&self, symbol: &str) -> String {
        format!("{}@ticker", symbol.to_lowercase())
    }

    /// Get the 24hr ticker stream for all symbols.
    ///
    /// Stream: `!ticker@arr`
    pub fn all_ticker_stream(&self) -> String {
        "!ticker@arr".to_string()
    }

    /// Get the book ticker stream name for a symbol.
    ///
    /// Stream: `<symbol>@bookTicker`
    pub fn book_ticker_stream(&self, symbol: &str) -> String {
        format!("{}@bookTicker", symbol.to_lowercase())
    }

    /// Get the book ticker stream for all symbols.
    ///
    /// Stream: `!bookTicker`
    pub fn all_book_ticker_stream(&self) -> String {
        "!bookTicker".to_string()
    }

    /// Get the partial book depth stream name.
    ///
    /// Stream: `<symbol>@depth<levels>` or `<symbol>@depth<levels>@100ms`
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `levels` - Depth levels (5, 10, or 20)
    /// * `fast` - If true, use 100ms update speed instead of 1000ms
    pub fn partial_depth_stream(&self, symbol: &str, levels: u8, fast: bool) -> String {
        let base = format!("{}@depth{}", symbol.to_lowercase(), levels);
        if fast {
            format!("{}@100ms", base)
        } else {
            base
        }
    }

    /// Get the diff depth stream name.
    ///
    /// Stream: `<symbol>@depth` or `<symbol>@depth@100ms`
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `fast` - If true, use 100ms update speed instead of 1000ms
    pub fn diff_depth_stream(&self, symbol: &str, fast: bool) -> String {
        let base = format!("{}@depth", symbol.to_lowercase());
        if fast {
            format!("{}@100ms", base)
        } else {
            base
        }
    }
}

// Basic WebSocket connection.

/// An active WebSocket connection.
///
/// Use `next()` to receive events, or convert to a `Stream` for async iteration.
pub struct WebSocketConnection {
    inner: TungsteniteStream<MaybeTlsStream<TcpStream>>,
    last_ping: Instant,
}

impl WebSocketConnection {
    fn new(stream: TungsteniteStream<MaybeTlsStream<TcpStream>>) -> Self {
        Self {
            inner: stream,
            last_ping: Instant::now(),
        }
    }

    /// Receive the next WebSocket event.
    ///
    /// Returns `None` if the connection is closed.
    pub async fn next(&mut self) -> Option<Result<WebSocketEvent>> {
        loop {
            match self.inner.next().await? {
                Ok(Message::Text(text)) => {
                    // Try to parse as a combined stream message first
                    if let Ok(combined) = serde_json::from_str::<CombinedStreamMessage>(&text) {
                        return Some(Ok(combined.data));
                    }
                    // Otherwise parse as a regular event
                    return Some(serde_json::from_str(&text).map_err(Error::Serialization));
                }
                Ok(Message::Binary(data)) => {
                    if let Ok(combined) = serde_json::from_slice::<CombinedStreamMessage>(&data) {
                        return Some(Ok(combined.data));
                    }
                    return Some(serde_json::from_slice(&data).map_err(Error::Serialization));
                }
                Ok(Message::Ping(data)) => {
                    self.last_ping = Instant::now();
                    // Respond to ping with pong
                    if let Err(e) = self.inner.send(Message::Pong(data)).await {
                        return Some(Err(Error::WebSocket(e)));
                    }
                }
                Ok(Message::Pong(_)) => {
                    // Ignore pong messages
                    continue;
                }
                Ok(Message::Close(_)) => {
                    return None;
                }
                Ok(Message::Frame(_)) => {
                    // Raw frames shouldn't appear in normal operation
                    continue;
                }
                Err(e) => {
                    return Some(Err(Error::WebSocket(e)));
                }
            }
        }
    }

    /// Receive the next raw message (for depth cache management).
    pub(crate) async fn next_raw(&mut self) -> Option<Result<serde_json::Value>> {
        loop {
            match self.inner.next().await? {
                Ok(Message::Text(text)) => {
                    return Some(serde_json::from_str(&text).map_err(Error::Serialization));
                }
                Ok(Message::Binary(data)) => {
                    return Some(serde_json::from_slice(&data).map_err(Error::Serialization));
                }
                Ok(Message::Ping(data)) => {
                    self.last_ping = Instant::now();
                    if let Err(e) = self.inner.send(Message::Pong(data)).await {
                        return Some(Err(Error::WebSocket(e)));
                    }
                }
                Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => continue,
                Ok(Message::Close(_)) => return None,
                Err(e) => return Some(Err(Error::WebSocket(e))),
            }
        }
    }

    /// Send a ping message.
    pub async fn ping(&mut self) -> Result<()> {
        self.inner
            .send(Message::Ping(Bytes::new()))
            .await
            .map_err(Error::WebSocket)
    }

    /// Close the WebSocket connection gracefully.
    pub async fn close(&mut self) -> Result<()> {
        self.inner.close(None).await.map_err(Error::WebSocket)
    }

    /// Get the time since the last ping was received.
    pub fn time_since_last_ping(&self) -> Duration {
        self.last_ping.elapsed()
    }

    /// Convert this connection into a `Stream` of events.
    pub fn into_stream(self) -> WebSocketEventStream {
        WebSocketEventStream { inner: self }
    }
}

/// A `Stream` wrapper for WebSocket events.
pub struct WebSocketEventStream {
    inner: WebSocketConnection,
}

impl Stream for WebSocketEventStream {
    type Item = Result<WebSocketEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let future = self.inner.next();
        tokio::pin!(future);
        future.poll(cx)
    }
}

// Reconnecting WebSocket.

/// Configuration for auto-reconnection behavior.
#[derive(Debug, Clone)]
pub struct ReconnectConfig {
    /// Maximum number of reconnection attempts.
    pub max_reconnects: u32,
    /// Maximum delay between reconnection attempts.
    pub max_reconnect_delay: Duration,
    /// Base delay for exponential backoff.
    pub base_delay: Duration,
    /// Whether to enable health check pings.
    pub health_check_enabled: bool,
    /// Interval for health check pings.
    pub health_check_interval: Duration,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            max_reconnects: MAX_RECONNECTS,
            max_reconnect_delay: Duration::from_secs(MAX_RECONNECT_DELAY_SECS),
            base_delay: Duration::from_millis(BASE_RECONNECT_DELAY_MS),
            health_check_enabled: true,
            health_check_interval: Duration::from_secs(HEALTH_CHECK_INTERVAL_SECS),
        }
    }
}

/// Connection state for reconnecting WebSocket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is being established.
    Connecting,
    /// Connection is active and streaming.
    Connected,
    /// Connection is being reconnected.
    Reconnecting,
    /// Connection has been closed.
    Closed,
}

/// A WebSocket connection with automatic reconnection support.
///
/// This wrapper handles connection failures by automatically reconnecting
/// with exponential backoff.
pub struct ReconnectingWebSocket {
    connection: Arc<Mutex<Option<WebSocketConnection>>>,
    state: Arc<RwLock<ConnectionState>>,
    reconnect_count: Arc<AtomicU64>,
    is_closed: Arc<AtomicBool>,
    event_rx: mpsc::Receiver<Result<WebSocketEvent>>,
}

impl ReconnectingWebSocket {
    /// Create a new reconnecting WebSocket connection.
    pub async fn new(url: String, config: ReconnectConfig) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::channel(1000);
        let connection = Arc::new(Mutex::new(None));
        let state = Arc::new(RwLock::new(ConnectionState::Connecting));
        let reconnect_count = Arc::new(AtomicU64::new(0));
        let is_closed = Arc::new(AtomicBool::new(false));

        // Perform initial connection
        let (ws_stream, _) = connect_async(&url).await.map_err(Error::WebSocket)?;
        {
            let mut conn = connection.lock().await;
            *conn = Some(WebSocketConnection::new(ws_stream));
        }
        *state.write().await = ConnectionState::Connected;

        let ws = Self {
            connection: connection.clone(),
            state: state.clone(),
            reconnect_count: reconnect_count.clone(),
            is_closed: is_closed.clone(),
            event_rx,
        };

        // Start the read loop in a background task
        tokio::spawn(async move {
            Self::read_loop(
                url,
                config,
                connection,
                state,
                reconnect_count,
                is_closed,
                event_tx,
            )
            .await;
        });

        Ok(ws)
    }

    async fn read_loop(
        url: String,
        config: ReconnectConfig,
        connection: Arc<Mutex<Option<WebSocketConnection>>>,
        state: Arc<RwLock<ConnectionState>>,
        reconnect_count: Arc<AtomicU64>,
        is_closed: Arc<AtomicBool>,
        event_tx: mpsc::Sender<Result<WebSocketEvent>>,
    ) {
        loop {
            if is_closed.load(Ordering::SeqCst) {
                break;
            }

            // Read from connection
            let event = {
                let mut conn_guard = connection.lock().await;
                if let Some(ref mut conn) = *conn_guard {
                    match timeout(Duration::from_secs(WS_TIMEOUT_SECS), conn.next()).await {
                        Ok(Some(event)) => Some(event),
                        Ok(None) => None, // Connection closed
                        Err(_) => {
                            // Timeout - connection might be stale
                            None
                        }
                    }
                } else {
                    None
                }
            };

            match event {
                Some(Ok(ev)) => {
                    if event_tx.send(Ok(ev)).await.is_err() {
                        // Receiver dropped, exit
                        break;
                    }
                }
                Some(Err(e)) => {
                    // Send error and attempt reconnect
                    let _ = event_tx.send(Err(e)).await;
                    Self::attempt_reconnect(
                        &url,
                        &config,
                        &connection,
                        &state,
                        &reconnect_count,
                        &is_closed,
                    )
                    .await;
                }
                None => {
                    // Connection closed or timed out, attempt reconnect
                    Self::attempt_reconnect(
                        &url,
                        &config,
                        &connection,
                        &state,
                        &reconnect_count,
                        &is_closed,
                    )
                    .await;
                }
            }
        }

        *state.write().await = ConnectionState::Closed;
    }

    async fn attempt_reconnect(
        url: &str,
        config: &ReconnectConfig,
        connection: &Arc<Mutex<Option<WebSocketConnection>>>,
        state: &Arc<RwLock<ConnectionState>>,
        reconnect_count: &Arc<AtomicU64>,
        is_closed: &Arc<AtomicBool>,
    ) {
        if is_closed.load(Ordering::SeqCst) {
            return;
        }

        *state.write().await = ConnectionState::Reconnecting;

        let count = reconnect_count.fetch_add(1, Ordering::SeqCst) + 1;

        if count > config.max_reconnects as u64 {
            is_closed.store(true, Ordering::SeqCst);
            *state.write().await = ConnectionState::Closed;
            return;
        }

        // Calculate delay with exponential backoff and jitter
        let delay = Self::calculate_backoff_delay(count, config);
        sleep(delay).await;

        // Attempt to reconnect
        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                let mut conn = connection.lock().await;
                *conn = Some(WebSocketConnection::new(ws_stream));
                *state.write().await = ConnectionState::Connected;
                reconnect_count.store(0, Ordering::SeqCst);
            }
            Err(_) => {
                // Will retry on next loop iteration
            }
        }
    }

    fn calculate_backoff_delay(attempt: u64, config: &ReconnectConfig) -> Duration {
        let base_ms = config.base_delay.as_millis() as u64;
        let exp_delay = base_ms.saturating_mul(2u64.saturating_pow(attempt as u32));
        let max_delay_ms = config.max_reconnect_delay.as_millis() as u64;
        let delay_ms = exp_delay.min(max_delay_ms);

        // Add jitter (±25%)
        let jitter = (delay_ms as f64 * 0.25 * (rand_simple() * 2.0 - 1.0)) as i64;
        let final_delay = (delay_ms as i64 + jitter).max(0) as u64;

        Duration::from_millis(final_delay)
    }

    /// Receive the next WebSocket event.
    pub async fn next(&mut self) -> Option<Result<WebSocketEvent>> {
        self.event_rx.recv().await
    }

    /// Get the current connection state.
    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    /// Get the number of reconnection attempts.
    pub fn reconnect_count(&self) -> u64 {
        self.reconnect_count.load(Ordering::SeqCst)
    }

    /// Check if the connection is closed.
    pub fn is_closed(&self) -> bool {
        self.is_closed.load(Ordering::SeqCst)
    }

    /// Close the connection.
    pub async fn close(&self) {
        self.is_closed.store(true, Ordering::SeqCst);
        let mut conn = self.connection.lock().await;
        if let Some(ref mut c) = *conn {
            let _ = c.close().await;
        }
        *conn = None;
        *self.state.write().await = ConnectionState::Closed;
    }
}

// Simple pseudo-random number generator for jitter.
fn rand_simple() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    nanos as f64 / u32::MAX as f64
}

// Depth cache.

/// A local order book cache that maintains bid/ask levels.
///
/// This struct provides efficient access to order book data with
/// sorted bids (highest first) and asks (lowest first).
#[derive(Debug, Clone)]
pub struct DepthCache {
    /// Trading pair symbol.
    pub symbol: String,
    /// Bid levels (price -> quantity), sorted descending by price.
    bids: BTreeMap<OrderedFloat, f64>,
    /// Ask levels (price -> quantity), sorted ascending by price.
    asks: BTreeMap<OrderedFloat, f64>,
    /// Last update ID from the exchange.
    pub last_update_id: u64,
    /// Last update time.
    pub update_time: Option<u64>,
}

/// Wrapper for f64 that implements Ord for use in BTreeMap.
#[derive(Debug, Clone, Copy, PartialEq)]
struct OrderedFloat(f64);

impl Eq for OrderedFloat {}

impl PartialOrd for OrderedFloat {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedFloat {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0
            .partial_cmp(&other.0)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl DepthCache {
    /// Create a new depth cache for a symbol.
    pub fn new(symbol: &str) -> Self {
        Self {
            symbol: symbol.to_string(),
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            last_update_id: 0,
            update_time: None,
        }
    }

    /// Initialize the cache from a REST API order book snapshot.
    pub fn initialize_from_snapshot(&mut self, order_book: &OrderBook) {
        self.bids.clear();
        self.asks.clear();

        for bid in &order_book.bids {
            if bid.quantity > 0.0 {
                self.bids.insert(OrderedFloat(bid.price), bid.quantity);
            }
        }

        for ask in &order_book.asks {
            if ask.quantity > 0.0 {
                self.asks.insert(OrderedFloat(ask.price), ask.quantity);
            }
        }

        self.last_update_id = order_book.last_update_id;
    }

    /// Apply a depth update event to the cache.
    ///
    /// Returns `true` if the update was applied, `false` if it was skipped
    /// (e.g., due to sequence issues).
    pub fn apply_update(&mut self, event: &DepthEvent) -> bool {
        // Skip if update is older than our snapshot
        if event.final_update_id <= self.last_update_id {
            return false;
        }

        // Check for sequence gap - would need to reinitialize
        if event.first_update_id > self.last_update_id + 1 {
            return false;
        }

        // Apply bid updates
        for bid in &event.bids {
            if bid.quantity == 0.0 {
                self.bids.remove(&OrderedFloat(bid.price));
            } else {
                self.bids.insert(OrderedFloat(bid.price), bid.quantity);
            }
        }

        // Apply ask updates
        for ask in &event.asks {
            if ask.quantity == 0.0 {
                self.asks.remove(&OrderedFloat(ask.price));
            } else {
                self.asks.insert(OrderedFloat(ask.price), ask.quantity);
            }
        }

        self.last_update_id = event.final_update_id;
        self.update_time = Some(event.event_time);

        true
    }

    /// Get the best bid (highest bid price and quantity).
    pub fn best_bid(&self) -> Option<(f64, f64)> {
        self.bids.iter().next_back().map(|(p, q)| (p.0, *q))
    }

    /// Get the best ask (lowest ask price and quantity).
    pub fn best_ask(&self) -> Option<(f64, f64)> {
        self.asks.iter().next().map(|(p, q)| (p.0, *q))
    }

    /// Get the bid-ask spread.
    pub fn spread(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some(ask - bid),
            _ => None,
        }
    }

    /// Get the mid price.
    pub fn mid_price(&self) -> Option<f64> {
        match (self.best_bid(), self.best_ask()) {
            (Some((bid, _)), Some((ask, _))) => Some((bid + ask) / 2.0),
            _ => None,
        }
    }

    /// Get all bids sorted by price (highest first).
    pub fn get_bids(&self) -> Vec<(f64, f64)> {
        self.bids.iter().rev().map(|(p, q)| (p.0, *q)).collect()
    }

    /// Get all asks sorted by price (lowest first).
    pub fn get_asks(&self) -> Vec<(f64, f64)> {
        self.asks.iter().map(|(p, q)| (p.0, *q)).collect()
    }

    /// Get the top N bids.
    pub fn get_top_bids(&self, n: usize) -> Vec<(f64, f64)> {
        self.bids
            .iter()
            .rev()
            .take(n)
            .map(|(p, q)| (p.0, *q))
            .collect()
    }

    /// Get the top N asks.
    pub fn get_top_asks(&self, n: usize) -> Vec<(f64, f64)> {
        self.asks.iter().take(n).map(|(p, q)| (p.0, *q)).collect()
    }

    /// Get the total bid volume.
    pub fn total_bid_volume(&self) -> f64 {
        self.bids.values().sum()
    }

    /// Get the total ask volume.
    pub fn total_ask_volume(&self) -> f64 {
        self.asks.values().sum()
    }
}

// Depth cache manager.

/// Configuration for the depth cache manager.
#[derive(Debug, Clone)]
pub struct DepthCacheConfig {
    /// Depth limit for initial snapshot (5, 10, 20, 50, 100, 500, 1000, 5000).
    pub depth_limit: u32,
    /// Whether to use fast (100ms) update speed.
    pub fast_updates: bool,
    /// Optional refresh interval to re-fetch snapshot.
    pub refresh_interval: Option<Duration>,
}

impl Default for DepthCacheConfig {
    fn default() -> Self {
        Self {
            depth_limit: 1000,
            fast_updates: false,
            refresh_interval: None,
        }
    }
}

/// State of the depth cache manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepthCacheState {
    /// Initializing the cache from snapshot.
    Initializing,
    /// Cache is synced and receiving updates.
    Synced,
    /// Cache is out of sync and needs reinitialization.
    OutOfSync,
    /// Cache manager has been stopped.
    Stopped,
}

/// Manages a local order book cache with WebSocket updates.
///
/// This manager follows Binance's recommended approach for maintaining
/// a local order book:
///
/// 1. Open a WebSocket connection for depth updates
/// 2. Buffer incoming events
/// 3. Fetch a REST API snapshot
/// 4. Apply buffered events that are newer than the snapshot
/// 5. Continue applying real-time updates
///
/// # Example
///
/// ```rust,ignore
/// use binance_api_client::Binance;
/// use binance_api_client::websocket::{DepthCacheManager, DepthCacheConfig};
///
/// let client = Binance::new_unauthenticated()?;
/// let config = DepthCacheConfig::default();
/// let mut manager = DepthCacheManager::new(client, "BTCUSDT", config).await?;
///
/// // Wait for initial sync
/// manager.wait_for_sync().await?;
///
/// // Get current order book
/// let cache = manager.get_cache().await;
/// println!("Best bid: {:?}", cache.best_bid());
/// println!("Best ask: {:?}", cache.best_ask());
///
/// // Receive updates
/// while let Some(cache) = manager.next().await {
///     println!("Mid price: {:?}", cache.mid_price());
/// }
/// ```
pub struct DepthCacheManager {
    symbol: String,
    cache: Arc<RwLock<DepthCache>>,
    state: Arc<RwLock<DepthCacheState>>,
    is_stopped: Arc<AtomicBool>,
    cache_rx: mpsc::Receiver<DepthCache>,
}

impl DepthCacheManager {
    /// Create a new depth cache manager.
    ///
    /// This will start the WebSocket connection and begin syncing the order book.
    pub async fn new(
        client: crate::Binance,
        symbol: &str,
        config: DepthCacheConfig,
    ) -> Result<Self> {
        let symbol = symbol.to_uppercase();
        let cache = Arc::new(RwLock::new(DepthCache::new(&symbol)));
        let state = Arc::new(RwLock::new(DepthCacheState::Initializing));
        let is_stopped = Arc::new(AtomicBool::new(false));
        let (cache_tx, cache_rx) = mpsc::channel(100);

        // Clone for the background task
        let symbol_clone = symbol.clone();
        let cache_clone = cache.clone();
        let state_clone = state.clone();
        let is_stopped_clone = is_stopped.clone();

        // Start the background sync task
        tokio::spawn(async move {
            Self::sync_loop(
                client,
                symbol_clone,
                config,
                cache_clone,
                state_clone,
                is_stopped_clone,
                cache_tx,
            )
            .await;
        });

        Ok(Self {
            symbol,
            cache,
            state,
            is_stopped,
            cache_rx,
        })
    }

    async fn sync_loop(
        client: crate::Binance,
        symbol: String,
        config: DepthCacheConfig,
        cache: Arc<RwLock<DepthCache>>,
        state: Arc<RwLock<DepthCacheState>>,
        is_stopped: Arc<AtomicBool>,
        cache_tx: mpsc::Sender<DepthCache>,
    ) {
        let ws = client.websocket();
        let stream = ws.diff_depth_stream(&symbol, config.fast_updates);

        loop {
            if is_stopped.load(Ordering::SeqCst) {
                break;
            }

            // Reset state
            *state.write().await = DepthCacheState::Initializing;

            // Connect to WebSocket
            let mut conn = match ws.connect(&stream).await {
                Ok(c) => c,
                Err(_) => {
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            // Buffer some initial events
            let mut initial_events = Vec::new();
            let buffer_timeout = Duration::from_secs(2);
            let start = Instant::now();

            while start.elapsed() < buffer_timeout {
                match timeout(Duration::from_millis(500), conn.next_raw()).await {
                    Ok(Some(Ok(raw))) => {
                        if let Ok(event) = serde_json::from_value::<DepthEvent>(raw) {
                            initial_events.push(event);
                        }
                    }
                    _ => break,
                }
            }

            // Fetch snapshot
            let snapshot = match client
                .market()
                .depth(&symbol, Some(config.depth_limit as u16))
                .await
            {
                Ok(s) => s,
                Err(_) => {
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

            // Initialize cache from snapshot
            {
                let mut cache_guard = cache.write().await;
                cache_guard.initialize_from_snapshot(&snapshot);

                // Apply buffered events
                for event in &initial_events {
                    cache_guard.apply_update(event);
                }
            }

            *state.write().await = DepthCacheState::Synced;

            // Send initial cache state
            {
                let cache_guard = cache.read().await;
                let _ = cache_tx.send(cache_guard.clone()).await;
            }

            // Main update loop
            let mut last_refresh = Instant::now();
            loop {
                if is_stopped.load(Ordering::SeqCst) {
                    break;
                }

                // Check if we need to refresh
                if let Some(refresh_interval) = config.refresh_interval {
                    if last_refresh.elapsed() >= refresh_interval {
                        // Re-fetch snapshot
                        if let Ok(snapshot) = client
                            .market()
                            .depth(&symbol, Some(config.depth_limit as u16))
                            .await
                        {
                            let mut cache_guard = cache.write().await;
                            cache_guard.initialize_from_snapshot(&snapshot);
                        }
                        last_refresh = Instant::now();
                    }
                }

                match timeout(Duration::from_secs(WS_TIMEOUT_SECS), conn.next_raw()).await {
                    Ok(Some(Ok(raw))) => {
                        if let Ok(event) = serde_json::from_value::<DepthEvent>(raw) {
                            let mut cache_guard = cache.write().await;
                            if cache_guard.apply_update(&event) {
                                // Successfully applied, send updated cache
                                let _ = cache_tx.send(cache_guard.clone()).await;
                            } else {
                                // Update failed (sequence gap), need to reinitialize
                                drop(cache_guard);
                                *state.write().await = DepthCacheState::OutOfSync;
                                break;
                            }
                        }
                    }
                    Ok(Some(Err(_))) | Ok(None) | Err(_) => {
                        // Connection error or timeout, reconnect
                        *state.write().await = DepthCacheState::OutOfSync;
                        break;
                    }
                }
            }

            // Brief delay before reconnecting
            sleep(Duration::from_millis(100)).await;
        }

        *state.write().await = DepthCacheState::Stopped;
    }

    /// Wait for the cache to be synchronized.
    pub async fn wait_for_sync(&self) -> Result<()> {
        let timeout_duration = Duration::from_secs(30);
        let start = Instant::now();

        loop {
            let state = *self.state.read().await;
            match state {
                DepthCacheState::Synced => return Ok(()),
                DepthCacheState::Stopped => {
                    return Err(Error::InvalidCredentials(
                        "Depth cache manager stopped".to_string(),
                    ));
                }
                _ => {
                    if start.elapsed() > timeout_duration {
                        return Err(Error::InvalidCredentials(
                            "Timeout waiting for depth cache sync".to_string(),
                        ));
                    }
                    sleep(Duration::from_millis(100)).await;
                }
            }
        }
    }

    /// Get the current depth cache.
    pub async fn get_cache(&self) -> DepthCache {
        self.cache.read().await.clone()
    }

    /// Get the current state of the manager.
    pub async fn state(&self) -> DepthCacheState {
        *self.state.read().await
    }

    /// Receive the next cache update.
    pub async fn next(&mut self) -> Option<DepthCache> {
        self.cache_rx.recv().await
    }

    /// Stop the depth cache manager.
    pub fn stop(&self) {
        self.is_stopped.store(true, Ordering::SeqCst);
    }

    /// Get the symbol being tracked.
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

// User data stream manager.

/// Manages a user data stream with automatic keep-alive.
///
/// This manager automatically refreshes the listen key every 30 minutes
/// to prevent the stream from expiring (listen keys expire after 60 minutes).
///
/// # Example
///
/// ```rust,ignore
/// use binance_api_client::Binance;
/// use binance_api_client::websocket::UserDataStreamManager;
///
/// let client = Binance::new("api_key", "secret_key")?;
/// let mut manager = UserDataStreamManager::new(client).await?;
///
/// while let Some(event) = manager.next().await {
///     match event? {
///         WebSocketEvent::ExecutionReport(report) => {
///             println!("Order update: {:?}", report);
///         }
///         WebSocketEvent::AccountPosition(position) => {
///             println!("Account update: {:?}", position);
///         }
///         _ => {}
///     }
/// }
/// ```
pub struct UserDataStreamManager {
    listen_key: Arc<RwLock<String>>,
    is_stopped: Arc<AtomicBool>,
    event_rx: mpsc::Receiver<Result<WebSocketEvent>>,
}

impl UserDataStreamManager {
    /// Create a new user data stream manager.
    ///
    /// This will start the listen key and begin receiving user data events.
    pub async fn new(client: crate::Binance) -> Result<Self> {
        // Get initial listen key
        let listen_key = client.user_stream().start().await?;
        let listen_key = Arc::new(RwLock::new(listen_key));
        let is_stopped = Arc::new(AtomicBool::new(false));
        let (event_tx, event_rx) = mpsc::channel(1000);

        // Clone for background tasks
        let listen_key_clone = listen_key.clone();
        let is_stopped_clone = is_stopped.clone();
        let client_clone = client.clone();

        // Start keep-alive task
        tokio::spawn(async move {
            Self::keepalive_loop(
                client_clone.clone(),
                listen_key_clone.clone(),
                is_stopped_clone.clone(),
            )
            .await;
        });

        // Start WebSocket connection task
        let listen_key_ws = listen_key.clone();
        let is_stopped_ws = is_stopped.clone();

        tokio::spawn(async move {
            Self::connection_loop(client, listen_key_ws, is_stopped_ws, event_tx).await;
        });

        Ok(Self {
            listen_key,
            is_stopped,
            event_rx,
        })
    }

    async fn keepalive_loop(
        client: crate::Binance,
        listen_key: Arc<RwLock<String>>,
        is_stopped: Arc<AtomicBool>,
    ) {
        let mut interval_timer = interval(Duration::from_secs(USER_STREAM_KEEPALIVE_SECS));

        loop {
            interval_timer.tick().await;

            if is_stopped.load(Ordering::SeqCst) {
                break;
            }

            let key = listen_key.read().await.clone();
            if client.user_stream().keepalive(&key).await.is_err() {
                // If keepalive fails, try to get a new listen key
                if let Ok(new_key) = client.user_stream().start().await {
                    *listen_key.write().await = new_key;
                }
            }
        }

        // Close the listen key when stopping
        let key = listen_key.read().await.clone();
        let _ = client.user_stream().close(&key).await;
    }

    async fn connection_loop(
        client: crate::Binance,
        listen_key: Arc<RwLock<String>>,
        is_stopped: Arc<AtomicBool>,
        event_tx: mpsc::Sender<Result<WebSocketEvent>>,
    ) {
        let reconnect_config = ReconnectConfig::default();

        loop {
            if is_stopped.load(Ordering::SeqCst) {
                break;
            }

            let key = listen_key.read().await.clone();
            let ws = client.websocket();

            match ws.connect_user_stream(&key).await {
                Ok(mut conn) => {
                    loop {
                        if is_stopped.load(Ordering::SeqCst) {
                            break;
                        }

                        match timeout(Duration::from_secs(WS_TIMEOUT_SECS), conn.next()).await {
                            Ok(Some(event)) => {
                                if event_tx.send(event).await.is_err() {
                                    // Receiver dropped
                                    return;
                                }
                            }
                            Ok(None) => {
                                // Connection closed
                                break;
                            }
                            Err(_) => {
                                // Timeout, continue
                                continue;
                            }
                        }
                    }
                }
                Err(_) => {
                    // Connection failed, wait before retry
                    sleep(reconnect_config.base_delay).await;
                }
            }

            // Brief delay before reconnecting
            sleep(Duration::from_millis(100)).await;
        }
    }

    /// Receive the next user data event.
    pub async fn next(&mut self) -> Option<Result<WebSocketEvent>> {
        self.event_rx.recv().await
    }

    /// Get the current listen key.
    pub async fn listen_key(&self) -> String {
        self.listen_key.read().await.clone()
    }

    /// Stop the user data stream manager.
    pub fn stop(&self) {
        self.is_stopped.store(true, Ordering::SeqCst);
    }

    /// Check if the manager is stopped.
    pub fn is_stopped(&self) -> bool {
        self.is_stopped.load(Ordering::SeqCst)
    }
}

// Connection health monitor.

/// Monitors WebSocket connection health with periodic pings.
///
/// This can be used to detect stale connections that are not receiving
/// any messages (including pings from the server).
pub struct ConnectionHealthMonitor {
    last_activity: Arc<RwLock<Instant>>,
    is_healthy: Arc<AtomicBool>,
    max_idle_duration: Duration,
}

impl ConnectionHealthMonitor {
    /// Create a new connection health monitor.
    ///
    /// # Arguments
    ///
    /// * `max_idle_duration` - Maximum time without activity before considering unhealthy.
    pub fn new(max_idle_duration: Duration) -> Self {
        Self {
            last_activity: Arc::new(RwLock::new(Instant::now())),
            is_healthy: Arc::new(AtomicBool::new(true)),
            max_idle_duration,
        }
    }

    /// Record activity on the connection.
    pub async fn record_activity(&self) {
        *self.last_activity.write().await = Instant::now();
        self.is_healthy.store(true, Ordering::SeqCst);
    }

    /// Check if the connection is healthy.
    pub async fn is_healthy(&self) -> bool {
        let last = *self.last_activity.read().await;
        let healthy = last.elapsed() < self.max_idle_duration;
        self.is_healthy.store(healthy, Ordering::SeqCst);
        healthy
    }

    /// Get the time since last activity.
    pub async fn time_since_last_activity(&self) -> Duration {
        self.last_activity.read().await.elapsed()
    }

    /// Start a background health check task that updates is_healthy periodically.
    pub fn start_background_check(
        self: Arc<Self>,
        check_interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        let monitor = self;
        tokio::spawn(async move {
            let mut interval_timer = interval(check_interval);
            loop {
                interval_timer.tick().await;
                monitor.is_healthy().await;
            }
        })
    }
}

// Combined stream message.

/// Combined stream message wrapper.
#[derive(serde::Deserialize)]
struct CombinedStreamMessage {
    #[allow(dead_code)]
    stream: String,
    data: WebSocketEvent,
}

// Tests.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_names() {
        let config = Config::default();
        let ws = WebSocketClient::new(config);

        assert_eq!(ws.agg_trade_stream("BTCUSDT"), "btcusdt@aggTrade");
        assert_eq!(ws.trade_stream("BTCUSDT"), "btcusdt@trade");
        assert_eq!(
            ws.kline_stream("BTCUSDT", KlineInterval::Hours1),
            "btcusdt@kline_1h"
        );
        assert_eq!(ws.ticker_stream("BTCUSDT"), "btcusdt@ticker");
        assert_eq!(ws.book_ticker_stream("BTCUSDT"), "btcusdt@bookTicker");
        assert_eq!(ws.all_mini_ticker_stream(), "!miniTicker@arr");
        assert_eq!(ws.all_ticker_stream(), "!ticker@arr");
        assert_eq!(ws.all_book_ticker_stream(), "!bookTicker");
    }

    #[test]
    fn test_depth_stream_names() {
        let config = Config::default();
        let ws = WebSocketClient::new(config);

        assert_eq!(
            ws.partial_depth_stream("BTCUSDT", 10, false),
            "btcusdt@depth10"
        );
        assert_eq!(
            ws.partial_depth_stream("BTCUSDT", 10, true),
            "btcusdt@depth10@100ms"
        );
        assert_eq!(ws.diff_depth_stream("BTCUSDT", false), "btcusdt@depth");
        assert_eq!(ws.diff_depth_stream("BTCUSDT", true), "btcusdt@depth@100ms");
    }

    #[test]
    fn test_depth_cache() {
        let mut cache = DepthCache::new("BTCUSDT");

        // Add some bids and asks
        cache.bids.insert(OrderedFloat(50000.0), 1.0);
        cache.bids.insert(OrderedFloat(49999.0), 2.0);
        cache.asks.insert(OrderedFloat(50001.0), 1.5);
        cache.asks.insert(OrderedFloat(50002.0), 2.5);

        assert_eq!(cache.best_bid(), Some((50000.0, 1.0)));
        assert_eq!(cache.best_ask(), Some((50001.0, 1.5)));
        assert_eq!(cache.spread(), Some(1.0));
        assert_eq!(cache.mid_price(), Some(50000.5));
    }

    #[test]
    fn test_reconnect_config_default() {
        let config = ReconnectConfig::default();
        assert_eq!(config.max_reconnects, MAX_RECONNECTS);
        assert_eq!(
            config.max_reconnect_delay,
            Duration::from_secs(MAX_RECONNECT_DELAY_SECS)
        );
        assert!(config.health_check_enabled);
    }

    #[test]
    fn test_depth_cache_config_default() {
        let config = DepthCacheConfig::default();
        assert_eq!(config.depth_limit, 1000);
        assert!(!config.fast_updates);
        assert!(config.refresh_interval.is_none());
    }

    #[test]
    fn test_connection_state() {
        assert_eq!(ConnectionState::Connecting, ConnectionState::Connecting);
        assert_ne!(ConnectionState::Connected, ConnectionState::Closed);
    }

    #[test]
    fn test_depth_cache_state() {
        assert_eq!(DepthCacheState::Initializing, DepthCacheState::Initializing);
        assert_ne!(DepthCacheState::Synced, DepthCacheState::OutOfSync);
    }

    #[test]
    fn test_ordered_float() {
        let a = OrderedFloat(1.0);
        let b = OrderedFloat(2.0);
        assert!(a < b);
        assert_eq!(a, OrderedFloat(1.0));
    }

    #[test]
    fn test_backoff_delay() {
        let config = ReconnectConfig::default();

        // First attempt should be around base delay
        let delay1 = ReconnectingWebSocket::calculate_backoff_delay(1, &config);
        assert!(delay1.as_millis() > 0);
        assert!(delay1 <= config.max_reconnect_delay);

        // Later attempts should have longer delays (on average)
        let delay5 = ReconnectingWebSocket::calculate_backoff_delay(5, &config);
        assert!(delay5 <= config.max_reconnect_delay);
    }
}
