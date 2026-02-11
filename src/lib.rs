//! Async Rust client for the Binance API.
//!
//! This library provides a type-safe, async interface to the Binance cryptocurrency
//! exchange API, supporting both REST and WebSocket endpoints.
//!
//! # Features
//!
//! - Full coverage of Binance Spot REST API
//! - WebSocket support for real-time market data streams
//! - User data stream support for account updates
//! - Automatic request signing for authenticated endpoints
//! - Production and testnet environment support
//! - Binance.US support
//!
//! # Quick Start
//!
//! ## Public API (No Authentication Required)
//!
//! ```rust,ignore
//! use binance_api_client::Binance;
//!
//! #[tokio::main]
//! async fn main() -> binance_api_client::Result<()> {
//!     // Create a client for public endpoints
//!     let client = Binance::new_unauthenticated()?;
//!
//!     // Ping the server
//!     client.market().ping().await?;
//!
//!     // Get server time
//!     let time = client.market().server_time().await?;
//!     println!("Server time: {}", time.server_time);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Authenticated API
//!
//! ```rust,ignore
//! use binance_api_client::Binance;
//!
//! #[tokio::main]
//! async fn main() -> binance_api_client::Result<()> {
//!     // Create an authenticated client
//!     let client = Binance::new("your_api_key", "your_secret_key")?;
//!
//!     // Access account information
//!     let account = client.account().get_account().await?;
//!     println!("Account balances: {:?}", account.balances);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Using Testnet
//!
//! ```rust,ignore
//! use binance_api_client::{Binance, Config};
//!
//! #[tokio::main]
//! async fn main() -> binance_api_client::Result<()> {
//!     let config = Config::testnet();
//!     let client = Binance::with_config(config, Some(("api_key", "secret_key")))?;
//!
//!     // Now all requests go to testnet
//!     client.market().ping().await?;
//!
//!     Ok(())
//! }
//! ```

#![deny(
    unstable_features,
    unused_must_use,
    unused_mut,
    unused_imports,
    unused_import_braces
)]

pub mod rest;
pub mod client;
pub mod config;
pub mod credentials;
pub mod error;
pub mod models;
pub mod types;
pub mod ws;

// Re-export main types at crate root
pub use client::Client;
pub use config::{Config, ConfigBuilder};
pub use credentials::{Credentials, SignatureType};
pub use error::{Error, Result};
pub use ws::{
    ConnectionHealthMonitor, ConnectionState, DepthCache, DepthCacheConfig, DepthCacheManager,
    DepthCacheState, ReconnectConfig, ReconnectingWebSocket, UserDataStreamManager,
    WebSocketClient, WebSocketConnection, WebSocketEventStream,
};

// Re-export commonly used types
pub use types::{
    AccountType, CancelReplaceMode, CancelReplaceResult, CancelRestrictions, ContingencyType,
    ExecutionType, KlineInterval, OcoOrderStatus, OcoStatus, OrderRateLimitExceededMode,
    OrderResponseType, OrderSide, OrderStatus, OrderType, RateLimitInterval, RateLimitType,
    SymbolPermission, SymbolStatus, TickerType, TimeInForce,
};

