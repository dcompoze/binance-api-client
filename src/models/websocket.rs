//! WebSocket event models.
//!
//! These models represent events received from Binance WebSocket streams.

use serde::{Deserialize, Serialize};

use crate::types::{ExecutionType, KlineInterval, OrderSide, OrderStatus, OrderType, TimeInForce};

use super::market::string_or_float;

/// WebSocket event wrapper.
///
/// All WebSocket events have an "e" field indicating the event type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "e")]
pub enum WebSocketEvent {
    /// Aggregate trade event.
    #[serde(rename = "aggTrade")]
    AggTrade(AggTradeEvent),
    /// Trade event.
    #[serde(rename = "trade")]
    Trade(TradeEvent),
    /// Kline/candlestick event.
    #[serde(rename = "kline")]
    Kline(KlineEvent),
    /// 24hr mini ticker event.
    #[serde(rename = "24hrMiniTicker")]
    MiniTicker(MiniTickerEvent),
    /// 24hr ticker event.
    #[serde(rename = "24hrTicker")]
    Ticker(TickerEvent),
    /// Book ticker event.
    #[serde(rename = "bookTicker")]
    BookTicker(BookTickerEvent),
    /// Depth update event.
    #[serde(rename = "depthUpdate")]
    Depth(DepthEvent),
    /// Account position update (user data stream).
    #[serde(rename = "outboundAccountPosition")]
    AccountPosition(AccountPositionEvent),
    /// Balance update (user data stream).
    #[serde(rename = "balanceUpdate")]
    BalanceUpdate(BalanceUpdateEvent),
    /// Order update (user data stream).
    #[serde(rename = "executionReport")]
    ExecutionReport(ExecutionReportEvent),
    /// OCO order update (user data stream).
    #[serde(rename = "listStatus")]
    ListStatus(ListStatusEvent),
}

/// Aggregate trade event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggTradeEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: u64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Aggregate trade ID.
    #[serde(rename = "a")]
    pub agg_trade_id: u64,
    /// Price.
    #[serde(rename = "p", with = "string_or_float")]
    pub price: f64,
    /// Quantity.
    #[serde(rename = "q", with = "string_or_float")]
    pub quantity: f64,
    /// First trade ID.
    #[serde(rename = "f")]
    pub first_trade_id: u64,
    /// Last trade ID.
    #[serde(rename = "l")]
    pub last_trade_id: u64,
    /// Trade time.
    #[serde(rename = "T")]
    pub trade_time: u64,
    /// Is buyer the maker.
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
    /// Ignore.
    #[serde(rename = "M")]
    pub is_best_match: bool,
}

/// Trade event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: u64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Trade ID.
    #[serde(rename = "t")]
    pub trade_id: u64,
    /// Price.
    #[serde(rename = "p", with = "string_or_float")]
    pub price: f64,
    /// Quantity.
    #[serde(rename = "q", with = "string_or_float")]
    pub quantity: f64,
    /// Buyer order ID.
    #[serde(rename = "b")]
    pub buyer_order_id: u64,
    /// Seller order ID.
    #[serde(rename = "a")]
    pub seller_order_id: u64,
    /// Trade time.
    #[serde(rename = "T")]
    pub trade_time: u64,
    /// Is buyer the maker.
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
    /// Ignore.
    #[serde(rename = "M")]
    pub is_best_match: bool,
}

/// Kline/candlestick event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: u64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Kline data.
    #[serde(rename = "k")]
    pub kline: KlineData,
}

/// Kline data within a kline event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineData {
    /// Kline start time.
    #[serde(rename = "t")]
    pub start_time: i64,
    /// Kline close time.
    #[serde(rename = "T")]
    pub close_time: i64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Interval.
    #[serde(rename = "i")]
    pub interval: KlineInterval,
    /// First trade ID.
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    /// Last trade ID.
    #[serde(rename = "L")]
    pub last_trade_id: i64,
    /// Open price.
    #[serde(rename = "o", with = "string_or_float")]
    pub open: f64,
    /// Close price.
    #[serde(rename = "c", with = "string_or_float")]
    pub close: f64,
    /// High price.
    #[serde(rename = "h", with = "string_or_float")]
    pub high: f64,
    /// Low price.
    #[serde(rename = "l", with = "string_or_float")]
    pub low: f64,
    /// Volume.
    #[serde(rename = "v", with = "string_or_float")]
    pub volume: f64,
    /// Number of trades.
    #[serde(rename = "n")]
    pub number_of_trades: i64,
    /// Is this kline closed.
    #[serde(rename = "x")]
    pub is_closed: bool,
    /// Quote asset volume.
    #[serde(rename = "q", with = "string_or_float")]
    pub quote_asset_volume: f64,
    /// Taker buy base asset volume.
    #[serde(rename = "V", with = "string_or_float")]
    pub taker_buy_base_volume: f64,
    /// Taker buy quote asset volume.
    #[serde(rename = "Q", with = "string_or_float")]
    pub taker_buy_quote_volume: f64,
}

