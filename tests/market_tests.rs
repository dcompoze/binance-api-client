//! Integration tests for market API endpoints.
//!
//! These tests use wiremock to mock HTTP responses from the Binance API.

use binance_api_client::{Binance, Config, KlineInterval};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to create a test client with a mock server
async fn test_client(mock_server: &MockServer) -> Binance {
    let config = Config::builder()
        .rest_api_endpoint(mock_server.uri())
        .build();
    Binance::with_config(config, None::<(&str, &str)>).unwrap()
}

/// Helper to load mock response from file
fn load_mock(filename: &str) -> String {
    std::fs::read_to_string(format!("tests/mocks/market/{}", filename))
        .unwrap_or_else(|_| panic!("Failed to load mock file: {}", filename))
}

#[tokio::test]
async fn test_ping() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/ping"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("ping.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client.market().ping().await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_server_time() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/time"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("server_time.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client.market().server_time().await;

    assert!(result.is_ok());
    let time = result.unwrap();
    assert_eq!(time.server_time, 1704067200000);
}

#[tokio::test]
async fn test_exchange_info() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/exchangeInfo"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("exchange_info.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client.market().exchange_info().await;

    assert!(result.is_ok());
    let info = result.unwrap();
    assert_eq!(info.timezone, "UTC");
    assert_eq!(info.symbols.len(), 1);
    assert_eq!(info.symbols[0].symbol, "BTCUSDT");
    assert_eq!(info.symbols[0].base_asset, "BTC");
    assert_eq!(info.symbols[0].quote_asset, "USDT");
}

#[tokio::test]
async fn test_depth() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/depth"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("depth.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client.market().depth("BTCUSDT", Some(10)).await;

    assert!(result.is_ok());
    let depth = result.unwrap();
    assert_eq!(depth.last_update_id, 1027024);
    assert_eq!(depth.bids.len(), 3);
    assert_eq!(depth.asks.len(), 3);
    assert_eq!(depth.bids[0].price, 50000.0);
    assert_eq!(depth.bids[0].quantity, 1.5);
    assert_eq!(depth.asks[0].price, 50001.0);
    assert_eq!(depth.asks[0].quantity, 0.75);
}

#[tokio::test]
async fn test_trades() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/trades"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("trades.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client.market().trades("BTCUSDT", Some(10)).await;

    assert!(result.is_ok());
    let trades = result.unwrap();
    assert_eq!(trades.len(), 2);
    assert_eq!(trades[0].id, 28457);
    assert_eq!(trades[0].price, 50000.0);
    assert_eq!(trades[0].quantity, 0.01);
    assert!(trades[0].is_buyer_maker);
}

#[tokio::test]
async fn test_agg_trades() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/aggTrades"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("agg_trades.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client
        .market()
        .agg_trades("BTCUSDT", None, None, None, Some(10))
        .await;

    assert!(result.is_ok());
    let trades = result.unwrap();
    assert_eq!(trades.len(), 2);
    assert_eq!(trades[0].agg_trade_id, 26129);
    assert_eq!(trades[0].price, 50000.0);
    assert_eq!(trades[0].quantity, 0.01);
    assert!(trades[0].is_buyer_maker);
}

#[tokio::test]
async fn test_klines() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/klines"))
        .and(query_param("symbol", "BTCUSDT"))
        .and(query_param("interval", "1h"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("klines.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client
        .market()
        .klines("BTCUSDT", KlineInterval::Hours1, None, None, Some(10))
        .await;

    assert!(result.is_ok());
    let klines = result.unwrap();
    assert_eq!(klines.len(), 2);
    assert_eq!(klines[0].open_time, 1704067200000);
    assert_eq!(klines[0].open, 50000.0);
    assert_eq!(klines[0].high, 50500.0);
    assert_eq!(klines[0].low, 49500.0);
    assert_eq!(klines[0].close, 50250.0);
    assert_eq!(klines[0].volume, 100.0);
}

#[tokio::test]
async fn test_avg_price() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/avgPrice"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("avg_price.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client.market().avg_price("BTCUSDT").await;

    assert!(result.is_ok());
    let avg = result.unwrap();
    assert_eq!(avg.mins, 5);
    assert_eq!(avg.price, 50125.1234);
}

#[tokio::test]
async fn test_ticker_24h() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/ticker/24hr"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("ticker_24h.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client.market().ticker_24h("BTCUSDT").await;

    assert!(result.is_ok());
    let ticker = result.unwrap();
    assert_eq!(ticker.symbol, "BTCUSDT");
    assert_eq!(ticker.price_change, 500.0);
    assert_eq!(ticker.price_change_percent, 1.010);
    assert_eq!(ticker.high_price, 51000.0);
    assert_eq!(ticker.low_price, 49000.0);
    assert_eq!(ticker.volume, 1000.0);
    assert_eq!(ticker.count, 76);
}

#[tokio::test]
async fn test_price() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/ticker/price"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("ticker_price.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client.market().price("BTCUSDT").await;

    assert!(result.is_ok());
    let price = result.unwrap();
    assert_eq!(price.symbol, "BTCUSDT");
    assert_eq!(price.price, 50000.0);
}

#[tokio::test]
async fn test_prices() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/ticker/price"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("ticker_prices.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client.market().prices().await;

    assert!(result.is_ok());
    let prices = result.unwrap();
    assert_eq!(prices.len(), 2);
    assert_eq!(prices[0].symbol, "BTCUSDT");
    assert_eq!(prices[0].price, 50000.0);
    assert_eq!(prices[1].symbol, "ETHUSDT");
    assert_eq!(prices[1].price, 3000.0);
}

#[tokio::test]
async fn test_book_ticker() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/ticker/bookTicker"))
        .and(query_param("symbol", "BTCUSDT"))
        .respond_with(ResponseTemplate::new(200).set_body_string(load_mock("book_ticker.json")))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server).await;
    let result = client.market().book_ticker("BTCUSDT").await;

    assert!(result.is_ok());
    let ticker = result.unwrap();
    assert_eq!(ticker.symbol, "BTCUSDT");
    assert_eq!(ticker.bid_price, 50000.0);
    assert_eq!(ticker.bid_qty, 1.5);
    assert_eq!(ticker.ask_price, 50001.0);
    assert_eq!(ticker.ask_qty, 0.75);
}
