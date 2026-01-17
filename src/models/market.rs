//! Market data models.
//!
//! These models represent responses from public market data endpoints.

use serde::{Deserialize, Serialize};

use crate::types::{OrderType, RateLimitInterval, RateLimitType, SymbolPermission, SymbolStatus};

/// Server time response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerTime {
    /// Server timestamp in milliseconds.
    pub server_time: u64,
}

/// Exchange information response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeInfo {
    /// Exchange timezone.
    pub timezone: String,
    /// Server timestamp.
    pub server_time: u64,
    /// Rate limits.
    pub rate_limits: Vec<RateLimit>,
    /// Trading symbols.
    pub symbols: Vec<Symbol>,
    /// Exchange-level filters.
    #[serde(default)]
    pub exchange_filters: Vec<SymbolFilter>,
}

/// Rate limit information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateLimit {
    /// Rate limit type.
    pub rate_limit_type: RateLimitType,
    /// Interval.
    pub interval: RateLimitInterval,
    /// Interval number.
    pub interval_num: i32,
    /// Limit value.
    pub limit: i32,
}

/// Trading symbol information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Symbol {
    /// Symbol name (e.g., "BTCUSDT").
    pub symbol: String,
    /// Symbol status.
    pub status: SymbolStatus,
    /// Base asset (e.g., "BTC").
    pub base_asset: String,
    /// Base asset precision.
    pub base_asset_precision: u8,
    /// Quote asset (e.g., "USDT").
    pub quote_asset: String,
    /// Quote asset precision.
    pub quote_precision: u8,
    /// Quote asset precision (duplicate field in API).
    pub quote_asset_precision: u8,
    /// Base commission precision.
    #[serde(default)]
    pub base_commission_precision: u8,
    /// Quote commission precision.
    #[serde(default)]
    pub quote_commission_precision: u8,
    /// Allowed order types.
    pub order_types: Vec<OrderType>,
    /// Whether iceberg orders are allowed.
    pub iceberg_allowed: bool,
    /// Whether OCO orders are allowed.
    pub oco_allowed: bool,
    /// Whether quote order quantity is allowed for market orders.
    #[serde(default)]
    pub quote_order_qty_market_allowed: bool,
    /// Whether spot trading is allowed.
    #[serde(default = "default_true")]
    pub is_spot_trading_allowed: bool,
    /// Whether margin trading is allowed.
    #[serde(default)]
    pub is_margin_trading_allowed: bool,
    /// Symbol filters.
    pub filters: Vec<SymbolFilter>,
    /// Symbol permissions.
    #[serde(default)]
    pub permissions: Vec<SymbolPermission>,
}

fn default_true() -> bool {
    true
}

impl Symbol {
    /// Get the LOT_SIZE filter for this symbol.
    pub fn lot_size(&self) -> Option<&SymbolFilter> {
        self.filters
            .iter()
            .find(|f| matches!(f, SymbolFilter::LotSize { .. }))
    }

    /// Get the PRICE_FILTER filter for this symbol.
    pub fn price_filter(&self) -> Option<&SymbolFilter> {
        self.filters
            .iter()
            .find(|f| matches!(f, SymbolFilter::PriceFilter { .. }))
    }

    /// Get the MIN_NOTIONAL filter for this symbol.
    pub fn min_notional(&self) -> Option<&SymbolFilter> {
        self.filters
            .iter()
            .find(|f| matches!(f, SymbolFilter::MinNotional { .. }))
    }
}

