//! Common types used across the Binance API.
//!
//! This module contains enums and types that are shared between
//! different API endpoints.

use serde::{Deserialize, Serialize};

/// Order side (buy or sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    /// Buy order
    #[default]
    Buy,
    /// Sell order
    Sell,
}

/// Order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderType {
    /// Limit order - specify price and quantity
    Limit,
    /// Market order - execute at current market price
    #[default]
    Market,
    /// Stop loss order - triggers market order when stop price is reached
    StopLoss,
    /// Stop loss limit order - triggers limit order when stop price is reached
    StopLossLimit,
    /// Take profit order - triggers market order when target price is reached
    TakeProfit,
    /// Take profit limit order - triggers limit order when target price is reached
    TakeProfitLimit,
    /// Limit maker order - rejected if it would immediately match
    LimitMaker,
    /// Unknown order type
    #[serde(other)]
    Other,
}

/// Time in force - how long an order remains active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good Till Canceled - order remains until filled or canceled
    #[default]
    GTC,
    /// Immediate Or Cancel - fill as much as possible, cancel the rest
    IOC,
    /// Fill Or Kill - fill completely or cancel entirely
    FOK,
    /// Good Till Crossing - only for Post Only orders
    GTX,
    /// Unknown time in force
    #[serde(other)]
    Other,
}

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    /// The order has been accepted by the engine
    New,
    /// A part of the order has been filled
    PartiallyFilled,
    /// The order has been completely filled
    Filled,
    /// The order has been canceled by the user
    Canceled,
    /// Currently unused
    PendingCancel,
    /// The order was not accepted by the engine and not processed
    Rejected,
    /// The order was canceled according to the order type's rules
    Expired,
    /// The order was canceled by the exchange due to STP trigger
    ExpiredInMatch,
}

/// Execution type for order updates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionType {
    /// The order has been accepted into the engine
    New,
    /// The order has been canceled by the user
    Canceled,
    /// Currently unused
    Replaced,
    /// The order has been rejected
    Rejected,
    /// Part of the order or all of the order's quantity has filled
    Trade,
    /// The order was canceled according to the order type's rules
    Expired,
    /// The order has expired due to STP trigger
    TradePrevention,
    /// Order modified
    Amendment,
}

/// Kline/candlestick interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KlineInterval {
    /// 1 second
    #[serde(rename = "1s")]
    Seconds1,
    /// 1 minute
    #[serde(rename = "1m")]
    Minutes1,
    /// 3 minutes
    #[serde(rename = "3m")]
    Minutes3,
    /// 5 minutes
    #[serde(rename = "5m")]
    Minutes5,
    /// 15 minutes
    #[serde(rename = "15m")]
    Minutes15,
    /// 30 minutes
    #[serde(rename = "30m")]
    Minutes30,
    /// 1 hour
    #[serde(rename = "1h")]
    Hours1,
    /// 2 hours
    #[serde(rename = "2h")]
    Hours2,
    /// 4 hours
    #[serde(rename = "4h")]
    Hours4,
    /// 6 hours
    #[serde(rename = "6h")]
    Hours6,
    /// 8 hours
    #[serde(rename = "8h")]
    Hours8,
    /// 12 hours
    #[serde(rename = "12h")]
    Hours12,
    /// 1 day
    #[serde(rename = "1d")]
    Days1,
    /// 3 days
    #[serde(rename = "3d")]
    Days3,
    /// 1 week
    #[serde(rename = "1w")]
    Weeks1,
    /// 1 month
    #[serde(rename = "1M")]
    Months1,
}

