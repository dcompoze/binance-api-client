//! Example demonstrating User Data Streams for real-time account updates.
//!
//! This example shows how to:
//! - Start a user data stream (listen key)
//! - Connect to WebSocket for account events
//! - Receive order updates, balance changes, and account position updates
//! - Use the automatic keep-alive manager
//!
//! User data streams provide real-time updates for:
//! - Order execution reports (new, filled, canceled, etc.)
//! - Account balance updates
//! - Account position updates (outbound account info)
//! - OCO order list status changes
//!
//! Before running, set your testnet API credentials:
//!   export BINANCE_API_KEY=your_testnet_api_key
//!   export BINANCE_SECRET_KEY=your_testnet_secret_key
//!
//! Get testnet credentials at: https://testnet.binance.vision/
//!
//! Run with: cargo run --example user_data_stream

use binance_api_client::{Binance, UserDataStreamManager, WebSocketEvent};
use std::time::Duration;

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    // Initialize logging (optional)
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file if present
    let _ = dotenv::dotenv();

    println!("=== Binance User Data Stream Example ===\n");

    // Check for credentials
    let api_key = match std::env::var("BINANCE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("BINANCE_API_KEY not set. Showing example code only.\n");
            show_example_code();
            return Ok(());
        }
    };

    let secret_key = match std::env::var("BINANCE_SECRET_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("BINANCE_SECRET_KEY not set. Showing example code only.\n");
            show_example_code();
            return Ok(());
        }
    };

    // Create a testnet client
    let client = Binance::testnet(&api_key, &secret_key)?;
    println!("Using testnet: {}\n", client.config().rest_api_endpoint);

    // Method 1: Manual listen key management
    println!("=== Method 1: Manual Listen Key Management ===\n");

    // Start a user data stream
    println!("Starting user data stream...");
    let listen_key = client.user_stream().start().await?;
    println!("Listen key obtained: {}...", &listen_key[..20]);

    // Connect to the WebSocket
    println!("Connecting to WebSocket...\n");
    let mut conn = client.websocket().connect_user_stream(&listen_key).await?;

    println!("Connected! Waiting for events (will timeout after 10 seconds)...");
    println!("(Place an order in another terminal to see events)\n");

    // Wait for events with timeout
    let timeout_duration = Duration::from_secs(10);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout_duration {
            println!("Timeout reached. No events received.");
            break;
        }

        // Use tokio::select! to add timeout to event waiting
        tokio::select! {
            event = conn.next() => {
                match event {
                    Some(Ok(ev)) => {
                        print_event(&ev);
                    }
                    Some(Err(e)) => {
                        println!("Error: {}", e);
                    }
                    None => {
                        println!("Connection closed");
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Check timeout
            }
        }
    }

    // Important: Keep the listen key alive (should be done every 30 minutes)
    println!("\nRefreshing listen key...");
    client.user_stream().keepalive(&listen_key).await?;
    println!("Listen key refreshed.");

    // Close the connection
    println!("Closing WebSocket connection...");
    conn.close().await?;

    // When done, close the listen key
    println!("Closing listen key...");
    client.user_stream().close(&listen_key).await?;
    println!("Listen key closed.\n");

    // Method 2: Automatic keep-alive with UserDataStreamManager
    println!("=== Method 2: Automatic Keep-Alive Manager ===\n");

    println!("Creating UserDataStreamManager...");
    println!("(This automatically refreshes the listen key every 30 minutes)\n");

    // Create the manager - it handles listen key lifecycle automatically
    let mut manager = UserDataStreamManager::new(client.clone()).await?;

    println!(
        "Manager created! Listen key: {}...",
        &manager.listen_key().await[..20]
    );
    println!("Waiting for events (will timeout after 10 seconds)...\n");

    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout_duration {
            println!("Timeout reached. No events received.");
            break;
        }

        tokio::select! {
            event = manager.next() => {
                match event {
                    Some(Ok(ev)) => {
                        print_event(&ev);
                    }
                    Some(Err(e)) => {
                        println!("Error: {}", e);
                    }
                    None => {
                        println!("Stream ended");
                        break;
                    }
                }
            }
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                // Check timeout
            }
        }
    }

    // Stop the manager (closes listen key automatically)
    println!("\nStopping manager...");
    manager.stop();
    println!("Manager stopped. Is stopped: {}", manager.is_stopped());

    println!("\n=== Example completed! ===");
    Ok(())
}