/// Symbol filter types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "filterType")]
pub enum SymbolFilter {
    /// Price filter - valid price range and tick size.
    #[serde(rename = "PRICE_FILTER")]
    #[serde(rename_all = "camelCase")]
    PriceFilter {
        /// Minimum price.
        #[serde(with = "string_or_float")]
        min_price: f64,
        /// Maximum price.
        #[serde(with = "string_or_float")]
        max_price: f64,
        /// Tick size (price must be a multiple of this).
        #[serde(with = "string_or_float")]
        tick_size: f64,
    },
    /// Percent price filter - price relative to average.
    #[serde(rename = "PERCENT_PRICE")]
    #[serde(rename_all = "camelCase")]
    PercentPrice {
        /// Upper multiplier.
        #[serde(with = "string_or_float")]
        multiplier_up: f64,
        /// Lower multiplier.
        #[serde(with = "string_or_float")]
        multiplier_down: f64,
        /// Average price minutes.
        avg_price_mins: u64,
    },
    /// Lot size filter - valid quantity range and step size.
    #[serde(rename = "LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    LotSize {
        /// Minimum quantity.
        #[serde(with = "string_or_float")]
        min_qty: f64,
        /// Maximum quantity.
        #[serde(with = "string_or_float")]
        max_qty: f64,
        /// Step size (quantity must be a multiple of this).
        #[serde(with = "string_or_float")]
        step_size: f64,
    },
    /// Market lot size filter.
    #[serde(rename = "MARKET_LOT_SIZE")]
    #[serde(rename_all = "camelCase")]
    MarketLotSize {
        /// Minimum quantity.
        #[serde(with = "string_or_float")]
        min_qty: f64,
        /// Maximum quantity.
        #[serde(with = "string_or_float")]
        max_qty: f64,
        /// Step size.
        #[serde(with = "string_or_float")]
        step_size: f64,
    },
    /// Minimum notional filter.
    #[serde(rename = "MIN_NOTIONAL")]
    #[serde(rename_all = "camelCase")]
    MinNotional {
        /// Minimum notional value (price * quantity).
        #[serde(with = "string_or_float")]
        min_notional: f64,
        /// Apply to market orders.
        apply_to_market: bool,
        /// Average price minutes.
        avg_price_mins: u64,
    },
    /// Notional filter (newer version).
    #[serde(rename = "NOTIONAL")]
    #[serde(rename_all = "camelCase")]
    Notional {
        /// Minimum notional value.
        #[serde(with = "string_or_float")]
        min_notional: f64,
        /// Apply minimum to market orders.
        apply_min_to_market: bool,
        /// Maximum notional value.
        #[serde(with = "string_or_float")]
        max_notional: f64,
        /// Apply maximum to market orders.
        apply_max_to_market: bool,
        /// Average price minutes.
        avg_price_mins: u64,
    },
    /// Iceberg parts filter.
    #[serde(rename = "ICEBERG_PARTS")]
    #[serde(rename_all = "camelCase")]
    IcebergParts {
        /// Maximum iceberg parts.
        limit: u16,
    },
    /// Max orders filter.
    #[serde(rename = "MAX_NUM_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumOrders {
        /// Maximum number of orders.
        max_num_orders: u16,
    },
    /// Max algo orders filter.
    #[serde(rename = "MAX_NUM_ALGO_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumAlgoOrders {
        /// Maximum number of algo orders.
        max_num_algo_orders: u16,
    },
    /// Max iceberg orders filter.
    #[serde(rename = "MAX_NUM_ICEBERG_ORDERS")]
    #[serde(rename_all = "camelCase")]
    MaxNumIcebergOrders {
        /// Maximum number of iceberg orders.
        max_num_iceberg_orders: u16,
    },
    /// Max position filter.
    #[serde(rename = "MAX_POSITION")]
    #[serde(rename_all = "camelCase")]
    MaxPosition {
        /// Maximum position value.
        #[serde(with = "string_or_float")]
        max_position: f64,
    },
    /// Exchange max orders filter.
    #[serde(rename = "EXCHANGE_MAX_NUM_ORDERS")]
    #[serde(rename_all = "camelCase")]
    ExchangeMaxNumOrders {
        /// Maximum number of orders.
        max_num_orders: u16,
    },
    /// Exchange max algo orders filter.
    #[serde(rename = "EXCHANGE_MAX_NUM_ALGO_ORDERS")]
    #[serde(rename_all = "camelCase")]
    ExchangeMaxNumAlgoOrders {
        /// Maximum number of algo orders.
        max_num_algo_orders: u16,
    },
    /// Trailing delta filter.
    #[serde(rename = "TRAILING_DELTA")]
    #[serde(rename_all = "camelCase")]
    TrailingDelta {
        /// Minimum trailing delta.
        min_trailing_above_delta: u32,
        /// Maximum trailing delta.
        max_trailing_above_delta: u32,
        /// Minimum trailing delta below.
        min_trailing_below_delta: u32,
        /// Maximum trailing delta below.
        max_trailing_below_delta: u32,
    },
    /// Unknown filter type.
    #[serde(other)]
    Other,
}

/// Order book response.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderBook {
    /// Last update ID.
    pub last_update_id: u64,
    /// Bid entries (price, quantity).
    pub bids: Vec<OrderBookEntry>,
    /// Ask entries (price, quantity).
    pub asks: Vec<OrderBookEntry>,
}

/// Order book entry (price level).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEntry {
    /// Price level.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Quantity at this price level.
    #[serde(with = "string_or_float")]
    pub quantity: f64,
}

