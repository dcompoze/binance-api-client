//! Example demonstrating the Depth Cache Manager for local order book.
//!
//! This example shows how to maintain a local order book that stays
//! synchronized with Binance's order book via WebSocket updates.
//!
//! No API key required - uses public market data.
//!
//! Run with: cargo run --example depth_cache

use binance_api_client::{Binance, DepthCacheConfig, DepthCacheManager};
use std::time::Duration;

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    // Initialize logging (optional)
    tracing_subscriber::fmt::init();

    println!("=== Binance Depth Cache Manager Example ===\n");

    // Create an unauthenticated client
    let client = Binance::new_unauthenticated()?;

    // Configure the depth cache
    let config = DepthCacheConfig {
        depth_limit: 100,       // Number of levels to fetch in snapshot
        fast_updates: true,     // Use 100ms update speed (vs 1000ms)
        refresh_interval: None, // Optional: periodically re-fetch snapshot
    };

    let symbol = "BTCUSDT";
    println!("Starting depth cache for {}...", symbol);
    println!("Config: {:?}\n", config);

    // Create the depth cache manager
    // This will:
    // 1. Connect to the WebSocket depth stream
    // 2. Fetch an initial order book snapshot via REST API
    // 3. Apply WebSocket updates to maintain sync
    let mut manager = DepthCacheManager::new(client.clone(), symbol, config).await?;

    // Wait for the initial sync to complete
    println!("Waiting for initial sync...");
    manager.wait_for_sync().await?;
    println!("Synced!\n");

    // Get the current cache state
    let cache = manager.get_cache().await;
    println!("=== Initial Order Book State ===");
    println!("Symbol: {}", cache.symbol);
    println!("Last Update ID: {}", cache.last_update_id);

    if let Some((bid_price, bid_qty)) = cache.best_bid() {
        println!("Best Bid: {} @ {}", bid_qty, bid_price);
    }
    if let Some((ask_price, ask_qty)) = cache.best_ask() {
        println!("Best Ask: {} @ {}", ask_qty, ask_price);
    }
    if let Some(spread) = cache.spread() {
        println!("Spread: {:.2}", spread);
    }
    if let Some(mid) = cache.mid_price() {
        println!("Mid Price: {:.2}", mid);
    }
    println!();

    // Show top 5 levels
    println!("=== Top 5 Bid/Ask Levels ===");
    println!("BIDS:");
    for (price, qty) in cache.get_top_bids(5) {
        println!("  {} @ {:.2}", qty, price);
    }
    println!("ASKS:");
    for (price, qty) in cache.get_top_asks(5) {
        println!("  {} @ {:.2}", qty, price);
    }
    println!();

    // Volume statistics
    println!("=== Volume Statistics ===");
    println!("Total Bid Volume: {:.4}", cache.total_bid_volume());
    println!("Total Ask Volume: {:.4}", cache.total_ask_volume());
    println!();

    // Listen for updates
    println!("=== Real-time Updates (10 updates) ===\n");

    let mut update_count = 0;
    while let Some(updated_cache) = manager.next().await {
        update_count += 1;

        let best_bid = updated_cache.best_bid();
        let best_ask = updated_cache.best_ask();
        let spread = updated_cache.spread();

        println!(
            "Update #{}: Bid: {:?} | Ask: {:?} | Spread: {:?}",
            update_count,
            best_bid.map(|(p, _)| format!("{:.2}", p)),
            best_ask.map(|(p, _)| format!("{:.2}", p)),
            spread.map(|s| format!("{:.2}", s))
        );

        if update_count >= 10 {
            break;
        }
    }

    // Check manager state
    println!("\n=== Manager State ===");
    println!("State: {:?}", manager.state().await);

    // Stop the manager
    println!("\nStopping depth cache manager...");
    manager.stop();

    // Give it a moment to clean up
    tokio::time::sleep(Duration::from_millis(100)).await;

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