impl std::fmt::Display for KlineInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Seconds1 => "1s",
            Self::Minutes1 => "1m",
            Self::Minutes3 => "3m",
            Self::Minutes5 => "5m",
            Self::Minutes15 => "15m",
            Self::Minutes30 => "30m",
            Self::Hours1 => "1h",
            Self::Hours2 => "2h",
            Self::Hours4 => "4h",
            Self::Hours6 => "6h",
            Self::Hours8 => "8h",
            Self::Hours12 => "12h",
            Self::Days1 => "1d",
            Self::Days3 => "3d",
            Self::Weeks1 => "1w",
            Self::Months1 => "1M",
        };
        write!(f, "{}", s)
    }
}

/// Ticker response type for market data endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TickerType {
    /// Full response payload.
    Full,
    /// Mini response payload.
    Mini,
}

impl std::fmt::Display for TickerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Full => "FULL",
            Self::Mini => "MINI",
        };
        write!(f, "{}", s)
    }
}

/// Symbol status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolStatus {
    /// Pre-trading period
    PreTrading,
    /// Currently trading
    Trading,
    /// Post-trading period
    PostTrading,
    /// End of day
    EndOfDay,
    /// Trading halted
    Halt,
    /// Auction match
    AuctionMatch,
    /// Trading break
    Break,
    /// Pending trading
    PendingTrading,
    /// Unknown status
    #[serde(other)]
    Other,
}

impl std::fmt::Display for SymbolStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::PreTrading => "PRE_TRADING",
            Self::Trading => "TRADING",
            Self::PostTrading => "POST_TRADING",
            Self::EndOfDay => "END_OF_DAY",
            Self::Halt => "HALT",
            Self::AuctionMatch => "AUCTION_MATCH",
            Self::Break => "BREAK",
            Self::PendingTrading => "PENDING_TRADING",
            Self::Other => "OTHER",
        };
        write!(f, "{}", s)
    }
}

/// Symbol permission type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolPermission {
    /// Spot trading
    Spot,
    /// Margin trading
    Margin,
    /// Unknown permission
    #[serde(other)]
    Other,
}

/// Account type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    /// Spot account
    Spot,
    /// USDT futures account
    UsdtFuture,
    /// Coin futures account
    CoinFuture,
    /// Leveraged account
    Leveraged,
    /// Unknown account type
    #[serde(other)]
    Other,
}

/// Rate limit type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitType {
    /// Request weight limit
    RequestWeight,
    /// Orders limit
    Orders,
    /// Raw requests limit
    RawRequests,
    /// Unknown limit type
    #[serde(other)]
    Other,
}

/// Rate limit interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RateLimitInterval {
    /// Per second
    Second,
    /// Per minute
    Minute,
    /// Per day
    Day,
}

/// Order response type for new orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderResponseType {
    /// Acknowledgement only
    Ack,
    /// Result with order details
    Result,
    /// Full response with fills
    Full,
    /// Unknown response type
    #[serde(other)]
    Other,
}

/// OCO order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OcoStatus {
    /// Response received
    Response,
    /// Execution started
    ExecStarted,
    /// All done
    AllDone,
}

/// OCO order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OcoOrderStatus {
    /// Executing
    Executing,
    /// All done
    AllDone,
    /// Rejected
    Reject,
}

/// Contingency type for OCO orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContingencyType {
    /// One Cancels Other
    Oco,
    /// One-Triggers-the-Other
    Oto,
    /// One-Triggers-One-Cancels-the-Other
    Otoco,
    /// One-Places-the-Other
    Opo,
    /// One-Places-One-Cancels-the-Other
    Opoco,
    /// Unknown contingency type
    #[serde(other)]
    Other,
}

/// Cancel-replace mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CancelReplaceMode {
    /// Stop if the cancel fails.
    StopOnFailure,
    /// Allow new order placement even if cancel fails.
    AllowFailure,
}

impl std::fmt::Display for CancelReplaceMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::StopOnFailure => "STOP_ON_FAILURE",
            Self::AllowFailure => "ALLOW_FAILURE",
        };
        write!(f, "{}", s)
    }
}