/// 24hr mini ticker event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniTickerEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: u64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Close price.
    #[serde(rename = "c", with = "string_or_float")]
    pub close: f64,
    /// Open price.
    #[serde(rename = "o", with = "string_or_float")]
    pub open: f64,
    /// High price.
    #[serde(rename = "h", with = "string_or_float")]
    pub high: f64,
    /// Low price.
    #[serde(rename = "l", with = "string_or_float")]
    pub low: f64,
    /// Total traded base asset volume.
    #[serde(rename = "v", with = "string_or_float")]
    pub volume: f64,
    /// Total traded quote asset volume.
    #[serde(rename = "q", with = "string_or_float")]
    pub quote_volume: f64,
}

/// 24hr ticker event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: u64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Price change.
    #[serde(rename = "p", with = "string_or_float")]
    pub price_change: f64,
    /// Price change percent.
    #[serde(rename = "P", with = "string_or_float")]
    pub price_change_percent: f64,
    /// Weighted average price.
    #[serde(rename = "w", with = "string_or_float")]
    pub weighted_avg_price: f64,
    /// Previous day's close price.
    #[serde(rename = "x", with = "string_or_float")]
    pub prev_close_price: f64,
    /// Current day's close price.
    #[serde(rename = "c", with = "string_or_float")]
    pub close_price: f64,
    /// Close trade quantity.
    #[serde(rename = "Q", with = "string_or_float")]
    pub close_quantity: f64,
    /// Best bid price.
    #[serde(rename = "b", with = "string_or_float")]
    pub bid_price: f64,
    /// Best bid quantity.
    #[serde(rename = "B", with = "string_or_float")]
    pub bid_quantity: f64,
    /// Best ask price.
    #[serde(rename = "a", with = "string_or_float")]
    pub ask_price: f64,
    /// Best ask quantity.
    #[serde(rename = "A", with = "string_or_float")]
    pub ask_quantity: f64,
    /// Open price.
    #[serde(rename = "o", with = "string_or_float")]
    pub open_price: f64,
    /// High price.
    #[serde(rename = "h", with = "string_or_float")]
    pub high_price: f64,
    /// Low price.
    #[serde(rename = "l", with = "string_or_float")]
    pub low_price: f64,
    /// Total traded base asset volume.
    #[serde(rename = "v", with = "string_or_float")]
    pub volume: f64,
    /// Total traded quote asset volume.
    #[serde(rename = "q", with = "string_or_float")]
    pub quote_volume: f64,
    /// Statistics open time.
    #[serde(rename = "O")]
    pub open_time: u64,
    /// Statistics close time.
    #[serde(rename = "C")]
    pub close_time: u64,
    /// First trade ID.
    #[serde(rename = "F")]
    pub first_trade_id: i64,
    /// Last trade ID.
    #[serde(rename = "L")]
    pub last_trade_id: i64,
    /// Total number of trades.
    #[serde(rename = "n")]
    pub number_of_trades: u64,
}

/// Book ticker event (best bid/ask).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookTickerEvent {
    /// Update ID.
    #[serde(rename = "u")]
    pub update_id: u64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Best bid price.
    #[serde(rename = "b", with = "string_or_float")]
    pub bid_price: f64,
    /// Best bid quantity.
    #[serde(rename = "B", with = "string_or_float")]
    pub bid_quantity: f64,
    /// Best ask price.
    #[serde(rename = "a", with = "string_or_float")]
    pub ask_price: f64,
    /// Best ask quantity.
    #[serde(rename = "A", with = "string_or_float")]
    pub ask_quantity: f64,
}

