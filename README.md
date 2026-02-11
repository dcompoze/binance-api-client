# binance-api-client

Async Rust client for Binance Spot REST and WebSocket APIs:

- Async REST client for market, account, wallet, and margin endpoints.
- WebSocket support for market streams and user data streams.
- Auth support for HMAC-SHA256, RSA-SHA256, and Ed25519 signatures.
- Production, testnet, and Binance.US configuration.
- Typed request builders and typed response models.

## Library

Unauthenticated usage:

```rust,ignore
use binance_api_client::Binance;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Binance::new_unauthenticated()?;
    let ticker = client.market().price("BTCUSDT").await?;
    println!("BTCUSDT: {}", ticker.price);
    Ok(())
}
```

Authenticated usage:

```rust,ignore
use binance_api_client::Binance;

#[tokio::main]
async fn main() -> Result<()> {
    let client = Binance::new("your_api_key", "your_secret_key")?;
    let account = client.account().get_account().await?;
    println!("Balances: {}", account.balances.len());
    Ok(())
}
```

## Configuration

- `Config::default()` for Binance production.
- `Config::testnet()` for Binance Spot testnet.
- `Config::binance_us()` for Binance.US.

## Project structure

```text
.
├── examples/            # Runnable examples showing common client usage.
├── src/                 # Library implementation.
│   ├── rest/            # REST endpoint clients.
│   ├── ws/              # WebSocket client and stream management.
│   └── models/          # Typed request and response models.
└── tests/               # Integration tests.
    └── mocks/           # Mock fixtures used by tests.
```
