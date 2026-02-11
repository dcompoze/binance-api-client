//! Market data API endpoints.
//!
//! This module provides access to public market data endpoints that don't
//! require authentication.

use serde_json::Value;

use crate::Result;
use crate::client::Client;
use crate::models::{
    AggTrade, AveragePrice, BookTicker, ExchangeInfo, Kline, OrderBook, RollingWindowTicker,
    RollingWindowTickerMini, ServerTime, Ticker24h, TickerPrice, Trade, TradingDayTicker,
    TradingDayTickerMini,
};
use crate::types::{KlineInterval, SymbolStatus, TickerType};

// API endpoints
const API_V3_PING: &str = "/api/v3/ping";
const API_V3_TIME: &str = "/api/v3/time";
const API_V3_EXCHANGE_INFO: &str = "/api/v3/exchangeInfo";
const API_V3_DEPTH: &str = "/api/v3/depth";
const API_V3_TRADES: &str = "/api/v3/trades";
const API_V3_HISTORICAL_TRADES: &str = "/api/v3/historicalTrades";
const API_V3_AGG_TRADES: &str = "/api/v3/aggTrades";
const API_V3_KLINES: &str = "/api/v3/klines";
const API_V3_UI_KLINES: &str = "/api/v3/uiKlines";
const API_V3_AVG_PRICE: &str = "/api/v3/avgPrice";
const API_V3_TICKER_24HR: &str = "/api/v3/ticker/24hr";
const API_V3_TICKER_TRADING_DAY: &str = "/api/v3/ticker/tradingDay";
const API_V3_TICKER_PRICE: &str = "/api/v3/ticker/price";
const API_V3_TICKER_BOOK_TICKER: &str = "/api/v3/ticker/bookTicker";
const API_V3_TICKER: &str = "/api/v3/ticker";

/// Market data API client.
///
/// Provides access to public market data endpoints.
#[derive(Clone)]
pub struct Market {
    client: Client,
}

impl Market {
    /// Create a new Market API client.
    pub(crate) fn new(client: Client) -> Self {
        Self { client }
    }

    /// Test connectivity to the API.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// client.market().ping().await?;
    /// ```
    pub async fn ping(&self) -> Result<()> {
        let _: Value = self.client.get(API_V3_PING, None).await?;
        Ok(())
    }

    /// Get the current server time.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let time = client.market().server_time().await?;
    /// println!("Server time: {}", time.server_time);
    /// ```
    pub async fn server_time(&self) -> Result<ServerTime> {
        self.client.get(API_V3_TIME, None).await
    }

    /// Get exchange information (trading rules and symbol info).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let info = client.market().exchange_info().await?;
    /// for symbol in info.symbols {
    ///     println!("{}: {}", symbol.symbol, symbol.status);
    /// }
    /// ```
    pub async fn exchange_info(&self) -> Result<ExchangeInfo> {
        self.client.get(API_V3_EXCHANGE_INFO, None).await
    }

    /// Get exchange information for specific symbols.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols to get info for
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let info = client.market().exchange_info_for_symbols(&["BTCUSDT", "ETHUSDT"]).await?;
    /// ```
    pub async fn exchange_info_for_symbols(&self, symbols: &[&str]) -> Result<ExchangeInfo> {
        let symbols_json = serde_json::to_string(symbols).unwrap_or_default();
        let query = format!("symbols={}", urlencoding::encode(&symbols_json));
        self.client.get(API_V3_EXCHANGE_INFO, Some(&query)).await
    }

    /// Get order book depth.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol (e.g., "BTCUSDT")
    /// * `limit` - Number of entries to return. Valid limits: 5, 10, 20, 50, 100, 500, 1000, 5000.
    ///   Default is 100; max is 5000.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let depth = client.market().depth("BTCUSDT", Some(10)).await?;
    /// for bid in depth.bids {
    ///     println!("Bid: {} @ {}", bid.quantity, bid.price);
    /// }
    /// ```
    pub async fn depth(&self, symbol: &str, limit: Option<u16>) -> Result<OrderBook> {
        let mut query = format!("symbol={}", symbol);
        if let Some(l) = limit {
            query.push_str(&format!("&limit={}", l));
        }
        self.client.get(API_V3_DEPTH, Some(&query)).await
    }