/// Recent trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Trade ID.
    pub id: u64,
    /// Price.
    #[serde(with = "string_or_float")]
    pub price: f64,
    /// Quantity.
    #[serde(rename = "qty", with = "string_or_float")]
    pub quantity: f64,
    /// Quote quantity.
    #[serde(rename = "quoteQty", with = "string_or_float")]
    pub quote_quantity: f64,
    /// Trade time in milliseconds.
    pub time: u64,
    /// Was the buyer the maker.
    pub is_buyer_maker: bool,
    /// Was this the best price match.
    pub is_best_match: bool,
}

/// Aggregate trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggTrade {
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
    /// Timestamp.
    #[serde(rename = "T")]
    pub timestamp: u64,
    /// Was the buyer the maker.
    #[serde(rename = "m")]
    pub is_buyer_maker: bool,
    /// Was this the best price match.
    #[serde(rename = "M")]
    pub is_best_match: bool,
}

/// Kline/candlestick data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Kline {
    /// Kline open time.
    pub open_time: i64,
    /// Open price.
    pub open: f64,
    /// High price.
    pub high: f64,
    /// Low price.
    pub low: f64,
    /// Close price.
    pub close: f64,
    /// Volume.
    pub volume: f64,
    /// Kline close time.
    pub close_time: i64,
    /// Quote asset volume.
    pub quote_asset_volume: f64,
    /// Number of trades.
    pub number_of_trades: i64,
    /// Taker buy base asset volume.
    pub taker_buy_base_asset_volume: f64,
    /// Taker buy quote asset volume.
    pub taker_buy_quote_asset_volume: f64,
}

/// 24hr ticker price change statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ticker24h {
    /// Symbol.
    pub symbol: String,
    /// Price change.
    #[serde(with = "string_or_float")]
    pub price_change: f64,
    /// Price change percent.
    #[serde(with = "string_or_float")]
    pub price_change_percent: f64,
    /// Weighted average price.
    #[serde(with = "string_or_float")]
    pub weighted_avg_price: f64,
    /// Previous close price.
    #[serde(with = "string_or_float")]
    pub prev_close_price: f64,
    /// Last price.
    #[serde(with = "string_or_float")]
    pub last_price: f64,
    /// Last quantity.
    #[serde(with = "string_or_float")]
    pub last_qty: f64,
    /// Bid price.
    #[serde(with = "string_or_float")]
    pub bid_price: f64,
    /// Bid quantity.
    #[serde(with = "string_or_float")]
    pub bid_qty: f64,
    /// Ask price.
    #[serde(with = "string_or_float")]
    pub ask_price: f64,
    /// Ask quantity.
    #[serde(with = "string_or_float")]
    pub ask_qty: f64,
    /// Open price.
    #[serde(with = "string_or_float")]
    pub open_price: f64,
    /// High price.
    #[serde(with = "string_or_float")]
    pub high_price: f64,
    /// Low price.
    #[serde(with = "string_or_float")]
    pub low_price: f64,
    /// Total volume.
    #[serde(with = "string_or_float")]
    pub volume: f64,
    /// Quote volume.
    #[serde(with = "string_or_float")]
    pub quote_volume: f64,
    /// Open time.
    pub open_time: u64,
    /// Close time.
    pub close_time: u64,
    /// First trade ID.
    pub first_id: i64,
    /// Last trade ID.
    pub last_id: i64,
    /// Trade count.
    pub count: u64,
}

/// Trading day ticker statistics (FULL).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingDayTicker {
    /// Symbol.
    pub symbol: String,
    /// Price change.
    #[serde(with = "string_or_float")]
    pub price_change: f64,
    /// Price change percent.
    #[serde(with = "string_or_float")]
    pub price_change_percent: f64,
    /// Weighted average price.
    #[serde(with = "string_or_float")]
    pub weighted_avg_price: f64,
    /// Open price.
    #[serde(with = "string_or_float")]
    pub open_price: f64,
    /// High price.
    #[serde(with = "string_or_float")]
    pub high_price: f64,
    /// Low price.
    #[serde(with = "string_or_float")]
    pub low_price: f64,
    /// Last price.
    #[serde(with = "string_or_float")]
    pub last_price: f64,
    /// Total volume.
    #[serde(with = "string_or_float")]
    pub volume: f64,
    /// Quote volume.
    #[serde(with = "string_or_float")]
    pub quote_volume: f64,
    /// Open time.
    pub open_time: u64,
    /// Close time.
    pub close_time: u64,
    /// First trade ID.
    pub first_id: i64,
    /// Last trade ID.
    pub last_id: i64,
    /// Trade count.
    pub count: u64,
}