// Re-export commonly used models
pub use models::{
    // Account models
    AccountCommission,
    AccountInfo,
    // Wallet models
    AccountSnapshot,
    AccountSnapshotType,
    AccountStatus,
    // Market models
    AggTrade,
    Allocation,
    AmendListStatus,
    AmendOrderResponse,
    AmendedOrderInfo,
    ApiKeyPermissions,
    ApiTradingStatus,
    AssetDetail,
    AveragePrice,
    Balance,
    // Margin models
    BnbBurnStatus,
    BookTicker,
    CancelOrderResponse,
    CancelReplaceErrorData,
    CancelReplaceErrorInfo,
    CancelReplaceErrorResponse,
    CancelReplaceResponse,
    CancelReplaceSideResponse,
    CoinInfo,
    CoinNetwork,
    DepositAddress,
    DepositRecord,
    DepositStatus,
    ExchangeInfo,
    Fill,
    FundingAsset,
    InterestHistoryRecord,
    InterestRateRecord,
    IsolatedAccountLimit,
    IsolatedAssetDetails,
    IsolatedMarginAccountAsset,
    IsolatedMarginAccountDetails,
    IsolatedMarginTransferType,
    Kline,
    ListenKey,
    LoanRecord,
    MarginAccountDetails,
    MarginAsset,
    MarginAssetInfo,
    MarginOrderCancellation,
    MarginOrderResult,
    MarginOrderState,
    MarginPairDetails,
    MarginPriceIndex,
    MarginTrade,
    MarginTransferType,
    MaxBorrowableAmount,
    MaxTransferableAmount,
    OcoOrder,
    OcoOrderDetail,
    OcoOrderReport,
    Order,
    OrderAck,
    OrderAmendment,
    OrderBook,
    OrderBookEntry,
    OrderFull,
    OrderResponse,
    OrderResult,
    PreventedMatch,
    RateLimit,
    RecordsQueryResult,
    RepayRecord,
    RollingWindowTicker,
    RollingWindowTickerMini,
    ServerTime,
    SideEffectType,
    SorOrderCommissionRates,
    SorOrderTestResponse,
    Symbol,
    SymbolFilter,
    SystemStatus,
    Ticker24h,
    TickerPrice,
    Trade,
    TradeFee,
    TradingDayTicker,
    TradingDayTickerMini,
    TransactionId,
    TransferHistory,
    TransferRecord,
    TransferResponse,
    UnfilledOrderCount,
    UniversalTransferType,
    UserTrade,
    WalletBalance,
    WithdrawRecord,
    WithdrawResponse,
    WithdrawStatus,
    // WebSocket models
    websocket::{
        AccountBalance, AccountPositionEvent, AggTradeEvent, BalanceUpdateEvent, BookTickerEvent,
        DepthEvent, DepthLevel, ExecutionReportEvent, KlineData, KlineEvent, ListStatusEvent,
        ListStatusOrder, MiniTickerEvent, TickerEvent, TradeEvent, WebSocketEvent,
    },
};

// Re-export order builders for convenience
pub use rest::{
    CancelReplaceOrder, CancelReplaceOrderBuilder, NewOcoOrder, NewOpoOrder, NewOpocoOrder,
    NewOrder, NewOtoOrder, NewOtocoOrder, OcoOrderBuilder, OpoOrderBuilder, OpocoOrderBuilder,
    OrderBuilder, OtoOrderBuilder, OtocoOrderBuilder,
};

/// Main entry point for the Binance API client.
///
/// The `Binance` struct provides access to all API modules and handles
/// configuration and authentication.
#[derive(Clone)]
pub struct Binance {
    client: Client,
}

