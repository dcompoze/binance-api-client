//! Tests for WebSocket event parsing and serialization.
//!
//! These tests verify that WebSocket events are correctly deserialized
//! from JSON messages.

use binance_api_client::WebSocketEvent;

#[test]
fn test_parse_agg_trade_event() {
    let json = r#"{
        "e": "aggTrade",
        "E": 1704067200000,
        "s": "BTCUSDT",
        "a": 26129,
        "p": "50000.00000000",
        "q": "0.01000000",
        "f": 100,
        "l": 105,
        "T": 1704067199999,
        "m": true,
        "M": true
    }"#;

    let event: WebSocketEvent = serde_json::from_str(json).unwrap();

    match event {
        WebSocketEvent::AggTrade(trade) => {
            assert_eq!(trade.symbol, "BTCUSDT");
            assert_eq!(trade.agg_trade_id, 26129);
            assert_eq!(trade.price, 50000.0);
            assert_eq!(trade.quantity, 0.01);
            assert_eq!(trade.first_trade_id, 100);
            assert_eq!(trade.last_trade_id, 105);
            assert!(trade.is_buyer_maker);
        }
        _ => panic!("Expected AggTrade event"),
    }
}

#[test]
fn test_parse_trade_event() {
    let json = r#"{
        "e": "trade",
        "E": 1704067200000,
        "s": "BTCUSDT",
        "t": 12345,
        "p": "50000.00000000",
        "q": "0.01000000",
        "b": 88,
        "a": 50,
        "T": 1704067199999,
        "m": true,
        "M": true
    }"#;

    let event: WebSocketEvent = serde_json::from_str(json).unwrap();

    match event {
        WebSocketEvent::Trade(trade) => {
            assert_eq!(trade.symbol, "BTCUSDT");
            assert_eq!(trade.trade_id, 12345);
            assert_eq!(trade.price, 50000.0);
            assert_eq!(trade.quantity, 0.01);
            assert_eq!(trade.buyer_order_id, 88);
            assert_eq!(trade.seller_order_id, 50);
            assert!(trade.is_buyer_maker);
        }
        _ => panic!("Expected Trade event"),
    }
}

#[test]
fn test_parse_kline_event() {
    let json = r#"{
        "e": "kline",
        "E": 1704067200000,
        "s": "BTCUSDT",
        "k": {
            "t": 1704067200000,
            "T": 1704070799999,
            "s": "BTCUSDT",
            "i": "1h",
            "f": 100,
            "L": 200,
            "o": "50000.00000000",
            "c": "50250.00000000",
            "h": "50500.00000000",
            "l": "49500.00000000",
            "v": "100.00000000",
            "n": 150,
            "x": false,
            "q": "5000000.00000000",
            "V": "60.00000000",
            "Q": "3000000.00000000",
            "B": "0"
        }
    }"#;

    let event: WebSocketEvent = serde_json::from_str(json).unwrap();

    match event {
        WebSocketEvent::Kline(kline) => {
            assert_eq!(kline.symbol, "BTCUSDT");
            assert_eq!(kline.kline.open, 50000.0);
            assert_eq!(kline.kline.close, 50250.0);
            assert_eq!(kline.kline.high, 50500.0);
            assert_eq!(kline.kline.low, 49500.0);
            assert_eq!(kline.kline.volume, 100.0);
            assert!(!kline.kline.is_closed);
        }
        _ => panic!("Expected Kline event"),
    }
}

#[test]
fn test_parse_mini_ticker_event() {
    let json = r#"{
        "e": "24hrMiniTicker",
        "E": 1704067200000,
        "s": "BTCUSDT",
        "c": "50000.00000000",
        "o": "49500.00000000",
        "h": "51000.00000000",
        "l": "49000.00000000",
        "v": "1000.00000000",
        "q": "50125000.00000000"
    }"#;

    let event: WebSocketEvent = serde_json::from_str(json).unwrap();

    match event {
        WebSocketEvent::MiniTicker(ticker) => {
            assert_eq!(ticker.symbol, "BTCUSDT");
            assert_eq!(ticker.close, 50000.0);
            assert_eq!(ticker.open, 49500.0);
            assert_eq!(ticker.high, 51000.0);
            assert_eq!(ticker.low, 49000.0);
            assert_eq!(ticker.volume, 1000.0);
            assert_eq!(ticker.quote_volume, 50125000.0);
        }
        _ => panic!("Expected MiniTicker event"),
    }
}

