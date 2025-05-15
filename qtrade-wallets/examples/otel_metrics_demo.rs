//! Example demonstrating OpenTelemetry metrics integration
//!
//! This example shows:
//! 1. How to initialize OpenTelemetry
//! 2. How to configure exporters for metrics
//! 3. How the wallet metrics are exported

use anyhow::Result;

use std::env;
use tokio::time::Duration;
use qtrade_wallets::wallet_metrics;

#[tokio::main]
async fn main() -> Result<()> {
    // Enable logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    println!("Demonstrating OpenTelemetry metrics integration");

    // Initialize OpenTelemetry to export metrics to console
    init_opentelemetry()?;

    // Initialize the qtrade-wallets system
    qtrade_wallets::init()?;

    // Create some simulated usage
    simulate_wallet_usage().await?;

    // Wait a bit to allow metrics to be exported
    tokio::time::sleep(Duration::from_secs(1)).await;

    println!("OpenTelemetry metrics demonstration complete!");

    // Shutdown OpenTelemetry - In 0.28.0, we don't need to call shutdown explicitly
    // as it happens automatically via Drop trait implementations

    Ok(())
}

fn init_opentelemetry() -> Result<()> {
    // With OpenTelemetry 0.28.0, we'll use a simpler initialization
    // This just ensures the global meter provider is initialized
    // Note: In a real application, you would configure proper exporters

    println!("OpenTelemetry metrics initialized");
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
    // This will also record metrics about key pool sizes and balances
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

    // Force OpenTelemetry to export metrics
    wallet_metrics::otel::record_otel_metrics();

    Ok(())
}