/// Trading day ticker statistics (MINI).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingDayTickerMini {
    /// Symbol.
    pub symbol: String,
    /// Open price.
    #[serde(with = "string_or_float")]
    pub open_price: f64,
    /// High price.
    #[serde(with = "string_or_float")]
    pub high_price: f64,
    /// Low price.
    #[serde(with = "string_or_float")]
    pub low_price: f64,
    /// Last price.
    #[serde(with = "string_or_float")]
    pub last_price: f64,
    /// Total volume.
    #[serde(with = "string_or_float")]
    pub volume: f64,
    /// Quote volume.
    #[serde(with = "string_or_float")]
    pub quote_volume: f64,
    /// Open time.
    pub open_time: u64,
    /// Close time.
    pub close_time: u64,
    /// First trade ID.
    pub first_id: i64,
    /// Last trade ID.
    pub last_id: i64,
    /// Trade count.
    pub count: u64,
}

/// Rolling window ticker statistics (FULL).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollingWindowTicker {
    /// Symbol.
    pub symbol: String,
    /// Price change.
    #[serde(with = "string_or_float")]
    pub price_change: f64,
    /// Price change percent.
    #[serde(with = "string_or_float")]
    pub price_change_percent: f64,
    /// Weighted average price.
    #[serde(with = "string_or_float")]
    pub weighted_avg_price: f64,
    /// Open price.
    #[serde(with = "string_or_float")]
    pub open_price: f64,
    /// High price.
    #[serde(with = "string_or_float")]
    pub high_price: f64,
    /// Low price.
    #[serde(with = "string_or_float")]
    pub low_price: f64,
    /// Last price.
    #[serde(with = "string_or_float")]
    pub last_price: f64,
    /// Total volume.
    #[serde(with = "string_or_float")]
    pub volume: f64,
    /// Quote volume.
    #[serde(with = "string_or_float")]
    pub quote_volume: f64,
    /// Open time.
    pub open_time: u64,
    /// Close time.
    pub close_time: u64,
    /// First trade ID.
    pub first_id: i64,
    /// Last trade ID.
    pub last_id: i64,
    /// Trade count.
    pub count: u64,
}

/// Rolling window ticker statistics (MINI).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RollingWindowTickerMini {
    /// Symbol.
    pub symbol: String,
    /// Open price.
    #[serde(with = "string_or_float")]
    pub open_price: f64,
    /// High price.
    #[serde(with = "string_or_float")]
    pub high_price: f64,
    /// Low price.
    #[serde(with = "string_or_float")]
    pub low_price: f64,
    /// Last price.
    #[serde(with = "string_or_float")]
    pub last_price: f64,
    /// Total volume.
    #[serde(with = "string_or_float")]
    pub volume: f64,
    /// Quote volume.
    #[serde(with = "string_or_float")]
    pub quote_volume: f64,
    /// Open time.
    pub open_time: u64,
    /// Close time.
    pub close_time: u64,
    /// First trade ID.
    pub first_id: i64,
    /// Last trade ID.
    pub last_id: i64,
    /// Trade count.
    pub count: u64,
}

/// Symbol price ticker.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TickerPrice {
    /// Symbol.
    pub symbol: String,
    /// Current price.
    #[serde(with = "string_or_float")]
    pub price: f64,
}

/// Symbol order book ticker (best bid/ask).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BookTicker {
    /// Symbol.
    pub symbol: String,
    /// Best bid price.
    #[serde(with = "string_or_float")]
    pub bid_price: f64,
    /// Best bid quantity.
    #[serde(with = "string_or_float")]
    pub bid_qty: f64,
    /// Best ask price.
    #[serde(with = "string_or_float")]
    pub ask_price: f64,
    /// Best ask quantity.
    #[serde(with = "string_or_float")]
    pub ask_qty: f64,
}

/// Average price response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AveragePrice {
    /// Number of minutes the average is calculated over.
    pub mins: u64,
    /// Average price.
    #[serde(with = "string_or_float")]
    pub price: f64,
}

/// Helper module for deserializing string or float values.
///
/// Binance API sometimes returns numbers as strings and sometimes as numbers.
pub mod string_or_float {
    use serde::{Deserialize, Deserializer, Serializer, de};
    use std::fmt;

    pub fn serialize<T, S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        serializer.collect_str(value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String(String),
            Float(f64),
        }

        match StringOrFloat::deserialize(deserializer)? {
            StringOrFloat::String(s) => s.parse().map_err(de::Error::custom),
            StringOrFloat::Float(f) => Ok(f),
        }
    }
}