impl Binance {
    /// Create a new authenticated Binance client with default production configuration.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Binance API key
    /// * `secret_key` - Your Binance secret key
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use binance_api_client::Binance;
    ///
    /// # fn run() -> binance_api_client::Result<()> {
    /// let client = Binance::new("api_key", "secret_key")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(api_key: impl Into<String>, secret_key: impl Into<String>) -> Result<Self> {
        let config = Config::default();
        let credentials = Credentials::new(api_key, secret_key);
        let client = Client::new(config, credentials)?;
        Ok(Self { client })
    }

    /// Create a new unauthenticated Binance client for public endpoints only.
    ///
    /// This client can only access public market data endpoints.
    /// For account and trading endpoints, use `Binance::new()` with credentials.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use binance_api_client::Binance;
    ///
    /// # fn run() -> binance_api_client::Result<()> {
    /// let client = Binance::new_unauthenticated()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new_unauthenticated() -> Result<Self> {
        let config = Config::default();
        let client = Client::new_unauthenticated(config)?;
        Ok(Self { client })
    }

    /// Create a new Binance client with custom configuration.
    ///
    /// # Arguments
    ///
    /// * `config` - Custom configuration (testnet, Binance.US, etc.)
    /// * `credentials` - Optional credentials tuple (api_key, secret_key)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use binance_api_client::{Binance, Config};
    ///
    /// # fn run() -> binance_api_client::Result<()> {
    /// // Testnet with credentials
    /// let config = Config::testnet();
    /// let client = Binance::with_config(config, Some(("api_key", "secret_key")))?;
    ///
    /// // Production without credentials
    /// let config = Config::default();
    /// let client = Binance::with_config(config, None::<(&str, &str)>)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_config<S: Into<String>>(
        config: Config,
        credentials: Option<(S, S)>,
    ) -> Result<Self> {
        let client = match credentials {
            Some((api_key, secret_key)) => {
                let creds = Credentials::new(api_key, secret_key);
                Client::new(config, creds)?
            }
            None => Client::new_unauthenticated(config)?,
        };
        Ok(Self { client })
    }

    /// Create a new Binance client from environment variables.
    ///
    /// Expects `BINANCE_API_KEY` and `BINANCE_SECRET_KEY` environment variables.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use binance_api_client::Binance;
    ///
    /// # fn run() -> binance_api_client::Result<()> {
    /// let client = Binance::from_env()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_env() -> Result<Self> {
        let config = Config::default();
        let credentials = Credentials::from_env()?;
        let client = Client::new(config, credentials)?;
        Ok(Self { client })
    }

    /// Create a new testnet Binance client with credentials.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use binance_api_client::Binance;
    ///
    /// # fn run() -> binance_api_client::Result<()> {
    /// let client = Binance::testnet("api_key", "secret_key")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn testnet(api_key: impl Into<String>, secret_key: impl Into<String>) -> Result<Self> {
        let config = Config::testnet();
        let credentials = Credentials::new(api_key, secret_key);
        let client = Client::new(config, credentials)?;
        Ok(Self { client })
    }

    /// Create a new testnet Binance client without credentials.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use binance_api_client::Binance;
    ///
    /// # fn run() -> binance_api_client::Result<()> {
    /// let client = Binance::testnet_unauthenticated()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn testnet_unauthenticated() -> Result<Self> {
        let config = Config::testnet();
        let client = Client::new_unauthenticated(config)?;
        Ok(Self { client })
    }

    /// Create a new Binance.US client with credentials.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use binance_api_client::Binance;
    ///
    /// # fn run() -> binance_api_client::Result<()> {
    /// let client = Binance::binance_us("api_key", "secret_key")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn binance_us(api_key: impl Into<String>, secret_key: impl Into<String>) -> Result<Self> {
        let config = Config::binance_us();
        let credentials = Credentials::new(api_key, secret_key);
        let client = Client::new(config, credentials)?;
        Ok(Self { client })
    }

    /// Get the underlying HTTP client.
    ///
    /// This is useful for advanced use cases where you need direct access
    /// to the client.
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Get the current configuration.
    pub fn config(&self) -> &Config {
        self.client.config()
    }

    /// Check if this client has credentials for authenticated endpoints.
    pub fn has_credentials(&self) -> bool {
        self.client.has_credentials()
    }

    /// Access market data API endpoints.
    ///
    /// Market data endpoints are public and don't require authentication.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    ///
    /// // Get current BTC price
    /// let price = client.market().price("BTCUSDT").await?;
    /// println!("BTC/USDT: {}", price.price);
    ///
    /// // Get order book
    /// let depth = client.market().depth("BTCUSDT", Some(10)).await?;
    /// ```
    pub fn market(&self) -> rest::Market {
        rest::Market::new(self.client.clone())
    }

    /// Access user data stream API endpoints.
    ///
    /// User data streams provide real-time updates for account balance changes,
    /// order updates, and other account events via WebSocket.
    ///
    /// **Requires authentication.**
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    ///
    /// // Start a user data stream
    /// let listen_key = client.user_stream().start().await?;
    ///
    /// // Keep it alive (call every 30 minutes)
    /// client.user_stream().keepalive(&listen_key).await?;
    ///
    /// // Close when done
    /// client.user_stream().close(&listen_key).await?;
    /// ```
    pub fn user_stream(&self) -> rest::UserStream {
        rest::UserStream::new(self.client.clone())
    }

    /// Access account and trading API endpoints.
    ///
    /// Account and trading endpoints require authentication. Use these to
    /// manage orders, query account balances, and view trade history.
    ///
    /// **Requires authentication.**
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    ///
    /// // Get account info
    /// let account = client.account().get_account().await?;
    ///
    /// // Place a limit buy order
    /// let order = client.account().limit_buy("BTCUSDT", "0.001", "50000.00").await?;
    ///
    /// // Or use the order builder for more control
    /// use binance_api_client::{OrderBuilder, OrderSide, OrderType, TimeInForce};
    ///
    /// let order = OrderBuilder::new("BTCUSDT", OrderSide::Buy, OrderType::Limit)
    ///     .quantity("0.001")
    ///     .price("50000.00")
    ///     .time_in_force(TimeInForce::GTC)
    ///     .build();
    ///
    /// let response = client.account().create_order(&order).await?;
    /// ```
    pub fn account(&self) -> rest::Account {
        rest::Account::new(self.client.clone())
    }

    /// Access wallet SAPI endpoints.
    ///
    /// Wallet endpoints provide access to deposit/withdrawal operations,
    /// asset management, universal transfers, and account status.
    ///
    /// **Requires authentication.**
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    ///
    /// // Check system status
    /// let status = client.wallet().system_status().await?;
    /// if status.is_normal() {
    ///     println!("System is operational");
    /// }
    ///
    /// // Get all coin information
    /// let coins = client.wallet().all_coins().await?;
    ///
    /// // Get deposit address
    /// let address = client.wallet().deposit_address("BTC", None).await?;
    /// println!("Deposit BTC to: {}", address.address);
    ///
    /// // Get trade fees
    /// let fees = client.wallet().trade_fee(Some("BTCUSDT")).await?;
    /// ```
    pub fn wallet(&self) -> rest::Wallet {
        rest::Wallet::new(self.client.clone())
    }

    /// Access margin trading SAPI endpoints.
    ///
    /// Margin endpoints provide access to cross-margin and isolated margin
    /// trading, loans, transfers, and account management.
    ///
    /// **Requires authentication.**
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    ///
    /// // Get cross-margin account details
    /// let account = client.margin().account().await?;
    /// println!("Margin level: {}", account.margin_level);
    ///
    /// // Check max borrowable
    /// let max = client.margin().max_borrowable("BTC", None).await?;
    /// println!("Max borrowable BTC: {}", max.amount);
    ///
    /// // Transfer to margin account
    /// use binance_api_client::MarginTransferType;
    /// let result = client.margin()
    ///     .transfer("USDT", "100.0", MarginTransferType::MainToMargin)
    ///     .await?;
    ///
    /// // Borrow
    /// let loan = client.margin().loan("USDT", "50.0", false, None).await?;
    /// ```
    pub fn margin(&self) -> rest::Margin {
        rest::Margin::new(self.client.clone())
    }

    /// Access WebSocket streaming API.
    ///
    /// The WebSocket client provides real-time market data streams including
    /// trades, klines, tickers, and order book updates. It also supports
    /// user data streams for account updates when connected with a listen key.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::Binance;
    ///
    /// let client = Binance::new_unauthenticated()?;
    /// let ws = client.websocket();
    ///
    /// // Connect to aggregate trade stream
    /// let stream_name = ws.agg_trade_stream("btcusdt");
    /// let mut conn = ws.connect(&stream_name).await?;
    ///
    /// // Receive events
    /// while let Some(event) = conn.next().await {
    ///     println!("{:?}", event?);
    /// }
    /// ```
    pub fn websocket(&self) -> ws::WebSocketClient {
        ws::WebSocketClient::new(self.client.config().clone())
    }
}

