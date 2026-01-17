//! Account and trading models.
//!
//! These models represent responses from authenticated account and trading endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{
    AccountType, CancelReplaceResult, ContingencyType, OcoOrderStatus, OcoStatus, OrderSide,
    OrderStatus, OrderType, TimeInForce,
};

use super::market::string_or_float;

/// Account information response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    /// Maker commission rate (bps).
    pub maker_commission: i32,
    /// Taker commission rate (bps).
    pub taker_commission: i32,
    /// Buyer commission rate (bps).
    pub buyer_commission: i32,
    /// Seller commission rate (bps).
    pub seller_commission: i32,
    /// Commission rates for specific assets.
    #[serde(default)]
    pub commission_rates: Option<CommissionRates>,
    /// Whether trading is enabled.
    pub can_trade: bool,
    /// Whether withdrawals are enabled.
    pub can_withdraw: bool,
    /// Whether deposits are enabled.
    pub can_deposit: bool,
    /// Whether self-trade prevention is being used.
    #[serde(default)]
    pub brokered: bool,
    /// Whether this account requires self-trade prevention.
    #[serde(default)]
    pub require_self_trade_prevention: bool,
    /// Account update time.
    pub update_time: u64,
    /// Account type.
    pub account_type: AccountType,
    /// Account balances.
    pub balances: Vec<Balance>,
    /// Account permissions.
    #[serde(default)]
    pub permissions: Vec<AccountType>,
    /// UID (user ID).
    #[serde(default)]
    pub uid: Option<u64>,
}

/// Commission rates.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRates {
    /// Maker rate.
    #[serde(with = "string_or_float")]
    pub maker: f64,
    /// Taker rate.
    #[serde(with = "string_or_float")]
    pub taker: f64,
    /// Buyer rate.
    #[serde(with = "string_or_float")]
    pub buyer: f64,
    /// Seller rate.
    #[serde(with = "string_or_float")]
    pub seller: f64,
}

/// Commission rate details for a symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionRateDetail {
    /// Maker rate.
    #[serde(with = "string_or_float")]
    pub maker: f64,
    /// Taker rate.
    #[serde(with = "string_or_float")]
    pub taker: f64,
    /// Buyer rate.
    #[serde(with = "string_or_float")]
    pub buyer: f64,
    /// Seller rate.
    #[serde(with = "string_or_float")]
    pub seller: f64,
}

/// Discount commission information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommissionDiscount {
    /// Whether discount is enabled for account.
    pub enabled_for_account: bool,
    /// Whether discount is enabled for symbol.
    pub enabled_for_symbol: bool,
    /// Discount asset (e.g., "BNB").
    pub discount_asset: String,
    /// Discount rate.
    #[serde(with = "string_or_float")]
    pub discount: f64,
}

/// Account commission rates for a symbol.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountCommission {
    /// Symbol.
    pub symbol: String,
    /// Standard commission rates from the order.
    pub standard_commission: CommissionRateDetail,
    /// Special commission rates from the order.
    pub special_commission: CommissionRateDetail,
    /// Tax commission rates from the order.
    pub tax_commission: CommissionRateDetail,
    /// Discount information for BNB commissions.
    pub discount: CommissionDiscount,
}

/// Commission rate detail for SOR test orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCommissionRate {
    /// Maker rate.
    #[serde(with = "string_or_float")]
    pub maker: f64,
    /// Taker rate.
    #[serde(with = "string_or_float")]
    pub taker: f64,
}

/// Commission rates returned by test SOR order when enabled.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SorOrderCommissionRates {
    /// Standard commission rates for the order.
    pub standard_commission_for_order: OrderCommissionRate,
    /// Tax commission rates for the order.
    pub tax_commission_for_order: OrderCommissionRate,
    /// Discount information for BNB commissions.
    pub discount: CommissionDiscount,
}

/// Test SOR order response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SorOrderTestResponse {
    /// Empty response when commission rates are not requested.
    Empty(EmptyResponse),
    /// Commission rates response.
    Rates(SorOrderCommissionRates),
}

/// Prevented match entry (self-trade prevention).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreventedMatch {
    /// Symbol.
    pub symbol: String,
    /// Prevented match ID.
    pub prevented_match_id: u64,
    /// Taker order ID.
    pub taker_order_id: u64,
    /// Maker symbol.
    pub maker_symbol: String,
    /// Maker order ID.
    pub maker_order_id: u64,
    /// Trade group ID.
    pub trade_group_id: u64,
    /// Self-trade prevention mode.
    pub self_trade_prevention_mode: String,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Maker prevented quantity.
    #[serde(with = "string_or_float")]
    pub maker_prevented_quantity: f64,
    /// Transaction time.
    pub transact_time: u64,
}

