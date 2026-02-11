//! Account and trading API endpoints.
//!
//! This module provides authenticated endpoints for account information,
//! order management, and trading.

use serde::Serialize;

use crate::client::Client;
use reqwest::StatusCode;

use crate::Result;
use crate::error::{BinanceApiError, Error};
use crate::models::{
    AccountCommission, AccountInfo, Allocation, AmendOrderResponse, CancelOrderResponse,
    CancelReplaceErrorResponse, CancelReplaceResponse, OcoOrder, Order, OrderAmendment, OrderFull,
    PreventedMatch, SorOrderTestResponse, UnfilledOrderCount, UserTrade,
};
use crate::types::{
    CancelReplaceMode, CancelRestrictions, OrderRateLimitExceededMode, OrderResponseType,
    OrderSide, OrderType, TimeInForce,
};

// API endpoints.
const API_V3_ACCOUNT: &str = "/api/v3/account";
const API_V3_MY_TRADES: &str = "/api/v3/myTrades";
const API_V3_ORDER: &str = "/api/v3/order";
const API_V3_ORDER_TEST: &str = "/api/v3/order/test";
const API_V3_OPEN_ORDERS: &str = "/api/v3/openOrders";
const API_V3_ALL_ORDERS: &str = "/api/v3/allOrders";
const API_V3_ORDER_OCO: &str = "/api/v3/order/oco";
const API_V3_ORDER_LIST_OTO: &str = "/api/v3/orderList/oto";
const API_V3_ORDER_LIST_OTOCO: &str = "/api/v3/orderList/otoco";
const API_V3_ORDER_LIST_OPO: &str = "/api/v3/orderList/opo";
const API_V3_ORDER_LIST_OPOCO: &str = "/api/v3/orderList/opoco";
const API_V3_ORDER_LIST: &str = "/api/v3/orderList";
const API_V3_ALL_ORDER_LIST: &str = "/api/v3/allOrderList";
const API_V3_OPEN_ORDER_LIST: &str = "/api/v3/openOrderList";
const API_V3_MY_PREVENTED_MATCHES: &str = "/api/v3/myPreventedMatches";
const API_V3_MY_ALLOCATIONS: &str = "/api/v3/myAllocations";
const API_V3_ACCOUNT_COMMISSION: &str = "/api/v3/account/commission";
const API_V3_ORDER_CANCEL_REPLACE: &str = "/api/v3/order/cancelReplace";
const API_V3_SOR_ORDER: &str = "/api/v3/sor/order";
const API_V3_SOR_ORDER_TEST: &str = "/api/v3/sor/order/test";
const API_V3_RATE_LIMIT_ORDER: &str = "/api/v3/rateLimit/order";
const API_V3_ORDER_AMEND: &str = "/api/v3/order/amend/keepPriority";
const API_V3_ORDER_AMENDMENTS: &str = "/api/v3/order/amendments";

/// Account and trading API client.
///
/// Provides authenticated endpoints for account information and trading.
/// All methods require authentication.
#[derive(Clone)]
pub struct Account {
    client: Client,
}

impl Account {
    /// Create a new Account API client.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    // Account Endpoints.

    /// Get current account information including balances.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let account = client.account().get_account().await?;
    ///
    /// for balance in account.balances {
    ///     if balance.free > 0.0 || balance.locked > 0.0 {
    ///         println!("{}: free={}, locked={}", balance.asset, balance.free, balance.locked);
    ///     }
    /// }
    /// ```
    pub async fn get_account(&self) -> Result<AccountInfo> {
        self.client.get_signed(API_V3_ACCOUNT, &[]).await
    }