    /// Get recent trades.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `limit` - Number of trades to return. Default 500; max 1000.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let trades = client.market().trades("BTCUSDT", Some(10)).await?;
    /// ```
    pub async fn trades(&self, symbol: &str, limit: Option<u16>) -> Result<Vec<Trade>> {
        let mut query = format!("symbol={}", symbol);
        if let Some(l) = limit {
            query.push_str(&format!("&limit={}", l));
        }
        self.client.get(API_V3_TRADES, Some(&query)).await
    }

    /// Get older/historical trades.
    ///
    /// This endpoint requires an API key but not a signature.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `from_id` - Trade ID to fetch from
    /// * `limit` - Number of trades to return. Default 500; max 1000.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new("api_key", "secret_key")?;
    /// let trades = client.market().historical_trades("BTCUSDT", Some(12345), Some(100)).await?;
    /// ```
    pub async fn historical_trades(
        &self,
        symbol: &str,
        from_id: Option<u64>,
        limit: Option<u16>,
    ) -> Result<Vec<Trade>> {
        let mut query = format!("symbol={}", symbol);
        if let Some(id) = from_id {
            query.push_str(&format!("&fromId={}", id));
        }
        if let Some(l) = limit {
            query.push_str(&format!("&limit={}", l));
        }
        // This endpoint requires API key but not signature
        self.client
            .get_with_api_key(API_V3_HISTORICAL_TRADES, Some(&query))
            .await
    }

