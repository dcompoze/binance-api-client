//! Deserialization tests for models added for the 2026 API surface.

use binance_api_client::models::websocket::WebSocketEvent;
use binance_api_client::{
    BlockTrade, ExecutionRule, ExecutionRules, ExpiryReason, MyFilters, Order, ReferencePrice,
    ReferencePriceCalculation, Symbol, SymbolFilter, SymbolStatus,
};

#[test]
fn test_execution_rules_deserialization() {
    let json = r#"{
        "symbolRules": [
            {
                "symbol": "BAZUSD",
                "rules": [
                    {
                        "ruleType": "PRICE_RANGE",
                        "bidLimitMultUp": "1.0001",
                        "bidLimitMultDown": "0.9999",
                        "askLimitMultUp": "1.0001",
                        "askLimitMultDown": "0.9999"
                    }
                ]
            }
        ]
    }"#;
    let rules: ExecutionRules = serde_json::from_str(json).unwrap();
    assert_eq!(rules.symbol_rules.len(), 1);
    assert_eq!(rules.symbol_rules[0].symbol, "BAZUSD");
    match &rules.symbol_rules[0].rules[0] {
        ExecutionRule::PriceRange {
            bid_limit_mult_up, ..
        } => assert_eq!(*bid_limit_mult_up, 1.0001),
        other => panic!("Unexpected rule: {:?}", other),
    }
}

#[test]
fn test_reference_price_null_deserialization() {
    let json = r#"{"symbol": "BAZUSD", "referencePrice": null, "timestamp": 1770736694138}"#;
    let price: ReferencePrice = serde_json::from_str(json).unwrap();
    assert_eq!(price.symbol, "BAZUSD");
    assert!(price.reference_price.is_none());
}

#[test]
fn test_reference_price_calculation_deserialization() {
    let json = r#"{
        "symbol": "BAZUSD",
        "calculationType": "ARITHMETIC_MEAN",
        "bucketCount": 10,
        "bucketWidthMs": 1000
    }"#;
    let calc: ReferencePriceCalculation = serde_json::from_str(json).unwrap();
    assert_eq!(
        calc,
        ReferencePriceCalculation::ArithmeticMean {
            bucket_count: 10,
            bucket_width_ms: 1000
        }
    );

    let json =
        r#"{"symbol": "BAZUSD", "calculationType": "EXTERNAL", "externalCalculationId": 42}"#;
    let calc: ReferencePriceCalculation = serde_json::from_str(json).unwrap();
    assert_eq!(
        calc,
        ReferencePriceCalculation::External {
            external_calculation_id: 42
        }
    );
}

#[test]
fn test_block_trade_deserialization() {
    let json = r#"[
        {
            "id": 582,
            "price": "0.052",
            "qty": "5838",
            "quoteQty": "303.576",
            "time": 1772506983321,
            "isBuyerMaker": true
        }
    ]"#;
    let trades: Vec<BlockTrade> = serde_json::from_str(json).unwrap();
    assert_eq!(trades[0].id, 582);
    assert_eq!(trades[0].price, 0.052);
    assert!(trades[0].is_buyer_maker);
}

#[test]
fn test_my_filters_with_max_asset() {
    let json = r#"{
        "exchangeFilters": [
            {"filterType": "EXCHANGE_MAX_NUM_ORDERS", "maxNumOrders": 1000}
        ],
        "symbolFilters": [
            {"filterType": "MAX_NUM_ORDER_LISTS", "maxNumOrderLists": 20},
            {"filterType": "MAX_NUM_ORDER_AMENDS", "maxNumOrderAmends": 10}
        ],
        "assetFilters": [
            {"filterType": "MAX_ASSET", "asset": "JPY", "limit": "1000000.00000000"}
        ]
    }"#;
    let filters: MyFilters = serde_json::from_str(json).unwrap();
    assert_eq!(
        filters.symbol_filters[0],
        SymbolFilter::MaxNumOrderLists {
            max_num_order_lists: 20
        }
    );
    assert_eq!(
        filters.symbol_filters[1],
        SymbolFilter::MaxNumOrderAmends {
            max_num_order_amends: 10
        }
    );
    match &filters.asset_filters[0] {
        SymbolFilter::MaxAsset { asset, limit } => {
            assert_eq!(asset, "JPY");
            assert_eq!(*limit, 1000000.0);
        }
        other => panic!("Unexpected filter: {:?}", other),
    }
}