/// Allocation entry from SOR order placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Allocation {
    /// Symbol.
    pub symbol: String,
    /// Allocation ID.
    pub allocation_id: u64,
    /// Allocation type (e.g., "SOR").
    pub allocation_type: String,
    /// Order ID.
    pub order_id: u64,
    /// Order list ID.
    pub order_list_id: i64,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Quantity.
    #[serde(with = "string_or_float")]
    pub qty: f64,
    /// Quote quantity.
    #[serde(with = "string_or_float")]
    pub quote_qty: f64,
    /// Commission.
    #[serde(with = "string_or_float")]
    pub commission: f64,
    /// Commission asset.
    pub commission_asset: String,
    /// Time.
    pub time: u64,
    /// Whether the allocation is buyer side.
    pub is_buyer: bool,
    /// Whether the allocation is maker side.
    pub is_maker: bool,
    /// Whether this allocation is allocator.
    pub is_allocator: bool,
}

/// Account balance for a single asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Balance {
    /// Asset symbol (e.g., "BTC").
    pub asset: String,
    /// Free (available) balance.
    #[serde(with = "string_or_float")]
    pub free: f64,
    /// Locked balance (in orders).
    #[serde(with = "string_or_float")]
    pub locked: f64,
}

impl Balance {
    /// Get the total balance (free + locked).
    pub fn total(&self) -> f64 {
        self.free + self.locked
    }

    /// Check if the balance is zero.
    pub fn is_zero(&self) -> bool {
        self.free == 0.0 && self.locked == 0.0
    }
}

/// Order information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Symbol.
    pub symbol: String,
    /// Order ID.
    pub order_id: u64,
    /// Order list ID (-1 if not part of an order list).
    pub order_list_id: i64,
    /// Client order ID.
    pub client_order_id: String,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Original quantity.
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    /// Executed quantity.
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    /// Cumulative quote quantity.
    #[serde(with = "string_or_float")]
    pub cummulative_quote_qty: f64,
    /// Order status.
    pub status: OrderStatus,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order side.
    pub side: OrderSide,
    /// Stop price.
    #[serde(with = "string_or_float")]
    pub stop_price: f64,
    /// Iceberg quantity.
    #[serde(with = "string_or_float")]
    pub iceberg_qty: f64,
    /// Order creation time.
    pub time: u64,
    /// Order update time.
    pub update_time: u64,
    /// Is the order working.
    pub is_working: bool,
    /// Original quote order quantity.
    #[serde(with = "string_or_float")]
    pub orig_quote_order_qty: f64,
    /// Working time.
    #[serde(default)]
    pub working_time: Option<u64>,
    /// Self-trade prevention mode.
    #[serde(default)]
    pub self_trade_prevention_mode: Option<String>,
}

impl Order {
    /// Get the average fill price.
    pub fn avg_price(&self) -> Option<f64> {
        if self.executed_qty > 0.0 {
            Some(self.cummulative_quote_qty / self.executed_qty)
        } else {
            None
        }
    }

    /// Check if the order is fully filled.
    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled
    }

    /// Check if the order is still active.
    pub fn is_active(&self) -> bool {
        matches!(self.status, OrderStatus::New | OrderStatus::PartiallyFilled)
    }
}

/// New order response (ACK type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderAck {
    /// Symbol.
    pub symbol: String,
    /// Order ID.
    pub order_id: u64,
    /// Order list ID.
    #[serde(default)]
    pub order_list_id: i64,
    /// Client order ID.
    pub client_order_id: String,
    /// Transaction time.
    pub transact_time: u64,
}

/// New order response (RESULT type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderResult {
    /// Symbol.
    pub symbol: String,
    /// Order ID.
    pub order_id: u64,
    /// Order list ID.
    #[serde(default)]
    pub order_list_id: i64,
    /// Client order ID.
    pub client_order_id: String,
    /// Transaction time.
    pub transact_time: u64,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Original quantity.
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    /// Executed quantity.
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    /// Cumulative quote quantity.
    #[serde(with = "string_or_float")]
    pub cummulative_quote_qty: f64,
    /// Order status.
    pub status: OrderStatus,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order side.
    pub side: OrderSide,
    /// Working time.
    #[serde(default)]
    pub working_time: Option<u64>,
    /// Self-trade prevention mode.
    #[serde(default)]
    pub self_trade_prevention_mode: Option<String>,
}

