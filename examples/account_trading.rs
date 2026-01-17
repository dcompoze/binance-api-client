//! Example demonstrating authenticated account and trading API usage.
//!
//! This example shows how to use authenticated endpoints for account
//! information and order management. Uses testnet by default to avoid
//! making real trades.
//!
//! Before running, set your testnet API credentials:
//!   export BINANCE_API_KEY=your_testnet_api_key
//!   export BINANCE_SECRET_KEY=your_testnet_secret_key
//!
//! Get testnet credentials at: https://testnet.binance.vision/
//!
//! Run with: cargo run --example account_trading

use binance_api_client::{Binance, OrderBuilder, OrderSide, OrderType, TimeInForce};

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    // Initialize logging (optional).
    tracing_subscriber::fmt::init();

    // Load environment variables from `.env` if present.
    let _ = dotenv::dotenv();

    println!("=== Binance Account & Trading Example ===\n");
    println!("NOTE: This example uses the Binance TESTNET.");
    println!("Get testnet API keys at: https://testnet.binance.vision/\n");

    // Create a testnet client from environment variables.
    let api_key = match std::env::var("BINANCE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("BINANCE_API_KEY not set. Showing example code only.");
            return Ok(());
        }
    };
    let secret_key = match std::env::var("BINANCE_SECRET_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("BINANCE_SECRET_KEY not set. Showing example code only.");
            return Ok(());
        }
    };

    let client = Binance::testnet(&api_key, &secret_key)?;

    println!(
        "Client created. Has credentials: {}",
        client.has_credentials()
    );
    println!("Using endpoint: {}\n", client.config().rest_api_endpoint);

    // Test connectivity first.
    println!("Testing connectivity...");
    client.market().ping().await?;
    println!("Connected successfully!\n");

    // Get account information.
    println!("=== Account Information ===\n");
    let account = client.account().get_account().await?;

    println!("Account type: {:?}", account.account_type);
    println!("Can trade: {}", account.can_trade);
    println!("Can withdraw: {}", account.can_withdraw);
    println!("Can deposit: {}", account.can_deposit);
    println!("\nBalances with funds:");

    for balance in &account.balances {
        if balance.free > 0.0 || balance.locked > 0.0 {
            println!(
                "  {}: free={}, locked={}",
                balance.asset, balance.free, balance.locked
            );
        }
    }
    println!();

    // Get current price for reference.
    let symbol = "BTCUSDT";
    println!("=== Market Price Reference ===\n");
    let price = client.market().price(symbol).await?;
    println!("{} current price: {}\n", symbol, price.price);

    // Test order validation without placing an order.
    println!("=== Test Order Validation ===\n");

    let test_order = OrderBuilder::new(symbol, OrderSide::Buy, OrderType::Limit)
        .quantity("0.001")
        .price("20000.00") // Low price that should not fill.
        .time_in_force(TimeInForce::GTC)
        .build();

    println!("Testing limit buy order: 0.001 BTC @ 20000 USDT...");
    match client.account().test_order(&test_order).await {
        Ok(()) => println!("Order parameters are valid!\n"),
        Err(e) => println!("Order validation failed: {}\n", e),
    }

    // Get open orders.
    println!("=== Open Orders ===\n");
    let open_orders = client.account().open_orders(Some(symbol)).await?;
    if open_orders.is_empty() {
        println!("No open orders for {}\n", symbol);
    } else {
        for order in &open_orders {
            println!(
                "Order #{}: {:?} {:?} {} @ {} (status: {:?})",
                order.order_id,
                order.side,
                order.order_type,
                order.orig_qty,
                order.price,
                order.status
            );
        }
        println!();
    }

    // Place a limit buy order at a low price so it should not fill.
    println!("=== Place Limit Order ===\n");
    println!("Placing limit buy order: 0.001 BTC @ 20000 USDT...");

    let order = OrderBuilder::new(symbol, OrderSide::Buy, OrderType::Limit)
        .quantity("0.001")
        .price("20000.00")
        .time_in_force(TimeInForce::GTC)
        .build();

    match client.account().create_order(&order).await {
        Ok(response) => {
            println!("Order placed successfully!");
            println!("  Order ID: {}", response.order_id);
            println!("  Client Order ID: {}", response.client_order_id);
            println!("  Status: {:?}", response.status);
            println!("  Symbol: {}", response.symbol);
            println!("  Side: {:?}", response.side);
            println!("  Type: {:?}", response.order_type);
            println!("  Price: {}", response.price);
            println!("  Quantity: {}", response.orig_qty);

            // Cancel the order.
            println!("\nCanceling order #{}...", response.order_id);
            match client
                .account()
                .cancel_order(symbol, Some(response.order_id), None)
                .await
            {
                Ok(cancel) => {
                    println!("Order canceled successfully!");
                    println!("  Status: {:?}", cancel.status);
                }
                Err(e) => println!("Failed to cancel order: {}", e),
            }
        }
        Err(e) => {
            println!("Failed to place order: {}", e);
        }
    }
    println!();

    // Show order history.
    println!("=== Recent Order History ===\n");
    let all_orders = client
        .account()
        .all_orders(symbol, None, None, None, Some(5))
        .await?;

    if all_orders.is_empty() {
        println!("No order history for {}", symbol);
    } else {
        for order in &all_orders {
            println!(
                "Order #{}: {:?} {:?} {} @ {} (status: {:?})",
                order.order_id,
                order.side,
                order.order_type,
                order.orig_qty,
                order.price,
                order.status
            );
        }
    }
    println!();

    // Show trade history.
    println!("=== Recent Trade History ===\n");
    let trades = client
        .account()
        .my_trades(symbol, None, None, None, Some(5))
        .await?;

    if trades.is_empty() {
        println!("No trade history for {}", symbol);
    } else {
        for trade in &trades {
            let side = if trade.is_buyer { "BUY" } else { "SELL" };
            println!(
                "Trade #{}: {} {} @ {} (commission: {} {})",
                trade.id,
                side,
                trade.quantity,
                trade.price,
                trade.commission,
                trade.commission_asset
            );
        }
    }
    println!();

    // Demonstrate convenience methods.
    println!("=== Convenience Methods ===\n");
    println!("Available convenience methods:");
    println!("  limit_buy(symbol, qty, price)");
    println!("  limit_sell(symbol, qty, price)");
    println!("  market_buy(symbol, qty)");
    println!("  market_sell(symbol, qty)");
    println!("  market_buy_quote(symbol, quote_qty)");
    println!();

    // Show an OCO order example without placing real orders.
    println!("=== OCO Order Example (Not Executed) ===\n");
    println!("To create an OCO order:");
    println!("```rust");
    println!("use binance_api_client::{{OcoOrderBuilder, OrderSide}};");
    println!();
    println!("let oco = OcoOrderBuilder::new(");
    println!("    \"BTCUSDT\",");
    println!("    OrderSide::Sell,");
    println!("    \"0.001\",      // quantity");
    println!("    \"70000.00\",   // limit price (take profit)");
    println!("    \"45000.00\",   // stop price (stop loss)");
    println!(")");
    println!(".stop_limit_price(\"44900.00\")");
    println!(".build();");
    println!();
    println!("let result = client.account().create_oco(&oco).await?;");
    println!("```\n");

    println!("=== Example completed successfully! ===");
    Ok(())
}