    /// Get account trade history for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `from_id` - Trade ID to fetch from
    /// * `start_time` - Start time in milliseconds
    /// * `end_time` - End time in milliseconds
    /// * `limit` - Max number of trades (default 500, max 1000)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let trades = client.account().my_trades("BTCUSDT", None, None, None, Some(10)).await?;
    /// ```
    pub async fn my_trades(
        &self,
        symbol: &str,
        from_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<UserTrade>> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(id) = from_id {
            params.push(("fromId", id.to_string()));
        }
        if let Some(start) = start_time {
            params.push(("startTime", start.to_string()));
        }
        if let Some(end) = end_time {
            params.push(("endTime", end.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client.get_signed(API_V3_MY_TRADES, &params_ref).await
    }

    /// Get orders that were expired due to self-trade prevention.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `prevented_match_id` - Prevented match ID
    /// * `order_id` - Order ID
    /// * `from_prevented_match_id` - Start from prevented match ID
    /// * `limit` - Max number of entries (default 500, max 1000)
    pub async fn my_prevented_matches(
        &self,
        symbol: &str,
        prevented_match_id: Option<u64>,
        order_id: Option<u64>,
        from_prevented_match_id: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<PreventedMatch>> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(id) = prevented_match_id {
            params.push(("preventedMatchId", id.to_string()));
        }
        if let Some(id) = order_id {
            params.push(("orderId", id.to_string()));
        }
        if let Some(id) = from_prevented_match_id {
            params.push(("fromPreventedMatchId", id.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(API_V3_MY_PREVENTED_MATCHES, &params_ref)
            .await
    }

    /// Get SOR allocations for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `start_time` - Start time in milliseconds
    /// * `end_time` - End time in milliseconds
    /// * `from_allocation_id` - Allocation ID to fetch from
    /// * `limit` - Max number of entries (default 500, max 1000)
    /// * `order_id` - Optional order ID to filter
    pub async fn my_allocations(
        &self,
        symbol: &str,
        start_time: Option<u64>,
        end_time: Option<u64>,
        from_allocation_id: Option<u64>,
        limit: Option<u32>,
        order_id: Option<u64>,
    ) -> Result<Vec<Allocation>> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(start) = start_time {
            params.push(("startTime", start.to_string()));
        }
        if let Some(end) = end_time {
            params.push(("endTime", end.to_string()));
        }
        if let Some(id) = from_allocation_id {
            params.push(("fromAllocationId", id.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }
        if let Some(id) = order_id {
            params.push(("orderId", id.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(API_V3_MY_ALLOCATIONS, &params_ref)
            .await
    }

    /// Get commission rates for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    pub async fn commission_rates(&self, symbol: &str) -> Result<AccountCommission> {
        let params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];
        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(API_V3_ACCOUNT_COMMISSION, &params_ref)
            .await
    }

    /// Query unfilled order count for all rate limit intervals.
    ///
    /// Returns the current count of unfilled orders for each rate limit interval
    /// (e.g., per second, per day). This is useful for monitoring order placement
    /// rate limits.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let counts = client.account().unfilled_order_count().await?;
    /// for count in counts {
    ///     println!("{}: {}/{} orders", count.interval, count.count, count.limit);
    /// }
    /// ```
    pub async fn unfilled_order_count(&self) -> Result<Vec<UnfilledOrderCount>> {
        self.client.get_signed(API_V3_RATE_LIMIT_ORDER, &[]).await
    }

    /// Query amendment history for a specific order.
    ///
    /// Returns all amendments made to a single order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `order_id` - Order ID to query amendments for
    /// * `from_execution_id` - Optional execution ID to start from
    /// * `limit` - Max number of entries (default 500, max 1000)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let amendments = client.account().order_amendments("BTCUSDT", 12345, None, None).await?;
    /// for amendment in amendments {
    ///     println!("Qty changed from {} to {}", amendment.orig_qty, amendment.new_qty);
    /// }
    /// ```
    pub async fn order_amendments(
        &self,
        symbol: &str,
        order_id: u64,
        from_execution_id: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<OrderAmendment>> {
        let mut params: Vec<(&str, String)> = vec![
            ("symbol", symbol.to_string()),
            ("orderId", order_id.to_string()),
        ];

        if let Some(from_id) = from_execution_id {
            params.push(("fromExecutionId", from_id.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(API_V3_ORDER_AMENDMENTS, &params_ref)
            .await
    }

    // Order Endpoints.

    /// Create a new order.
    ///
    /// Use `OrderBuilder` to construct orders with the desired parameters.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::{OrderBuilder, OrderSide, OrderType, TimeInForce};
    ///
    /// let client = Binance::new("api_key", "secret_key")?;
    ///
    /// // Limit buy order
    /// let order = OrderBuilder::new("BTCUSDT", OrderSide::Buy, OrderType::Limit)
    ///     .quantity("0.001")
    ///     .price("50000.00")
    ///     .time_in_force(TimeInForce::GTC)
    ///     .build();
    ///
    /// let response = client.account().create_order(&order).await?;
    /// ```
    pub async fn create_order(&self, order: &NewOrder) -> Result<OrderFull> {
        let params = order.to_params();
        let params_ref: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.client.post_signed(API_V3_ORDER, &params_ref).await
    }

    /// Test a new order without executing it.
    ///
    /// Validates order parameters but doesn't place the order.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let order = OrderBuilder::new("BTCUSDT", OrderSide::Buy, OrderType::Market)
    ///     .quantity("0.001")
    ///     .build();
    ///
    /// client.account().test_order(&order).await?;
    /// println!("Order parameters are valid");
    /// ```
    pub async fn test_order(&self, order: &NewOrder) -> Result<()> {
        let params = order.to_params();
        let params_ref: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let _: serde_json::Value = self
            .client
            .post_signed(API_V3_ORDER_TEST, &params_ref)
            .await?;
        Ok(())
    }

    /// Amend an order's quantity while keeping queue priority.
    ///
    /// This endpoint allows reducing the quantity of an existing open order
    /// without losing its place in the order queue. The new quantity must be
    /// greater than 0 and less than the current order quantity.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `order_id` - Order ID to amend (either order_id or orig_client_order_id required)
    /// * `orig_client_order_id` - Client order ID to amend
    /// * `new_qty` - New quantity (must be less than current quantity)
    /// * `new_client_order_id` - Optional new client order ID after amendment
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    ///
    /// // Reduce an order's quantity from 10 to 5
    /// let response = client.account().amend_order_keep_priority(
    ///     "BTCUSDT",
    ///     Some(12345),
    ///     None,
    ///     "5.0",
    ///     None,
    /// ).await?;
    ///
    /// println!("Amended order ID: {}", response.amended_order.order_id);
    /// ```
    pub async fn amend_order_keep_priority(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        orig_client_order_id: Option<&str>,
        new_qty: &str,
        new_client_order_id: Option<&str>,
    ) -> Result<AmendOrderResponse> {
        let mut params: Vec<(&str, String)> = vec![
            ("symbol", symbol.to_string()),
            ("newQty", new_qty.to_string()),
        ];

        if let Some(id) = order_id {
            params.push(("orderId", id.to_string()));
        }
        if let Some(cid) = orig_client_order_id {
            params.push(("origClientOrderId", cid.to_string()));
        }
        if let Some(new_cid) = new_client_order_id {
            params.push(("newClientOrderId", new_cid.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .put_signed(API_V3_ORDER_AMEND, &params_ref)
            .await
    }

    /// Cancel an existing order and place a new order.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::rest::account::CancelReplaceOrderBuilder;
    /// use binance_api_client::{CancelReplaceMode, OrderSide, OrderType, TimeInForce};
    ///
    /// let request = CancelReplaceOrderBuilder::new("BTCUSDT", OrderSide::Buy, OrderType::Limit, CancelReplaceMode::StopOnFailure)
    ///     .cancel_order_id(12345)
    ///     .price("25000.00")
    ///     .quantity("0.01")
    ///     .time_in_force(TimeInForce::GTC)
    ///     .build();
    ///
    /// let response = client.account().cancel_replace_order(&request).await?;
    /// println!("Cancel result: {:?}", response.cancel_result);
    /// ```
    pub async fn cancel_replace_order(
        &self,
        request: &CancelReplaceOrder,
    ) -> Result<CancelReplaceResponse> {
        let params = request.to_params();
        let params_ref: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        let response = self
            .client
            .post_signed_raw(API_V3_ORDER_CANCEL_REPLACE, &params_ref)
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            StatusCode::BAD_REQUEST | StatusCode::CONFLICT => {
                let error: CancelReplaceErrorResponse = response.json().await?;
                Err(Error::from_cancel_replace_error(error))
            }
            StatusCode::UNAUTHORIZED => Err(Error::Api {
                code: 401,
                message: "Unauthorized".to_string(),
            }),
            StatusCode::FORBIDDEN | StatusCode::TOO_MANY_REQUESTS => {
                let error: BinanceApiError = response.json().await?;
                Err(Error::from_binance_error(error))
            }
            StatusCode::INTERNAL_SERVER_ERROR => Err(Error::Api {
                code: 500,
                message: "Internal server error".to_string(),
            }),
            StatusCode::SERVICE_UNAVAILABLE => Err(Error::Api {
                code: 503,
                message: "Service unavailable".to_string(),
            }),
            status => Err(Error::Api {
                code: status.as_u16() as i32,
                message: format!("Unexpected status code: {}", status),
            }),
        }
    }

    /// Place an order using smart order routing (SOR).
    pub async fn create_sor_order(&self, order: &NewOrder) -> Result<OrderFull> {
        let params = order.to_params();
        let params_ref: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.client.post_signed(API_V3_SOR_ORDER, &params_ref).await
    }

    /// Test a new SOR order without executing it.
    pub async fn test_sor_order(
        &self,
        order: &NewOrder,
        compute_commission_rates: bool,
    ) -> Result<SorOrderTestResponse> {
        let mut params = order.to_params();
        if compute_commission_rates {
            params.push((
                "computeCommissionRates".to_string(),
                compute_commission_rates.to_string(),
            ));
        }
        let params_ref: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.client
            .post_signed(API_V3_SOR_ORDER_TEST, &params_ref)
            .await
    }

    /// Query an order's status.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `order_id` - Order ID to query (either order_id or client_order_id required)
    /// * `client_order_id` - Client order ID to query
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let order = client.account().get_order("BTCUSDT", Some(12345), None).await?;
    /// println!("Order status: {:?}", order.status);
    /// ```
    pub async fn get_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        client_order_id: Option<&str>,
    ) -> Result<Order> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(id) = order_id {
            params.push(("orderId", id.to_string()));
        }
        if let Some(cid) = client_order_id {
            params.push(("origClientOrderId", cid.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client.get_signed(API_V3_ORDER, &params_ref).await
    }

    /// Cancel an order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `order_id` - Order ID to cancel (either order_id or client_order_id required)
    /// * `client_order_id` - Client order ID to cancel
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let result = client.account().cancel_order("BTCUSDT", Some(12345), None).await?;
    /// println!("Canceled order: {}", result.order_id);
    /// ```
    pub async fn cancel_order(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        client_order_id: Option<&str>,
    ) -> Result<CancelOrderResponse> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(id) = order_id {
            params.push(("orderId", id.to_string()));
        }
        if let Some(cid) = client_order_id {
            params.push(("origClientOrderId", cid.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client.delete_signed(API_V3_ORDER, &params_ref).await
    }

    /// Get all open orders for a symbol, or all symbols if none specified.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Optional trading pair symbol
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    ///
    /// // Get open orders for a specific symbol
    /// let orders = client.account().open_orders(Some("BTCUSDT")).await?;
    ///
    /// // Get all open orders
    /// let all_orders = client.account().open_orders(None).await?;
    /// ```
    pub async fn open_orders(&self, symbol: Option<&str>) -> Result<Vec<Order>> {
        let params: Vec<(&str, String)> = match symbol {
            Some(s) => vec![("symbol", s.to_string())],
            None => vec![],
        };

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(API_V3_OPEN_ORDERS, &params_ref)
            .await
    }

    /// Cancel all open orders for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let canceled = client.account().cancel_all_orders("BTCUSDT").await?;
    /// println!("Canceled {} orders", canceled.len());
    /// ```
    pub async fn cancel_all_orders(&self, symbol: &str) -> Result<Vec<CancelOrderResponse>> {
        let params = [("symbol", symbol)];
        self.client.delete_signed(API_V3_OPEN_ORDERS, &params).await
    }

    /// Get all orders for a symbol (active, canceled, or filled).
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `order_id` - If set, get orders >= this order ID
    /// * `start_time` - Start time in milliseconds
    /// * `end_time` - End time in milliseconds
    /// * `limit` - Max number of orders (default 500, max 1000)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let orders = client.account().all_orders("BTCUSDT", None, None, None, Some(10)).await?;
    /// ```
    pub async fn all_orders(
        &self,
        symbol: &str,
        order_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<Order>> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(id) = order_id {
            params.push(("orderId", id.to_string()));
        }
        if let Some(start) = start_time {
            params.push(("startTime", start.to_string()));
        }
        if let Some(end) = end_time {
            params.push(("endTime", end.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client.get_signed(API_V3_ALL_ORDERS, &params_ref).await
    }

    // OCO Order Endpoints.

    /// Create a new OCO (One-Cancels-Other) order.
    ///
    /// An OCO order combines a limit order with a stop-limit order.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let oco = OcoOrderBuilder::new("BTCUSDT", OrderSide::Sell, "1.0", "55000.00", "48000.00")
    ///     .stop_limit_price("47900.00")
    ///     .build();
    ///
    /// let result = client.account().create_oco(&oco).await?;
    /// ```
    pub async fn create_oco(&self, order: &NewOcoOrder) -> Result<OcoOrder> {
        let params = order.to_params();
        let params_ref: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.client.post_signed(API_V3_ORDER_OCO, &params_ref).await
    }

    /// Create a new OTO (One-Triggers-the-Other) order list.
    pub async fn create_oto(&self, order: &NewOtoOrder) -> Result<OcoOrder> {
        let params = order.to_params();
        let params_ref: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.client
            .post_signed(API_V3_ORDER_LIST_OTO, &params_ref)
            .await
    }

    /// Create a new OTOCO (One-Triggers-One-Cancels-the-Other) order list.
    pub async fn create_otoco(&self, order: &NewOtocoOrder) -> Result<OcoOrder> {
        let params = order.to_params();
        let params_ref: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.client
            .post_signed(API_V3_ORDER_LIST_OTOCO, &params_ref)
            .await
    }

    /// Create a new OPO (One-Places-the-Other) order list.
    pub async fn create_opo(&self, order: &NewOpoOrder) -> Result<OcoOrder> {
        let params = order.to_params();
        let params_ref: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.client
            .post_signed(API_V3_ORDER_LIST_OPO, &params_ref)
            .await
    }

    /// Create a new OPOCO (One-Places-One-Cancels-the-Other) order list.
    pub async fn create_opoco(&self, order: &NewOpocoOrder) -> Result<OcoOrder> {
        let params = order.to_params();
        let params_ref: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        self.client
            .post_signed(API_V3_ORDER_LIST_OPOCO, &params_ref)
            .await
    }

    /// Query an order list by ID or client order list ID.
    ///
    /// This applies to all order list types (OCO/OTO/OTOCO/OPO/OPOCO).
    pub async fn get_order_list(
        &self,
        order_list_id: Option<u64>,
        client_order_list_id: Option<&str>,
    ) -> Result<OcoOrder> {
        self.get_oco(order_list_id, client_order_list_id).await
    }

    /// Cancel an order list by symbol and list identifiers.
    ///
    /// This applies to all order list types (OCO/OTO/OTOCO/OPO/OPOCO).
    pub async fn cancel_order_list(
        &self,
        symbol: &str,
        order_list_id: Option<u64>,
        client_order_list_id: Option<&str>,
    ) -> Result<OcoOrder> {
        self.cancel_oco(symbol, order_list_id, client_order_list_id)
            .await
    }

    /// Get all order lists.
    ///
    /// This applies to all order list types (OCO/OTO/OTOCO/OPO/OPOCO).
    pub async fn all_order_lists(
        &self,
        from_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<OcoOrder>> {
        self.all_oco(from_id, start_time, end_time, limit).await
    }

    /// Get all open order lists.
    ///
    /// This applies to all order list types (OCO/OTO/OTOCO/OPO/OPOCO).
    pub async fn open_order_lists(&self) -> Result<Vec<OcoOrder>> {
        self.open_oco().await
    }

    /// Query an OCO order's status.
    ///
    /// # Arguments
    ///
    /// * `order_list_id` - OCO order list ID
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let oco = client.account().get_oco(Some(12345), None).await?;
    /// ```
    pub async fn get_oco(
        &self,
        order_list_id: Option<u64>,
        client_order_list_id: Option<&str>,
    ) -> Result<OcoOrder> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(id) = order_list_id {
            params.push(("orderListId", id.to_string()));
        }
        if let Some(cid) = client_order_list_id {
            params.push(("origClientOrderId", cid.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client.get_signed(API_V3_ORDER_LIST, &params_ref).await
    }

    /// Cancel an OCO order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `order_list_id` - OCO order list ID
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = client.account().cancel_oco("BTCUSDT", Some(12345), None).await?;
    /// ```
    pub async fn cancel_oco(
        &self,
        symbol: &str,
        order_list_id: Option<u64>,
        client_order_list_id: Option<&str>,
    ) -> Result<OcoOrder> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(id) = order_list_id {
            params.push(("orderListId", id.to_string()));
        }
        if let Some(cid) = client_order_list_id {
            params.push(("listClientOrderId", cid.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .delete_signed(API_V3_ORDER_LIST, &params_ref)
            .await
    }

    /// Get all OCO orders.
    ///
    /// # Arguments
    ///
    /// * `from_id` - If set, get orders >= this order list ID
    /// * `start_time` - Start time in milliseconds
    /// * `end_time` - End time in milliseconds
    /// * `limit` - Max number of orders (default 500, max 1000)
    pub async fn all_oco(
        &self,
        from_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<OcoOrder>> {
        let mut params: Vec<(&str, String)> = vec![];

        if let Some(id) = from_id {
            params.push(("fromId", id.to_string()));
        }
        if let Some(start) = start_time {
            params.push(("startTime", start.to_string()));
        }
        if let Some(end) = end_time {
            params.push(("endTime", end.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_signed(API_V3_ALL_ORDER_LIST, &params_ref)
            .await
    }

    /// Get all open OCO orders.
    pub async fn open_oco(&self) -> Result<Vec<OcoOrder>> {
        self.client.get_signed(API_V3_OPEN_ORDER_LIST, &[]).await
    }

    // Convenience Methods.

    /// Place a limit buy order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `quantity` - Order quantity
    /// * `price` - Limit price
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let order = client.account().limit_buy("BTCUSDT", "0.001", "50000.00").await?;
    /// ```
    pub async fn limit_buy(&self, symbol: &str, quantity: &str, price: &str) -> Result<OrderFull> {
        let order = OrderBuilder::new(symbol, OrderSide::Buy, OrderType::Limit)
            .quantity(quantity)
            .price(price)
            .time_in_force(TimeInForce::GTC)
            .build();
        self.create_order(&order).await
    }

    /// Place a limit sell order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `quantity` - Order quantity
    /// * `price` - Limit price
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let order = client.account().limit_sell("BTCUSDT", "0.001", "55000.00").await?;
    /// ```
    pub async fn limit_sell(&self, symbol: &str, quantity: &str, price: &str) -> Result<OrderFull> {
        let order = OrderBuilder::new(symbol, OrderSide::Sell, OrderType::Limit)
            .quantity(quantity)
            .price(price)
            .time_in_force(TimeInForce::GTC)
            .build();
        self.create_order(&order).await
    }

    /// Place a market buy order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `quantity` - Order quantity
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let order = client.account().market_buy("BTCUSDT", "0.001").await?;
    /// ```
    pub async fn market_buy(&self, symbol: &str, quantity: &str) -> Result<OrderFull> {
        let order = OrderBuilder::new(symbol, OrderSide::Buy, OrderType::Market)
            .quantity(quantity)
            .build();
        self.create_order(&order).await
    }

    /// Place a market sell order.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `quantity` - Order quantity
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let order = client.account().market_sell("BTCUSDT", "0.001").await?;
    /// ```
    pub async fn market_sell(&self, symbol: &str, quantity: &str) -> Result<OrderFull> {
        let order = OrderBuilder::new(symbol, OrderSide::Sell, OrderType::Market)
            .quantity(quantity)
            .build();
        self.create_order(&order).await
    }

    /// Place a market buy order using quote asset quantity.
    ///
    /// This allows you to specify how much of the quote asset (e.g., USDT) to spend.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `quote_quantity` - Amount of quote asset to spend
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// // Spend 100 USDT to buy BTC
    /// let order = client.account().market_buy_quote("BTCUSDT", "100.00").await?;
    /// ```
    pub async fn market_buy_quote(&self, symbol: &str, quote_quantity: &str) -> Result<OrderFull> {
        let order = OrderBuilder::new(symbol, OrderSide::Buy, OrderType::Market)
            .quote_quantity(quote_quantity)
            .build();
        self.create_order(&order).await
    }
}

/// Builder for creating new orders.
///
/// # Example
///
/// ```rust
/// use binance_api_client::rest::account::OrderBuilder;
/// use binance_api_client::{OrderSide, OrderType, TimeInForce};
///
/// let order = OrderBuilder::new("BTCUSDT", OrderSide::Buy, OrderType::Limit)
///     .quantity("0.001")
///     .price("50000.00")
///     .time_in_force(TimeInForce::GTC)
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct OrderBuilder {
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    quantity: Option<String>,
    quote_quantity: Option<String>,
    price: Option<String>,
    stop_price: Option<String>,
    time_in_force: Option<TimeInForce>,
    client_order_id: Option<String>,
    iceberg_qty: Option<String>,
    response_type: Option<OrderResponseType>,
}

/// Builder for cancel-replace orders.
#[derive(Debug, Clone)]
pub struct CancelReplaceOrderBuilder {
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    cancel_replace_mode: CancelReplaceMode,
    time_in_force: Option<TimeInForce>,
    quantity: Option<String>,
    quote_quantity: Option<String>,
    price: Option<String>,
    cancel_new_client_order_id: Option<String>,
    cancel_orig_client_order_id: Option<String>,
    cancel_order_id: Option<u64>,
    new_client_order_id: Option<String>,
    strategy_id: Option<u64>,
    strategy_type: Option<i32>,
    stop_price: Option<String>,
    trailing_delta: Option<u64>,
    iceberg_qty: Option<String>,
    response_type: Option<OrderResponseType>,
    self_trade_prevention_mode: Option<String>,
    cancel_restrictions: Option<CancelRestrictions>,
    order_rate_limit_exceeded_mode: Option<OrderRateLimitExceededMode>,
    peg_price_type: Option<String>,
    peg_offset_value: Option<i32>,
    peg_offset_type: Option<String>,
}

impl CancelReplaceOrderBuilder {
    /// Create a new cancel-replace order builder.
    pub fn new(
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        cancel_replace_mode: CancelReplaceMode,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            side,
            order_type,
            cancel_replace_mode,
            time_in_force: None,
            quantity: None,
            quote_quantity: None,
            price: None,
            cancel_new_client_order_id: None,
            cancel_orig_client_order_id: None,
            cancel_order_id: None,
            new_client_order_id: None,
            strategy_id: None,
            strategy_type: None,
            stop_price: None,
            trailing_delta: None,
            iceberg_qty: None,
            response_type: None,
            self_trade_prevention_mode: None,
            cancel_restrictions: None,
            order_rate_limit_exceeded_mode: None,
            peg_price_type: None,
            peg_offset_value: None,
            peg_offset_type: None,
        }
    }

    /// Set the order quantity.
    pub fn quantity(mut self, quantity: &str) -> Self {
        self.quantity = Some(quantity.to_string());
        self
    }

    /// Set the quote order quantity.
    pub fn quote_quantity(mut self, quantity: &str) -> Self {
        self.quote_quantity = Some(quantity.to_string());
        self
    }

    /// Set the order price.
    pub fn price(mut self, price: &str) -> Self {
        self.price = Some(price.to_string());
        self
    }

    /// Set the stop price.
    pub fn stop_price(mut self, price: &str) -> Self {
        self.stop_price = Some(price.to_string());
        self
    }

    /// Set the time in force.
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    /// Set a new client order ID for the cancel.
    pub fn cancel_new_client_order_id(mut self, id: &str) -> Self {
        self.cancel_new_client_order_id = Some(id.to_string());
        self
    }

    /// Set the original client order ID to cancel.
    pub fn cancel_orig_client_order_id(mut self, id: &str) -> Self {
        self.cancel_orig_client_order_id = Some(id.to_string());
        self
    }

    /// Set the order ID to cancel.
    pub fn cancel_order_id(mut self, id: u64) -> Self {
        self.cancel_order_id = Some(id);
        self
    }

    /// Set the new client order ID.
    pub fn new_client_order_id(mut self, id: &str) -> Self {
        self.new_client_order_id = Some(id.to_string());
        self
    }

    /// Set the strategy ID.
    pub fn strategy_id(mut self, id: u64) -> Self {
        self.strategy_id = Some(id);
        self
    }

    /// Set the strategy type.
    pub fn strategy_type(mut self, strategy_type: i32) -> Self {
        self.strategy_type = Some(strategy_type);
        self
    }

    /// Set the trailing delta.
    pub fn trailing_delta(mut self, delta: u64) -> Self {
        self.trailing_delta = Some(delta);
        self
    }

    /// Set the iceberg quantity.
    pub fn iceberg_qty(mut self, qty: &str) -> Self {
        self.iceberg_qty = Some(qty.to_string());
        self
    }

    /// Set the response type.
    pub fn response_type(mut self, resp_type: OrderResponseType) -> Self {
        self.response_type = Some(resp_type);
        self
    }

    /// Set self-trade prevention mode.
    pub fn self_trade_prevention_mode(mut self, mode: &str) -> Self {
        self.self_trade_prevention_mode = Some(mode.to_string());
        self
    }

    /// Set cancel restrictions.
    pub fn cancel_restrictions(mut self, restrictions: CancelRestrictions) -> Self {
        self.cancel_restrictions = Some(restrictions);
        self
    }

    /// Set order rate limit exceeded mode.
    pub fn order_rate_limit_exceeded_mode(mut self, mode: OrderRateLimitExceededMode) -> Self {
        self.order_rate_limit_exceeded_mode = Some(mode);
        self
    }

    /// Set pegged price type.
    pub fn peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.peg_price_type = Some(peg_price_type.to_string());
        self
    }

    /// Set pegged offset value.
    pub fn peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.peg_offset_value = Some(peg_offset_value);
        self
    }

    /// Set pegged offset type.
    pub fn peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    /// Build the cancel-replace order request.
    pub fn build(self) -> CancelReplaceOrder {
        CancelReplaceOrder {
            symbol: self.symbol,
            side: self.side,
            order_type: self.order_type,
            cancel_replace_mode: self.cancel_replace_mode,
            time_in_force: self.time_in_force,
            quantity: self.quantity,
            quote_quantity: self.quote_quantity,
            price: self.price,
            cancel_new_client_order_id: self.cancel_new_client_order_id,
            cancel_orig_client_order_id: self.cancel_orig_client_order_id,
            cancel_order_id: self.cancel_order_id,
            new_client_order_id: self.new_client_order_id,
            strategy_id: self.strategy_id,
            strategy_type: self.strategy_type,
            stop_price: self.stop_price,
            trailing_delta: self.trailing_delta,
            iceberg_qty: self.iceberg_qty,
            response_type: self.response_type,
            self_trade_prevention_mode: self.self_trade_prevention_mode,
            cancel_restrictions: self.cancel_restrictions,
            order_rate_limit_exceeded_mode: self.order_rate_limit_exceeded_mode,
            peg_price_type: self.peg_price_type,
            peg_offset_value: self.peg_offset_value,
            peg_offset_type: self.peg_offset_type,
        }
    }
}

/// Cancel-replace order request parameters.
#[derive(Debug, Clone)]
pub struct CancelReplaceOrder {
    symbol: String,
    side: OrderSide,
    order_type: OrderType,
    cancel_replace_mode: CancelReplaceMode,
    time_in_force: Option<TimeInForce>,
    quantity: Option<String>,
    quote_quantity: Option<String>,
    price: Option<String>,
    cancel_new_client_order_id: Option<String>,
    cancel_orig_client_order_id: Option<String>,
    cancel_order_id: Option<u64>,
    new_client_order_id: Option<String>,
    strategy_id: Option<u64>,
    strategy_type: Option<i32>,
    stop_price: Option<String>,
    trailing_delta: Option<u64>,
    iceberg_qty: Option<String>,
    response_type: Option<OrderResponseType>,
    self_trade_prevention_mode: Option<String>,
    cancel_restrictions: Option<CancelRestrictions>,
    order_rate_limit_exceeded_mode: Option<OrderRateLimitExceededMode>,
    peg_price_type: Option<String>,
    peg_offset_value: Option<i32>,
    peg_offset_type: Option<String>,
}

impl CancelReplaceOrder {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("symbol".to_string(), self.symbol.clone()),
            (
                "side".to_string(),
                format!("{:?}", self.side).to_uppercase(),
            ),
            (
                "type".to_string(),
                format!("{:?}", self.order_type).to_uppercase(),
            ),
            (
                "cancelReplaceMode".to_string(),
                self.cancel_replace_mode.to_string(),
            ),
        ];

        if let Some(ref tif) = self.time_in_force {
            params.push(("timeInForce".to_string(), format!("{:?}", tif)));
        }
        if let Some(ref qty) = self.quantity {
            params.push(("quantity".to_string(), qty.clone()));
        }
        if let Some(ref qty) = self.quote_quantity {
            params.push(("quoteOrderQty".to_string(), qty.clone()));
        }
        if let Some(ref price) = self.price {
            params.push(("price".to_string(), price.clone()));
        }
        if let Some(ref id) = self.cancel_new_client_order_id {
            params.push(("cancelNewClientOrderId".to_string(), id.clone()));
        }
        if let Some(ref id) = self.cancel_orig_client_order_id {
            params.push(("cancelOrigClientOrderId".to_string(), id.clone()));
        }
        if let Some(id) = self.cancel_order_id {
            params.push(("cancelOrderId".to_string(), id.to_string()));
        }
        if let Some(ref id) = self.new_client_order_id {
            params.push(("newClientOrderId".to_string(), id.clone()));
        }
        if let Some(id) = self.strategy_id {
            params.push(("strategyId".to_string(), id.to_string()));
        }
        if let Some(id) = self.strategy_type {
            params.push(("strategyType".to_string(), id.to_string()));
        }
        if let Some(ref stop) = self.stop_price {
            params.push(("stopPrice".to_string(), stop.clone()));
        }
        if let Some(delta) = self.trailing_delta {
            params.push(("trailingDelta".to_string(), delta.to_string()));
        }
        if let Some(ref ice) = self.iceberg_qty {
            params.push(("icebergQty".to_string(), ice.clone()));
        }
        if let Some(ref resp) = self.response_type {
            params.push((
                "newOrderRespType".to_string(),
                format!("{:?}", resp).to_uppercase(),
            ));
        }
        if let Some(ref mode) = self.self_trade_prevention_mode {
            params.push(("selfTradePreventionMode".to_string(), mode.clone()));
        }
        if let Some(restrictions) = self.cancel_restrictions {
            params.push(("cancelRestrictions".to_string(), restrictions.to_string()));
        }
        if let Some(mode) = self.order_rate_limit_exceeded_mode {
            params.push(("orderRateLimitExceededMode".to_string(), mode.to_string()));
        }
        if let Some(ref peg) = self.peg_price_type {
            params.push(("pegPriceType".to_string(), peg.clone()));
        }
        if let Some(value) = self.peg_offset_value {
            params.push(("pegOffsetValue".to_string(), value.to_string()));
        }
        if let Some(ref peg) = self.peg_offset_type {
            params.push(("pegOffsetType".to_string(), peg.clone()));
        }

        params
    }
}

impl OrderBuilder {
    /// Create a new order builder.
    pub fn new(symbol: &str, side: OrderSide, order_type: OrderType) -> Self {
        Self {
            symbol: symbol.to_string(),
            side,
            order_type,
            quantity: None,
            quote_quantity: None,
            price: None,
            stop_price: None,
            time_in_force: None,
            client_order_id: None,
            iceberg_qty: None,
            response_type: None,
        }
    }

    /// Set the order quantity.
    pub fn quantity(mut self, quantity: &str) -> Self {
        self.quantity = Some(quantity.to_string());
        self
    }

    /// Set the quote order quantity (for market orders).
    pub fn quote_quantity(mut self, quantity: &str) -> Self {
        self.quote_quantity = Some(quantity.to_string());
        self
    }

    /// Set the order price (required for limit orders).
    pub fn price(mut self, price: &str) -> Self {
        self.price = Some(price.to_string());
        self
    }

    /// Set the stop price (for stop orders).
    pub fn stop_price(mut self, price: &str) -> Self {
        self.stop_price = Some(price.to_string());
        self
    }

    /// Set the time in force.
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    /// Set a custom client order ID.
    pub fn client_order_id(mut self, id: &str) -> Self {
        self.client_order_id = Some(id.to_string());
        self
    }

    /// Set the iceberg quantity.
    pub fn iceberg_qty(mut self, qty: &str) -> Self {
        self.iceberg_qty = Some(qty.to_string());
        self
    }

    /// Set the response type.
    pub fn response_type(mut self, resp_type: OrderResponseType) -> Self {
        self.response_type = Some(resp_type);
        self
    }

    /// Build the order.
    pub fn build(self) -> NewOrder {
        NewOrder {
            symbol: self.symbol,
            side: self.side,
            order_type: self.order_type,
            quantity: self.quantity,
            quote_quantity: self.quote_quantity,
            price: self.price,
            stop_price: self.stop_price,
            time_in_force: self.time_in_force,
            client_order_id: self.client_order_id,
            iceberg_qty: self.iceberg_qty,
            response_type: self.response_type,
        }
    }
}

/// New order parameters.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewOrder {
    symbol: String,
    side: OrderSide,
    #[serde(rename = "type")]
    order_type: OrderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "quoteOrderQty")]
    quote_quantity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_price: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_in_force: Option<TimeInForce>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "newClientOrderId")]
    client_order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iceberg_qty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "newOrderRespType")]
    response_type: Option<OrderResponseType>,
}

impl NewOrder {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("symbol".to_string(), self.symbol.clone()),
            (
                "side".to_string(),
                format!("{:?}", self.side).to_uppercase(),
            ),
            (
                "type".to_string(),
                format!("{:?}", self.order_type).to_uppercase(),
            ),
        ];

        if let Some(ref qty) = self.quantity {
            params.push(("quantity".to_string(), qty.clone()));
        }
        if let Some(ref qty) = self.quote_quantity {
            params.push(("quoteOrderQty".to_string(), qty.clone()));
        }
        if let Some(ref price) = self.price {
            params.push(("price".to_string(), price.clone()));
        }
        if let Some(ref stop) = self.stop_price {
            params.push(("stopPrice".to_string(), stop.clone()));
        }
        if let Some(ref tif) = self.time_in_force {
            params.push(("timeInForce".to_string(), format!("{:?}", tif)));
        }
        if let Some(ref cid) = self.client_order_id {
            params.push(("newClientOrderId".to_string(), cid.clone()));
        }
        if let Some(ref ice) = self.iceberg_qty {
            params.push(("icebergQty".to_string(), ice.clone()));
        }
        if let Some(ref resp) = self.response_type {
            params.push((
                "newOrderRespType".to_string(),
                format!("{:?}", resp).to_uppercase(),
            ));
        }

        params
    }
}

/// Builder for creating OCO orders.
#[derive(Debug, Clone)]
pub struct OcoOrderBuilder {
    symbol: String,
    side: OrderSide,
    quantity: String,
    price: String,
    stop_price: String,
    stop_limit_price: Option<String>,
    stop_limit_time_in_force: Option<TimeInForce>,
    list_client_order_id: Option<String>,
    limit_client_order_id: Option<String>,
    stop_client_order_id: Option<String>,
}

impl OcoOrderBuilder {
    /// Create a new OCO order builder.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `side` - Order side (Buy or Sell)
    /// * `quantity` - Order quantity
    /// * `price` - Limit order price
    /// * `stop_price` - Stop order trigger price
    pub fn new(
        symbol: &str,
        side: OrderSide,
        quantity: &str,
        price: &str,
        stop_price: &str,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            side,
            quantity: quantity.to_string(),
            price: price.to_string(),
            stop_price: stop_price.to_string(),
            stop_limit_price: None,
            stop_limit_time_in_force: None,
            list_client_order_id: None,
            limit_client_order_id: None,
            stop_client_order_id: None,
        }
    }

    /// Set the stop limit price.
    pub fn stop_limit_price(mut self, price: &str) -> Self {
        self.stop_limit_price = Some(price.to_string());
        self
    }

    /// Set the stop limit time in force.
    pub fn stop_limit_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.stop_limit_time_in_force = Some(tif);
        self
    }

    /// Set a custom list client order ID.
    pub fn list_client_order_id(mut self, id: &str) -> Self {
        self.list_client_order_id = Some(id.to_string());
        self
    }

    /// Set a custom limit client order ID.
    pub fn limit_client_order_id(mut self, id: &str) -> Self {
        self.limit_client_order_id = Some(id.to_string());
        self
    }

    /// Set a custom stop client order ID.
    pub fn stop_client_order_id(mut self, id: &str) -> Self {
        self.stop_client_order_id = Some(id.to_string());
        self
    }

    /// Build the OCO order.
    pub fn build(self) -> NewOcoOrder {
        NewOcoOrder {
            symbol: self.symbol,
            side: self.side,
            quantity: self.quantity,
            price: self.price,
            stop_price: self.stop_price,
            stop_limit_price: self.stop_limit_price,
            stop_limit_time_in_force: self.stop_limit_time_in_force,
            list_client_order_id: self.list_client_order_id,
            limit_client_order_id: self.limit_client_order_id,
            stop_client_order_id: self.stop_client_order_id,
        }
    }
}

/// New OCO order parameters.
#[derive(Debug, Clone)]
pub struct NewOcoOrder {
    symbol: String,
    side: OrderSide,
    quantity: String,
    price: String,
    stop_price: String,
    stop_limit_price: Option<String>,
    stop_limit_time_in_force: Option<TimeInForce>,
    list_client_order_id: Option<String>,
    limit_client_order_id: Option<String>,
    stop_client_order_id: Option<String>,
}

impl NewOcoOrder {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("symbol".to_string(), self.symbol.clone()),
            (
                "side".to_string(),
                format!("{:?}", self.side).to_uppercase(),
            ),
            ("quantity".to_string(), self.quantity.clone()),
            ("price".to_string(), self.price.clone()),
            ("stopPrice".to_string(), self.stop_price.clone()),
        ];

        if let Some(ref slp) = self.stop_limit_price {
            params.push(("stopLimitPrice".to_string(), slp.clone()));
        }
        if let Some(ref tif) = self.stop_limit_time_in_force {
            params.push(("stopLimitTimeInForce".to_string(), format!("{:?}", tif)));
        }
        if let Some(ref id) = self.list_client_order_id {
            params.push(("listClientOrderId".to_string(), id.clone()));
        }
        if let Some(ref id) = self.limit_client_order_id {
            params.push(("limitClientOrderId".to_string(), id.clone()));
        }
        if let Some(ref id) = self.stop_client_order_id {
            params.push(("stopClientOrderId".to_string(), id.clone()));
        }

        params
    }
}

/// Builder for creating OTO order lists.
#[derive(Debug, Clone)]
pub struct OtoOrderBuilder {
    symbol: String,
    working_type: OrderType,
    working_side: OrderSide,
    working_price: String,
    working_quantity: String,
    pending_type: OrderType,
    pending_side: OrderSide,
    pending_quantity: String,
    list_client_order_id: Option<String>,
    response_type: Option<OrderResponseType>,
    self_trade_prevention_mode: Option<String>,
    working_client_order_id: Option<String>,
    working_iceberg_qty: Option<String>,
    working_time_in_force: Option<TimeInForce>,
    working_strategy_id: Option<u64>,
    working_strategy_type: Option<i32>,
    working_peg_price_type: Option<String>,
    working_peg_offset_type: Option<String>,
    working_peg_offset_value: Option<i32>,
    pending_client_order_id: Option<String>,
    pending_price: Option<String>,
    pending_stop_price: Option<String>,
    pending_trailing_delta: Option<u64>,
    pending_iceberg_qty: Option<String>,
    pending_time_in_force: Option<TimeInForce>,
    pending_strategy_id: Option<u64>,
    pending_strategy_type: Option<i32>,
    pending_peg_price_type: Option<String>,
    pending_peg_offset_type: Option<String>,
    pending_peg_offset_value: Option<i32>,
}

impl OtoOrderBuilder {
    /// Create a new OTO order list builder.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: &str,
        working_type: OrderType,
        working_side: OrderSide,
        working_price: &str,
        working_quantity: &str,
        pending_type: OrderType,
        pending_side: OrderSide,
        pending_quantity: &str,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            working_type,
            working_side,
            working_price: working_price.to_string(),
            working_quantity: working_quantity.to_string(),
            pending_type,
            pending_side,
            pending_quantity: pending_quantity.to_string(),
            list_client_order_id: None,
            response_type: None,
            self_trade_prevention_mode: None,
            working_client_order_id: None,
            working_iceberg_qty: None,
            working_time_in_force: None,
            working_strategy_id: None,
            working_strategy_type: None,
            working_peg_price_type: None,
            working_peg_offset_type: None,
            working_peg_offset_value: None,
            pending_client_order_id: None,
            pending_price: None,
            pending_stop_price: None,
            pending_trailing_delta: None,
            pending_iceberg_qty: None,
            pending_time_in_force: None,
            pending_strategy_id: None,
            pending_strategy_type: None,
            pending_peg_price_type: None,
            pending_peg_offset_type: None,
            pending_peg_offset_value: None,
        }
    }

    pub fn list_client_order_id(mut self, id: &str) -> Self {
        self.list_client_order_id = Some(id.to_string());
        self
    }

    pub fn response_type(mut self, resp_type: OrderResponseType) -> Self {
        self.response_type = Some(resp_type);
        self
    }

    pub fn self_trade_prevention_mode(mut self, mode: &str) -> Self {
        self.self_trade_prevention_mode = Some(mode.to_string());
        self
    }

    pub fn working_client_order_id(mut self, id: &str) -> Self {
        self.working_client_order_id = Some(id.to_string());
        self
    }

    pub fn working_iceberg_qty(mut self, qty: &str) -> Self {
        self.working_iceberg_qty = Some(qty.to_string());
        self
    }

    pub fn working_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.working_time_in_force = Some(tif);
        self
    }

    pub fn working_strategy_id(mut self, id: u64) -> Self {
        self.working_strategy_id = Some(id);
        self
    }

    pub fn working_strategy_type(mut self, strategy_type: i32) -> Self {
        self.working_strategy_type = Some(strategy_type);
        self
    }

    pub fn working_peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.working_peg_price_type = Some(peg_price_type.to_string());
        self
    }

    pub fn working_peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.working_peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    pub fn working_peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.working_peg_offset_value = Some(peg_offset_value);
        self
    }

    pub fn pending_client_order_id(mut self, id: &str) -> Self {
        self.pending_client_order_id = Some(id.to_string());
        self
    }

    pub fn pending_price(mut self, price: &str) -> Self {
        self.pending_price = Some(price.to_string());
        self
    }

    pub fn pending_stop_price(mut self, price: &str) -> Self {
        self.pending_stop_price = Some(price.to_string());
        self
    }

    pub fn pending_trailing_delta(mut self, delta: u64) -> Self {
        self.pending_trailing_delta = Some(delta);
        self
    }

    pub fn pending_iceberg_qty(mut self, qty: &str) -> Self {
        self.pending_iceberg_qty = Some(qty.to_string());
        self
    }

    pub fn pending_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.pending_time_in_force = Some(tif);
        self
    }

    pub fn pending_strategy_id(mut self, id: u64) -> Self {
        self.pending_strategy_id = Some(id);
        self
    }

    pub fn pending_strategy_type(mut self, strategy_type: i32) -> Self {
        self.pending_strategy_type = Some(strategy_type);
        self
    }

    pub fn pending_peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.pending_peg_price_type = Some(peg_price_type.to_string());
        self
    }

    pub fn pending_peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.pending_peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    pub fn pending_peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.pending_peg_offset_value = Some(peg_offset_value);
        self
    }

    pub fn build(self) -> NewOtoOrder {
        NewOtoOrder {
            symbol: self.symbol,
            working_type: self.working_type,
            working_side: self.working_side,
            working_price: self.working_price,
            working_quantity: self.working_quantity,
            pending_type: self.pending_type,
            pending_side: self.pending_side,
            pending_quantity: Some(self.pending_quantity),
            list_client_order_id: self.list_client_order_id,
            response_type: self.response_type,
            self_trade_prevention_mode: self.self_trade_prevention_mode,
            working_client_order_id: self.working_client_order_id,
            working_iceberg_qty: self.working_iceberg_qty,
            working_time_in_force: self.working_time_in_force,
            working_strategy_id: self.working_strategy_id,
            working_strategy_type: self.working_strategy_type,
            working_peg_price_type: self.working_peg_price_type,
            working_peg_offset_type: self.working_peg_offset_type,
            working_peg_offset_value: self.working_peg_offset_value,
            pending_client_order_id: self.pending_client_order_id,
            pending_price: self.pending_price,
            pending_stop_price: self.pending_stop_price,
            pending_trailing_delta: self.pending_trailing_delta,
            pending_iceberg_qty: self.pending_iceberg_qty,
            pending_time_in_force: self.pending_time_in_force,
            pending_strategy_id: self.pending_strategy_id,
            pending_strategy_type: self.pending_strategy_type,
            pending_peg_price_type: self.pending_peg_price_type,
            pending_peg_offset_type: self.pending_peg_offset_type,
            pending_peg_offset_value: self.pending_peg_offset_value,
        }
    }
}

/// New OTO order list parameters.
#[derive(Debug, Clone)]
pub struct NewOtoOrder {
    symbol: String,
    working_type: OrderType,
    working_side: OrderSide,
    working_price: String,
    working_quantity: String,
    pending_type: OrderType,
    pending_side: OrderSide,
    pending_quantity: Option<String>,
    list_client_order_id: Option<String>,
    response_type: Option<OrderResponseType>,
    self_trade_prevention_mode: Option<String>,
    working_client_order_id: Option<String>,
    working_iceberg_qty: Option<String>,
    working_time_in_force: Option<TimeInForce>,
    working_strategy_id: Option<u64>,
    working_strategy_type: Option<i32>,
    working_peg_price_type: Option<String>,
    working_peg_offset_type: Option<String>,
    working_peg_offset_value: Option<i32>,
    pending_client_order_id: Option<String>,
    pending_price: Option<String>,
    pending_stop_price: Option<String>,
    pending_trailing_delta: Option<u64>,
    pending_iceberg_qty: Option<String>,
    pending_time_in_force: Option<TimeInForce>,
    pending_strategy_id: Option<u64>,
    pending_strategy_type: Option<i32>,
    pending_peg_price_type: Option<String>,
    pending_peg_offset_type: Option<String>,
    pending_peg_offset_value: Option<i32>,
}

impl NewOtoOrder {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("symbol".to_string(), self.symbol.clone()),
            (
                "workingType".to_string(),
                format!("{:?}", self.working_type).to_uppercase(),
            ),
            (
                "workingSide".to_string(),
                format!("{:?}", self.working_side).to_uppercase(),
            ),
            ("workingPrice".to_string(), self.working_price.clone()),
            ("workingQuantity".to_string(), self.working_quantity.clone()),
            (
                "pendingType".to_string(),
                format!("{:?}", self.pending_type).to_uppercase(),
            ),
            (
                "pendingSide".to_string(),
                format!("{:?}", self.pending_side).to_uppercase(),
            ),
        ];

        if let Some(ref qty) = self.pending_quantity {
            params.push(("pendingQuantity".to_string(), qty.clone()));
        }
        if let Some(ref id) = self.list_client_order_id {
            params.push(("listClientOrderId".to_string(), id.clone()));
        }
        if let Some(ref resp) = self.response_type {
            params.push((
                "newOrderRespType".to_string(),
                format!("{:?}", resp).to_uppercase(),
            ));
        }
        if let Some(ref mode) = self.self_trade_prevention_mode {
            params.push(("selfTradePreventionMode".to_string(), mode.clone()));
        }
        if let Some(ref id) = self.working_client_order_id {
            params.push(("workingClientOrderId".to_string(), id.clone()));
        }
        if let Some(ref qty) = self.working_iceberg_qty {
            params.push(("workingIcebergQty".to_string(), qty.clone()));
        }
        if let Some(ref tif) = self.working_time_in_force {
            params.push(("workingTimeInForce".to_string(), format!("{:?}", tif)));
        }
        if let Some(id) = self.working_strategy_id {
            params.push(("workingStrategyId".to_string(), id.to_string()));
        }
        if let Some(id) = self.working_strategy_type {
            params.push(("workingStrategyType".to_string(), id.to_string()));
        }
        if let Some(ref peg) = self.working_peg_price_type {
            params.push(("workingPegPriceType".to_string(), peg.clone()));
        }
        if let Some(ref peg) = self.working_peg_offset_type {
            params.push(("workingPegOffsetType".to_string(), peg.clone()));
        }
        if let Some(value) = self.working_peg_offset_value {
            params.push(("workingPegOffsetValue".to_string(), value.to_string()));
        }
        if let Some(ref id) = self.pending_client_order_id {
            params.push(("pendingClientOrderId".to_string(), id.clone()));
        }
        if let Some(ref price) = self.pending_price {
            params.push(("pendingPrice".to_string(), price.clone()));
        }
        if let Some(ref price) = self.pending_stop_price {
            params.push(("pendingStopPrice".to_string(), price.clone()));
        }
        if let Some(delta) = self.pending_trailing_delta {
            params.push(("pendingTrailingDelta".to_string(), delta.to_string()));
        }
        if let Some(ref qty) = self.pending_iceberg_qty {
            params.push(("pendingIcebergQty".to_string(), qty.clone()));
        }
        if let Some(ref tif) = self.pending_time_in_force {
            params.push(("pendingTimeInForce".to_string(), format!("{:?}", tif)));
        }
        if let Some(id) = self.pending_strategy_id {
            params.push(("pendingStrategyId".to_string(), id.to_string()));
        }
        if let Some(id) = self.pending_strategy_type {
            params.push(("pendingStrategyType".to_string(), id.to_string()));
        }
        if let Some(ref peg) = self.pending_peg_price_type {
            params.push(("pendingPegPriceType".to_string(), peg.clone()));
        }
        if let Some(ref peg) = self.pending_peg_offset_type {
            params.push(("pendingPegOffsetType".to_string(), peg.clone()));
        }
        if let Some(value) = self.pending_peg_offset_value {
            params.push(("pendingPegOffsetValue".to_string(), value.to_string()));
        }

        params
    }
}

/// Builder for creating OPO order lists.
#[derive(Debug, Clone)]
pub struct OpoOrderBuilder {
    inner: NewOtoOrder,
}

impl OpoOrderBuilder {
    /// Create a new OPO order list builder.
    pub fn new(
        symbol: &str,
        working_type: OrderType,
        working_side: OrderSide,
        working_price: &str,
        working_quantity: &str,
        pending_type: OrderType,
        pending_side: OrderSide,
    ) -> Self {
        Self {
            inner: NewOtoOrder {
                symbol: symbol.to_string(),
                working_type,
                working_side,
                working_price: working_price.to_string(),
                working_quantity: working_quantity.to_string(),
                pending_type,
                pending_side,
                pending_quantity: None,
                list_client_order_id: None,
                response_type: None,
                self_trade_prevention_mode: None,
                working_client_order_id: None,
                working_iceberg_qty: None,
                working_time_in_force: None,
                working_strategy_id: None,
                working_strategy_type: None,
                working_peg_price_type: None,
                working_peg_offset_type: None,
                working_peg_offset_value: None,
                pending_client_order_id: None,
                pending_price: None,
                pending_stop_price: None,
                pending_trailing_delta: None,
                pending_iceberg_qty: None,
                pending_time_in_force: None,
                pending_strategy_id: None,
                pending_strategy_type: None,
                pending_peg_price_type: None,
                pending_peg_offset_type: None,
                pending_peg_offset_value: None,
            },
        }
    }

