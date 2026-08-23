# binance-api-client

A Rust client library for the [Binance Spot API](https://developers.binance.com/docs/binance-spot-api-docs):

- Broad Spot REST API coverage: market data, trading, order lists, account, wallet, and margin
- WebSocket API for low-latency trading, account queries, and user data streams
- WebSocket support for real-time market data streams with live subscribe/unsubscribe
- Local orderbook management with snapshot and delta synchronization
- HMAC-SHA256, RSA-SHA256, and Ed25519 authentication
- Async/await support with Tokio
- Strongly typed request builders and response models
- Automatic retry with exponential backoff for idempotent requests
- Rate limit visibility from response headers
- Server time synchronization
- Production, testnet, and Binance.US endpoints

User data streams use the WebSocket API (`userDataStream.subscribe`), since the listenKey endpoints were removed from Binance production on 2026-02-20.

## Library

Public REST API client:

```rust
use binance_api_client::Binance;

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    let client = Binance::new_unauthenticated()?;

    let ticker = client.market().price("BTCUSDT").await?;
    println!("BTCUSDT: {}", ticker.price);

    let depth = client.market().depth("BTCUSDT", Some(10)).await?;
    println!("Bids: {}, asks: {}", depth.bids.len(), depth.asks.len());

    Ok(())
}
```

Authenticated client:

```rust
use binance_api_client::{Binance, OrderBuilder, OrderSide, OrderType, TimeInForce};

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    let client = Binance::new("your_api_key", "your_secret_key")?;

    let account = client.account().get_account().await?;
    println!("Balances: {}", account.balances.len());

    let order = OrderBuilder::new("BTCUSDT", OrderSide::Buy, OrderType::Limit)
        .quantity("0.001")
        .price("50000.00")
        .time_in_force(TimeInForce::GTC)
        .build();
    let result = client.account().create_order(&order).await?;
    println!("Order ID: {}", result.order_id);

    Ok(())
}
```

WebSocket market streams:

```rust
use binance_api_client::{Binance, WebSocketEvent};

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    let client = Binance::new_unauthenticated()?;
    let ws = client.streams();

    let stream = ws.agg_trade_stream("btcusdt");
    let mut conn = ws.connect(&stream).await?;

    // Subscribe to more streams on the live connection.
    conn.subscribe(&[ws.trade_stream("ethusdt")]).await?;

    while let Some(event) = conn.next().await {
        match event? {
            WebSocketEvent::AggTrade(trade) => {
                println!("{}: {} @ {}", trade.symbol, trade.quantity, trade.price);
            }
            WebSocketEvent::Trade(trade) => {
                println!("{}: {} @ {}", trade.symbol, trade.quantity, trade.price);
            }
            _ => {}
        }
    }

    Ok(())
}
```

Local orderbook:

```rust
use binance_api_client::{Binance, DepthCacheConfig, DepthCacheManager};

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    let client = Binance::new_unauthenticated()?;

    let config = DepthCacheConfig::default();
    let manager = DepthCacheManager::new(client, "BTCUSDT", config).await?;
    manager.wait_for_sync().await?;

    let cache = manager.get_cache().await;
    if let (Some((bid, _)), Some((ask, _))) = (cache.best_bid(), cache.best_ask()) {
        println!("Best bid: {}, best ask: {}", bid, ask);
        println!("Spread: {:?}", cache.spread());
    }

    Ok(())
}
```

WebSocket API trading and user data streams:

```rust
use binance_api_client::{Binance, OrderBuilder, OrderSide, OrderType, WsApiEvent};

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    let client = Binance::new("your_api_key", "your_secret_key")?;
    let conn = client.ws_api().connect().await?;

    // Place an order over the websocket connection.
    let order = OrderBuilder::new("BTCUSDT", OrderSide::Buy, OrderType::Market)
        .quantity("0.001")
        .build();
    let result = conn.place_order(&order).await?;
    println!("Order response: {}", result);

    // Subscribe to the user data stream (works with any API key type).
    let subscription_id = conn.subscribe_user_data_with_signature().await?;
    println!("Subscribed with id {}", subscription_id);

    let mut events = conn.take_events().await.unwrap();
    while let Some(event) = events.recv().await {
        match event {
            WsApiEvent::UserData { event, .. } => println!("{:?}", event),
            WsApiEvent::ServerShutdown { .. } => break, // Reconnect immediately.
            _ => {}
        }
    }

    Ok(())
}
```

## API coverage

REST API services:

| Service | Description | Authentication |
|---------|-------------|----------------|
| `market()` | Market data (tickers, orderbook, klines, trades, block trades, execution rules, reference price) | Public |
| `account()` | Orders, order lists (OCO/OTO/OTOCO/OPO/OPOCO), SOR, cancel-replace, amend, trades, commission, filters | Required |
| `wallet()` | Deposits, withdrawals, transfers, balances, account status, API key permissions | Required |
| `margin()` | Cross and isolated margin trading, borrow/repay, transfers, interest history | Required |
| `user_stream()` | listenKey management (deprecated, Binance.US only) | Required |

WebSocket API (`ws_api()`):

- Session management (`session.logon` with Ed25519, status, logout, subscriptions)
- Trading (`order.place`, `order.test`, `order.cancel`, `openOrders.cancelAll`)
- Account queries (`account.status`, `order.status`, `openOrders.status`)
- User data stream subscriptions (`userDataStream.subscribe`, `userDataStream.subscribe.signature`, unsubscribe) with `subscriptionId` demultiplexing
- Generic `request` and `signed_request` methods for any other WebSocket API method
- Rate limit status from responses and `serverShutdown` handling

Market data streams (`streams()`):

- `<symbol>@aggTrade` - Aggregate trades
- `<symbol>@trade` - Individual trades
- `<symbol>@blockTrade` - Block trades
- `<symbol>@kline_<interval>` - Klines, with optional timezone offset
- `<symbol>@miniTicker`, `!miniTicker@arr` - Mini tickers
- `<symbol>@ticker` - 24hr ticker
- `<symbol>@ticker_<window>`, `!ticker_<window>@arr` - Rolling window tickers
- `<symbol>@bookTicker`, `!bookTicker` - Best bid/ask
- `<symbol>@avgPrice` - Average price
- `<symbol>@referencePrice` - Reference price
- `<symbol>@depth<levels>`, `<symbol>@depth` - Orderbook snapshots and diffs

Stream connections support live `SUBSCRIBE`/`UNSUBSCRIBE`, and `ReconnectingWebSocket` reconnects automatically with exponential backoff.

## Configuration

```rust
use binance_api_client::{Binance, Config};

fn main() -> binance_api_client::Result<()> {
    let config = Config::builder()
        .recv_window(10000)             // Set recv_window to 10 seconds
        .timeout_secs(30)               // Set request timeout to 30 seconds
        .proxy("http://localhost:8080") // Route REST requests through a proxy
        .microsecond_timestamps(true)   // Send timestamps in microseconds
        .build();

    let client = Binance::with_config(config, Some(("api_key", "secret_key")))?;
    Ok(())
}
```

Presets:

- `Config::default()` for Binance production
- `Config::testnet()` for the Binance Spot testnet (REST `/api` and WebSocket only, no SAPI)
- `Config::binance_us()` for Binance.US

Optional server time synchronization prevents `-1021` timestamp errors:

```rust
client.client().sync_time().await?;
```

## Environment variables

You can set credentials via environment variables:

```bash
export BINANCE_API_KEY=""
export BINANCE_SECRET_KEY=""
```

```rust
use binance_api_client::Binance;

fn main() -> binance_api_client::Result<()> {
    let client = Binance::from_env()?;
    Ok(())
}
```

## Project structure

```text
.
├── examples/            # Runnable examples showing common client usage.
├── src/                 # Library implementation.
│   ├── rest/            # REST endpoint clients.
│   ├── streams/         # WebSocket market stream client and stream management.
│   ├── ws_api/          # WebSocket API client (trading and user data streams).
│   └── models/          # Typed request and response models.
└── tests/               # Integration tests.
    └── mocks/           # Mock fixtures used by tests.
```
