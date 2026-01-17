//! Example demonstrating the Margin Trading SAPI endpoints.
//!
//! This example shows how to use margin trading endpoints for:
//! - Cross-margin and isolated margin accounts
//! - Borrowing and repaying
//! - Margin orders
//! - Interest and loan history
//!
//! WARNING: Margin trading involves borrowing funds and carries significant
//! risk. Only use with funds you can afford to lose.
//!
//! NOTE: Margin endpoints require:
//! - API key with margin trading permission
//! - Margin account enabled on Binance
//! - May not be available on testnet
//!
//! Before running, set your API credentials:
//!   export BINANCE_API_KEY=your_api_key
//!   export BINANCE_SECRET_KEY=your_secret_key
//!
//! Run with: cargo run --example margin_trading

use binance_api_client::Binance;

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    // Initialize logging (optional)
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file if present
    let _ = dotenv::dotenv();

    println!("=== Binance Margin Trading API Example ===\n");
    println!("WARNING: Margin trading carries significant risk!");
    println!("         Only use with funds you can afford to lose.\n");

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

    // Create client
    let client = Binance::new(&api_key, &secret_key)?;
    println!(
        "Client created. Using endpoint: {}\n",
        client.config().rest_api_endpoint
    );

    // 1. Cross-Margin Account Information
    println!("=== Cross-Margin Account ===\n");
    match client.margin().account().await {
        Ok(account) => {
            println!("Borrow Enabled: {}", account.borrow_enabled);
            println!("Trade Enabled: {}", account.trade_enabled);
            println!("Margin Level: {}", account.margin_level);
            println!("Total Asset (BTC): {}", account.total_asset_of_btc);
            println!("Total Liability (BTC): {}", account.total_liability_of_btc);
            println!("Total Net Asset (BTC): {}", account.total_net_asset_of_btc);
            println!("\nAssets with balance:");
            for asset in &account.user_assets {
                if asset.free > 0.0 || asset.locked > 0.0 || asset.borrowed > 0.0 {
                    println!(
                        "  {}: free={}, locked={}, borrowed={}, interest={}",
                        asset.asset, asset.free, asset.locked, asset.borrowed, asset.interest
                    );
                }
            }
        }
        Err(e) => println!(
            "Failed to get margin account: {} (Margin may not be enabled)",
            e
        ),
    }
    println!();

    // 2. Max Borrowable Amount
    println!("=== Max Borrowable (USDT) ===\n");
    match client.margin().max_borrowable("USDT", None).await {
        Ok(max) => {
            println!("Amount: {}", max.amount);
            println!("Borrow Limit: {:?}", max.borrow_limit);
        }
        Err(e) => println!("Failed to get max borrowable: {}", e),
    }
    println!();

    // 3. Max Transferable Amount
    println!("=== Max Transferable (BTC) ===\n");
    match client.margin().max_transferable("BTC", None).await {
        Ok(max) => {
            println!("Amount: {}", max.amount);
        }
        Err(e) => println!("Failed to get max transferable: {}", e),
    }
    println!();

    // 4. Margin Pair Information
    println!("=== Margin Pair Info (BTCUSDT) ===\n");
    match client.margin().pair("BTCUSDT").await {
        Ok(pair) => {
            println!("Symbol: {}", pair.symbol);
            println!(
                "Base Asset: {} (borrowable: {})",
                pair.base, pair.is_buy_allowed
            );
            println!(
                "Quote Asset: {} (borrowable: {})",
                pair.quote, pair.is_sell_allowed
            );
            println!("Is Margin Trade: {}", pair.is_margin_trade);
        }
        Err(e) => println!("Failed to get pair info: {}", e),
    }
    println!();

    // 5. All Margin Assets
    println!("=== Margin Assets (first 5) ===\n");
    match client.margin().all_assets().await {
        Ok(assets) => {
            for asset in assets.iter().take(5) {
                println!(
                    "{}: borrowable={}, full_name={}",
                    asset.asset_name, asset.is_borrowable, asset.asset_full_name
                );
            }
        }
        Err(e) => println!("Failed to get margin assets: {}", e),
    }
    println!();

    // 6. Price Index
    println!("=== Price Index (BTCUSDT) ===\n");
    match client.margin().price_index("BTCUSDT").await {
        Ok(index) => {
            println!("Symbol: {}", index.symbol);
            println!("Price: {}", index.price);
            println!("Calc Time: {}", index.calc_time);
        }
        Err(e) => println!("Failed to get price index: {}", e),
    }
    println!();

    // 7. Interest Rate History
    println!("=== Interest Rate History (BTC) ===\n");
    match client
        .margin()
        .interest_rate_history("BTC", None, None, None, Some(3))
        .await
    {
        Ok(records) => {
            if records.is_empty() {
                println!("No interest rate history found.");
            } else {
                for record in records {
                    println!(
                        "{}: daily_rate={}, timestamp={}",
                        record.asset, record.daily_interest_rate, record.timestamp
                    );
                }
            }
        }
        Err(e) => println!("Failed to get interest rate history: {}", e),
    }
    println!();

    // 8. BNB Burn Status
    println!("=== BNB Burn Status ===\n");
    match client.margin().bnb_burn_status().await {
        Ok(status) => {
            println!("Spot BNB Burn: {}", status.spot_bnb_burn);
            println!("Interest BNB Burn: {}", status.interest_bnb_burn);
        }
        Err(e) => println!("Failed to get BNB burn status: {}", e),
    }
    println!();

    // 9. Open Margin Orders
    println!("=== Open Margin Orders (BTCUSDT) ===\n");
    match client.margin().open_orders(Some("BTCUSDT"), None).await {
        Ok(orders) => {
            if orders.is_empty() {
                println!("No open margin orders.");
            } else {
                for order in orders {
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
        }
        Err(e) => println!("Failed to get open orders: {}", e),
    }
    println!();

    // 10. Loan Records
    println!("=== Recent Loan Records ===\n");
    match client
        .margin()
        .loan_records("USDT", None, None, None, None, Some(5))
        .await
    {
        Ok(result) => {
            if result.rows.is_empty() {
                println!("No loan records found.");
            } else {
                for record in result.rows {
                    println!(
                        "{}: {} (status: {})",
                        record.asset, record.principal, record.status
                    );
                }
            }
        }
        Err(e) => println!("Failed to get loan records: {}", e),
    }
    println!();

    // Note about write operations
    println!("=== Write Operations (Not Executed) ===\n");
    println!("The following operations modify your account and are shown as examples only:\n");

    println!("// Transfer from spot to margin");
    println!("let result = client.margin()");
    println!("    .transfer(\"USDT\", \"100.0\", MarginTransferType::MainToMargin)");
    println!("    .await?;\n");

    println!("// Borrow funds");
    println!("let result = client.margin()");
    println!("    .loan(\"USDT\", \"50.0\", false, None)  // false = cross margin");
    println!("    .await?;\n");

    println!("// Repay loan");
    println!("let result = client.margin()");
    println!("    .repay(\"USDT\", \"50.0\", false, None)");
    println!("    .await?;\n");

    println!("// Create margin order with auto-borrow");
    println!("let result = client.margin()");
    println!("    .create_order(");
    println!("        \"BTCUSDT\",");
    println!("        OrderSide::Buy,");
    println!("        OrderType::Limit,");
    println!("        Some(TimeInForce::GTC),");
    println!("        Some(\"0.001\"),   // quantity");
    println!("        None,             // quote quantity");
    println!("        Some(\"50000.0\"), // price");
    println!("        None,             // stop price");
    println!("        None,             // client order id");
    println!("        None,             // iceberg qty");
    println!("        Some(SideEffectType::MarginBuy), // auto borrow");
    println!("        None,             // is isolated");
    println!("    )");
    println!("    .await?;\n");

    println!("=== Example completed successfully! ===");
    Ok(())
}

fn show_example_code() {
    println!("=== Margin Trading API Example Code ===\n");

    println!("// Create authenticated client");
    println!("let client = Binance::new(\"api_key\", \"secret_key\")?;\n");

    println!("// Get cross-margin account details");
    println!("let account = client.margin().account().await?;");
    println!("println!(\"Margin Level: {{}}\", account.margin_level);\n");

    println!("// Get isolated margin account");
    println!("let isolated = client.margin().isolated_account(Some(vec![\"BTCUSDT\"])).await?;\n");

    println!("// Check max borrowable amount");
    println!("let max = client.margin().max_borrowable(\"USDT\", None).await?;");
    println!("println!(\"Can borrow up to: {{}}\", max.amount);\n");

    println!("// Transfer to margin account");
    println!("use binance_api_client::MarginTransferType;");
    println!("let tx = client.margin()");
    println!("    .transfer(\"USDT\", \"100.0\", MarginTransferType::MainToMargin)");
    println!("    .await?;\n");

    println!("// Borrow funds (cross margin)");
    println!("let tx = client.margin().loan(\"USDT\", \"50.0\", false, None).await?;\n");

    println!("// Borrow funds (isolated margin)");
    println!(
        "let tx = client.margin().loan(\"USDT\", \"50.0\", true, Some(\"BTCUSDT\")).await?;\n"
    );

    println!("// Repay loan");
    println!("let tx = client.margin().repay(\"USDT\", \"50.0\", false, None).await?;\n");

    println!("// Create margin order with side effect (auto-borrow)");
    println!("use binance_api_client::{{OrderSide, OrderType, TimeInForce, SideEffectType}};");
    println!("let order = client.margin().create_order(");
    println!("    \"BTCUSDT\",");
    println!("    OrderSide::Buy,");
    println!("    OrderType::Limit,");
    println!("    Some(TimeInForce::GTC),");
    println!("    Some(\"0.001\"),");
    println!("    None,");
    println!("    Some(\"50000.0\"),");
    println!("    None,");
    println!("    None,");
    println!("    None,");
    println!("    Some(SideEffectType::MarginBuy), // AUTO_BORROW_REPAY");
    println!("    None,");
    println!(").await?;\n");

    println!("// Get margin trades");
    println!("let trades = client.margin()");
    println!("    .my_trades(\"BTCUSDT\", None, None, None, None, Some(10), None)");
    println!("    .await?;\n");

    println!("// Get loan/repay records");
    println!("let loans = client.margin()");
    println!("    .loan_records(\"USDT\", None, None, None, None, Some(10))");
    println!("    .await?;\n");

    println!("// Get interest history");
    println!("let interest = client.margin()");
    println!("    .interest_history(Some(\"USDT\"), None, None, None, Some(10))");
    println!("    .await?;\n");

    println!("=== End of Example Code ===");
}