    pub fn list_client_order_id(mut self, id: &str) -> Self {
        self.inner.list_client_order_id = Some(id.to_string());
        self
    }

    pub fn response_type(mut self, resp_type: OrderResponseType) -> Self {
        self.inner.response_type = Some(resp_type);
        self
    }

    pub fn self_trade_prevention_mode(mut self, mode: &str) -> Self {
        self.inner.self_trade_prevention_mode = Some(mode.to_string());
        self
    }

    pub fn working_client_order_id(mut self, id: &str) -> Self {
        self.inner.working_client_order_id = Some(id.to_string());
        self
    }

    pub fn working_iceberg_qty(mut self, qty: &str) -> Self {
        self.inner.working_iceberg_qty = Some(qty.to_string());
        self
    }

    pub fn working_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.inner.working_time_in_force = Some(tif);
        self
    }

    pub fn working_strategy_id(mut self, id: u64) -> Self {
        self.inner.working_strategy_id = Some(id);
        self
    }

    pub fn working_strategy_type(mut self, strategy_type: i32) -> Self {
        self.inner.working_strategy_type = Some(strategy_type);
        self
    }

    pub fn working_peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.inner.working_peg_price_type = Some(peg_price_type.to_string());
        self
    }

    pub fn working_peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.inner.working_peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    pub fn working_peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.inner.working_peg_offset_value = Some(peg_offset_value);
        self
    }

    pub fn pending_client_order_id(mut self, id: &str) -> Self {
        self.inner.pending_client_order_id = Some(id.to_string());
        self
    }

    pub fn pending_quantity(mut self, qty: &str) -> Self {
        self.inner.pending_quantity = Some(qty.to_string());
        self
    }

    pub fn pending_price(mut self, price: &str) -> Self {
        self.inner.pending_price = Some(price.to_string());
        self
    }

    pub fn pending_stop_price(mut self, price: &str) -> Self {
        self.inner.pending_stop_price = Some(price.to_string());
        self
    }

    pub fn pending_trailing_delta(mut self, delta: u64) -> Self {
        self.inner.pending_trailing_delta = Some(delta);
        self
    }

    pub fn pending_iceberg_qty(mut self, qty: &str) -> Self {
        self.inner.pending_iceberg_qty = Some(qty.to_string());
        self
    }

    pub fn pending_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.inner.pending_time_in_force = Some(tif);
        self
    }

    pub fn pending_strategy_id(mut self, id: u64) -> Self {
        self.inner.pending_strategy_id = Some(id);
        self
    }

    pub fn pending_strategy_type(mut self, strategy_type: i32) -> Self {
        self.inner.pending_strategy_type = Some(strategy_type);
        self
    }

    pub fn pending_peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.inner.pending_peg_price_type = Some(peg_price_type.to_string());
        self
    }

    pub fn pending_peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.inner.pending_peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    pub fn pending_peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.inner.pending_peg_offset_value = Some(peg_offset_value);
        self
    }

    pub fn build(self) -> NewOpoOrder {
        NewOpoOrder { inner: self.inner }
    }
}