/// Depth update event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: u64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// First update ID in event.
    #[serde(rename = "U")]
    pub first_update_id: u64,
    /// Final update ID in event.
    #[serde(rename = "u")]
    pub final_update_id: u64,
    /// Bids to be updated.
    #[serde(rename = "b")]
    pub bids: Vec<DepthLevel>,
    /// Asks to be updated.
    #[serde(rename = "a")]
    pub asks: Vec<DepthLevel>,
}

/// Depth level (price/quantity pair).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepthLevel {
    /// Price level.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Quantity.
    #[serde(with = "string_or_float")]
    pub quantity: f64,
}

/// Account position update event (user data stream).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountPositionEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: u64,
    /// Time of last account update.
    #[serde(rename = "u")]
    pub last_update_time: u64,
    /// Balances.
    #[serde(rename = "B")]
    pub balances: Vec<AccountBalance>,
}

/// Account balance in position event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    /// Asset.
    #[serde(rename = "a")]
    pub asset: String,
    /// Free balance.
    #[serde(rename = "f", with = "string_or_float")]
    pub free: f64,
    /// Locked balance.
    #[serde(rename = "l", with = "string_or_float")]
    pub locked: f64,
}

/// Balance update event (user data stream).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceUpdateEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: u64,
    /// Asset.
    #[serde(rename = "a")]
    pub asset: String,
    /// Balance delta.
    #[serde(rename = "d", with = "string_or_float")]
    pub balance_delta: f64,
    /// Clear time.
    #[serde(rename = "T")]
    pub clear_time: u64,
}

/// Order execution report event (user data stream).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReportEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: u64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Client order ID.
    #[serde(rename = "c")]
    pub client_order_id: String,
    /// Side.
    #[serde(rename = "S")]
    pub side: OrderSide,
    /// Order type.
    #[serde(rename = "o")]
    pub order_type: OrderType,
    /// Time in force.
    #[serde(rename = "f")]
    pub time_in_force: TimeInForce,
    /// Order quantity.
    #[serde(rename = "q", with = "string_or_float")]
    pub quantity: f64,
    /// Order price.
    #[serde(rename = "p", with = "string_or_float")]
    pub price: f64,
    /// Stop price.
    #[serde(rename = "P", with = "string_or_float")]
    pub stop_price: f64,
    /// Iceberg quantity.
    #[serde(rename = "F", with = "string_or_float")]
    pub iceberg_quantity: f64,
    /// Order list ID.
    #[serde(rename = "g")]
    pub order_list_id: i64,
    /// Original client order ID (for cancel/replace).
    #[serde(rename = "C")]
    pub orig_client_order_id: String,
    /// Current execution type.
    #[serde(rename = "x")]
    pub execution_type: ExecutionType,
    /// Current order status.
    #[serde(rename = "X")]
    pub order_status: OrderStatus,
    /// Order reject reason.
    #[serde(rename = "r")]
    pub reject_reason: String,
    /// Order ID.
    #[serde(rename = "i")]
    pub order_id: u64,
    /// Last executed quantity.
    #[serde(rename = "l", with = "string_or_float")]
    pub last_executed_quantity: f64,
    /// Cumulative filled quantity.
    #[serde(rename = "z", with = "string_or_float")]
    pub cumulative_filled_quantity: f64,
    /// Last executed price.
    #[serde(rename = "L", with = "string_or_float")]
    pub last_executed_price: f64,
    /// Commission amount.
    #[serde(rename = "n", with = "string_or_float")]
    pub commission: f64,
    /// Commission asset.
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    /// Transaction time.
    #[serde(rename = "T")]
    pub transaction_time: u64,
    /// Trade ID.
    #[serde(rename = "t")]
    pub trade_id: i64,
    /// Ignore.
    #[serde(rename = "I")]
    pub ignore_a: u64,
    /// Is the order on the book.
    #[serde(rename = "w")]
    pub is_on_book: bool,
    /// Is this trade the maker side.
    #[serde(rename = "m")]
    pub is_maker: bool,
    /// Ignore.
    #[serde(rename = "M")]
    pub ignore_b: bool,
    /// Order creation time.
    #[serde(rename = "O")]
    pub order_creation_time: u64,
    /// Cumulative quote asset transacted quantity.
    #[serde(rename = "Z", with = "string_or_float")]
    pub cumulative_quote_quantity: f64,
    /// Last quote asset transacted quantity.
    #[serde(rename = "Y", with = "string_or_float")]
    pub last_quote_quantity: f64,
    /// Quote order quantity.
    #[serde(rename = "Q", with = "string_or_float")]
    pub quote_order_quantity: f64,
}

