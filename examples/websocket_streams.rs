//! Example demonstrating WebSocket streaming API usage.
//!
//! This example shows how to connect to Binance WebSocket streams
//! for real-time market data.
//!
//! Run with: cargo run --example websocket_streams

use binance_api_client::{Binance, KlineInterval, WebSocketEvent};

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    // Initialize logging (optional)
    tracing_subscriber::fmt::init();

    println!("=== Binance WebSocket Streams Example ===\n");

    // Create a client
    let client = Binance::new_unauthenticated()?;
    let ws = client.websocket();

    // Example 1: Single stream - Aggregate Trades
    println!("Connecting to aggregate trade stream for BTCUSDT...");
    let stream = ws.agg_trade_stream("btcusdt");
    println!("Stream name: {}", stream);

    let mut conn = ws.connect(&stream).await?;
    println!("Connected! Waiting for trades (will exit after 5 events)...\n");

    let mut count = 0;
    while let Some(event) = conn.next().await {
        match event? {
            WebSocketEvent::AggTrade(trade) => {
                let side = if trade.is_buyer_maker { "SELL" } else { "BUY" };
                println!(
                    "[AggTrade] {} {} @ {} (qty: {}) time: {}",
                    side, trade.symbol, trade.price, trade.quantity, trade.trade_time
                );
                count += 1;
                if count >= 5 {
                    break;
                }
            }
            other => {
                println!("Received other event: {:?}", other);
            }
        }
    }

    conn.close().await?;
    println!("\nClosed aggregate trade stream.\n");

    // Example 2: Kline stream
    println!("Connecting to 1-minute kline stream for ETHUSDT...");
    let kline_stream = ws.kline_stream("ethusdt", KlineInterval::Minutes1);
    println!("Stream name: {}", kline_stream);

    let mut conn = ws.connect(&kline_stream).await?;
    println!("Connected! Waiting for klines (will exit after 3 events)...\n");

    count = 0;
    while let Some(event) = conn.next().await {
        match event? {
            WebSocketEvent::Kline(kline) => {
                let k = &kline.kline;
                println!(
                    "[Kline] {} {} O:{} H:{} L:{} C:{} V:{}",
                    kline.symbol, k.interval, k.open, k.high, k.low, k.close, k.volume
                );
                count += 1;
                if count >= 3 {
                    break;
                }
            }
            other => {
                println!("Received other event: {:?}", other);
            }
        }
    }

    conn.close().await?;
    println!("\nClosed kline stream.\n");

    // Example 3: Combined streams (multiple symbols)
    println!("Connecting to combined streams for BTC and ETH tickers...");
    let streams = vec![ws.ticker_stream("btcusdt"), ws.ticker_stream("ethusdt")];
    println!("Stream names: {:?}", streams);

    let mut conn = ws.connect_combined(&streams).await?;
    println!("Connected! Waiting for ticker updates (will exit after 6 events)...\n");

    count = 0;
    while let Some(event) = conn.next().await {
        match event? {
            WebSocketEvent::Ticker(ticker) => {
                println!(
                    "[Ticker] {} Price: {} Change: {}% Volume: {}",
                    ticker.symbol,
                    ticker.close_price,
                    ticker.price_change_percent,
                    ticker.quote_volume
                );
                count += 1;
                if count >= 6 {
                    break;
                }
            }
            other => {
                println!("Received other event: {:?}", other);
            }
        }
    }

    conn.close().await?;
    println!("\nClosed combined ticker streams.\n");

    // Example 4: Book ticker stream
    println!("Connecting to book ticker stream for BNBUSDT...");
    let book_stream = ws.book_ticker_stream("bnbusdt");

    let mut conn = ws.connect(&book_stream).await?;
    println!("Connected! Waiting for book updates (will exit after 3 events)...\n");

    count = 0;
    while let Some(event) = conn.next().await {
        match event? {
            WebSocketEvent::BookTicker(book) => {
                println!(
                    "[BookTicker] {} Bid: {} @ {} | Ask: {} @ {}",
                    book.symbol,
                    book.bid_quantity,
                    book.bid_price,
                    book.ask_quantity,
                    book.ask_price
                );
                count += 1;
                if count >= 3 {
                    break;
                }
            }
            other => {
                println!("Received other event: {:?}", other);
            }
        }
    }

    conn.close().await?;
    println!("\nClosed book ticker stream.");

    // Show available stream helpers
    println!("\n=== Available Stream Types ===");
    println!("  agg_trade_stream(symbol)     - Aggregate trades");
    println!("  trade_stream(symbol)         - Individual trades");
    println!("  kline_stream(symbol, interval) - Candlesticks");
    println!("  mini_ticker_stream(symbol)   - Mini ticker (24h)");
    println!("  ticker_stream(symbol)        - Full ticker (24h)");
    println!("  book_ticker_stream(symbol)   - Best bid/ask");
    println!("  partial_depth_stream(symbol, levels, fast) - Order book updates");
    println!("  diff_depth_stream(symbol, fast) - Order book diffs");
    println!("  all_mini_ticker_stream()     - All symbols mini ticker");
    println!("  all_ticker_stream()          - All symbols ticker");
    println!("  all_book_ticker_stream()     - All symbols book ticker");

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