/// New OPO order list parameters.
#[derive(Debug, Clone)]
pub struct NewOpoOrder {
    inner: NewOtoOrder,
}

impl NewOpoOrder {
    fn to_params(&self) -> Vec<(String, String)> {
        self.inner.to_params()
    }
}

/// Builder for creating OTOCO order lists.
#[derive(Debug, Clone)]
pub struct OtocoOrderBuilder {
    symbol: String,
    working_type: OrderType,
    working_side: OrderSide,
    working_price: String,
    working_quantity: String,
    pending_side: OrderSide,
    pending_quantity: String,
    pending_above_type: OrderType,
    list_client_order_id: Option<String>,
    response_type: Option<OrderResponseType>,
    self_trade_prevention_mode: Option<String>,
    working_client_order_id: Option<String>,
    working_iceberg_qty: Option<String>,
    working_time_in_force: Option<TimeInForce>,
    working_strategy_id: Option<u64>,
    working_strategy_type: Option<i32>,
    working_peg_price_type: Option<String>,
    working_peg_offset_type: Option<String>,
    working_peg_offset_value: Option<i32>,
    pending_above_client_order_id: Option<String>,
    pending_above_price: Option<String>,
    pending_above_stop_price: Option<String>,
    pending_above_trailing_delta: Option<u64>,
    pending_above_iceberg_qty: Option<String>,
    pending_above_time_in_force: Option<TimeInForce>,
    pending_above_strategy_id: Option<u64>,
    pending_above_strategy_type: Option<i32>,
    pending_above_peg_price_type: Option<String>,
    pending_above_peg_offset_type: Option<String>,
    pending_above_peg_offset_value: Option<i32>,
    pending_below_type: Option<OrderType>,
    pending_below_client_order_id: Option<String>,
    pending_below_price: Option<String>,
    pending_below_stop_price: Option<String>,
    pending_below_trailing_delta: Option<u64>,
    pending_below_iceberg_qty: Option<String>,
    pending_below_time_in_force: Option<TimeInForce>,
    pending_below_strategy_id: Option<u64>,
    pending_below_strategy_type: Option<i32>,
    pending_below_peg_price_type: Option<String>,
    pending_below_peg_offset_type: Option<String>,
    pending_below_peg_offset_value: Option<i32>,
}

