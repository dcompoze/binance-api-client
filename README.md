# binance-api-client

Async Rust client for Binance Spot REST and WebSocket APIs.

- Async REST client for market, account, wallet, and margin endpoints.
- WebSocket support for market streams and user data streams.
- Auth support for HMAC-SHA256, RSA-SHA256, and Ed25519 signatures.
- Production, testnet, and Binance.US configuration.
- Typed request builders and typed response models.

# Usage

```rust,ignore
use binance_api_client::Binance;

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
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
async fn main() -> binance_api_client::Result<()> {
    let client = Binance::new("your_api_key", "your_secret_key")?;
    let account = client.account().get_account().await?;
    println!("Balances: {}", account.balances.len());
    Ok(())
}
```

# Configuration

- `Config::default()` for Binance production.
- `Config::testnet()` for Binance Spot testnet.
- `Config::binance_us()` for Binance.US.

# Security

- Do not commit real API keys or secrets.
- Use environment variables, for example `BINANCE_API_KEY` and `BINANCE_SECRET_KEY`.
- Start from `.env.example` for local development.

# Examples

Run examples with:

```bash
cargo run --example market_data
cargo run --example websocket_streams
```
