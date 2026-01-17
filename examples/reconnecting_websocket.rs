//! Example demonstrating the Reconnecting WebSocket client.
//!
//! This example shows how to use a WebSocket connection that automatically
//! reconnects on disconnection with exponential backoff.
//!
//! No API key required - uses public market data.
//!
//! Run with: cargo run --example reconnecting_websocket

use binance_api_client::{Binance, ReconnectConfig, WebSocketEvent};

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    // Initialize logging (optional)
    tracing_subscriber::fmt::init();

    println!("=== Binance Reconnecting WebSocket Example ===\n");

    // Create an unauthenticated client
    let client = Binance::new_unauthenticated()?;
    let ws = client.websocket();

    // Show the reconnection configuration
    let config = ReconnectConfig::default();
    println!("Reconnection Configuration:");
    println!("  Max reconnects: {}", config.max_reconnects);
    println!("  Max reconnect delay: {:?}", config.max_reconnect_delay);
    println!("  Base delay: {:?}", config.base_delay);
    println!("  Health check enabled: {}", config.health_check_enabled);
    println!(
        "  Health check interval: {:?}",
        config.health_check_interval
    );
    println!();

    // Connect to aggregate trade stream with auto-reconnection
    let stream = ws.agg_trade_stream("btcusdt");
    println!("Connecting to stream: {}", stream);
    println!("(With auto-reconnection enabled)\n");

    let mut conn = ws.connect_with_reconnect(&stream).await?;

    println!("Connected! State: {:?}", conn.state().await);
    println!("Waiting for trades...\n");

    // Receive events - connection will auto-reconnect if it drops
    let mut count = 0;
    while let Some(event) = conn.next().await {
        match event {
            Ok(WebSocketEvent::AggTrade(trade)) => {
                let side = if trade.is_buyer_maker { "SELL" } else { "BUY" };
                println!(
                    "[{}] Trade #{}: {} {} @ {} (qty: {})",
                    count + 1,
                    trade.agg_trade_id,
                    side,
                    trade.symbol,
                    trade.price,
                    trade.quantity
                );

                count += 1;

                // Show connection state periodically
                if count % 5 == 0 {
                    println!(
                        "  -> Connection state: {:?}, Reconnect attempts: {}",
                        conn.state().await,
                        conn.reconnect_count()
                    );
                }

                if count >= 15 {
                    break;
                }
            }
            Ok(other) => {
                println!("Received: {:?}", other);
            }
            Err(e) => {
                println!("Error (will auto-reconnect): {}", e);
                // The reconnecting websocket will handle reconnection automatically
            }
        }
    }

    // Check final state
    println!("\n=== Final State ===");
    println!("Connection state: {:?}", conn.state().await);
    println!("Total reconnect attempts: {}", conn.reconnect_count());
    println!("Is closed: {}", conn.is_closed());

    // Close the connection gracefully
    println!("\nClosing connection...");
    conn.close().await;
    println!("Connection closed. State: {:?}", conn.state().await);

    // Example: Custom reconnection config
    println!("\n=== Custom Reconnection Config Example ===\n");
    println!("You can customize reconnection behavior:");
    println!("```rust");
    println!("let config = ReconnectConfig {{");
    println!("    max_reconnects: 10,");
    println!("    max_reconnect_delay: Duration::from_secs(120),");
    println!("    base_delay: Duration::from_millis(500),");
    println!("    health_check_enabled: true,");
    println!("    health_check_interval: Duration::from_secs(60),");
    println!("}};");
    println!();
    println!("let conn = ReconnectingWebSocket::new(url, config).await?;");
    println!("```\n");

    // Example: Combined streams with reconnection
    println!("=== Combined Streams with Reconnection ===\n");
    let streams = vec![ws.ticker_stream("btcusdt"), ws.ticker_stream("ethusdt")];
    println!("Connecting to combined streams: {:?}", streams);

    let mut combined_conn = ws.connect_combined_with_reconnect(&streams).await?;
    println!("Connected!\n");

    let mut ticker_count = 0;
    while let Some(event) = combined_conn.next().await {
        if let Ok(WebSocketEvent::Ticker(ticker)) = event {
            println!(
                "[Ticker] {} Price: {} Change: {}%",
                ticker.symbol, ticker.close_price, ticker.price_change_percent
            );
            ticker_count += 1;
            if ticker_count >= 4 {
                break;
            }
        }
    }

    combined_conn.close().await;
    println!("\nCombined streams closed.");

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