impl OtocoOrderBuilder {
    /// Create a new OTOCO order list builder.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: &str,
        working_type: OrderType,
        working_side: OrderSide,
        working_price: &str,
        working_quantity: &str,
        pending_side: OrderSide,
        pending_quantity: &str,
        pending_above_type: OrderType,
    ) -> Self {
        Self {
            symbol: symbol.to_string(),
            working_type,
            working_side,
            working_price: working_price.to_string(),
            working_quantity: working_quantity.to_string(),
            pending_side,
            pending_quantity: pending_quantity.to_string(),
            pending_above_type,
            list_client_order_id: None,
            response_type: None,
            self_trade_prevention_mode: None,
            working_client_order_id: None,
            working_iceberg_qty: None,
            working_time_in_force: None,
            working_strategy_id: None,
            working_strategy_type: None,
            working_peg_price_type: None,
            working_peg_offset_type: None,
            working_peg_offset_value: None,
            pending_above_client_order_id: None,
            pending_above_price: None,
            pending_above_stop_price: None,
            pending_above_trailing_delta: None,
            pending_above_iceberg_qty: None,
            pending_above_time_in_force: None,
            pending_above_strategy_id: None,
            pending_above_strategy_type: None,
            pending_above_peg_price_type: None,
            pending_above_peg_offset_type: None,
            pending_above_peg_offset_value: None,
            pending_below_type: None,
            pending_below_client_order_id: None,
            pending_below_price: None,
            pending_below_stop_price: None,
            pending_below_trailing_delta: None,
            pending_below_iceberg_qty: None,
            pending_below_time_in_force: None,
            pending_below_strategy_id: None,
            pending_below_strategy_type: None,
            pending_below_peg_price_type: None,
            pending_below_peg_offset_type: None,
            pending_below_peg_offset_value: None,
        }
    }

    pub fn list_client_order_id(mut self, id: &str) -> Self {
        self.list_client_order_id = Some(id.to_string());
        self
    }

    pub fn response_type(mut self, resp_type: OrderResponseType) -> Self {
        self.response_type = Some(resp_type);
        self
    }

    pub fn self_trade_prevention_mode(mut self, mode: &str) -> Self {
        self.self_trade_prevention_mode = Some(mode.to_string());
        self
    }

    pub fn working_client_order_id(mut self, id: &str) -> Self {
        self.working_client_order_id = Some(id.to_string());
        self
    }

    pub fn working_iceberg_qty(mut self, qty: &str) -> Self {
        self.working_iceberg_qty = Some(qty.to_string());
        self
    }

    pub fn working_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.working_time_in_force = Some(tif);
        self
    }

    pub fn working_strategy_id(mut self, id: u64) -> Self {
        self.working_strategy_id = Some(id);
        self
    }

    pub fn working_strategy_type(mut self, strategy_type: i32) -> Self {
        self.working_strategy_type = Some(strategy_type);
        self
    }

    pub fn working_peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.working_peg_price_type = Some(peg_price_type.to_string());
        self
    }

    pub fn working_peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.working_peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    pub fn working_peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.working_peg_offset_value = Some(peg_offset_value);
        self
    }

    pub fn pending_above_client_order_id(mut self, id: &str) -> Self {
        self.pending_above_client_order_id = Some(id.to_string());
        self
    }

    pub fn pending_above_price(mut self, price: &str) -> Self {
        self.pending_above_price = Some(price.to_string());
        self
    }

    pub fn pending_above_stop_price(mut self, price: &str) -> Self {
        self.pending_above_stop_price = Some(price.to_string());
        self
    }

    pub fn pending_above_trailing_delta(mut self, delta: u64) -> Self {
        self.pending_above_trailing_delta = Some(delta);
        self
    }

    pub fn pending_above_iceberg_qty(mut self, qty: &str) -> Self {
        self.pending_above_iceberg_qty = Some(qty.to_string());
        self
    }

    pub fn pending_above_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.pending_above_time_in_force = Some(tif);
        self
    }

    pub fn pending_above_strategy_id(mut self, id: u64) -> Self {
        self.pending_above_strategy_id = Some(id);
        self
    }

    pub fn pending_above_strategy_type(mut self, strategy_type: i32) -> Self {
        self.pending_above_strategy_type = Some(strategy_type);
        self
    }

    pub fn pending_above_peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.pending_above_peg_price_type = Some(peg_price_type.to_string());
        self
    }

    pub fn pending_above_peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.pending_above_peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    pub fn pending_above_peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.pending_above_peg_offset_value = Some(peg_offset_value);
        self
    }

    pub fn pending_below_type(mut self, order_type: OrderType) -> Self {
        self.pending_below_type = Some(order_type);
        self
    }

    pub fn pending_below_client_order_id(mut self, id: &str) -> Self {
        self.pending_below_client_order_id = Some(id.to_string());
        self
    }

    pub fn pending_below_price(mut self, price: &str) -> Self {
        self.pending_below_price = Some(price.to_string());
        self
    }

    pub fn pending_below_stop_price(mut self, price: &str) -> Self {
        self.pending_below_stop_price = Some(price.to_string());
        self
    }

    pub fn pending_below_trailing_delta(mut self, delta: u64) -> Self {
        self.pending_below_trailing_delta = Some(delta);
        self
    }

    pub fn pending_below_iceberg_qty(mut self, qty: &str) -> Self {
        self.pending_below_iceberg_qty = Some(qty.to_string());
        self
    }

    pub fn pending_below_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.pending_below_time_in_force = Some(tif);
        self
    }

    pub fn pending_below_strategy_id(mut self, id: u64) -> Self {
        self.pending_below_strategy_id = Some(id);
        self
    }

    pub fn pending_below_strategy_type(mut self, strategy_type: i32) -> Self {
        self.pending_below_strategy_type = Some(strategy_type);
        self
    }

    pub fn pending_below_peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.pending_below_peg_price_type = Some(peg_price_type.to_string());
        self
    }

    pub fn pending_below_peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.pending_below_peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    pub fn pending_below_peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.pending_below_peg_offset_value = Some(peg_offset_value);
        self
    }

    pub fn build(self) -> NewOtocoOrder {
        NewOtocoOrder {
            symbol: self.symbol,
            working_type: self.working_type,
            working_side: self.working_side,
            working_price: self.working_price,
            working_quantity: self.working_quantity,
            pending_side: self.pending_side,
            pending_quantity: Some(self.pending_quantity),
            pending_above_type: self.pending_above_type,
            list_client_order_id: self.list_client_order_id,
            response_type: self.response_type,
            self_trade_prevention_mode: self.self_trade_prevention_mode,
            working_client_order_id: self.working_client_order_id,
            working_iceberg_qty: self.working_iceberg_qty,
            working_time_in_force: self.working_time_in_force,
            working_strategy_id: self.working_strategy_id,
            working_strategy_type: self.working_strategy_type,
            working_peg_price_type: self.working_peg_price_type,
            working_peg_offset_type: self.working_peg_offset_type,
            working_peg_offset_value: self.working_peg_offset_value,
            pending_above_client_order_id: self.pending_above_client_order_id,
            pending_above_price: self.pending_above_price,
            pending_above_stop_price: self.pending_above_stop_price,
            pending_above_trailing_delta: self.pending_above_trailing_delta,
            pending_above_iceberg_qty: self.pending_above_iceberg_qty,
            pending_above_time_in_force: self.pending_above_time_in_force,
            pending_above_strategy_id: self.pending_above_strategy_id,
            pending_above_strategy_type: self.pending_above_strategy_type,
            pending_above_peg_price_type: self.pending_above_peg_price_type,
            pending_above_peg_offset_type: self.pending_above_peg_offset_type,
            pending_above_peg_offset_value: self.pending_above_peg_offset_value,
            pending_below_type: self.pending_below_type,
            pending_below_client_order_id: self.pending_below_client_order_id,
            pending_below_price: self.pending_below_price,
            pending_below_stop_price: self.pending_below_stop_price,
            pending_below_trailing_delta: self.pending_below_trailing_delta,
            pending_below_iceberg_qty: self.pending_below_iceberg_qty,
            pending_below_time_in_force: self.pending_below_time_in_force,
            pending_below_strategy_id: self.pending_below_strategy_id,
            pending_below_strategy_type: self.pending_below_strategy_type,
            pending_below_peg_price_type: self.pending_below_peg_price_type,
            pending_below_peg_offset_type: self.pending_below_peg_offset_type,
            pending_below_peg_offset_value: self.pending_below_peg_offset_value,
        }
    }
}

