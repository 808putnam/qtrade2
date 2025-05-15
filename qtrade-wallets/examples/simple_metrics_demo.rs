//! Example demonstrating how to directly use the wallet metrics system
//!
//! This example shows:
//! 1. How to record metrics directly using the wallet_metrics module
//! 2. How to access and display recorded metrics

use anyhow::Result;
use std::env;
use std::sync::atomic::Ordering;
use qtrade_wallets::wallet_metrics;

fn main() -> Result<()> {
    // Enable logging
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    println!("Demonstrating wallet metrics system (simple version)");

    // Initialize the metrics system
    wallet_metrics::init();

    // Simulate recording some metrics directly
    println!("\n--- Recording metrics directly ---");

    // Record explorer key metrics
    println!("Recording explorer key metrics...");
    wallet_metrics::record_explorer_key_acquired();
    wallet_metrics::record_explorer_key_acquired();
    wallet_metrics::record_explorer_key_acquired();
    wallet_metrics::record_explorer_key_retired();
    wallet_metrics::record_explorer_key_retired();
    wallet_metrics::record_explorer_keys_created(2);
    wallet_metrics::record_explorer_keys_funds_recovered(1, 10_000_000); // 0.01 SOL

    // Record bank key metrics
    println!("Recording bank key metrics...");
    wallet_metrics::record_bank_keys_funded(1);

    // Record key pool sizes
    println!("Recording key pool size metrics...");
    wallet_metrics::record_key_pool_sizes(
        5, 3,  // HODL keys: 5 total, 3 available
        10, 7, // Bank keys: 10 total, 7 available
        15, 8  // Explorer keys: 15 total, 8 available
    );

    // Record key balance metrics
    println!("Recording key balance metrics...");
    wallet_metrics::record_key_balance("hodl", 100.0);   // 100 SOL
    wallet_metrics::record_key_balance("bank", 10.5);    // 10.5 SOL
    wallet_metrics::record_key_balance("explorer", 0.1); // 0.1 SOL

    // Print the metrics
    print_metrics();

    // Force OpenTelemetry to export metrics
    wallet_metrics::otel::record_otel_metrics();

    println!("\nWallet metrics demonstration complete!");
    Ok(())
}

fn print_metrics() {
    // Access the wallet metrics directly
    let explorer_keys_acquired = wallet_metrics::WALLET_METRICS.explorer_keys_acquired.load(Ordering::SeqCst);
    let explorer_keys_retired = wallet_metrics::WALLET_METRICS.explorer_keys_retired.load(Ordering::SeqCst);
    let explorer_keys_created = wallet_metrics::WALLET_METRICS.explorer_keys_created.load(Ordering::SeqCst);
    let explorer_keys_recovered = wallet_metrics::WALLET_METRICS.explorer_keys_funds_recovered.load(Ordering::SeqCst);
    let bank_keys_funded = wallet_metrics::WALLET_METRICS.bank_keys_funded.load(Ordering::SeqCst);
    let sol_recovered = wallet_metrics::get_total_sol_recovered();

    println!("\n--- WALLET METRICS ---");
    println!("Explorer keys acquired: {}", explorer_keys_acquired);
    println!("Explorer keys retired: {}", explorer_keys_retired);
    println!("Explorer keys created: {}", explorer_keys_created);
    println!("Explorer keys recovered: {}", explorer_keys_recovered);
    println!("Bank keys funded: {}", bank_keys_funded);
    println!("Total SOL recovered: {:.6} SOL", sol_recovered);
}