/// New order response (FULL type).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderFull {
    /// Symbol.
    pub symbol: String,
    /// Order ID.
    pub order_id: u64,
    /// Order list ID.
    #[serde(default)]
    pub order_list_id: i64,
    /// Client order ID.
    pub client_order_id: String,
    /// Transaction time.
    pub transact_time: u64,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Original quantity.
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    /// Executed quantity.
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    /// Cumulative quote quantity.
    #[serde(with = "string_or_float")]
    pub cummulative_quote_qty: f64,
    /// Order status.
    pub status: OrderStatus,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order side.
    pub side: OrderSide,
    /// Working time.
    #[serde(default)]
    pub working_time: Option<u64>,
    /// Self-trade prevention mode.
    #[serde(default)]
    pub self_trade_prevention_mode: Option<String>,
    /// Fills (trades that filled this order).
    #[serde(default)]
    pub fills: Vec<Fill>,
}

/// Order fill information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
    /// Fill price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Fill quantity.
    #[serde(rename = "qty", with = "string_or_float")]
    pub quantity: f64,
    /// Commission amount.
    #[serde(with = "string_or_float")]
    pub commission: f64,
    /// Commission asset.
    pub commission_asset: String,
    /// Trade ID.
    #[serde(default)]
    pub trade_id: Option<u64>,
}

/// Cancel order response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderResponse {
    /// Symbol.
    pub symbol: String,
    /// Original client order ID.
    pub orig_client_order_id: String,
    /// Order ID.
    pub order_id: u64,
    /// Order list ID.
    #[serde(default)]
    pub order_list_id: i64,
    /// Client order ID.
    pub client_order_id: String,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Original quantity.
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    /// Executed quantity.
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    /// Cumulative quote quantity.
    #[serde(with = "string_or_float")]
    pub cummulative_quote_qty: f64,
    /// Order status.
    pub status: OrderStatus,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order side.
    pub side: OrderSide,
    /// Self-trade prevention mode.
    #[serde(default)]
    pub self_trade_prevention_mode: Option<String>,
}

/// Cancel-replace error info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelReplaceErrorInfo {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub msg: String,
}

/// Cancel-replace response payload for a failed request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelReplaceErrorData {
    /// Cancel result.
    pub cancel_result: CancelReplaceResult,
    /// New order result.
    pub new_order_result: CancelReplaceResult,
    /// Cancel response payload.
    pub cancel_response: CancelReplaceSideResponse,
    /// New order response payload.
    pub new_order_response: Option<CancelReplaceSideResponse>,
}

/// Cancel-replace error response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelReplaceErrorResponse {
    /// Error code.
    pub code: i32,
    /// Error message.
    pub msg: String,
    /// Error data payload.
    pub data: CancelReplaceErrorData,
}

/// Cancel-replace sub-response payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CancelReplaceSideResponse {
    /// Error response.
    Error(CancelReplaceErrorInfo),
    /// Cancel order response.
    Cancel(CancelOrderResponse),
    /// New order response.
    Order(OrderResponse),
}

/// Cancel-replace order response for a successful request.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelReplaceResponse {
    /// Cancel result.
    pub cancel_result: CancelReplaceResult,
    /// New order result.
    pub new_order_result: CancelReplaceResult,
    /// Cancel order response.
    pub cancel_response: CancelOrderResponse,
    /// New order response.
    pub new_order_response: OrderResponse,
}

/// New order response variants for cancel-replace.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OrderResponse {
    /// ACK response.
    Ack(OrderAck),
    /// RESULT response.
    Result(OrderResult),
    /// FULL response.
    Full(OrderFull),
}

/// User trade (my trades).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTrade {
    /// Symbol.
    pub symbol: String,
    /// Trade ID.
    pub id: u64,
    /// Order ID.
    pub order_id: u64,
    /// Order list ID.
    #[serde(default)]
    pub order_list_id: i64,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Quantity.
    #[serde(rename = "qty", with = "string_or_float")]
    pub quantity: f64,
    /// Quote quantity.
    #[serde(rename = "quoteQty", with = "string_or_float")]
    pub quote_quantity: f64,
    /// Commission.
    #[serde(with = "string_or_float")]
    pub commission: f64,
    /// Commission asset.
    pub commission_asset: String,
    /// Trade time.
    pub time: u64,
    /// Was the buyer.
    pub is_buyer: bool,
    /// Was the maker.
    pub is_maker: bool,
    /// Was best match.
    pub is_best_match: bool,
}