#[test]
fn test_symbol_new_flags_and_cancel_only_status() {
    let json = r#"{
        "symbol": "BTCUSDT",
        "status": "CANCEL_ONLY",
        "baseAsset": "BTC",
        "baseAssetPrecision": 8,
        "quoteAsset": "USDT",
        "quotePrecision": 8,
        "quoteAssetPrecision": 8,
        "orderTypes": ["LIMIT", "MARKET"],
        "icebergAllowed": true,
        "ocoAllowed": true,
        "otoAllowed": true,
        "opoAllowed": true,
        "pegInstructionsAllowed": true,
        "filters": []
    }"#;
    let symbol: Symbol = serde_json::from_str(json).unwrap();
    assert_eq!(symbol.status, SymbolStatus::CancelOnly);
    assert!(symbol.oto_allowed);
    assert!(symbol.opo_allowed);
    assert!(symbol.peg_instructions_allowed);
}

#[test]
fn test_order_with_expiry_reason_and_pegged_fields() {
    let json = r#"{
        "symbol": "BTCUSDT",
        "orderId": 1,
        "orderListId": -1,
        "clientOrderId": "abc",
        "price": "0.1",
        "origQty": "1.0",
        "executedQty": "0.0",
        "cummulativeQuoteQty": "0.0",
        "status": "EXPIRED",
        "timeInForce": "GTC",
        "type": "LIMIT",
        "side": "BUY",
        "stopPrice": "0.0",
        "icebergQty": "0.0",
        "time": 1499827319559,
        "updateTime": 1499827319559,
        "isWorking": true,
        "origQuoteOrderQty": "0.0",
        "expiryReason": "UNFILLED_IOC_QUANTITY_EXPIRED",
        "pegPriceType": "PRIMARY_PEG",
        "pegOffsetType": "PRICE_LEVEL",
        "pegOffsetValue": 5,
        "peggedPrice": "0.09"
    }"#;
    let order: Order = serde_json::from_str(json).unwrap();
    assert_eq!(
        order.expiry_reason,
        Some(ExpiryReason::UnfilledIocQuantityExpired)
    );
    assert_eq!(order.peg_price_type.as_deref(), Some("PRIMARY_PEG"));
    assert_eq!(order.peg_offset_value, Some(5));
    assert_eq!(order.pegged_price.as_deref(), Some("0.09"));
}