/// New OTOCO order list parameters.
#[derive(Debug, Clone)]
pub struct NewOtocoOrder {
    symbol: String,
    working_type: OrderType,
    working_side: OrderSide,
    working_price: String,
    working_quantity: String,
    pending_side: OrderSide,
    pending_quantity: Option<String>,
    pending_above_type: OrderType,
    list_client_order_id: Option<String>,
    response_type: Option<OrderResponseType>,
    self_trade_prevention_mode: Option<String>,
    working_client_order_id: Option<String>,
    working_iceberg_qty: Option<String>,
    working_time_in_force: Option<TimeInForce>,
    working_strategy_id: Option<u64>,
    working_strategy_type: Option<i32>,
    working_peg_price_type: Option<String>,
    working_peg_offset_type: Option<String>,
    working_peg_offset_value: Option<i32>,
    pending_above_client_order_id: Option<String>,
    pending_above_price: Option<String>,
    pending_above_stop_price: Option<String>,
    pending_above_trailing_delta: Option<u64>,
    pending_above_iceberg_qty: Option<String>,
    pending_above_time_in_force: Option<TimeInForce>,
    pending_above_strategy_id: Option<u64>,
    pending_above_strategy_type: Option<i32>,
    pending_above_peg_price_type: Option<String>,
    pending_above_peg_offset_type: Option<String>,
    pending_above_peg_offset_value: Option<i32>,
    pending_below_type: Option<OrderType>,
    pending_below_client_order_id: Option<String>,
    pending_below_price: Option<String>,
    pending_below_stop_price: Option<String>,
    pending_below_trailing_delta: Option<u64>,
    pending_below_iceberg_qty: Option<String>,
    pending_below_time_in_force: Option<TimeInForce>,
    pending_below_strategy_id: Option<u64>,
    pending_below_strategy_type: Option<i32>,
    pending_below_peg_price_type: Option<String>,
    pending_below_peg_offset_type: Option<String>,
    pending_below_peg_offset_value: Option<i32>,
}