/// OCO order information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcoOrder {
    /// Order list ID.
    pub order_list_id: u64,
    /// Contingency type.
    pub contingency_type: ContingencyType,
    /// List status type.
    pub list_status_type: OcoStatus,
    /// List order status.
    pub list_order_status: OcoOrderStatus,
    /// List client order ID.
    pub list_client_order_id: String,
    /// Transaction time.
    pub transaction_time: u64,
    /// Symbol.
    pub symbol: String,
    /// Orders in this OCO.
    pub orders: Vec<OcoOrderDetail>,
    /// Order reports (detailed info about each order).
    #[serde(default)]
    pub order_reports: Vec<OcoOrderReport>,
}

/// OCO order detail (minimal info).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcoOrderDetail {
    /// Symbol.
    pub symbol: String,
    /// Order ID.
    pub order_id: u64,
    /// Client order ID.
    pub client_order_id: String,
}

/// OCO order report (detailed info).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcoOrderReport {
    /// Symbol.
    pub symbol: String,
    /// Order ID.
    pub order_id: u64,
    /// Order list ID.
    pub order_list_id: i64,
    /// Client order ID.
    pub client_order_id: String,
    /// Transaction time.
    pub transact_time: u64,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Original quantity.
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    /// Executed quantity.
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    /// Cumulative quote quantity.
    #[serde(with = "string_or_float")]
    pub cummulative_quote_qty: f64,
    /// Order status.
    pub status: OrderStatus,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order side.
    pub side: OrderSide,
    /// Stop price.
    #[serde(default, with = "super::market::string_or_float_opt")]
    pub stop_price: Option<f64>,
    /// Self-trade prevention mode.
    #[serde(default)]
    pub self_trade_prevention_mode: Option<String>,
}

/// User data stream listen key response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKey {
    /// The listen key.
    pub listen_key: String,
}

/// Empty response (used for endpoints that return {}).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmptyResponse {}

/// Unfilled order count for a rate limit interval.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnfilledOrderCount {
    /// Rate limit type (always "ORDERS").
    pub rate_limit_type: String,
    /// Time interval (e.g., "SECOND", "DAY").
    pub interval: String,
    /// Interval number (e.g., 10 for "10 seconds").
    pub interval_num: u32,
    /// Maximum allowed orders in this interval.
    pub limit: u32,
    /// Current count of unfilled orders.
    pub count: u32,
}

/// Order amendment history entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderAmendment {
    /// Symbol.
    pub symbol: String,
    /// Order ID.
    pub order_id: u64,
    /// Execution ID.
    pub execution_id: u64,
    /// Original client order ID.
    pub orig_client_order_id: String,
    /// New client order ID after amendment.
    pub new_client_order_id: String,
    /// Original quantity before amendment.
    #[serde(with = "string_or_float")]
    pub orig_qty: f64,
    /// New quantity after amendment.
    #[serde(with = "string_or_float")]
    pub new_qty: f64,
    /// Amendment time.
    pub time: u64,
}

/// Amended order details returned from amend endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmendedOrderInfo {
    /// Symbol.
    pub symbol: String,
    /// Order ID.
    pub order_id: u64,
    /// Order list ID (-1 if not part of a list).
    pub order_list_id: i64,
    /// Original client order ID.
    pub orig_client_order_id: String,
    /// Client order ID after amendment.
    pub client_order_id: String,
    /// Order price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Order quantity after amendment.
    #[serde(rename = "qty", with = "string_or_float")]
    pub quantity: f64,
    /// Executed quantity.
    #[serde(with = "string_or_float")]
    pub executed_qty: f64,
    /// Prevented quantity.
    #[serde(with = "string_or_float")]
    pub prevented_qty: f64,
    /// Quote order quantity.
    #[serde(with = "string_or_float")]
    pub quote_order_qty: f64,
    /// Cumulative quote quantity.
    #[serde(with = "string_or_float")]
    pub cumulative_quote_qty: f64,
    /// Order status.
    pub status: OrderStatus,
    /// Time in force.
    pub time_in_force: TimeInForce,
    /// Order type.
    #[serde(rename = "type")]
    pub order_type: OrderType,
    /// Order side.
    pub side: OrderSide,
    /// Working time.
    #[serde(default)]
    pub working_time: Option<u64>,
    /// Self-trade prevention mode.
    #[serde(default)]
    pub self_trade_prevention_mode: Option<String>,
}

/// Order list status for amended orders that are part of order lists.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmendListStatus {
    /// Order list ID.
    pub order_list_id: u64,
    /// Contingency type (e.g., "OTO", "OCO").
    pub contingency_type: ContingencyType,
    /// List order status.
    pub list_order_status: OcoOrderStatus,
    /// List client order ID.
    pub list_client_order_id: String,
    /// Symbol.
    pub symbol: String,
    /// Orders in this list.
    pub orders: Vec<OcoOrderDetail>,
}