/// Helper module for deserializing optional string or float values.
pub mod string_or_float_opt {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        T: fmt::Display,
        S: Serializer,
    {
        match value {
            Some(v) => super::string_or_float::serialize(v, serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrFloat {
            String(String),
            Float(f64),
            Null,
        }

        match StringOrFloat::deserialize(deserializer)? {
            StringOrFloat::String(s) if s.is_empty() => Ok(None),
            StringOrFloat::String(s) => s.parse().map(Some).map_err(serde::de::Error::custom),
            StringOrFloat::Float(f) => Ok(Some(f)),
            StringOrFloat::Null => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_time_deserialize() {
        let json = r#"{"serverTime": 1234567890123}"#;
        let time: ServerTime = serde_json::from_str(json).unwrap();
        assert_eq!(time.server_time, 1234567890123);
    }

    #[test]
    fn test_ticker_price_deserialize() {
        let json = r#"{"symbol": "BTCUSDT", "price": "50000.00"}"#;
        let ticker: TickerPrice = serde_json::from_str(json).unwrap();
        assert_eq!(ticker.symbol, "BTCUSDT");
        assert_eq!(ticker.price, 50000.0);
    }

    #[test]
    fn test_book_ticker_deserialize() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "bidPrice": "50000.00",
            "bidQty": "1.5",
            "askPrice": "50001.00",
            "askQty": "2.0"
        }"#;
        let ticker: BookTicker = serde_json::from_str(json).unwrap();
        assert_eq!(ticker.symbol, "BTCUSDT");
        assert_eq!(ticker.bid_price, 50000.0);
        assert_eq!(ticker.bid_qty, 1.5);
        assert_eq!(ticker.ask_price, 50001.0);
        assert_eq!(ticker.ask_qty, 2.0);
    }

    #[test]
    fn test_order_book_entry_deserialize() {
        // Order book entries come as arrays: [price, quantity]
        let json = r#"["50000.00", "1.5"]"#;
        let entry: OrderBookEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.price, 50000.0);
        assert_eq!(entry.quantity, 1.5);
    }

    #[test]
    fn test_agg_trade_deserialize() {
        let json = r#"{
            "a": 12345,
            "p": "50000.00",
            "q": "1.5",
            "f": 100,
            "l": 105,
            "T": 1234567890123,
            "m": true,
            "M": true
        }"#;
        let trade: AggTrade = serde_json::from_str(json).unwrap();
        assert_eq!(trade.agg_trade_id, 12345);
        assert_eq!(trade.price, 50000.0);
        assert_eq!(trade.quantity, 1.5);
        assert_eq!(trade.first_trade_id, 100);
        assert_eq!(trade.last_trade_id, 105);
        assert_eq!(trade.timestamp, 1234567890123);
        assert!(trade.is_buyer_maker);
        assert!(trade.is_best_match);
    }

    #[test]
    fn test_average_price_deserialize() {
        let json = r#"{"mins": 5, "price": "50000.00"}"#;
        let avg: AveragePrice = serde_json::from_str(json).unwrap();
        assert_eq!(avg.mins, 5);
        assert_eq!(avg.price, 50000.0);
    }

    #[test]
    fn test_symbol_filter_price_filter() {
        let json = r#"{
            "filterType": "PRICE_FILTER",
            "minPrice": "0.00001000",
            "maxPrice": "1000000.00000000",
            "tickSize": "0.00001000"
        }"#;
        let filter: SymbolFilter = serde_json::from_str(json).unwrap();
        match filter {
            SymbolFilter::PriceFilter {
                min_price,
                max_price,
                tick_size,
            } => {
                assert_eq!(min_price, 0.00001);
                assert_eq!(max_price, 1000000.0);
                assert_eq!(tick_size, 0.00001);
            }
            _ => panic!("Expected PriceFilter"),
        }
    }

    #[test]
    fn test_symbol_filter_lot_size() {
        let json = r#"{
            "filterType": "LOT_SIZE",
            "minQty": "0.00100000",
            "maxQty": "100000.00000000",
            "stepSize": "0.00100000"
        }"#;
        let filter: SymbolFilter = serde_json::from_str(json).unwrap();
        match filter {
            SymbolFilter::LotSize {
                min_qty,
                max_qty,
                step_size,
            } => {
                assert_eq!(min_qty, 0.001);
                assert_eq!(max_qty, 100000.0);
                assert_eq!(step_size, 0.001);
            }
            _ => panic!("Expected LotSize"),
        }
    }

    #[test]
    fn test_unknown_filter_type() {
        let json = r#"{"filterType": "UNKNOWN_FILTER_TYPE"}"#;
        let filter: SymbolFilter = serde_json::from_str(json).unwrap();
        assert_eq!(filter, SymbolFilter::Other);
    }
}
