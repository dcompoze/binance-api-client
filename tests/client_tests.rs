//! Integration tests for client behavior.
//!
//! Covers signed request encoding, error handling, rate limit handling,
//! time synchronization, and the testnet SAPI guard.

use binance_api_client::{Binance, Config, Credentials, Error};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

fn test_client(mock_server: &MockServer) -> Binance {
    let config = Config::builder()
        .rest_api_endpoint(mock_server.uri())
        .build();
    Binance::with_config(config, Some(("test_api_key", "test_secret_key"))).unwrap()
}

#[tokio::test]
async fn test_signed_request_is_percent_encoded_and_signed_over_encoded_payload() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/myTrades"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    // A symbol with reserved characters must be encoded before signing.
    let result = client
        .account()
        .my_trades("BTC&USDT=X", None, None, None, None)
        .await;
    assert!(result.is_ok());

    let requests = mock_server.received_requests().await.unwrap();
    assert_eq!(requests.len(), 1);
    let query = requests[0].url.query().unwrap().to_string();

    // The raw query must contain the encoded value, not the raw one.
    assert!(query.contains("symbol=BTC%26USDT%3DX"), "query: {}", query);

    // The signature must verify against the encoded payload as sent.
    let (payload, signature) = query.rsplit_once("&signature=").unwrap();
    let credentials = Credentials::new("test_api_key", "test_secret_key");
    assert_eq!(credentials.sign(payload), signature);
}

#[tokio::test]
async fn test_rate_limited_429_with_retry_after() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/account"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "37")
                .set_body_string(r#"{"code": -1003, "msg": "Too many requests."}"#),
        )
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let error = client.account().get_account().await.unwrap_err();

    match error {
        Error::RateLimited {
            code,
            retry_after,
            ip_banned,
            ..
        } => {
            assert_eq!(code, -1003);
            assert_eq!(retry_after, Some(37));
            assert!(!ip_banned);
        }
        other => panic!("Expected RateLimited, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_ip_ban_418() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/account"))
        .respond_with(
            ResponseTemplate::new(418)
                .insert_header("Retry-After", "86400")
                .set_body_string(r#"{"code": -1003, "msg": "IP banned."}"#),
        )
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let error = client.account().get_account().await.unwrap_err();

    match error {
        Error::RateLimited {
            retry_after,
            ip_banned,
            ..
        } => {
            assert_eq!(retry_after, Some(86400));
            assert!(ip_banned);
        }
        other => panic!("Expected RateLimited, got: {:?}", other),
    }
    assert!(error.is_rate_limit());
}

#[tokio::test]
async fn test_error_body_preserved_on_500() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/account"))
        .respond_with(
            ResponseTemplate::new(500)
                .set_body_string(r#"{"code": -1000, "msg": "An unknown error occurred."}"#),
        )
        .expect(1..)
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let error = client.account().get_account().await.unwrap_err();

    match error {
        Error::Api { code, message } => {
            assert_eq!(code, -1000);
            assert_eq!(message, "An unknown error occurred.");
        }
        other => panic!("Expected Api error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_rate_limit_usage_headers_recorded() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/v3/account"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-mbx-used-weight-1m", "321")
                .insert_header("x-mbx-order-count-10s", "12")
                .set_body_string(load_account_body()),
        )
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    client.account().get_account().await.unwrap();

    let usage = client.client().rate_limit_usage();
    assert_eq!(usage.get("x-mbx-used-weight-1m"), Some(&321));
    assert_eq!(usage.get("x-mbx-order-count-10s"), Some(&12));
}

#[tokio::test]
async fn test_sync_time_sets_offset() {
    let mock_server = MockServer::start().await;

    let future_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
        + 90_000;

    Mock::given(method("GET"))
        .and(path("/api/v3/time"))
        .respond_with(move |_: &Request| {
            ResponseTemplate::new(200)
                .set_body_string(format!(r#"{{"serverTime": {}}}"#, future_time))
        })
        .mount(&mock_server)
        .await;

    let client = test_client(&mock_server);
    let offset = client.client().sync_time().await.unwrap();

    // The mocked server is 90 seconds ahead.
    assert!(offset > 80_000 && offset < 100_000, "offset: {}", offset);
    assert_eq!(client.client().time_offset(), offset);
}

#[tokio::test]
async fn test_sapi_not_available_on_testnet() {
    let config = Config::testnet();
    let client = Binance::with_config(config, Some(("key", "secret"))).unwrap();

    let error = client.wallet().account_status().await.unwrap_err();
    match error {
        Error::InvalidConfig(message) => {
            assert!(message.contains("testnet"));
        }
        other => panic!("Expected InvalidConfig, got: {:?}", other),
    }
}

fn load_account_body() -> String {
    r#"{
        "makerCommission": 15,
        "takerCommission": 15,
        "buyerCommission": 0,
        "sellerCommission": 0,
        "canTrade": true,
        "canWithdraw": true,
        "canDeposit": true,
        "brokered": false,
        "requireSelfTradePrevention": false,
        "preventSor": false,
        "updateTime": 123456789,
        "accountType": "SPOT",
        "balances": [],
        "permissions": ["SPOT"],
        "uid": 354937868
    }"#
    .to_string()
}