#[test]
fn test_parse_ticker_event() {
    let json = r#"{
        "e": "24hrTicker",
        "E": 1704067200000,
        "s": "BTCUSDT",
        "p": "500.00000000",
        "P": "1.010",
        "w": "50125.00000000",
        "x": "49500.00000000",
        "c": "50000.00000000",
        "Q": "0.01000000",
        "b": "49999.00000000",
        "B": "1.50000000",
        "a": "50001.00000000",
        "A": "0.75000000",
        "o": "49500.00000000",
        "h": "51000.00000000",
        "l": "49000.00000000",
        "v": "1000.00000000",
        "q": "50125000.00000000",
        "O": 1703980800000,
        "C": 1704067199999,
        "F": 100,
        "L": 200,
        "n": 76
    }"#;

    let event: WebSocketEvent = serde_json::from_str(json).unwrap();

    match event {
        WebSocketEvent::Ticker(ticker) => {
            assert_eq!(ticker.symbol, "BTCUSDT");
            assert_eq!(ticker.price_change, 500.0);
            assert_eq!(ticker.price_change_percent, 1.010);
            assert_eq!(ticker.close_price, 50000.0);
            assert_eq!(ticker.bid_price, 49999.0);
            assert_eq!(ticker.ask_price, 50001.0);
            assert_eq!(ticker.open_price, 49500.0);
            assert_eq!(ticker.high_price, 51000.0);
            assert_eq!(ticker.low_price, 49000.0);
            assert_eq!(ticker.volume, 1000.0);
            assert_eq!(ticker.number_of_trades, 76);
        }
        _ => panic!("Expected Ticker event"),
    }
}

#[test]
fn test_parse_book_ticker_event() {
    let json = r#"{
        "e": "bookTicker",
        "u": 400900217,
        "s": "BTCUSDT",
        "b": "50000.00000000",
        "B": "1.50000000",
        "a": "50001.00000000",
        "A": "0.75000000"
    }"#;

    let event: WebSocketEvent = serde_json::from_str(json).unwrap();

    match event {
        WebSocketEvent::BookTicker(book) => {
            assert_eq!(book.symbol, "BTCUSDT");
            assert_eq!(book.update_id, 400900217);
            assert_eq!(book.bid_price, 50000.0);
            assert_eq!(book.bid_quantity, 1.5);
            assert_eq!(book.ask_price, 50001.0);
            assert_eq!(book.ask_quantity, 0.75);
        }
        _ => panic!("Expected BookTicker event"),
    }
}

#[test]
fn test_parse_depth_event() {
    let json = r#"{
        "e": "depthUpdate",
        "E": 1704067200000,
        "s": "BTCUSDT",
        "U": 157,
        "u": 160,
        "b": [
            ["50000.00000000", "1.50000000"],
            ["49999.00000000", "2.00000000"]
        ],
        "a": [
            ["50001.00000000", "0.75000000"],
            ["50002.00000000", "1.25000000"]
        ]
    }"#;

    let event: WebSocketEvent = serde_json::from_str(json).unwrap();

    match event {
        WebSocketEvent::Depth(depth) => {
            assert_eq!(depth.symbol, "BTCUSDT");
            assert_eq!(depth.first_update_id, 157);
            assert_eq!(depth.final_update_id, 160);
            assert_eq!(depth.bids.len(), 2);
            assert_eq!(depth.asks.len(), 2);
            assert_eq!(depth.bids[0].price, 50000.0);
            assert_eq!(depth.bids[0].quantity, 1.5);
            assert_eq!(depth.asks[0].price, 50001.0);
            assert_eq!(depth.asks[0].quantity, 0.75);
        }
        _ => panic!("Expected Depth event"),
    }
}