/// Cancel-replace result status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CancelReplaceResult {
    /// Operation succeeded.
    Success,
    /// Operation failed.
    Failure,
    /// Operation was not attempted.
    NotAttempted,
}

/// Cancel restrictions for cancel-replace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CancelRestrictions {
    /// Cancel only if the order is NEW.
    OnlyNew,
    /// Cancel only if the order is PARTIALLY_FILLED.
    OnlyPartiallyFilled,
}

impl std::fmt::Display for CancelRestrictions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::OnlyNew => "ONLY_NEW",
            Self::OnlyPartiallyFilled => "ONLY_PARTIALLY_FILLED",
        };
        write!(f, "{}", s)
    }
}

/// Order rate limit exceeded mode for cancel-replace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderRateLimitExceededMode {
    /// Do not attempt cancel when exceeded.
    DoNothing,
    /// Cancel only even if exceeded.
    CancelOnly,
}

impl std::fmt::Display for OrderRateLimitExceededMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::DoNothing => "DO_NOTHING",
            Self::CancelOnly => "CANCEL_ONLY",
        };
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_side_serde() {
        let buy: OrderSide = serde_json::from_str("\"BUY\"").unwrap();
        assert_eq!(buy, OrderSide::Buy);

        let sell: OrderSide = serde_json::from_str("\"SELL\"").unwrap();
        assert_eq!(sell, OrderSide::Sell);

        let serialized = serde_json::to_string(&OrderSide::Buy).unwrap();
        assert_eq!(serialized, "\"BUY\"");
    }

    #[test]
    fn test_order_type_serde() {
        let limit: OrderType = serde_json::from_str("\"LIMIT\"").unwrap();
        assert_eq!(limit, OrderType::Limit);

        let market: OrderType = serde_json::from_str("\"MARKET\"").unwrap();
        assert_eq!(market, OrderType::Market);

        let stop_loss: OrderType = serde_json::from_str("\"STOP_LOSS\"").unwrap();
        assert_eq!(stop_loss, OrderType::StopLoss);

        // Unknown type should deserialize to Other
        let other: OrderType = serde_json::from_str("\"UNKNOWN_TYPE\"").unwrap();
        assert_eq!(other, OrderType::Other);
    }

    #[test]
    fn test_time_in_force_serde() {
        let gtc: TimeInForce = serde_json::from_str("\"GTC\"").unwrap();
        assert_eq!(gtc, TimeInForce::GTC);

        let ioc: TimeInForce = serde_json::from_str("\"IOC\"").unwrap();
        assert_eq!(ioc, TimeInForce::IOC);

        let fok: TimeInForce = serde_json::from_str("\"FOK\"").unwrap();
        assert_eq!(fok, TimeInForce::FOK);
    }

    #[test]
    fn test_order_status_serde() {
        let new: OrderStatus = serde_json::from_str("\"NEW\"").unwrap();
        assert_eq!(new, OrderStatus::New);

        let filled: OrderStatus = serde_json::from_str("\"FILLED\"").unwrap();
        assert_eq!(filled, OrderStatus::Filled);

        let canceled: OrderStatus = serde_json::from_str("\"CANCELED\"").unwrap();
        assert_eq!(canceled, OrderStatus::Canceled);
    }

    #[test]
    fn test_kline_interval_display() {
        assert_eq!(KlineInterval::Minutes1.to_string(), "1m");
        assert_eq!(KlineInterval::Hours1.to_string(), "1h");
        assert_eq!(KlineInterval::Days1.to_string(), "1d");
        assert_eq!(KlineInterval::Months1.to_string(), "1M");
    }

    #[test]
    fn test_kline_interval_serde() {
        let interval: KlineInterval = serde_json::from_str("\"1h\"").unwrap();
        assert_eq!(interval, KlineInterval::Hours1);

        let serialized = serde_json::to_string(&KlineInterval::Minutes15).unwrap();
        assert_eq!(serialized, "\"15m\"");
    }
}