impl std::fmt::Debug for Binance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Binance")
            .field("config", self.config())
            .field("has_credentials", &self.has_credentials())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_unauthenticated() {
        let client = Binance::new_unauthenticated().unwrap();
        assert!(!client.has_credentials());
        assert_eq!(client.config().rest_api_endpoint, "https://api.binance.com");
    }

    #[test]
    fn test_new_authenticated() {
        let client = Binance::new("api_key", "secret_key").unwrap();
        assert!(client.has_credentials());
    }

    #[test]
    fn test_testnet() {
        let client = Binance::testnet("api_key", "secret_key").unwrap();
        assert!(client.has_credentials());
        assert_eq!(
            client.config().rest_api_endpoint,
            "https://testnet.binance.vision"
        );
    }

    #[test]
    fn test_testnet_unauthenticated() {
        let client = Binance::testnet_unauthenticated().unwrap();
        assert!(!client.has_credentials());
        assert_eq!(
            client.config().rest_api_endpoint,
            "https://testnet.binance.vision"
        );
    }

    #[test]
    fn test_binance_us() {
        let client = Binance::binance_us("api_key", "secret_key").unwrap();
        assert!(client.has_credentials());
        assert_eq!(client.config().rest_api_endpoint, "https://api.binance.us");
    }

    #[test]
    fn test_with_config() {
        let config = Config::builder()
            .rest_api_endpoint("https://custom.api.com")
            .build();
        let client = Binance::with_config(config, Some(("api_key", "secret_key"))).unwrap();
        assert!(client.has_credentials());
        assert_eq!(client.config().rest_api_endpoint, "https://custom.api.com");
    }

    #[test]
    fn test_with_config_no_credentials() {
        let config = Config::default();
        let client = Binance::with_config(config, None::<(&str, &str)>).unwrap();
        assert!(!client.has_credentials());
    }

    #[test]
    fn test_debug_output() {
        let client = Binance::new("api_key", "secret_key").unwrap();
        let debug_output = format!("{:?}", client);
        assert!(debug_output.contains("Binance"));
        assert!(debug_output.contains("has_credentials: true"));
        // Should not contain the actual secret key
        assert!(!debug_output.contains("secret_key"));
    }
}
