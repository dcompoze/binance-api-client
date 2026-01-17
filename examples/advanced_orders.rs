//! Example demonstrating Advanced Order Types.
//!
//! This example shows how to use various order list types:
//! - OCO (One-Cancels-the-Other): Take profit + Stop loss
//! - OTO (One-Triggers-the-Other): Working order triggers pending order
//! - OTOCO (One-Triggers-One-Cancels-Other): Working order triggers OCO
//! - Cancel-Replace: Atomically cancel and replace an order
//! - SOR (Smart Order Router): Route across multiple venues
//!
//! These order types are useful for:
//! - Automated profit taking and stop losses
//! - Entry orders that automatically set up exit strategies
//! - Modifying orders without missing fills
//!
//! Before running, set your testnet API credentials:
//!   export BINANCE_API_KEY=your_testnet_api_key
//!   export BINANCE_SECRET_KEY=your_testnet_secret_key
//!
//! Get testnet credentials at: https://testnet.binance.vision/
//!
//! Run with: cargo run --example advanced_orders

use binance_api_client::{
    Binance, CancelReplaceMode, CancelReplaceOrderBuilder, OcoOrderBuilder, OrderBuilder,
    OrderSide, OrderType, OtoOrderBuilder, OtocoOrderBuilder, TimeInForce,
};

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    // Initialize logging (optional).
    tracing_subscriber::fmt::init();

    // Load environment variables from `.env` if present.
    let _ = dotenv::dotenv();

    println!("=== Binance Advanced Order Types Example ===\n");

    // Check for credentials.
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

    // Create a testnet client.
    let client = Binance::testnet(&api_key, &secret_key)?;
    println!("Using testnet: {}", client.config().rest_api_endpoint);
    println!("NOTE: Using TESTNET - no real funds at risk!\n");

    // Get current price for reference
    let symbol = "BTCUSDT";
    let price = client.market().price(symbol).await?;
    let current_price: f64 = price.price;
    println!("Current {} price: {:.2}\n", symbol, current_price);

    // Calculate realistic price levels based on current price
    let take_profit_price = current_price * 1.05; // 5% above
    let stop_price = current_price * 0.95; // 5% below
    let stop_limit_price = current_price * 0.948; // Just below stop
    let entry_price = current_price * 0.98; // 2% below current

    // 1. OCO Order (One-Cancels-the-Other)
    println!("=== 1. OCO Order (One-Cancels-the-Other) ===\n");
    println!("OCO combines a limit order (take profit) and a stop-loss order.");
    println!("When one fills, the other is automatically canceled.\n");

    let oco = OcoOrderBuilder::new(
        symbol,
        OrderSide::Sell,                      // Selling BTC
        "0.001",                              // Quantity
        &format!("{:.2}", take_profit_price), // Take profit (limit price)
        &format!("{:.2}", stop_price),        // Stop price
    )
    .stop_limit_price(&format!("{:.2}", stop_limit_price))
    .stop_limit_time_in_force(TimeInForce::GTC)
    .build();

    println!("OCO Order:");
    println!("  Quantity: 0.001 BTC");
    println!("  Take Profit: {:.2} (limit sell)", take_profit_price);
    println!("  Stop Loss: {:.2} (stop trigger)", stop_price);
    println!("  Stop Limit: {:.2}\n", stop_limit_price);

    // Place the OCO order (will fail if no balance, but shows the API)
    print!("Placing OCO order... ");
    match client.account().create_oco(&oco).await {
        Ok(result) => {
            println!("Success! Order List ID: {}", result.order_list_id);
            // Cancel it immediately for cleanup
            let _ = client
                .account()
                .cancel_order_list(symbol, Some(result.order_list_id), None)
                .await;
            println!("  (Canceled for cleanup)");
        }
        Err(e) => println!("Failed: {} (expected if no balance)", e),
    }
    println!();

    // 2. OTO Order (One-Triggers-the-Other)
    println!("=== 2. OTO Order (One-Triggers-the-Other) ===\n");
    println!("OTO has a 'working' order that triggers a 'pending' order when filled.");
    println!("Example: Buy BTC at entry price, then sell at take profit.\n");

    let oto = OtoOrderBuilder::new(
        symbol,
        OrderType::Limit,               // Working order type
        OrderSide::Buy,                 // Working order side
        &format!("{:.2}", entry_price), // Working order price
        "0.001",                        // Working order quantity
        OrderType::Limit,               // Pending order type
        OrderSide::Sell,                // Pending order side
        "0.001",                        // Pending order quantity
    )
    .working_time_in_force(TimeInForce::GTC)
    .pending_price(&format!("{:.2}", take_profit_price))
    .pending_time_in_force(TimeInForce::GTC)
    .build();

    println!("OTO Order:");
    println!("  Working: BUY 0.001 BTC @ {:.2}", entry_price);
    println!(
        "  Pending: SELL 0.001 BTC @ {:.2} (triggered when working fills)\n",
        take_profit_price
    );

    print!("Placing OTO order... ");
    match client.account().create_oto(&oto).await {
        Ok(result) => {
            println!("Success! Order List ID: {}", result.order_list_id);
            let _ = client
                .account()
                .cancel_order_list(symbol, Some(result.order_list_id), None)
                .await;
            println!("  (Canceled for cleanup)");
        }
        Err(e) => println!("Failed: {} (expected if no balance)", e),
    }
    println!();

    // 3. OTOCO Order (One-Triggers-One-Cancels-Other)
    println!("=== 3. OTOCO Order (One-Triggers-One-Cancels-Other) ===\n");
    println!("OTOCO combines OTO + OCO: A working order triggers an OCO (TP + SL).");
    println!("Example: Entry order triggers both take-profit and stop-loss.\n");

    let otoco = OtocoOrderBuilder::new(
        symbol,
        OrderType::Limit,               // Working order type
        OrderSide::Buy,                 // Working order side
        &format!("{:.2}", entry_price), // Working order price
        "0.001",                        // Working order quantity
        OrderSide::Sell,                // Pending orders side (exit side)
        "0.001",                        // Pending orders quantity
        OrderType::LimitMaker,          // Take profit type (above price)
    )
    .working_time_in_force(TimeInForce::GTC)
    .pending_above_price(&format!("{:.2}", take_profit_price)) // Take profit price
    .pending_below_type(OrderType::StopLossLimit)
    .pending_below_stop_price(&format!("{:.2}", stop_price))
    .pending_below_price(&format!("{:.2}", stop_limit_price))
    .pending_below_time_in_force(TimeInForce::GTC)
    .build();

    println!("OTOCO Order:");
    println!("  Working: BUY 0.001 BTC @ {:.2}", entry_price);
    println!("  When filled, triggers:");
    println!("    - Take Profit: SELL @ {:.2}", take_profit_price);
    println!(
        "    - Stop Loss: SELL @ {:.2} (stop: {:.2})\n",
        stop_limit_price, stop_price
    );

    print!("Placing OTOCO order... ");
    match client.account().create_otoco(&otoco).await {
        Ok(result) => {
            println!("Success! Order List ID: {}", result.order_list_id);
            let _ = client
                .account()
                .cancel_order_list(symbol, Some(result.order_list_id), None)
                .await;
            println!("  (Canceled for cleanup)");
        }
        Err(e) => println!("Failed: {} (expected if no balance)", e),
    }
    println!();

    // 4. Cancel-Replace Order
    println!("=== 4. Cancel-Replace Order ===\n");
    println!("Atomically cancel an existing order and place a new one.");
    println!("Prevents the gap where your order might be filled during modification.\n");

    // First, let's place a test order to modify
    let initial_order = OrderBuilder::new(symbol, OrderSide::Buy, OrderType::Limit)
        .quantity("0.001")
        .price(&format!("{:.2}", entry_price))
        .time_in_force(TimeInForce::GTC)
        .build();

    println!("Placing initial order to modify...");
    match client.account().create_order(&initial_order).await {
        Ok(result) => {
            println!("  Order placed! ID: {}", result.order_id);
            println!("  Price: {}, Qty: {}\n", result.price, result.orig_qty);

            // Now cancel-replace it
            let new_price = entry_price * 0.99; // Adjust price down 1%
            println!("Cancel-replacing with new price {:.2}...", new_price);

            let cancel_replace = CancelReplaceOrderBuilder::new(
                symbol,
                OrderSide::Buy,
                OrderType::Limit,
                CancelReplaceMode::StopOnFailure,
            )
            .cancel_order_id(result.order_id)
            .quantity("0.001")
            .price(&format!("{:.2}", new_price))
            .time_in_force(TimeInForce::GTC)
            .build();

            match client.account().cancel_replace_order(&cancel_replace).await {
                Ok(response) => {
                    println!("Cancel-Replace successful!");
                    println!("  Cancel result: {:?}", response.cancel_result);
                    println!("  New order result: {:?}", response.new_order_result);
                    println!(
                        "  Canceled order: {} (status: {:?})",
                        response.cancel_response.order_id, response.cancel_response.status
                    );

                    // Get new order ID from the response and clean up
                    let new_order_id = match &response.new_order_response {
                        binance_api_client::OrderResponse::Ack(ack) => Some(ack.order_id),
                        binance_api_client::OrderResponse::Result(res) => Some(res.order_id),
                        binance_api_client::OrderResponse::Full(full) => Some(full.order_id),
                    };
                    if let Some(id) = new_order_id {
                        println!("  New order ID: {}", id);
                        let _ = client.account().cancel_order(symbol, Some(id), None).await;
                        println!("  (Cleaned up - canceled the new order)");
                    }
                }
                Err(e) => println!("Cancel-Replace failed: {}", e),
            }
        }
        Err(e) => {
            println!("Failed to place initial order: {}", e);
            println!("(This is expected if you don't have sufficient balance)");
        }
    }
    println!();

    // 5. Query Order Lists
    println!("=== 5. Query Order Lists ===\n");

    println!("Getting open order lists...");
    match client.account().open_order_lists().await {
        Ok(lists) => {
            if lists.is_empty() {
                println!("No open order lists.\n");
            } else {
                for list in &lists {
                    println!("Order List ID: {}", list.order_list_id);
                    println!("  Contingency Type: {:?}", list.contingency_type);
                    println!("  List Status Type: {:?}", list.list_status_type);
                    println!("  Orders: {}", list.orders.len());
                }
            }
        }
        Err(e) => println!("Failed to get order lists: {}\n", e),
    }

    println!("=== Example completed! ===");
    Ok(())
}

