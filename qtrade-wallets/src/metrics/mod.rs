//! Metrics module for wallet management operations
//!
//! This module provides metrics tracking for the tiered key management system.
//! It includes both internal atomic counters and OpenTelemetry integration.

// Reexport OpenTelemetry integration
pub mod otel;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use lazy_static::lazy_static;
// Create a global metric tracker for the qtrade-wallets module
pub const QTRADE_WALLETS_METER_NAME: &str = "qtrade-wallets";

/// Metrics for tracking wallet key management operations
pub struct WalletMetrics {
    /// Counter for explorer keys acquired for use
    pub explorer_keys_acquired: Arc<AtomicU64>,
    /// Counter for explorer keys retired
    pub explorer_keys_retired: Arc<AtomicU64>,
    /// Counter for explorer keys created
    pub explorer_keys_created: Arc<AtomicU64>,
    /// Counter for explorer keys with funds recovered
    pub explorer_keys_funds_recovered: Arc<AtomicU64>,
    /// Counter for bank keys funded
    pub bank_keys_funded: Arc<AtomicU64>,
    /// Counter for total SOL recovered from explorer keys (lamports * 10^-9)
    pub total_sol_recovered: Arc<AtomicU64>,
}

lazy_static! {
    /// Global static instance of wallet metrics
    pub static ref WALLET_METRICS: WalletMetrics = {
        WalletMetrics {
            explorer_keys_acquired: Arc::new(AtomicU64::new(0)),
            explorer_keys_retired: Arc::new(AtomicU64::new(0)),
            explorer_keys_created: Arc::new(AtomicU64::new(0)),
            explorer_keys_funds_recovered: Arc::new(AtomicU64::new(0)),
            bank_keys_funded: Arc::new(AtomicU64::new(0)),
            total_sol_recovered: Arc::new(AtomicU64::new(0)),
        }
    };
}

/// Record metrics for an explorer key being acquired for transaction signing
pub fn record_explorer_key_acquired() {
    WALLET_METRICS.explorer_keys_acquired.fetch_add(1, Ordering::SeqCst);
}

/// Record metrics for an explorer key being retired
pub fn record_explorer_key_retired() {
    WALLET_METRICS.explorer_keys_retired.fetch_add(1, Ordering::SeqCst);
}

/// Record metrics for new explorer keys being created
pub fn record_explorer_keys_created(count: u64) {
    WALLET_METRICS.explorer_keys_created.fetch_add(count, Ordering::SeqCst);
}

/// Record metrics for explorer keys with funds recovered
pub fn record_explorer_keys_funds_recovered(count: u64, total_lamports: u64) {
    WALLET_METRICS.explorer_keys_funds_recovered.fetch_add(count, Ordering::SeqCst);

    // Convert lamports to SOL with fixed precision (1 SOL = 10^9 lamports)
    // Storing as integer with 3 decimal precision (1 SOL = 1000 units)
    let sol = (total_lamports as f64) / 1_000_000.0; // Represents SOL with 3 decimal points
    let sol_integer = sol as u64;

    WALLET_METRICS.total_sol_recovered.fetch_add(sol_integer, Ordering::SeqCst);
}

/// Record metrics for bank keys being funded
pub fn record_bank_keys_funded(count: u64) {
    WALLET_METRICS.bank_keys_funded.fetch_add(count, Ordering::SeqCst);
}

/// Record metrics for key balance
pub fn record_key_balance(tier: &str, balance_sol: f64) {
    // Pass through to OpenTelemetry
    otel::record_key_balance(tier, balance_sol);
}

/// Record metrics for key pool sizes
pub fn record_key_pool_sizes(
    hodl_total: u64, hodl_available: u64,
    bank_total: u64, bank_available: u64,
    explorer_total: u64, explorer_available: u64
) {
    // Pass through to OpenTelemetry
    otel::record_key_pool_sizes(
        hodl_total, hodl_available,
        bank_total, bank_available,
        explorer_total, explorer_available
    );
}

/// Initialize the metrics module
pub fn init() {
    // Initialize OpenTelemetry metrics
    otel::init();
}

/// Get the total number of SOL recovered from explorer keys
pub fn get_total_sol_recovered() -> f64 {
    let total = WALLET_METRICS.total_sol_recovered.load(Ordering::SeqCst);
    // Convert back to SOL with 3 decimal precision
    total as f64 / 1000.0
}