/// Response from order amend keep priority endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AmendOrderResponse {
    /// Transaction time.
    pub transact_time: u64,
    /// Execution ID.
    pub execution_id: u64,
    /// Amended order details.
    pub amended_order: AmendedOrderInfo,
    /// List status (only present for orders that are part of order lists).
    #[serde(default)]
    pub list_status: Option<AmendListStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balance_deserialize() {
        let json = r#"{
            "asset": "BTC",
            "free": "1.5",
            "locked": "0.5"
        }"#;
        let balance: Balance = serde_json::from_str(json).unwrap();
        assert_eq!(balance.asset, "BTC");
        assert_eq!(balance.free, 1.5);
        assert_eq!(balance.locked, 0.5);
        assert_eq!(balance.total(), 2.0);
        assert!(!balance.is_zero());
    }

    #[test]
    fn test_balance_zero() {
        let balance = Balance {
            asset: "BTC".to_string(),
            free: 0.0,
            locked: 0.0,
        };
        assert!(balance.is_zero());
    }

    #[test]
    fn test_order_deserialize() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "orderId": 12345,
            "orderListId": -1,
            "clientOrderId": "test123",
            "price": "50000.00",
            "origQty": "1.0",
            "executedQty": "0.5",
            "cummulativeQuoteQty": "25000.00",
            "status": "PARTIALLY_FILLED",
            "timeInForce": "GTC",
            "type": "LIMIT",
            "side": "BUY",
            "stopPrice": "0.0",
            "icebergQty": "0.0",
            "time": 1234567890123,
            "updateTime": 1234567890123,
            "isWorking": true,
            "origQuoteOrderQty": "0.0"
        }"#;
        let order: Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.order_id, 12345);
        assert_eq!(order.price, 50000.0);
        assert_eq!(order.status, OrderStatus::PartiallyFilled);
        assert!(order.is_active());
        assert!(!order.is_filled());
        assert_eq!(order.avg_price(), Some(50000.0));
    }

    #[test]
    fn test_order_full_deserialize() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "orderId": 12345,
            "orderListId": -1,
            "clientOrderId": "test123",
            "transactTime": 1234567890123,
            "price": "50000.00",
            "origQty": "1.0",
            "executedQty": "1.0",
            "cummulativeQuoteQty": "50000.00",
            "status": "FILLED",
            "timeInForce": "GTC",
            "type": "LIMIT",
            "side": "BUY",
            "fills": [
                {
                    "price": "50000.00",
                    "qty": "1.0",
                    "commission": "0.001",
                    "commissionAsset": "BTC"
                }
            ]
        }"#;
        let order: OrderFull = serde_json::from_str(json).unwrap();
        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.status, OrderStatus::Filled);
        assert_eq!(order.fills.len(), 1);
        assert_eq!(order.fills[0].price, 50000.0);
        assert_eq!(order.fills[0].commission, 0.001);
    }

    #[test]
    fn test_user_trade_deserialize() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "id": 12345,
            "orderId": 67890,
            "price": "50000.00",
            "qty": "1.0",
            "quoteQty": "50000.00",
            "commission": "0.001",
            "commissionAsset": "BTC",
            "time": 1234567890123,
            "isBuyer": true,
            "isMaker": false,
            "isBestMatch": true
        }"#;
        let trade: UserTrade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.symbol, "BTCUSDT");
        assert_eq!(trade.id, 12345);
        assert_eq!(trade.price, 50000.0);
        assert!(trade.is_buyer);
        assert!(!trade.is_maker);
    }

    #[test]
    fn test_listen_key_deserialize() {
        let json = r#"{"listenKey": "abc123xyz"}"#;
        let key: ListenKey = serde_json::from_str(json).unwrap();
        assert_eq!(key.listen_key, "abc123xyz");
    }

    #[test]
    fn test_account_info_deserialize() {
        let json = r#"{
            "makerCommission": 10,
            "takerCommission": 10,
            "buyerCommission": 0,
            "sellerCommission": 0,
            "canTrade": true,
            "canWithdraw": true,
            "canDeposit": true,
            "updateTime": 1234567890123,
            "accountType": "SPOT",
            "balances": [
                {"asset": "BTC", "free": "1.0", "locked": "0.0"}
            ],
            "permissions": ["SPOT"]
        }"#;
        let account: AccountInfo = serde_json::from_str(json).unwrap();
        assert_eq!(account.maker_commission, 10);
        assert!(account.can_trade);
        assert_eq!(account.account_type, AccountType::Spot);
        assert_eq!(account.balances.len(), 1);
        assert_eq!(account.balances[0].asset, "BTC");
    }
}
