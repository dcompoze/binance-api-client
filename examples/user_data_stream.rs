//! Example demonstrating user data streams over the WebSocket API.
//!
//! The listenKey endpoints were removed from Binance production on
//! 2026-02-20, so user data streams are now subscribed through the
//! WebSocket API (`userDataStream.subscribe.signature` or
//! `userDataStream.subscribe` after an Ed25519 session logon).
//!
//! This example shows how to:
//! - Connect to the WebSocket API
//! - Subscribe to the user data stream with an API key signature
//! - Receive order updates, balance changes, and account position updates
//! - Unsubscribe and handle the `serverShutdown` event
//!
//! Before running, set your testnet API credentials:
//!   export BINANCE_API_KEY=your_testnet_api_key
//!   export BINANCE_SECRET_KEY=your_testnet_secret_key
//!
//! Get testnet credentials at: https://testnet.binance.vision/
//!
//! Run with: cargo run --example user_data_stream

use binance_api_client::{Binance, WebSocketEvent, WsApiEvent};
use std::time::Duration;

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    tracing_subscriber::fmt::init();
    let _ = dotenv::dotenv();

    println!("=== Binance User Data Stream Example (WebSocket API) ===\n");

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

    let client = Binance::testnet(&api_key, &secret_key)?;
    println!(
        "Using testnet WebSocket API: {}\n",
        client.config().ws_api_endpoint
    );

    // Connect to the WebSocket API.
    println!("Connecting...");
    let conn = client.ws_api().connect().await?;

    // Subscribe using an API key signature.
    // This works with any API key type and needs no session logon.
    println!("Subscribing to user data stream...");
    let subscription_id = conn.subscribe_user_data_with_signature().await?;
    println!("Subscribed with subscription id {}\n", subscription_id);

    // List active subscriptions on this session.
    let subscriptions = conn.session_subscriptions().await?;
    println!("Active subscriptions: {:?}\n", subscriptions);

    println!("Waiting for events (will timeout after 30 seconds)...");
    println!("(Place an order in another terminal to see events)\n");

    let mut events = conn.take_events().await.expect("events already taken");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);

    loop {
        let event = tokio::select! {
            event = events.recv() => event,
            _ = tokio::time::sleep_until(deadline) => {
                println!("Timeout reached.");
                break;
            }
        };

        match event {
            Some(WsApiEvent::UserData {
                subscription_id,
                event,
            }) => {
                println!("[subscription {:?}]", subscription_id);
                print_event(&event);
            }
            Some(WsApiEvent::ServerShutdown { event_time }) => {
                println!("Server shutdown at {}. Reconnect immediately.", event_time);
                break;
            }
            Some(WsApiEvent::Unknown(value)) => {
                println!("Unknown event: {}\n", value);
            }
            None => {
                println!("Connection closed");
                break;
            }
        }
    }

    // Unsubscribe when done.
    println!("\nUnsubscribing...");
    conn.unsubscribe_user_data(Some(subscription_id)).await?;
    println!("Unsubscribed.");

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
            println!();
        }
        WebSocketEvent::AccountPosition(position) => {
            println!("=== Account Position Update ===");
            println!("  Event Time: {}", position.event_time);
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
            println!();
        }
        WebSocketEvent::ListStatus(status) => {
            println!("=== List Status ===");
            println!("  Symbol: {}", status.symbol);
            println!("  Order List ID: {}", status.order_list_id);
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

    println!("// Connect to the WebSocket API");
    println!("let conn = client.ws_api().connect().await?;\n");

    println!("// Subscribe with an API key signature (any key type)");
    println!("let subscription_id = conn.subscribe_user_data_with_signature().await?;\n");

    println!("// Or with an Ed25519 session logon:");
    println!("// conn.session_logon().await?;");
    println!("// let subscription_id = conn.subscribe_user_data().await?;\n");

    println!("// Receive events");
    println!("let mut events = conn.take_events().await.unwrap();");
    println!("while let Some(event) = events.recv().await {{");
    println!("    match event {{");
    println!("        WsApiEvent::UserData {{ subscription_id, event }} => {{");
    println!("            // Handle WebSocketEvent (ExecutionReport, BalanceUpdate, ...)");
    println!("        }}");
    println!("        WsApiEvent::ServerShutdown {{ .. }} => {{");
    println!("            // Reconnect immediately");
    println!("        }}");
    println!("        _ => {{}}");
    println!("    }}");
    println!("}}\n");

    println!("// Unsubscribe when done");
    println!("conn.unsubscribe_user_data(Some(subscription_id)).await?;\n");

    println!("=== End of Example Code ===");
}