fn show_example_code() {
    println!("=== Advanced Order Types Example Code ===\n");

    println!("// Create authenticated client (testnet recommended for testing)");
    println!("let client = Binance::testnet(\"api_key\", \"secret_key\")?;\n");

    // OCO
    println!("// --- OCO (One-Cancels-the-Other) ---");
    println!("// Combines take-profit and stop-loss in one order");
    println!("use binance_api_client::{{OcoOrderBuilder, OrderSide, TimeInForce}};\n");
    println!("let oco = OcoOrderBuilder::new(");
    println!("    \"BTCUSDT\",");
    println!("    OrderSide::Sell,     // Exit side");
    println!("    \"0.001\",            // Quantity");
    println!("    \"70000.00\",         // Take profit (limit price)");
    println!("    \"60000.00\",         // Stop price");
    println!(")");
    println!(".stop_limit_price(\"59900.00\")  // Stop limit price");
    println!(".stop_limit_time_in_force(TimeInForce::GTC)");
    println!(".build();\n");
    println!("let result = client.account().create_oco(&oco).await?;\n");

    // OTO
    println!("// --- OTO (One-Triggers-the-Other) ---");
    println!("// Working order triggers a pending order when filled");
    println!("use binance_api_client::{{OtoOrderBuilder, OrderType}};\n");
    println!("let oto = OtoOrderBuilder::new(");
    println!("    \"BTCUSDT\",");
    println!("    OrderType::Limit,    // Working type");
    println!("    OrderSide::Buy,      // Working side");
    println!("    \"65000.00\",         // Working price (entry)");
    println!("    \"0.001\",            // Working quantity");
    println!("    OrderType::Limit,    // Pending type");
    println!("    OrderSide::Sell,     // Pending side (exit)");
    println!("    \"0.001\",            // Pending quantity");
    println!(")");
    println!(".working_time_in_force(TimeInForce::GTC)");
    println!(".pending_price(\"70000.00\")  // Take profit price");
    println!(".pending_time_in_force(TimeInForce::GTC)");
    println!(".build();\n");
    println!("let result = client.account().create_oto(&oto).await?;\n");

    // OTOCO
    println!("// --- OTOCO (One-Triggers-One-Cancels-Other) ---");
    println!("// Entry order triggers an OCO (take-profit + stop-loss)");
    println!("use binance_api_client::OtocoOrderBuilder;\n");
    println!("let otoco = OtocoOrderBuilder::new(");
    println!("    \"BTCUSDT\",");
    println!("    OrderType::Limit,    // Working (entry) type");
    println!("    OrderSide::Buy,      // Working side");
    println!("    \"65000.00\",         // Working price");
    println!("    \"0.001\",            // Working quantity");
    println!("    OrderSide::Sell,     // Exit side");
    println!("    \"0.001\",            // Exit quantity");
    println!("    OrderType::LimitMaker, // Take profit type");
    println!(")");
    println!(".working_time_in_force(TimeInForce::GTC)");
    println!(".pending_above_price(\"70000.00\")  // Take profit");
    println!(".pending_below_type(OrderType::StopLossLimit)");
    println!(".pending_below_stop_price(\"60000.00\")");
    println!(".pending_below_price(\"59900.00\")");
    println!(".pending_below_time_in_force(TimeInForce::GTC)");
    println!(".build();\n");
    println!("let result = client.account().create_otoco(&otoco).await?;\n");

    // Cancel-Replace
    println!("// --- Cancel-Replace ---");
    println!("// Atomically cancel and place a new order");
    println!("use binance_api_client::CancelReplaceMode;\n");
    println!("let cancel_replace = client.account().cancel_replace_order(");
    println!("    \"BTCUSDT\",");
    println!("    OrderSide::Buy,");
    println!("    OrderType::Limit,");
    println!("    CancelReplaceMode::StopOnFailure,");
    println!(")");
    println!(".cancel_order_id(12345678)  // Order to cancel");
    println!(".quantity(\"0.001\")");
    println!(".price(\"64000.00\")  // New price");
    println!(".time_in_force(TimeInForce::GTC)");
    println!(".build();\n");
    println!("let result = client.account().cancel_replace(&cancel_replace).await?;\n");

    // Query
    println!("// --- Query Order Lists ---");
    println!("// Get open OCO/OTO/OTOCO orders");
    println!("let open_lists = client.account().open_oco_orders().await?;");
    println!("let all_lists = client.account().all_oco_orders(None, None, Some(10)).await?;\n");

    // Cancel
    println!("// --- Cancel Order List ---");
    println!(
        "let result = client.account().cancel_oco(\"BTCUSDT\", Some(order_list_id), None).await?;\n"
    );

    println!("=== End of Example Code ===");
}