impl NewOtocoOrder {
    fn to_params(&self) -> Vec<(String, String)> {
        let mut params = vec![
            ("symbol".to_string(), self.symbol.clone()),
            (
                "workingType".to_string(),
                format!("{:?}", self.working_type).to_uppercase(),
            ),
            (
                "workingSide".to_string(),
                format!("{:?}", self.working_side).to_uppercase(),
            ),
            ("workingPrice".to_string(), self.working_price.clone()),
            ("workingQuantity".to_string(), self.working_quantity.clone()),
            (
                "pendingSide".to_string(),
                format!("{:?}", self.pending_side).to_uppercase(),
            ),
            (
                "pendingAboveType".to_string(),
                format!("{:?}", self.pending_above_type).to_uppercase(),
            ),
        ];

        if let Some(ref qty) = self.pending_quantity {
            params.push(("pendingQuantity".to_string(), qty.clone()));
        }
        if let Some(ref id) = self.list_client_order_id {
            params.push(("listClientOrderId".to_string(), id.clone()));
        }
        if let Some(ref resp) = self.response_type {
            params.push((
                "newOrderRespType".to_string(),
                format!("{:?}", resp).to_uppercase(),
            ));
        }
        if let Some(ref mode) = self.self_trade_prevention_mode {
            params.push(("selfTradePreventionMode".to_string(), mode.clone()));
        }
        if let Some(ref id) = self.working_client_order_id {
            params.push(("workingClientOrderId".to_string(), id.clone()));
        }
        if let Some(ref qty) = self.working_iceberg_qty {
            params.push(("workingIcebergQty".to_string(), qty.clone()));
        }
        if let Some(ref tif) = self.working_time_in_force {
            params.push(("workingTimeInForce".to_string(), format!("{:?}", tif)));
        }
        if let Some(id) = self.working_strategy_id {
            params.push(("workingStrategyId".to_string(), id.to_string()));
        }
        if let Some(id) = self.working_strategy_type {
            params.push(("workingStrategyType".to_string(), id.to_string()));
        }
        if let Some(ref peg) = self.working_peg_price_type {
            params.push(("workingPegPriceType".to_string(), peg.clone()));
        }
        if let Some(ref peg) = self.working_peg_offset_type {
            params.push(("workingPegOffsetType".to_string(), peg.clone()));
        }
        if let Some(value) = self.working_peg_offset_value {
            params.push(("workingPegOffsetValue".to_string(), value.to_string()));
        }
        if let Some(ref id) = self.pending_above_client_order_id {
            params.push(("pendingAboveClientOrderId".to_string(), id.clone()));
        }
        if let Some(ref price) = self.pending_above_price {
            params.push(("pendingAbovePrice".to_string(), price.clone()));
        }
        if let Some(ref price) = self.pending_above_stop_price {
            params.push(("pendingAboveStopPrice".to_string(), price.clone()));
        }
        if let Some(delta) = self.pending_above_trailing_delta {
            params.push(("pendingAboveTrailingDelta".to_string(), delta.to_string()));
        }
        if let Some(ref qty) = self.pending_above_iceberg_qty {
            params.push(("pendingAboveIcebergQty".to_string(), qty.clone()));
        }
        if let Some(ref tif) = self.pending_above_time_in_force {
            params.push(("pendingAboveTimeInForce".to_string(), format!("{:?}", tif)));
        }
        if let Some(id) = self.pending_above_strategy_id {
            params.push(("pendingAboveStrategyId".to_string(), id.to_string()));
        }
        if let Some(id) = self.pending_above_strategy_type {
            params.push(("pendingAboveStrategyType".to_string(), id.to_string()));
        }
        if let Some(ref peg) = self.pending_above_peg_price_type {
            params.push(("pendingAbovePegPriceType".to_string(), peg.clone()));
        }
        if let Some(ref peg) = self.pending_above_peg_offset_type {
            params.push(("pendingAbovePegOffsetType".to_string(), peg.clone()));
        }
        if let Some(value) = self.pending_above_peg_offset_value {
            params.push(("pendingAbovePegOffsetValue".to_string(), value.to_string()));
        }
        if let Some(order_type) = self.pending_below_type {
            params.push((
                "pendingBelowType".to_string(),
                format!("{:?}", order_type).to_uppercase(),
            ));
        }
        if let Some(ref id) = self.pending_below_client_order_id {
            params.push(("pendingBelowClientOrderId".to_string(), id.clone()));
        }
        if let Some(ref price) = self.pending_below_price {
            params.push(("pendingBelowPrice".to_string(), price.clone()));
        }
        if let Some(ref price) = self.pending_below_stop_price {
            params.push(("pendingBelowStopPrice".to_string(), price.clone()));
        }
        if let Some(delta) = self.pending_below_trailing_delta {
            params.push(("pendingBelowTrailingDelta".to_string(), delta.to_string()));
        }
        if let Some(ref qty) = self.pending_below_iceberg_qty {
            params.push(("pendingBelowIcebergQty".to_string(), qty.clone()));
        }
        if let Some(ref tif) = self.pending_below_time_in_force {
            params.push(("pendingBelowTimeInForce".to_string(), format!("{:?}", tif)));
        }
        if let Some(id) = self.pending_below_strategy_id {
            params.push(("pendingBelowStrategyId".to_string(), id.to_string()));
        }
        if let Some(id) = self.pending_below_strategy_type {
            params.push(("pendingBelowStrategyType".to_string(), id.to_string()));
        }
        if let Some(ref peg) = self.pending_below_peg_price_type {
            params.push(("pendingBelowPegPriceType".to_string(), peg.clone()));
        }
        if let Some(ref peg) = self.pending_below_peg_offset_type {
            params.push(("pendingBelowPegOffsetType".to_string(), peg.clone()));
        }
        if let Some(value) = self.pending_below_peg_offset_value {
            params.push(("pendingBelowPegOffsetValue".to_string(), value.to_string()));
        }

        params
    }
}

