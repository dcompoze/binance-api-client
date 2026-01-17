//! Margin Trading API endpoints (SAPI).
//!
//! This module provides access to Binance Margin SAPI endpoints for:
//! - Cross-margin and isolated margin account management
//! - Margin transfers, loans, and repayments
//! - Margin trading (orders)
//! - Interest and loan history

use crate::client::Client;
use crate::error::Result;
use crate::models::margin::{
    BnbBurnStatus, InterestHistoryRecord, InterestRateRecord, IsolatedAccountLimit,
    IsolatedMarginAccountDetails, IsolatedMarginTransferType, LoanRecord, MarginAccountDetails,
    MarginAssetInfo, MarginOrderCancellation, MarginOrderResult, MarginOrderState,
    MarginPairDetails, MarginPriceIndex, MarginTrade, MarginTransferType, MaxBorrowableAmount,
    MaxTransferableAmount, RecordsQueryResult, RepayRecord, SideEffectType, TransactionId,
};
use crate::types::{OrderSide, OrderType, TimeInForce};

// SAPI endpoints.
const SAPI_V1_MARGIN_TRANSFER: &str = "/sapi/v1/margin/transfer";
const SAPI_V1_MARGIN_ISOLATED_TRANSFER: &str = "/sapi/v1/margin/isolated/transfer";
const SAPI_V1_MARGIN_LOAN: &str = "/sapi/v1/margin/loan";
const SAPI_V1_MARGIN_REPAY: &str = "/sapi/v1/margin/repay";
const SAPI_V1_MARGIN_ACCOUNT: &str = "/sapi/v1/margin/account";
const SAPI_V1_MARGIN_ISOLATED_ACCOUNT: &str = "/sapi/v1/margin/isolated/account";
const SAPI_V1_MARGIN_ORDER: &str = "/sapi/v1/margin/order";
const SAPI_V1_MARGIN_OPEN_ORDERS: &str = "/sapi/v1/margin/openOrders";
const SAPI_V1_MARGIN_ALL_ORDERS: &str = "/sapi/v1/margin/allOrders";
const SAPI_V1_MARGIN_MY_TRADES: &str = "/sapi/v1/margin/myTrades";
const SAPI_V1_MARGIN_MAX_BORROWABLE: &str = "/sapi/v1/margin/maxBorrowable";
const SAPI_V1_MARGIN_MAX_TRANSFERABLE: &str = "/sapi/v1/margin/maxTransferable";
const SAPI_V1_MARGIN_INTEREST_HISTORY: &str = "/sapi/v1/margin/interestHistory";
const SAPI_V1_MARGIN_INTEREST_RATE_HISTORY: &str = "/sapi/v1/margin/interestRateHistory";
const SAPI_V1_MARGIN_PAIR: &str = "/sapi/v1/margin/pair";
const SAPI_V1_MARGIN_ALL_PAIRS: &str = "/sapi/v1/margin/allPairs";
const SAPI_V1_MARGIN_ASSET: &str = "/sapi/v1/margin/asset";
const SAPI_V1_MARGIN_ALL_ASSETS: &str = "/sapi/v1/margin/allAssets";
const SAPI_V1_MARGIN_PRICE_INDEX: &str = "/sapi/v1/margin/priceIndex";
const SAPI_V1_MARGIN_ISOLATED_ACCOUNT_LIMIT: &str = "/sapi/v1/margin/isolated/accountLimit";
const SAPI_V1_BNB_BURN: &str = "/sapi/v1/bnbBurn";

/// Margin Trading API client.
///
/// Provides access to Binance Margin SAPI endpoints for margin trading,
/// loans, transfers, and account management.
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
/// ```
#[derive(Clone)]
pub struct Margin {
    pub(crate) client: Client,
}

impl Margin {
    /// Create a new Margin API client.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    // Account Management.

    /// Get cross-margin account details.
    ///
    /// Returns margin account information including balances, margin level,
    /// and trading status.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let account = client.margin().account().await?;
    /// println!("Margin level: {}", account.margin_level);
    /// println!("Total assets (BTC): {}", account.total_asset_of_btc);
    /// for asset in account.user_assets {
    ///     if asset.net_asset > 0.0 {
    ///         println!("{}: free={}, borrowed={}", asset.asset, asset.free, asset.borrowed);
    ///     }
    /// }
    /// ```
    pub async fn account(&self) -> Result<MarginAccountDetails> {
        self.client.get_signed(SAPI_V1_MARGIN_ACCOUNT, &[]).await
    }