/// OCO list status event (user data stream).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStatusEvent {
    /// Event time.
    #[serde(rename = "E")]
    pub event_time: u64,
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Order list ID.
    #[serde(rename = "g")]
    pub order_list_id: u64,
    /// Contingency type.
    #[serde(rename = "c")]
    pub contingency_type: String,
    /// List status type.
    #[serde(rename = "l")]
    pub list_status_type: String,
    /// List order status.
    #[serde(rename = "L")]
    pub list_order_status: String,
    /// List reject reason.
    #[serde(rename = "r")]
    pub list_reject_reason: String,
    /// List client order ID.
    #[serde(rename = "C")]
    pub list_client_order_id: String,
    /// Transaction time.
    #[serde(rename = "T")]
    pub transaction_time: u64,
    /// Orders.
    #[serde(rename = "O")]
    pub orders: Vec<ListStatusOrder>,
}

/// Order in list status event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStatusOrder {
    /// Symbol.
    #[serde(rename = "s")]
    pub symbol: String,
    /// Order ID.
    #[serde(rename = "i")]
    pub order_id: u64,
    /// Client order ID.
    #[serde(rename = "c")]
    pub client_order_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agg_trade_event_deserialize() {
        let json = r#"{
            "e": "aggTrade",
            "E": 1234567890123,
            "s": "BTCUSDT",
            "a": 12345,
            "p": "50000.00",
            "q": "1.5",
            "f": 100,
            "l": 105,
            "T": 1234567890123,
            "m": true,
            "M": true
        }"#;

        let event: WebSocketEvent = serde_json::from_str(json).unwrap();
        match event {
            WebSocketEvent::AggTrade(e) => {
                assert_eq!(e.symbol, "BTCUSDT");
                assert_eq!(e.agg_trade_id, 12345);
                assert_eq!(e.price, 50000.0);
                assert_eq!(e.quantity, 1.5);
            }
            _ => panic!("Expected AggTrade event"),
        }
    }

    #[test]
    fn test_book_ticker_event_deserialize() {
        let json = r#"{
            "e": "bookTicker",
            "u": 400900217,
            "s": "BTCUSDT",
            "b": "50000.00",
            "B": "1.5",
            "a": "50001.00",
            "A": "2.0"
        }"#;

        let event: WebSocketEvent = serde_json::from_str(json).unwrap();
        match event {
            WebSocketEvent::BookTicker(e) => {
                assert_eq!(e.symbol, "BTCUSDT");
                assert_eq!(e.bid_price, 50000.0);
                assert_eq!(e.ask_price, 50001.0);
            }
            _ => panic!("Expected BookTicker event"),
        }
    }

    #[test]
    fn test_mini_ticker_event_deserialize() {
        let json = r#"{
            "e": "24hrMiniTicker",
            "E": 1234567890123,
            "s": "BTCUSDT",
            "c": "50000.00",
            "o": "49000.00",
            "h": "51000.00",
            "l": "48000.00",
            "v": "1000.00",
            "q": "50000000.00"
        }"#;

        let event: WebSocketEvent = serde_json::from_str(json).unwrap();
        match event {
            WebSocketEvent::MiniTicker(e) => {
                assert_eq!(e.symbol, "BTCUSDT");
                assert_eq!(e.close, 50000.0);
            }
            _ => panic!("Expected MiniTicker event"),
        }
    }

    #[test]
    fn test_depth_level_deserialize() {
        let json = r#"["50000.00", "1.5"]"#;
        let level: DepthLevel = serde_json::from_str(json).unwrap();
        assert_eq!(level.price, 50000.0);
        assert_eq!(level.quantity, 1.5);
    }

    #[test]
    fn test_account_balance_deserialize() {
        let json = r#"{"a": "BTC", "f": "1.5", "l": "0.5"}"#;
        let balance: AccountBalance = serde_json::from_str(json).unwrap();
        assert_eq!(balance.asset, "BTC");
        assert_eq!(balance.free, 1.5);
        assert_eq!(balance.locked, 0.5);
    }
}