#[test]
fn test_new_websocket_events() {
    let avg_price = r#"{
        "e": "avgPrice",
        "E": 1693907033000,
        "s": "BTCUSDT",
        "i": "5m",
        "w": "25776.86000000",
        "T": 1693907032213
    }"#;
    let event: WebSocketEvent = serde_json::from_str(avg_price).unwrap();
    assert!(matches!(event, WebSocketEvent::AvgPrice(_)));

    let reference_price = r#"{
        "e": "referencePrice",
        "s": "BAZUSD",
        "r": "1.00",
        "t": 1770313263917
    }"#;
    let event: WebSocketEvent = serde_json::from_str(reference_price).unwrap();
    match event {
        WebSocketEvent::ReferencePrice(p) => {
            assert_eq!(p.reference_price.as_deref(), Some("1.00"));
        }
        other => panic!("Unexpected event: {:?}", other),
    }

    let block_trade = r#"{
        "e": "blockTrade",
        "E": 1772506983582,
        "s": "BNBBTC",
        "t": 582,
        "p": "0.052",
        "q": "5838",
        "T": 1772506983321,
        "m": true
    }"#;
    let event: WebSocketEvent = serde_json::from_str(block_trade).unwrap();
    assert!(matches!(event, WebSocketEvent::BlockTrade(_)));

    let rolling_ticker = r#"{
        "e": "1hTicker",
        "E": 1672515782136,
        "s": "BNBBTC",
        "p": "0.0015",
        "P": "250.00",
        "o": "0.0010",
        "h": "0.0025",
        "l": "0.0010",
        "c": "0.0025",
        "w": "0.0018",
        "v": "10000",
        "q": "18",
        "O": 0,
        "C": 1675216573749,
        "F": 0,
        "L": 18150,
        "n": 18151
    }"#;
    let event: WebSocketEvent = serde_json::from_str(rolling_ticker).unwrap();
    assert!(matches!(event, WebSocketEvent::RollingWindowTicker1h(_)));

    let external_lock = r#"{
        "e": "externalLockUpdate",
        "E": 1581557507324,
        "a": "NEO",
        "d": "10.00000000",
        "T": 1581557507268
    }"#;
    let event: WebSocketEvent = serde_json::from_str(external_lock).unwrap();
    assert!(matches!(event, WebSocketEvent::ExternalLockUpdate(_)));

    let terminated = r#"{"e": "eventStreamTerminated", "E": 1728973001334}"#;
    let event: WebSocketEvent = serde_json::from_str(terminated).unwrap();
    assert!(matches!(event, WebSocketEvent::EventStreamTerminated(_)));

    let shutdown = r#"{"e": "serverShutdown", "E": 1770123456789}"#;
    let event: WebSocketEvent = serde_json::from_str(shutdown).unwrap();
    assert!(matches!(event, WebSocketEvent::ServerShutdown(_)));

    // Unknown event types must not fail deserialization.
    let unknown = r#"{"e": "someFutureEvent", "E": 1}"#;
    let event: WebSocketEvent = serde_json::from_str(unknown).unwrap();
    assert!(matches!(event, WebSocketEvent::Unknown));
}

#[test]
fn test_execution_report_with_expiry_and_pegged_fields() {
    let json = r#"{
        "e": "executionReport",
        "E": 1499405658658,
        "s": "ETHBTC",
        "c": "mUvoqJxFIILMdfAW5iGSOW",
        "S": "BUY",
        "o": "LIMIT",
        "f": "GTC",
        "q": "1.00000000",
        "p": "0.10264410",
        "P": "0.00000000",
        "F": "0.00000000",
        "g": -1,
        "C": "",
        "x": "EXPIRED",
        "X": "EXPIRED",
        "r": "NONE",
        "i": 4293153,
        "l": "0.00000000",
        "z": "0.00000000",
        "L": "0.00000000",
        "n": "0",
        "N": null,
        "T": 1499405658657,
        "t": -1,
        "I": 8641984,
        "w": true,
        "m": false,
        "M": false,
        "O": 1499405658657,
        "Z": "0.00000000",
        "Y": "0.00000000",
        "Q": "0.00000000",
        "eR": "INSUFFICIENT_LIQUIDITY",
        "gP": "PRIMARY_PEG",
        "gOT": "PRICE_LEVEL",
        "gOV": 5,
        "gp": "1.00000000"
    }"#;
    let event: WebSocketEvent = serde_json::from_str(json).unwrap();
    match event {
        WebSocketEvent::ExecutionReport(report) => {
            assert_eq!(
                report.expiry_reason,
                Some(ExpiryReason::InsufficientLiquidity)
            );
            assert_eq!(report.peg_price_type.as_deref(), Some("PRIMARY_PEG"));
            assert_eq!(report.peg_offset_type.as_deref(), Some("PRICE_LEVEL"));
            assert_eq!(report.peg_offset_value, Some(5));
            assert_eq!(report.pegged_price.as_deref(), Some("1.00000000"));
        }
        other => panic!("Unexpected event: {:?}", other),
    }
}