    /// Get compressed/aggregate trades.
    ///
    /// Trades that fill at the same time, from the same order, with the same
    /// price will have the aggregate quantity added.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `from_id` - Aggregate trade ID to get from
    /// * `start_time` - Start time in milliseconds
    /// * `end_time` - End time in milliseconds
    /// * `limit` - Default 500; max 1000
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let trades = client.market().agg_trades("BTCUSDT", None, None, None, Some(10)).await?;
    /// ```
    pub async fn agg_trades(
        &self,
        symbol: &str,
        from_id: Option<u64>,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> Result<Vec<AggTrade>> {
        let mut query = format!("symbol={}", symbol);
        if let Some(id) = from_id {
            query.push_str(&format!("&fromId={}", id));
        }
        if let Some(start) = start_time {
            query.push_str(&format!("&startTime={}", start));
        }
        if let Some(end) = end_time {
            query.push_str(&format!("&endTime={}", end));
        }
        if let Some(l) = limit {
            query.push_str(&format!("&limit={}", l));
        }
        self.client.get(API_V3_AGG_TRADES, Some(&query)).await
    }

    /// Get kline/candlestick data.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `interval` - Kline interval
    /// * `start_time` - Start time in milliseconds
    /// * `end_time` - End time in milliseconds
    /// * `limit` - Default 500; max 1000
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::KlineInterval;
    ///
    /// let client = Binance::new_unauthenticated()?;
    /// let klines = client.market().klines("BTCUSDT", KlineInterval::Hours1, None, None, Some(10)).await?;
    /// for kline in klines {
    ///     println!("Open: {}, Close: {}", kline.open, kline.close);
    /// }
    /// ```
    pub async fn klines(
        &self,
        symbol: &str,
        interval: KlineInterval,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> Result<Vec<Kline>> {
        let mut query = format!("symbol={}&interval={}", symbol, interval);
        if let Some(start) = start_time {
            query.push_str(&format!("&startTime={}", start));
        }
        if let Some(end) = end_time {
            query.push_str(&format!("&endTime={}", end));
        }
        if let Some(l) = limit {
            query.push_str(&format!("&limit={}", l));
        }

        // Klines come as arrays, need to parse manually
        let raw: Vec<Vec<Value>> = self.client.get(API_V3_KLINES, Some(&query)).await?;

        Ok(parse_klines(raw))
    }

    /// Get UI optimized kline/candlestick data.
    ///
    /// This endpoint mirrors the `/api/v3/klines` response format.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `interval` - Kline interval
    /// * `start_time` - Start time in milliseconds
    /// * `end_time` - End time in milliseconds
    /// * `limit` - Default 500; max 1000
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use binance_api_client::KlineInterval;
    ///
    /// let client = Binance::new_unauthenticated()?;
    /// let klines = client
    ///     .market()
    ///     .ui_klines("BTCUSDT", KlineInterval::Hours1, None, None, Some(10))
    ///     .await?;
    /// ```
    pub async fn ui_klines(
        &self,
        symbol: &str,
        interval: KlineInterval,
        start_time: Option<u64>,
        end_time: Option<u64>,
        limit: Option<u16>,
    ) -> Result<Vec<Kline>> {
        let mut query = format!("symbol={}&interval={}", symbol, interval);
        if let Some(start) = start_time {
            query.push_str(&format!("&startTime={}", start));
        }
        if let Some(end) = end_time {
            query.push_str(&format!("&endTime={}", end));
        }
        if let Some(l) = limit {
            query.push_str(&format!("&limit={}", l));
        }

        let raw: Vec<Vec<Value>> = self.client.get(API_V3_UI_KLINES, Some(&query)).await?;

        Ok(parse_klines(raw))
    }

    /// Get current average price for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let avg = client.market().avg_price("BTCUSDT").await?;
    /// println!("Average price over {} mins: {}", avg.mins, avg.price);
    /// ```
    pub async fn avg_price(&self, symbol: &str) -> Result<AveragePrice> {
        let query = format!("symbol={}", symbol);
        self.client.get(API_V3_AVG_PRICE, Some(&query)).await
    }

    /// Get 24hr ticker price change statistics.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let ticker = client.market().ticker_24h("BTCUSDT").await?;
    /// println!("Price change: {}%", ticker.price_change_percent);
    /// ```
    pub async fn ticker_24h(&self, symbol: &str) -> Result<Ticker24h> {
        let query = format!("symbol={}", symbol);
        self.client.get(API_V3_TICKER_24HR, Some(&query)).await
    }

    /// Get 24hr ticker price change statistics for all symbols.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let tickers = client.market().ticker_24h_all().await?;
    /// ```
    pub async fn ticker_24h_all(&self) -> Result<Vec<Ticker24h>> {
        self.client.get(API_V3_TICKER_24HR, None).await
    }

    /// Get trading day ticker statistics (FULL).
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `time_zone` - Optional timezone (e.g., "0" or "-1:00")
    /// * `symbol_status` - Optional symbol trading status filter
    pub async fn trading_day_ticker(
        &self,
        symbol: &str,
        time_zone: Option<&str>,
        symbol_status: Option<SymbolStatus>,
    ) -> Result<TradingDayTicker> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(tz) = time_zone {
            params.push(("timeZone", tz.to_string()));
        }
        if let Some(status) = symbol_status {
            params.push(("symbolStatus", status.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_with_params(API_V3_TICKER_TRADING_DAY, &params_ref)
            .await
    }

    /// Get trading day ticker statistics (MINI).
    pub async fn trading_day_ticker_mini(
        &self,
        symbol: &str,
        time_zone: Option<&str>,
        symbol_status: Option<SymbolStatus>,
    ) -> Result<TradingDayTickerMini> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        params.push(("type", TickerType::Mini.to_string()));

        if let Some(tz) = time_zone {
            params.push(("timeZone", tz.to_string()));
        }
        if let Some(status) = symbol_status {
            params.push(("symbolStatus", status.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_with_params(API_V3_TICKER_TRADING_DAY, &params_ref)
            .await
    }

    /// Get trading day ticker statistics (FULL) for multiple symbols.
    pub async fn trading_day_tickers(
        &self,
        symbols: &[&str],
        time_zone: Option<&str>,
        symbol_status: Option<SymbolStatus>,
    ) -> Result<Vec<TradingDayTicker>> {
        let symbols_json = serde_json::to_string(symbols).unwrap_or_default();
        let mut params: Vec<(&str, String)> =
            vec![("symbols", urlencoding::encode(&symbols_json).into_owned())];

        if let Some(tz) = time_zone {
            params.push(("timeZone", tz.to_string()));
        }
        if let Some(status) = symbol_status {
            params.push(("symbolStatus", status.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_with_params(API_V3_TICKER_TRADING_DAY, &params_ref)
            .await
    }

    /// Get trading day ticker statistics (MINI) for multiple symbols.
    pub async fn trading_day_tickers_mini(
        &self,
        symbols: &[&str],
        time_zone: Option<&str>,
        symbol_status: Option<SymbolStatus>,
    ) -> Result<Vec<TradingDayTickerMini>> {
        let symbols_json = serde_json::to_string(symbols).unwrap_or_default();
        let mut params: Vec<(&str, String)> =
            vec![("symbols", urlencoding::encode(&symbols_json).into_owned())];

        params.push(("type", TickerType::Mini.to_string()));

        if let Some(tz) = time_zone {
            params.push(("timeZone", tz.to_string()));
        }
        if let Some(status) = symbol_status {
            params.push(("symbolStatus", status.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_with_params(API_V3_TICKER_TRADING_DAY, &params_ref)
            .await
    }

    /// Get rolling window ticker statistics (FULL).
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    /// * `window_size` - Optional window size (e.g., "1d", "15m")
    /// * `symbol_status` - Optional symbol trading status filter
    pub async fn rolling_window_ticker(
        &self,
        symbol: &str,
        window_size: Option<&str>,
        symbol_status: Option<SymbolStatus>,
    ) -> Result<RollingWindowTicker> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        if let Some(window) = window_size {
            params.push(("windowSize", window.to_string()));
        }
        if let Some(status) = symbol_status {
            params.push(("symbolStatus", status.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_with_params(API_V3_TICKER, &params_ref)
            .await
    }

    /// Get rolling window ticker statistics (MINI).
    pub async fn rolling_window_ticker_mini(
        &self,
        symbol: &str,
        window_size: Option<&str>,
        symbol_status: Option<SymbolStatus>,
    ) -> Result<RollingWindowTickerMini> {
        let mut params: Vec<(&str, String)> = vec![("symbol", symbol.to_string())];

        params.push(("type", TickerType::Mini.to_string()));

        if let Some(window) = window_size {
            params.push(("windowSize", window.to_string()));
        }
        if let Some(status) = symbol_status {
            params.push(("symbolStatus", status.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_with_params(API_V3_TICKER, &params_ref)
            .await
    }

    /// Get rolling window ticker statistics (FULL) for multiple symbols.
    pub async fn rolling_window_tickers(
        &self,
        symbols: &[&str],
        window_size: Option<&str>,
        symbol_status: Option<SymbolStatus>,
    ) -> Result<Vec<RollingWindowTicker>> {
        let symbols_json = serde_json::to_string(symbols).unwrap_or_default();
        let mut params: Vec<(&str, String)> =
            vec![("symbols", urlencoding::encode(&symbols_json).into_owned())];

        if let Some(window) = window_size {
            params.push(("windowSize", window.to_string()));
        }
        if let Some(status) = symbol_status {
            params.push(("symbolStatus", status.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_with_params(API_V3_TICKER, &params_ref)
            .await
    }

    /// Get rolling window ticker statistics (MINI) for multiple symbols.
    pub async fn rolling_window_tickers_mini(
        &self,
        symbols: &[&str],
        window_size: Option<&str>,
        symbol_status: Option<SymbolStatus>,
    ) -> Result<Vec<RollingWindowTickerMini>> {
        let symbols_json = serde_json::to_string(symbols).unwrap_or_default();
        let mut params: Vec<(&str, String)> =
            vec![("symbols", urlencoding::encode(&symbols_json).into_owned())];

        params.push(("type", TickerType::Mini.to_string()));

        if let Some(window) = window_size {
            params.push(("windowSize", window.to_string()));
        }
        if let Some(status) = symbol_status {
            params.push(("symbolStatus", status.to_string()));
        }

        let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .get_with_params(API_V3_TICKER, &params_ref)
            .await
    }

    /// Get latest price for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let price = client.market().price("BTCUSDT").await?;
    /// println!("BTC/USDT: {}", price.price);
    /// ```
    pub async fn price(&self, symbol: &str) -> Result<TickerPrice> {
        let query = format!("symbol={}", symbol);
        self.client.get(API_V3_TICKER_PRICE, Some(&query)).await
    }

    /// Get latest prices for all symbols.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let prices = client.market().prices().await?;
    /// for price in prices {
    ///     println!("{}: {}", price.symbol, price.price);
    /// }
    /// ```
    pub async fn prices(&self) -> Result<Vec<TickerPrice>> {
        self.client.get(API_V3_TICKER_PRICE, None).await
    }

    /// Get latest prices for specific symbols.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let prices = client.market().prices_for(&["BTCUSDT", "ETHUSDT"]).await?;
    /// ```
    pub async fn prices_for(&self, symbols: &[&str]) -> Result<Vec<TickerPrice>> {
        let symbols_json = serde_json::to_string(symbols).unwrap_or_default();
        let query = format!("symbols={}", urlencoding::encode(&symbols_json));
        self.client.get(API_V3_TICKER_PRICE, Some(&query)).await
    }

    /// Get best price/qty on the order book for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Trading pair symbol
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let ticker = client.market().book_ticker("BTCUSDT").await?;
    /// println!("Best bid: {} @ {}", ticker.bid_qty, ticker.bid_price);
    /// println!("Best ask: {} @ {}", ticker.ask_qty, ticker.ask_price);
    /// ```
    pub async fn book_ticker(&self, symbol: &str) -> Result<BookTicker> {
        let query = format!("symbol={}", symbol);
        self.client
            .get(API_V3_TICKER_BOOK_TICKER, Some(&query))
            .await
    }

    /// Get best price/qty on the order book for all symbols.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let tickers = client.market().book_tickers().await?;
    /// ```
    pub async fn book_tickers(&self) -> Result<Vec<BookTicker>> {
        self.client.get(API_V3_TICKER_BOOK_TICKER, None).await
    }

    /// Get best price/qty on the order book for specific symbols.
    ///
    /// # Arguments
    ///
    /// * `symbols` - List of symbols
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let client = Binance::new_unauthenticated()?;
    /// let tickers = client.market().book_tickers_for(&["BTCUSDT", "ETHUSDT"]).await?;
    /// ```
    pub async fn book_tickers_for(&self, symbols: &[&str]) -> Result<Vec<BookTicker>> {
        let symbols_json = serde_json::to_string(symbols).unwrap_or_default();
        let query = format!("symbols={}", urlencoding::encode(&symbols_json));
        self.client
            .get(API_V3_TICKER_BOOK_TICKER, Some(&query))
            .await
    }
}

/// Parse a serde_json::Value as f64, handling both strings and numbers.
fn parse_value_as_f64(value: &Value) -> f64 {
    match value {
        Value::String(s) => s.parse().unwrap_or_default(),
        Value::Number(n) => n.as_f64().unwrap_or_default(),
        _ => 0.0,
    }
}

fn parse_klines(raw: Vec<Vec<Value>>) -> Vec<Kline> {
    raw.into_iter()
        .map(|row| Kline {
            open_time: row[0].as_i64().unwrap_or_default(),
            open: parse_value_as_f64(&row[1]),
            high: parse_value_as_f64(&row[2]),
            low: parse_value_as_f64(&row[3]),
            close: parse_value_as_f64(&row[4]),
            volume: parse_value_as_f64(&row[5]),
            close_time: row[6].as_i64().unwrap_or_default(),
            quote_asset_volume: parse_value_as_f64(&row[7]),
            number_of_trades: row[8].as_i64().unwrap_or_default(),
            taker_buy_base_asset_volume: parse_value_as_f64(&row[9]),
            taker_buy_quote_asset_volume: parse_value_as_f64(&row[10]),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_value_as_f64_string() {
        let value = Value::String("123.456".to_string());
        assert_eq!(parse_value_as_f64(&value), 123.456);
    }

    #[test]
    fn test_parse_value_as_f64_number() {
        let value = serde_json::json!(123.456);
        assert_eq!(parse_value_as_f64(&value), 123.456);
    }

    #[test]
    fn test_parse_value_as_f64_invalid() {
        let value = Value::Null;
        assert_eq!(parse_value_as_f64(&value), 0.0);
    }
}