#[test]
fn test_parse_combined_stream_message() {
    // Combined stream messages have a wrapper with stream name and data
    let json = r#"{
        "stream": "btcusdt@aggTrade",
        "data": {
            "e": "aggTrade",
            "E": 1704067200000,
            "s": "BTCUSDT",
            "a": 26129,
            "p": "50000.00000000",
            "q": "0.01000000",
            "f": 100,
            "l": 105,
            "T": 1704067199999,
            "m": true,
            "M": true
        }
    }"#;

    #[derive(serde::Deserialize)]
    struct CombinedMessage {
        stream: String,
        data: WebSocketEvent,
    }

    let msg: CombinedMessage = serde_json::from_str(json).unwrap();

    assert_eq!(msg.stream, "btcusdt@aggTrade");
    match msg.data {
        WebSocketEvent::AggTrade(trade) => {
            assert_eq!(trade.symbol, "BTCUSDT");
            assert_eq!(trade.agg_trade_id, 26129);
        }
        _ => panic!("Expected AggTrade event"),
    }
}

#[test]
fn test_parse_account_position_event() {
    let json = r#"{
        "e": "outboundAccountPosition",
        "E": 1704067200000,
        "u": 1704067199999,
        "B": [
            {
                "a": "BTC",
                "f": "1.00000000",
                "l": "0.50000000"
            },
            {
                "a": "USDT",
                "f": "10000.00000000",
                "l": "500.00000000"
            }
        ]
    }"#;

    let event: WebSocketEvent = serde_json::from_str(json).unwrap();

    match event {
        WebSocketEvent::AccountPosition(account) => {
            assert_eq!(account.balances.len(), 2);
            assert_eq!(account.balances[0].asset, "BTC");
            assert_eq!(account.balances[0].free, 1.0);
            assert_eq!(account.balances[0].locked, 0.5);
            assert_eq!(account.balances[1].asset, "USDT");
            assert_eq!(account.balances[1].free, 10000.0);
        }
        _ => panic!("Expected AccountPosition event"),
    }
}

#[test]
fn test_parse_balance_update_event() {
    let json = r#"{
        "e": "balanceUpdate",
        "E": 1704067200000,
        "a": "BTC",
        "d": "0.01000000",
        "T": 1704067199999
    }"#;

    let event: WebSocketEvent = serde_json::from_str(json).unwrap();

    match event {
        WebSocketEvent::BalanceUpdate(update) => {
            assert_eq!(update.asset, "BTC");
            assert_eq!(update.balance_delta, 0.01);
            assert_eq!(update.clear_time, 1704067199999);
        }
        _ => panic!("Expected BalanceUpdate event"),
    }
}

#[test]
fn test_parse_execution_report_event() {
    let json = r#"{
        "e": "executionReport",
        "E": 1704067200000,
        "s": "BTCUSDT",
        "c": "my_order_123",
        "S": "BUY",
        "o": "LIMIT",
        "f": "GTC",
        "q": "0.01000000",
        "p": "50000.00000000",
        "P": "0.00000000",
        "F": "0.00000000",
        "g": -1,
        "C": "",
        "x": "NEW",
        "X": "NEW",
        "r": "NONE",
        "i": 4293153,
        "l": "0.00000000",
        "z": "0.00000000",
        "L": "0.00000000",
        "n": "0",
        "N": null,
        "T": 1704067199999,
        "t": -1,
        "I": 8641984,
        "w": true,
        "m": false,
        "M": false,
        "O": 1704067199999,
        "Z": "0.00000000",
        "Y": "0.00000000",
        "Q": "0.00000000"
    }"#;

    let event: WebSocketEvent = serde_json::from_str(json).unwrap();

    match event {
        WebSocketEvent::ExecutionReport(report) => {
            assert_eq!(report.symbol, "BTCUSDT");
            assert_eq!(report.client_order_id, "my_order_123");
            assert_eq!(report.order_id, 4293153);
            assert_eq!(report.quantity, 0.01);
            assert_eq!(report.price, 50000.0);
        }
        _ => panic!("Expected ExecutionReport event"),
    }
}