fn print_event(event: &WebSocketEvent) {
    match event {
        WebSocketEvent::ExecutionReport(report) => {
            println!("=== Execution Report ===");
            println!("  Event: {:?}", report.execution_type);
            println!("  Symbol: {}", report.symbol);
            println!("  Side: {:?}", report.side);
            println!("  Order Type: {:?}", report.order_type);
            println!("  Order ID: {}", report.order_id);
            println!("  Client Order ID: {}", report.client_order_id);
            println!("  Status: {:?}", report.order_status);
            println!("  Price: {}", report.price);
            println!("  Quantity: {}", report.quantity);
            println!("  Last Executed Qty: {}", report.last_executed_quantity);
            println!("  Cumulative Qty: {}", report.cumulative_filled_quantity);
            println!(
                "  Commission: {} {}",
                report.commission,
                report.commission_asset.as_deref().unwrap_or("")
            );
            println!();
        }
        WebSocketEvent::AccountPosition(position) => {
            println!("=== Account Position Update ===");
            println!("  Event Time: {}", position.event_time);
            println!("  Last Update Time: {}", position.last_update_time);
            println!("  Balances:");
            for balance in &position.balances {
                if balance.free > 0.0 || balance.locked > 0.0 {
                    println!(
                        "    {}: free={}, locked={}",
                        balance.asset, balance.free, balance.locked
                    );
                }
            }
            println!();
        }
        WebSocketEvent::BalanceUpdate(update) => {
            println!("=== Balance Update ===");
            println!("  Asset: {}", update.asset);
            println!("  Balance Delta: {}", update.balance_delta);
            println!("  Clear Time: {}", update.clear_time);
            println!();
        }
        WebSocketEvent::ListStatus(status) => {
            println!("=== List Status (OCO) ===");
            println!("  Symbol: {}", status.symbol);
            println!("  Order List ID: {}", status.order_list_id);
            println!("  Contingency Type: {}", status.contingency_type);
            println!("  List Status Type: {}", status.list_status_type);
            println!("  List Order Status: {}", status.list_order_status);
            println!("  Orders:");
            for order in &status.orders {
                println!(
                    "    - Symbol: {}, Order ID: {}, Client Order ID: {}",
                    order.symbol, order.order_id, order.client_order_id
                );
            }
            println!();
        }
        other => {
            println!("Other event: {:?}\n", other);
        }
    }
}

fn show_example_code() {
    println!("=== User Data Stream Example Code ===\n");

    println!("// Create authenticated client");
    println!("let client = Binance::new(\"api_key\", \"secret_key\")?;\n");

    println!("// --- Manual Listen Key Management ---\n");

    println!("// Start user data stream (get listen key)");
    println!("let listen_key = client.user_stream().start().await?;\n");

    println!("// Connect to WebSocket");
    println!("let mut conn = client.websocket().connect_user_stream(&listen_key).await?;\n");

    println!("// Receive events");
    println!("while let Some(event) = conn.next().await {{");
    println!("    match event? {{");
    println!("        WebSocketEvent::ExecutionReport(report) => {{");
    println!(
        "            println!(\"Order {{}}: {{:?}}\", report.order_id, report.execution_type);"
    );
    println!("        }}");
    println!("        WebSocketEvent::AccountPosition(pos) => {{");
    println!("            println!(\"Account updated: {{}} balances\", pos.balances.len());");
    println!("        }}");
    println!("        WebSocketEvent::BalanceUpdate(update) => {{");
    println!(
        "            println!(\"Balance change: {{}} {{}}\", update.balance_delta, update.asset);"
    );
    println!("        }}");
    println!("        _ => {{}}");
    println!("    }}");
    println!("}}\n");

    println!("// Keep alive (call every 30 minutes)");
    println!("client.user_stream().keepalive(&listen_key).await?;\n");

    println!("// Close when done");
    println!("conn.close().await?;");
    println!("client.user_stream().close(&listen_key).await?;\n");

    println!("// --- Automatic Keep-Alive with Manager ---\n");

    println!("use binance_api_client::UserDataStreamManager;\n");

    println!("// Create manager (handles listen key lifecycle automatically)");
    println!("let mut manager = UserDataStreamManager::new(client).await?;\n");

    println!("// Receive events (keep-alive happens automatically)");
    println!("while let Some(event) = manager.next().await {{");
    println!("    match event? {{");
    println!("        WebSocketEvent::ExecutionReport(report) => {{");
    println!("            // Handle order update");
    println!("        }}");
    println!("        _ => {{}}");
    println!("    }}");
    println!("}}\n");

    println!("// Stop when done");
    println!("manager.stop();\n");

    println!("=== Event Types ===\n");
    println!("User data streams provide these event types:");
    println!("  - ExecutionReport: Order updates (new, filled, canceled, etc.)");
    println!("  - AccountPosition: Account balance changes (outboundAccountPosition)");
    println!("  - BalanceUpdate: Individual balance changes");
    println!("  - ListStatus: OCO order list status changes\n");

    println!("=== End of Example Code ===");
}