/// Builder for creating OPOCO order lists.
#[derive(Debug, Clone)]
pub struct OpocoOrderBuilder {
    inner: NewOtocoOrder,
}

impl OpocoOrderBuilder {
    /// Create a new OPOCO order list builder.
    pub fn new(
        symbol: &str,
        working_type: OrderType,
        working_side: OrderSide,
        working_price: &str,
        working_quantity: &str,
        pending_side: OrderSide,
        pending_above_type: OrderType,
    ) -> Self {
        Self {
            inner: NewOtocoOrder {
                symbol: symbol.to_string(),
                working_type,
                working_side,
                working_price: working_price.to_string(),
                working_quantity: working_quantity.to_string(),
                pending_side,
                pending_quantity: None,
                pending_above_type,
                list_client_order_id: None,
                response_type: None,
                self_trade_prevention_mode: None,
                working_client_order_id: None,
                working_iceberg_qty: None,
                working_time_in_force: None,
                working_strategy_id: None,
                working_strategy_type: None,
                working_peg_price_type: None,
                working_peg_offset_type: None,
                working_peg_offset_value: None,
                pending_above_client_order_id: None,
                pending_above_price: None,
                pending_above_stop_price: None,
                pending_above_trailing_delta: None,
                pending_above_iceberg_qty: None,
                pending_above_time_in_force: None,
                pending_above_strategy_id: None,
                pending_above_strategy_type: None,
                pending_above_peg_price_type: None,
                pending_above_peg_offset_type: None,
                pending_above_peg_offset_value: None,
                pending_below_type: None,
                pending_below_client_order_id: None,
                pending_below_price: None,
                pending_below_stop_price: None,
                pending_below_trailing_delta: None,
                pending_below_iceberg_qty: None,
                pending_below_time_in_force: None,
                pending_below_strategy_id: None,
                pending_below_strategy_type: None,
                pending_below_peg_price_type: None,
                pending_below_peg_offset_type: None,
                pending_below_peg_offset_value: None,
            },
        }
    }

    pub fn list_client_order_id(mut self, id: &str) -> Self {
        self.inner.list_client_order_id = Some(id.to_string());
        self
    }

    pub fn response_type(mut self, resp_type: OrderResponseType) -> Self {
        self.inner.response_type = Some(resp_type);
        self
    }

    pub fn self_trade_prevention_mode(mut self, mode: &str) -> Self {
        self.inner.self_trade_prevention_mode = Some(mode.to_string());
        self
    }

    pub fn working_client_order_id(mut self, id: &str) -> Self {
        self.inner.working_client_order_id = Some(id.to_string());
        self
    }

    pub fn working_iceberg_qty(mut self, qty: &str) -> Self {
        self.inner.working_iceberg_qty = Some(qty.to_string());
        self
    }

    pub fn working_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.inner.working_time_in_force = Some(tif);
        self
    }

    pub fn working_strategy_id(mut self, id: u64) -> Self {
        self.inner.working_strategy_id = Some(id);
        self
    }

    pub fn working_strategy_type(mut self, strategy_type: i32) -> Self {
        self.inner.working_strategy_type = Some(strategy_type);
        self
    }

    pub fn working_peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.inner.working_peg_price_type = Some(peg_price_type.to_string());
        self
    }

    pub fn working_peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.inner.working_peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    pub fn working_peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.inner.working_peg_offset_value = Some(peg_offset_value);
        self
    }

    pub fn pending_quantity(mut self, qty: &str) -> Self {
        self.inner.pending_quantity = Some(qty.to_string());
        self
    }

    pub fn pending_above_client_order_id(mut self, id: &str) -> Self {
        self.inner.pending_above_client_order_id = Some(id.to_string());
        self
    }

    pub fn pending_above_price(mut self, price: &str) -> Self {
        self.inner.pending_above_price = Some(price.to_string());
        self
    }

    pub fn pending_above_stop_price(mut self, price: &str) -> Self {
        self.inner.pending_above_stop_price = Some(price.to_string());
        self
    }

    pub fn pending_above_trailing_delta(mut self, delta: u64) -> Self {
        self.inner.pending_above_trailing_delta = Some(delta);
        self
    }

    pub fn pending_above_iceberg_qty(mut self, qty: &str) -> Self {
        self.inner.pending_above_iceberg_qty = Some(qty.to_string());
        self
    }

    pub fn pending_above_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.inner.pending_above_time_in_force = Some(tif);
        self
    }

    pub fn pending_above_strategy_id(mut self, id: u64) -> Self {
        self.inner.pending_above_strategy_id = Some(id);
        self
    }

    pub fn pending_above_strategy_type(mut self, strategy_type: i32) -> Self {
        self.inner.pending_above_strategy_type = Some(strategy_type);
        self
    }

    pub fn pending_above_peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.inner.pending_above_peg_price_type = Some(peg_price_type.to_string());
        self
    }

    pub fn pending_above_peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.inner.pending_above_peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    pub fn pending_above_peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.inner.pending_above_peg_offset_value = Some(peg_offset_value);
        self
    }

    pub fn pending_below_type(mut self, order_type: OrderType) -> Self {
        self.inner.pending_below_type = Some(order_type);
        self
    }

    pub fn pending_below_client_order_id(mut self, id: &str) -> Self {
        self.inner.pending_below_client_order_id = Some(id.to_string());
        self
    }

    pub fn pending_below_price(mut self, price: &str) -> Self {
        self.inner.pending_below_price = Some(price.to_string());
        self
    }

    pub fn pending_below_stop_price(mut self, price: &str) -> Self {
        self.inner.pending_below_stop_price = Some(price.to_string());
        self
    }

    pub fn pending_below_trailing_delta(mut self, delta: u64) -> Self {
        self.inner.pending_below_trailing_delta = Some(delta);
        self
    }

    pub fn pending_below_iceberg_qty(mut self, qty: &str) -> Self {
        self.inner.pending_below_iceberg_qty = Some(qty.to_string());
        self
    }

    pub fn pending_below_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.inner.pending_below_time_in_force = Some(tif);
        self
    }

    pub fn pending_below_strategy_id(mut self, id: u64) -> Self {
        self.inner.pending_below_strategy_id = Some(id);
        self
    }

    pub fn pending_below_strategy_type(mut self, strategy_type: i32) -> Self {
        self.inner.pending_below_strategy_type = Some(strategy_type);
        self
    }

    pub fn pending_below_peg_price_type(mut self, peg_price_type: &str) -> Self {
        self.inner.pending_below_peg_price_type = Some(peg_price_type.to_string());
        self
    }

    pub fn pending_below_peg_offset_type(mut self, peg_offset_type: &str) -> Self {
        self.inner.pending_below_peg_offset_type = Some(peg_offset_type.to_string());
        self
    }

    pub fn pending_below_peg_offset_value(mut self, peg_offset_value: i32) -> Self {
        self.inner.pending_below_peg_offset_value = Some(peg_offset_value);
        self
    }

    pub fn build(self) -> NewOpocoOrder {
        NewOpocoOrder { inner: self.inner }
    }
}

/// New OPOCO order list parameters.
#[derive(Debug, Clone)]
pub struct NewOpocoOrder {
    inner: NewOtocoOrder,
}

impl NewOpocoOrder {
    fn to_params(&self) -> Vec<(String, String)> {
        self.inner.to_params()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_builder_limit() {
        let order = OrderBuilder::new("BTCUSDT", OrderSide::Buy, OrderType::Limit)
            .quantity("0.001")
            .price("50000.00")
            .time_in_force(TimeInForce::GTC)
            .build();

        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.quantity, Some("0.001".to_string()));
        assert_eq!(order.price, Some("50000.00".to_string()));
        assert_eq!(order.time_in_force, Some(TimeInForce::GTC));
    }

    #[test]
    fn test_order_builder_market() {
        let order = OrderBuilder::new("BTCUSDT", OrderSide::Sell, OrderType::Market)
            .quantity("1.0")
            .build();

        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.side, OrderSide::Sell);
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.quantity, Some("1.0".to_string()));
        assert!(order.price.is_none());
    }

    #[test]
    fn test_order_to_params() {
        let order = OrderBuilder::new("BTCUSDT", OrderSide::Buy, OrderType::Limit)
            .quantity("0.001")
            .price("50000.00")
            .time_in_force(TimeInForce::GTC)
            .build();

        let params = order.to_params();

        assert!(params.iter().any(|(k, v)| k == "symbol" && v == "BTCUSDT"));
        assert!(params.iter().any(|(k, v)| k == "side" && v == "BUY"));
        assert!(params.iter().any(|(k, v)| k == "type" && v == "LIMIT"));
        assert!(params.iter().any(|(k, v)| k == "quantity" && v == "0.001"));
        assert!(params.iter().any(|(k, v)| k == "price" && v == "50000.00"));
    }

    #[test]
    fn test_oco_order_builder() {
        let order = OcoOrderBuilder::new("BTCUSDT", OrderSide::Sell, "1.0", "55000.00", "48000.00")
            .stop_limit_price("47900.00")
            .stop_limit_time_in_force(TimeInForce::GTC)
            .build();

        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.side, OrderSide::Sell);
        assert_eq!(order.quantity, "1.0");
        assert_eq!(order.price, "55000.00");
        assert_eq!(order.stop_price, "48000.00");
        assert_eq!(order.stop_limit_price, Some("47900.00".to_string()));
    }
}
