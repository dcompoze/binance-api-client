//! Example demonstrating the Wallet SAPI endpoints.
//!
//! This example shows how to use wallet-related endpoints for:
//! - System status
//! - Coin information
//! - Deposit/withdrawal addresses and history
//! - Asset transfers
//! - Trade fees
//!
//! NOTE: Most wallet endpoints require authentication and may have
//! restrictions on testnet. This example uses production by default
//! but only performs read operations.
//!
//! Before running, set your API credentials:
//!   export BINANCE_API_KEY=your_api_key
//!   export BINANCE_SECRET_KEY=your_secret_key
//!
//! Run with: cargo run --example wallet_api

use binance_api_client::Binance;

#[tokio::main]
async fn main() -> binance_api_client::Result<()> {
    // Initialize logging (optional)
    tracing_subscriber::fmt::init();

    // Load environment variables from .env file if present
    let _ = dotenv::dotenv();

    println!("=== Binance Wallet API Example ===\n");
    println!("NOTE: This example uses the production API with read-only operations.");
    println!("      Some endpoints may require specific API permissions.\n");

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

    // Create client (production by default for wallet SAPI)
    let client = Binance::new(&api_key, &secret_key)?;
    println!(
        "Client created. Using endpoint: {}\n",
        client.config().rest_api_endpoint
    );

    // 1. System Status (no auth required)
    println!("=== System Status ===\n");
    match client.wallet().system_status().await {
        Ok(status) => {
            println!("Status: {}", status.status);
            println!("Message: {}", status.msg);
            println!("Is Normal: {}", status.is_normal());
        }
        Err(e) => println!("Failed to get system status: {}", e),
    }
    println!();

    // 2. All Coins Information
    println!("=== Coin Information (first 5) ===\n");
    match client.wallet().all_coins().await {
        Ok(coins) => {
            for coin in coins.iter().take(5) {
                println!("Coin: {} ({})", coin.coin, coin.name);
                println!("  Deposit enabled: {}", coin.deposit_all_enable);
                println!("  Withdraw enabled: {}", coin.withdraw_all_enable);
                println!("  Free: {}", coin.free);
                println!("  Locked: {}", coin.locked);
                if !coin.network_list.is_empty() {
                    println!("  Networks: {}", coin.network_list.len());
                    if let Some(network) = coin.network_list.first() {
                        println!(
                            "    - {}: min withdraw {}",
                            network.network, network.withdraw_min
                        );
                    }
                }
                println!();
            }
        }
        Err(e) => println!("Failed to get coins: {}", e),
    }

    // 3. Deposit Address
    println!("=== Deposit Address (BTC) ===\n");
    match client.wallet().deposit_address("BTC", None).await {
        Ok(addr) => {
            println!("Coin: {}", addr.coin);
            println!("Address: {}", addr.address);
            if !addr.tag.is_empty() {
                println!("Tag/Memo: {}", addr.tag);
            }
            if let Some(url) = &addr.url {
                println!("URL: {}", url);
            }
        }
        Err(e) => println!("Failed to get deposit address: {}", e),
    }
    println!();

    // 4. Deposit History
    println!("=== Recent Deposits (last 5) ===\n");
    match client
        .wallet()
        .deposit_history(None, None, None, None, None, Some(5))
        .await
    {
        Ok(deposits) => {
            if deposits.is_empty() {
                println!("No deposit history found.");
            } else {
                for dep in deposits {
                    println!(
                        "{}: {} {} (status: {:?})",
                        dep.coin, dep.amount, dep.network, dep.status
                    );
                    println!("  Address: {}", dep.address);
                    println!("  TxId: {}", dep.tx_id);
                    println!();
                }
            }
        }
        Err(e) => println!("Failed to get deposit history: {}", e),
    }

    // 5. Withdrawal History
    println!("=== Recent Withdrawals (last 5) ===\n");
    match client
        .wallet()
        .withdraw_history(None, None, None, None, None, None, Some(5))
        .await
    {
        Ok(withdrawals) => {
            if withdrawals.is_empty() {
                println!("No withdrawal history found.");
            } else {
                for wd in withdrawals {
                    println!(
                        "{}: {} {} (status: {:?})",
                        wd.coin, wd.amount, wd.network, wd.status
                    );
                    println!("  Address: {}", wd.address);
                    if let Some(tx_id) = &wd.tx_id {
                        println!("  TxId: {}", tx_id);
                    }
                    println!();
                }
            }
        }
        Err(e) => println!("Failed to get withdrawal history: {}", e),
    }

    // 6. Trade Fees
    println!("=== Trade Fees (BTCUSDT) ===\n");
    match client.wallet().trade_fee(Some("BTCUSDT")).await {
        Ok(fees) => {
            for fee in fees {
                println!("Symbol: {}", fee.symbol);
                println!("  Maker fee: {}%", fee.maker_commission * 100.0);
                println!("  Taker fee: {}%", fee.taker_commission * 100.0);
            }
        }
        Err(e) => println!("Failed to get trade fees: {}", e),
    }
    println!();

    // 7. Wallet Balance
    println!("=== Spot Wallet Balance ===\n");
    match client.wallet().wallet_balance().await {
        Ok(balances) => {
            let non_zero: Vec<_> = balances.iter().filter(|b| b.balance > 0.0).collect();

            if non_zero.is_empty() {
                println!("No balances found.");
            } else {
                for bal in non_zero.iter().take(10) {
                    println!("{}: {}", bal.wallet_name, bal.balance);
                }
                if non_zero.len() > 10 {
                    println!("... and {} more", non_zero.len() - 10);
                }
            }
        }
        Err(e) => println!("Failed to get wallet balance: {}", e),
    }
    println!();

    // 8. Account Status
    println!("=== Account Status ===\n");
    match client.wallet().account_status().await {
        Ok(status) => {
            println!("Data: {}", status.data);
        }
        Err(e) => println!("Failed to get account status: {}", e),
    }
    println!();

    // 9. API Trading Status
    println!("=== API Trading Status ===\n");
    match client.wallet().api_trading_status().await {
        Ok(status) => {
            println!("Is Locked: {}", status.data.is_locked);
            if let Some(recover_time) = status.data.planned_recover_time {
                println!("Planned Recover Time: {}", recover_time);
            }
            println!("Update Time: {}", status.data.update_time);
        }
        Err(e) => println!("Failed to get API trading status: {}", e),
    }
    println!();

    // 10. API Key Permissions
    println!("=== API Key Permissions ===\n");
    match client.wallet().api_key_permissions().await {
        Ok(perms) => {
            println!("IP Restricted: {}", perms.ip_restrict);
            println!("Enable Withdrawals: {}", perms.enable_withdrawals);
            println!(
                "Enable Internal Transfer: {}",
                perms.enable_internal_transfer
            );
            println!(
                "Enable Spot Trading: {}",
                perms.enable_spot_and_margin_trading
            );
            println!("Enable Futures: {}", perms.enable_futures);
        }
        Err(e) => println!("Failed to get API key permissions: {}", e),
    }
    println!();

    println!("=== Example completed successfully! ===");
    Ok(())
}

