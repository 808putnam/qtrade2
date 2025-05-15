//! Example demonstrating the wallet metrics system
//!
//! This example shows:
//! 1. How to initialize the wallet metrics system
//! 2. How metrics are recorded during key operations
//! 3. How to view the metrics

use anyhow::Result;

use std::env;
use tokio::time::Duration;
use qtrade_wallets::wallet_metrics;

#[tokio::main]
async fn main() -> Result<()> {
    // Enable logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    println!("Demonstrating wallet metrics system");

    // First initialize the qtrade-wallets system
    qtrade_wallets::init()?;

    // Create some simulated usage
    simulate_wallet_usage().await?;

    // Print the metrics
    print_metrics();

    println!("Wallet metrics demonstration complete!");
    Ok(())
}

async fn simulate_wallet_usage() -> Result<()> {
    println!("\n--- Simulating wallet usage ---");

    for i in 0..5 {
        // Get explorer keypair (this will record metrics)
        if let Some((pubkey, _keypair)) = qtrade_wallets::get_explorer_keypair() {
            println!("{i}. Got explorer key: {}", pubkey);

            // Simulate a small delay for the "transaction"
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Return the keypair as used (this will record metrics)
            qtrade_wallets::return_explorer_keypair(&pubkey, true)?;
            println!("{i}. Retired explorer key: {}", pubkey);
        } else {
            println!("{i}. No explorer keypairs available");
        }
    }

    // Run the balancer which will create new explorer keys as needed
    println!("\n--- Running wallet balancer ---");
    qtrade_wallets::balancer().await?;

    // Let's use a few more explorer keys
    println!("\n--- Using more explorer keys ---");
    for i in 0..3 {
        if let Some((pubkey, _keypair)) = qtrade_wallets::get_explorer_keypair() {
            println!("{i}. Got explorer key: {}", pubkey);

            // Simulate a small delay for the "transaction"
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Return the keypair as used
            qtrade_wallets::return_explorer_keypair(&pubkey, true)?;
            println!("{i}. Retired explorer key: {}", pubkey);
        }
    }

    // Run the balancer again
    println!("\n--- Running wallet balancer again ---");
    qtrade_wallets::balancer().await?;

    Ok(())
}

fn print_metrics() {
    // Access the wallet metrics directly
    let explorer_keys_acquired = wallet_metrics::WALLET_METRICS.explorer_keys_acquired.load(std::sync::atomic::Ordering::SeqCst);
    let explorer_keys_retired = wallet_metrics::WALLET_METRICS.explorer_keys_retired.load(std::sync::atomic::Ordering::SeqCst);
    let explorer_keys_created = wallet_metrics::WALLET_METRICS.explorer_keys_created.load(std::sync::atomic::Ordering::SeqCst);
    let explorer_keys_recovered = wallet_metrics::WALLET_METRICS.explorer_keys_funds_recovered.load(std::sync::atomic::Ordering::SeqCst);
    let bank_keys_funded = wallet_metrics::WALLET_METRICS.bank_keys_funded.load(std::sync::atomic::Ordering::SeqCst);
    let sol_recovered = wallet_metrics::get_total_sol_recovered();

    println!("\n--- WALLET METRICS ---");
    println!("Explorer keys acquired: {}", explorer_keys_acquired);
    println!("Explorer keys retired: {}", explorer_keys_retired);
    println!("Explorer keys created: {}", explorer_keys_created);
    println!("Explorer keys recovered: {}", explorer_keys_recovered);
    println!("Bank keys funded: {}", bank_keys_funded);
    println!("Total SOL recovered: {:.6} SOL", sol_recovered);
}