    /// Get isolated margin account details.
    ///
    /// # Arguments
    ///
    /// * `symbols` - Optional list of symbols to filter (comma-separated if multiple)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let account = client.margin().isolated_account(Some("BTCUSDT")).await?;
    /// for asset in account.assets {
    ///     println!("{}: margin_level={}", asset.symbol, asset.margin_level);
    /// }
    /// ```
    pub async fn isolated_account(
        &self,
        symbols: Option<&str>,
    ) -> Result<IsolatedMarginAccountDetails> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(s) = symbols {
            params.push(("symbols", s.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_ISOLATED_ACCOUNT, &params_ref)
            .await
    }

    /// Get max borrowable amount for an asset.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to query
    /// * `isolated_symbol` - Isolated margin symbol (for isolated margin)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let max = client.margin().max_borrowable("BTC", None).await?;
    /// println!("Max borrowable: {} BTC", max.amount);
    /// ```
    pub async fn max_borrowable(
        &self,
        asset: &str,
        isolated_symbol: Option<&str>,
    ) -> Result<MaxBorrowableAmount> {
        let mut params: Vec<(&str, String)> = vec![("asset", asset.to_string())];

        if let Some(s) = isolated_symbol {
            params.push(("isolatedSymbol", s.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_MAX_BORROWABLE, &params_ref)
            .await
    }

    /// Get max transferable amount for an asset.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to query
    /// * `isolated_symbol` - Isolated margin symbol (for isolated margin)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let max = client.margin().max_transferable("USDT", None).await?;
    /// println!("Max transferable: {} USDT", max.amount);
    /// ```
    pub async fn max_transferable(
        &self,
        asset: &str,
        isolated_symbol: Option<&str>,
    ) -> Result<MaxTransferableAmount> {
        let mut params: Vec<(&str, String)> = vec![("asset", asset.to_string())];

        if let Some(s) = isolated_symbol {
            params.push(("isolatedSymbol", s.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_MAX_TRANSFERABLE, &params_ref)
            .await
    }

    /// Get isolated margin account limit.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let limit = client.margin().isolated_account_limit().await?;
    /// println!("Enabled: {}, Max: {}", limit.enabled_account, limit.max_account);
    /// ```
    pub async fn isolated_account_limit(&self) -> Result<IsolatedAccountLimit> {
        self.client
            .get_signed(SAPI_V1_MARGIN_ISOLATED_ACCOUNT_LIMIT, &[])
            .await
    }

    // Transfer.

    /// Execute a cross-margin transfer between spot and margin accounts.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to transfer
    /// * `amount` - Amount to transfer
    /// * `transfer_type` - Direction of transfer
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::MarginTransferType;
    ///
    /// // Transfer 100 USDT from spot to margin
    /// let result = client.margin()
    ///     .transfer("USDT", "100.0", MarginTransferType::MainToMargin)
    ///     .await?;
    /// println!("Transfer ID: {}", result.tran_id);
    /// ```
    pub async fn transfer(
        &self,
        asset: &str,
        amount: &str,
        transfer_type: MarginTransferType,
    ) -> Result<TransactionId> {
        let type_val = match transfer_type {
            MarginTransferType::MainToMargin => "1",
            MarginTransferType::MarginToMain => "2",
        };

        let params: Vec<(&str, String)> = vec![
            ("asset", asset.to_string()),
            ("amount", amount.to_string()),
            ("type", type_val.to_string()),
        ];

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .post_signed(SAPI_V1_MARGIN_TRANSFER, &params_ref)
            .await
    }

    /// Execute an isolated margin transfer.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to transfer
    /// * `symbol` - Isolated margin symbol
    /// * `amount` - Amount to transfer
    /// * `trans_from` - Source account type
    /// * `trans_to` - Destination account type
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::IsolatedMarginTransferType;
    ///
    /// // Transfer 100 USDT from spot to BTCUSDT isolated margin
    /// let result = client.margin()
    ///     .isolated_transfer(
    ///         "USDT",
    ///         "BTCUSDT",
    ///         "100.0",
    ///         IsolatedMarginTransferType::Spot,
    ///         IsolatedMarginTransferType::IsolatedMargin,
    ///     )
    ///     .await?;
    /// ```
    pub async fn isolated_transfer(
        &self,
        asset: &str,
        symbol: &str,
        amount: &str,
        trans_from: IsolatedMarginTransferType,
        trans_to: IsolatedMarginTransferType,
    ) -> Result<TransactionId> {
        let from_str = match trans_from {
            IsolatedMarginTransferType::Spot => "SPOT",
            IsolatedMarginTransferType::IsolatedMargin => "ISOLATED_MARGIN",
        };
        let to_str = match trans_to {
            IsolatedMarginTransferType::Spot => "SPOT",
            IsolatedMarginTransferType::IsolatedMargin => "ISOLATED_MARGIN",
        };

        let params: Vec<(&str, String)> = vec![
            ("asset", asset.to_string()),
            ("symbol", symbol.to_string()),
            ("amount", amount.to_string()),
            ("transFrom", from_str.to_string()),
            ("transTo", to_str.to_string()),
        ];

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .post_signed(SAPI_V1_MARGIN_ISOLATED_TRANSFER, &params_ref)
            .await
    }

    // Borrow/Repay.

    /// Apply for a margin loan.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to borrow
    /// * `amount` - Amount to borrow
    /// * `is_isolated` - Whether this is isolated margin
    /// * `symbol` - Symbol for isolated margin (required if is_isolated is true)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Borrow 0.1 BTC on cross margin
    /// let result = client.margin().loan("BTC", "0.1", false, None).await?;
    /// println!("Loan transaction ID: {}", result.tran_id);
    /// ```
    pub async fn loan(
        &self,
        asset: &str,
        amount: &str,
        is_isolated: bool,
        symbol: Option<&str>,
    ) -> Result<TransactionId> {
        let mut params: Vec<(&str, String)> =
            vec![("asset", asset.to_string()), ("amount", amount.to_string())];

        if is_isolated {
            params.push(("isIsolated", "TRUE".to_string()));
            if let Some(s) = symbol {
                params.push(("symbol", s.to_string()));
            }
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .post_signed(SAPI_V1_MARGIN_LOAN, &params_ref)
            .await
    }

    /// Repay a margin loan.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to repay
    /// * `amount` - Amount to repay
    /// * `is_isolated` - Whether this is isolated margin
    /// * `symbol` - Symbol for isolated margin (required if is_isolated is true)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// // Repay 0.1 BTC on cross margin
    /// let result = client.margin().repay("BTC", "0.1", false, None).await?;
    /// println!("Repay transaction ID: {}", result.tran_id);
    /// ```
    pub async fn repay(
        &self,
        asset: &str,
        amount: &str,
        is_isolated: bool,
        symbol: Option<&str>,
    ) -> Result<TransactionId> {
        let mut params: Vec<(&str, String)> =
            vec![("asset", asset.to_string()), ("amount", amount.to_string())];

        if is_isolated {
            params.push(("isIsolated", "TRUE".to_string()));
            if let Some(s) = symbol {
                params.push(("symbol", s.to_string()));
            }
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .post_signed(SAPI_V1_MARGIN_REPAY, &params_ref)
            .await
    }

    /// Get loan records.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to query
    /// * `isolated_symbol` - Isolated margin symbol (optional)
    /// * `start_time` - Start timestamp (optional)
    /// * `end_time` - End timestamp (optional)
    /// * `current` - Page number (default 1)
    /// * `size` - Page size (default 10, max 100)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let records = client.margin()
    ///     .loan_records("BTC", None, None, None, None, Some(20))
    ///     .await?;
    /// for record in records.rows {
    ///     println!("Loan: {} {} at {}", record.principal, record.asset, record.timestamp);
    /// }
    /// ```
    pub async fn loan_records(
        &self,
        asset: &str,
        isolated_symbol: Option<&str>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        current: Option<u32>,
        size: Option<u32>,
    ) -> Result<RecordsQueryResult<LoanRecord>> {
        let mut params: Vec<(&str, String)> = vec![("asset", asset.to_string())];

        if let Some(s) = isolated_symbol {
            params.push(("isolatedSymbol", s.to_string()));
        }
        if let Some(st) = start_time {
            params.push(("startTime", st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime", et.to_string()));
        }
        if let Some(c) = current {
            params.push(("current", c.to_string()));
        }
        if let Some(s) = size {
            params.push(("size", s.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_LOAN, &params_ref)
            .await
    }

    /// Get repay records.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to query
    /// * `isolated_symbol` - Isolated margin symbol (optional)
    /// * `start_time` - Start timestamp (optional)
    /// * `end_time` - End timestamp (optional)
    /// * `current` - Page number (default 1)
    /// * `size` - Page size (default 10, max 100)
    pub async fn repay_records(
        &self,
        asset: &str,
        isolated_symbol: Option<&str>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        current: Option<u32>,
        size: Option<u32>,
    ) -> Result<RecordsQueryResult<RepayRecord>> {
        let mut params: Vec<(&str, String)> = vec![("asset", asset.to_string())];

        if let Some(s) = isolated_symbol {
            params.push(("isolatedSymbol", s.to_string()));
        }
        if let Some(st) = start_time {
            params.push(("startTime", st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime", et.to_string()));
        }
        if let Some(c) = current {
            params.push(("current", c.to_string()));
        }
        if let Some(s) = size {
            params.push(("size", s.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_REPAY, &params_ref)
            .await
    }

    // Trading.

    /// Create a new margin order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `side` - Order side (Buy/Sell)
    /// * `order_type` - Order type
    /// * `quantity` - Order quantity (optional for some order types)
    /// * `quote_order_qty` - Quote order quantity (optional)
    /// * `price` - Price (required for limit orders)
    /// * `stop_price` - Stop price (for stop orders)
    /// * `time_in_force` - Time in force (optional)
    /// * `new_client_order_id` - Client order ID (optional)
    /// * `side_effect_type` - Side effect type (optional)
    /// * `is_isolated` - Whether isolated margin (optional)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::{OrderSide, OrderType, TimeInForce, SideEffectType};
    ///
    /// let order = client.margin().create_order(
    ///     "BTCUSDT",
    ///     OrderSide::Buy,
    ///     OrderType::Limit,
    ///     Some("0.001"),
    ///     None,
    ///     Some("50000.00"),
    ///     None,
    ///     Some(TimeInForce::GTC),
    ///     None,
    ///     Some(SideEffectType::MarginBuy),
    ///     None,
    /// ).await?;
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub async fn create_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        quantity: Option<&str>,
        quote_order_qty: Option<&str>,
        price: Option<&str>,
        stop_price: Option<&str>,
        time_in_force: Option<TimeInForce>,
        new_client_order_id: Option<&str>,
        side_effect_type: Option<SideEffectType>,
        is_isolated: Option<bool>,
    ) -> Result<MarginOrderResult> {
        let mut params: Vec<(&str, String)> = vec![
            ("symbol", symbol.to_string()),
            ("side", format!("{:?}", side).to_uppercase()),
            ("type", format!("{:?}", order_type).to_uppercase()),
        ];

        if let Some(qty) = quantity {
            params.push(("quantity", qty.to_string()));
        }
        if let Some(qty) = quote_order_qty {
            params.push(("quoteOrderQty", qty.to_string()));
        }
        if let Some(p) = price {
            params.push(("price", p.to_string()));
        }
        if let Some(sp) = stop_price {
            params.push(("stopPrice", sp.to_string()));
        }
        if let Some(tif) = time_in_force {
            params.push(("timeInForce", format!("{:?}", tif).to_uppercase()));
        }
        if let Some(id) = new_client_order_id {
            params.push(("newClientOrderId", id.to_string()));
        }
        if let Some(se) = side_effect_type {
            params.push((
                "sideEffectType",
                match se {
                    SideEffectType::NoSideEffect => "NO_SIDE_EFFECT",
                    SideEffectType::MarginBuy => "MARGIN_BUY",
                    SideEffectType::AutoRepay => "AUTO_REPAY",
                }
                .to_string(),
            ));
        }
        if let Some(isolated) = is_isolated {
            params.push((
                "isIsolated",
                if isolated { "TRUE" } else { "FALSE" }.to_string(),
            ));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .post_signed(SAPI_V1_MARGIN_ORDER, &params_ref)
            .await
    }

    /// Cancel a margin order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `order_id` - Order ID (optional if orig_client_order_id provided)
    /// * `orig_client_order_id` - Original client order ID (optional if order_id provided)
    /// * `is_isolated` - Whether isolated margin
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = client.margin()
    ///     .cancel_order("BTCUSDT", Some(12345), None, None)
    ///     .await?;
    /// ```
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
        is_isolated: Option<bool>,
    ) -> Result<MarginOrderCancellation> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(id) = order_id {
            params.push(("orderId", id.to_string()));
        }
        if let Some(cid) = orig_client_order_id {
            params.push(("origClientOrderId", cid.to_string()));
        }
        if let Some(isolated) = is_isolated {
            params.push((
                "isIsolated",
                if isolated { "TRUE" } else { "FALSE" }.to_string(),
            ));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .delete_signed(SAPI_V1_MARGIN_ORDER, &params_ref)
            .await
    }

    /// Cancel all open orders on a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `is_isolated` - Whether isolated margin
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let results = client.margin()
    ///     .cancel_all_orders("BTCUSDT", None)
    ///     .await?;
    /// ```
    pub async fn cancel_all_orders(
        &self,
        symbol: &str,
        is_isolated: Option<bool>,
    ) -> Result<Vec<MarginOrderCancellation>> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(isolated) = is_isolated {
            params.push((
                "isIsolated",
                if isolated { "TRUE" } else { "FALSE" }.to_string(),
            ));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .delete_signed(SAPI_V1_MARGIN_OPEN_ORDERS, &params_ref)
            .await
    }

    /// Query a margin order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `order_id` - Order ID (optional)
    /// * `orig_client_order_id` - Original client order ID (optional)
    /// * `is_isolated` - Whether isolated margin
    pub async fn get_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
        is_isolated: Option<bool>,
    ) -> Result<MarginOrderState> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(id) = order_id {
            params.push(("orderId", id.to_string()));
        }
        if let Some(cid) = orig_client_order_id {
            params.push(("origClientOrderId", cid.to_string()));
        }
        if let Some(isolated) = is_isolated {
            params.push((
                "isIsolated",
                if isolated { "TRUE" } else { "FALSE" }.to_string(),
            ));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_ORDER, &params_ref)
            .await
    }

    /// Get all open margin orders.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (optional)
    /// * `is_isolated` - Whether isolated margin
    pub async fn open_orders(
        &self,
        symbol: Option<&str>,
        is_isolated: Option<bool>,
    ) -> Result<Vec<MarginOrderState>> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(s) = symbol {
            params.push(("symbol", s.to_string()));
        }
        if let Some(isolated) = is_isolated {
            params.push((
                "isIsolated",
                if isolated { "TRUE" } else { "FALSE" }.to_string(),
            ));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_OPEN_ORDERS, &params_ref)
            .await
    }

    /// Get all margin orders (active, cancelled, or filled).
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `order_id` - Order ID to start from (optional)
    /// * `start_time` - Start timestamp (optional)
    /// * `end_time` - End timestamp (optional)
    /// * `limit` - Number of records (default 500, max 500)
    /// * `is_isolated` - Whether isolated margin
    pub async fn all_orders(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
        is_isolated: Option<bool>,
    ) -> Result<Vec<MarginOrderState>> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(id) = order_id {
            params.push(("orderId", id.to_string()));
        }
        if let Some(st) = start_time {
            params.push(("startTime", st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime", et.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }
        if let Some(isolated) = is_isolated {
            params.push((
                "isIsolated",
                if isolated { "TRUE" } else { "FALSE" }.to_string(),
            ));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_ALL_ORDERS, &params_ref)
            .await
    }

    /// Get margin trades.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `order_id` - Filter by order ID (optional)
    /// * `start_time` - Start timestamp (optional)
    /// * `end_time` - End timestamp (optional)
    /// * `from_id` - Trade ID to start from (optional)
    /// * `limit` - Number of records (default 500, max 1000)
    /// * `is_isolated` - Whether isolated margin
    #[allow(clippy::too_many_arguments)]
    pub async fn my_trades(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        from_id: Option<u64>,
        limit: Option<u32>,
        is_isolated: Option<bool>,
    ) -> Result<Vec<MarginTrade>> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(id) = order_id {
            params.push(("orderId", id.to_string()));
        }
        if let Some(st) = start_time {
            params.push(("startTime", st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime", et.to_string()));
        }
        if let Some(id) = from_id {
            params.push(("fromId", id.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }
        if let Some(isolated) = is_isolated {
            params.push((
                "isIsolated",
                if isolated { "TRUE" } else { "FALSE" }.to_string(),
            ));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_MY_TRADES, &params_ref)
            .await
    }

    // Interest.

    /// Get interest history.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to query (optional)
    /// * `isolated_symbol` - Isolated margin symbol (optional)
    /// * `start_time` - Start timestamp (optional)
    /// * `end_time` - End timestamp (optional)
    /// * `current` - Page number (default 1)
    /// * `size` - Page size (default 10, max 100)
    pub async fn interest_history(
        &self,
        asset: Option<&str>,
        isolated_symbol: Option<&str>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        current: Option<u32>,
        size: Option<u32>,
    ) -> Result<RecordsQueryResult<InterestHistoryRecord>> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(a) = asset {
            params.push(("asset", a.to_string()));
        }
        if let Some(s) = isolated_symbol {
            params.push(("isolatedSymbol", s.to_string()));
        }
        if let Some(st) = start_time {
            params.push(("startTime", st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime", et.to_string()));
        }
        if let Some(c) = current {
            params.push(("current", c.to_string()));
        }
        if let Some(s) = size {
            params.push(("size", s.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_INTEREST_HISTORY, &params_ref)
            .await
    }

    /// Get interest rate history.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset to query
    /// * `vip_level` - VIP level (optional, default uses user's vip level)
    /// * `start_time` - Start timestamp (optional)
    /// * `end_time` - End timestamp (optional)
    /// * `limit` - Number of records (default 20, max 100)
    pub async fn interest_rate_history(
        &self,
        asset: &str,
        vip_level: Option<u32>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<InterestRateRecord>> {
        let mut params: Vec<(&str, String)> = vec![("asset", asset.to_string())];

        if let Some(vip) = vip_level {
            params.push(("vipLevel", vip.to_string()));
        }
        if let Some(st) = start_time {
            params.push(("startTime", st.to_string()));
        }
        if let Some(et) = end_time {
            params.push(("endTime", et.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_INTEREST_RATE_HISTORY, &params_ref)
            .await
    }

    // Market Data.

    /// Get cross margin pair details.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    pub async fn pair(&self, symbol: &str) -> Result<MarginPairDetails> {
        let params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_PAIR, &params_ref)
            .await
    }

    /// Get all cross margin pairs.
    pub async fn all_pairs(&self) -> Result<Vec<MarginPairDetails>> {
        self.client.get_signed(SAPI_V1_MARGIN_ALL_PAIRS, &[]).await
    }

    /// Get margin asset info.
    ///
    /// # Arguments
    ///
    /// * `asset` - Asset symbol
    pub async fn asset(&self, asset: &str) -> Result<MarginAssetInfo> {
        let params: Vec<(&str, String)> = vec![("asset", asset.to_string())];
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_ASSET, &params_ref)
            .await
    }

    /// Get all margin assets info.
    pub async fn all_assets(&self) -> Result<Vec<MarginAssetInfo>> {
        self.client.get_signed(SAPI_V1_MARGIN_ALL_ASSETS, &[]).await
    }

    /// Get margin price index for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    pub async fn price_index(&self, symbol: &str) -> Result<MarginPriceIndex> {
        let params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(SAPI_V1_MARGIN_PRICE_INDEX, &params_ref)
            .await
    }

    // BNB Burn.

    /// Get BNB burn status for spot trading and margin interest.
    pub async fn bnb_burn_status(&self) -> Result<BnbBurnStatus> {
        self.client.get_signed(SAPI_V1_BNB_BURN, &[]).await
    }

    /// Toggle BNB burn on spot trade and margin interest.
    ///
    /// # Arguments
    ///
    /// * `spot_bnb_burn` - Enable BNB for spot trading fees (optional)
    /// * `interest_bnb_burn` - Enable BNB for margin interest (optional)
    pub async fn toggle_bnb_burn(
        &self,
        spot_bnb_burn: Option<bool>,
        interest_bnb_burn: Option<bool>,
    ) -> Result<BnbBurnStatus> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(spot) = spot_bnb_burn {
            params.push(("spotBNBBurn", spot.to_string()));
        }
        if let Some(interest) = interest_bnb_burn {
            params.push(("interestBNBBurn", interest.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client.post_signed(SAPI_V1_BNB_BURN, &params_ref).await
    }
}