fn show_example_code() {
    println!("=== Wallet API Example Code ===\n");

    println!("// Create authenticated client");
    println!("let client = Binance::new(\"api_key\", \"secret_key\")?;\n");

    println!("// Check system status");
    println!("let status = client.wallet().system_status().await?;");
    println!("println!(\"System is normal: {{}}\", status.is_normal());\n");

    println!("// Get all coin information");
    println!("let coins = client.wallet().all_coins().await?;");
    println!("for coin in coins {{");
    println!("    println!(\"{{}}: free={{}}\", coin.coin, coin.free);");
    println!("}}\n");

    println!("// Get deposit address for BTC");
    println!("let addr = client.wallet().deposit_address(\"BTC\", None).await?;");
    println!("println!(\"Deposit to: {{}}\", addr.address);\n");

    println!("// Get deposit history");
    println!("let deposits = client.wallet()");
    println!("    .deposit_history(Some(\"BTC\"), None, None, None, None, Some(10))");
    println!("    .await?;\n");

    println!("// Withdraw (requires withdrawal permission)");
    println!("let result = client.wallet()");
    println!("    .withdraw(\"USDT\", \"0x...\", \"100.0\", None, Some(\"ETH\"), None)");
    println!("    .await?;");
    println!("println!(\"Withdrawal ID: {{}}\", result.id);\n");

    println!("// Universal transfer (between wallets)");
    println!("use binance_api_client::UniversalTransferType;");
    println!("let result = client.wallet()");
    println!("    .universal_transfer(");
    println!("        UniversalTransferType::MainToFunding,");
    println!("        \"USDT\",");
    println!("        \"50.0\",");
    println!("    )");
    println!("    .await?;\n");

    println!("// Get trade fees");
    println!("let fees = client.wallet().trade_fee(Some(\"BTCUSDT\")).await?;\n");

    println!("// Get wallet balance");
    println!("let balance = client.wallet().wallet_balance().await?;\n");

    println!("=== End of Example Code ===");
}
