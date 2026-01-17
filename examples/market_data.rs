//! Example demonstrating public market data API usage.
//!
//! This example shows how to use the Binance client to fetch market data
//! without requiring authentication.
//!
//! Run with: cargo run --example market_data

use binance_api_client::{Binance, KlineInterval};

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    // Initialize logging (optional)
    tracing_subscriber::fmt::init();

    // Create an unauthenticated client for public endpoints
    let client = Binance::new_unauthenticated()?;

    println!("=== Binance Market Data Example ===\n");

    // Test connectivity
    println!("Testing connectivity...");
    client.market().ping().await?;
    println!("Connected successfully!\n");

    // Get server time
    println!("Fetching server time...");
    let time = client.market().server_time().await?;
    println!("Server time: {}\n", time.server_time);

    // Get exchange info for specific symbols
    println!("Fetching exchange info for BTCUSDT...");
    let info = client
        .market()
        .exchange_info_for_symbols(&["BTCUSDT"])
        .await?;
    if let Some(symbol) = info.symbols.first() {
        println!("Symbol: {}", symbol.symbol);
        println!("Status: {:?}", symbol.status);
        println!("Base asset: {}", symbol.base_asset);
        println!("Quote asset: {}\n", symbol.quote_asset);
    }

    // Get current price
    println!("Fetching BTC/USDT price...");
    let price = client.market().price("BTCUSDT").await?;
    println!("BTC/USDT: {}\n", price.price);

    // Get multiple prices at once
    println!("Fetching prices for BTC, ETH, BNB...");
    let prices = client
        .market()
        .prices_for(&["BTCUSDT", "ETHUSDT", "BNBUSDT"])
        .await?;
    for p in prices {
        println!("{}: {}", p.symbol, p.price);
    }
    println!();

    // Get average price
    println!("Fetching average price for BTCUSDT...");
    let avg = client.market().avg_price("BTCUSDT").await?;
    println!("Average price (over {} mins): {}\n", avg.mins, avg.price);

    // Get order book depth
    println!("Fetching order book depth (top 5 levels)...");
    let depth = client.market().depth("BTCUSDT", Some(5)).await?;
    println!("Bids:");
    for bid in depth.bids.iter().take(3) {
        println!("  {} @ {}", bid.quantity, bid.price);
    }
    println!("Asks:");
    for ask in depth.asks.iter().take(3) {
        println!("  {} @ {}", ask.quantity, ask.price);
    }
    println!();

    // Get book ticker (best bid/ask)
    println!("Fetching best bid/ask...");
    let ticker = client.market().book_ticker("BTCUSDT").await?;
    println!("Best bid: {} @ {}", ticker.bid_qty, ticker.bid_price);
    println!("Best ask: {} @ {}\n", ticker.ask_qty, ticker.ask_price);

    // Get 24hr ticker
    println!("Fetching 24hr ticker...");
    let ticker_24h = client.market().ticker_24h("BTCUSDT").await?;
    println!(
        "Price change: {} ({}%)",
        ticker_24h.price_change, ticker_24h.price_change_percent
    );
    println!("High: {}", ticker_24h.high_price);
    println!("Low: {}", ticker_24h.low_price);
    println!("Volume: {}\n", ticker_24h.volume);

    // Get recent trades
    println!("Fetching recent trades (last 5)...");
    let trades = client.market().trades("BTCUSDT", Some(5)).await?;
    for trade in trades {
        let side = if trade.is_buyer_maker { "SELL" } else { "BUY" };
        println!(
            "  {} {} @ {} (qty: {})",
            side, trade.id, trade.price, trade.quantity
        );
    }
    println!();

    // Get aggregate trades
    println!("Fetching aggregate trades (last 5)...");
    let agg_trades = client
        .market()
        .agg_trades("BTCUSDT", None, None, None, Some(5))
        .await?;
    for trade in agg_trades {
        let side = if trade.is_buyer_maker { "SELL" } else { "BUY" };
        println!(
            "  {} {} @ {} (qty: {})",
            side, trade.agg_trade_id, trade.price, trade.quantity
        );
    }
    println!();

    // Get klines (candlesticks)
    println!("Fetching hourly klines (last 5)...");
    let klines = client
        .market()
        .klines("BTCUSDT", KlineInterval::Hours1, None, None, Some(5))
        .await?;
    for kline in klines {
        println!(
            "  Open: {}, High: {}, Low: {}, Close: {}, Volume: {}",
            kline.open, kline.high, kline.low, kline.close, kline.volume
        );
    }

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
