//! Metrics for tracking arbitrage operations
use crate::QTRADE_RUNTIME_METER;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use lazy_static::lazy_static;
use opentelemetry::metrics::{Counter, Histogram};

/// Metrics for tracking arbitrage operations
pub struct ArbitrageMetrics {
    /// Counter for total number of arbitrage results received
    pub total_results_received: Arc<AtomicU64>,
    /// Counter for total number of arbitrage opportunities processed
    pub total_opportunities_processed: Arc<AtomicU64>,
    /// Counter for total number of successful arbitrage transactions
    pub total_successful_transactions: Arc<AtomicU64>,
    /// Counter for total number of failed arbitrage transactions
    pub total_failed_transactions: Arc<AtomicU64>,
    /// Counter for total profit in USD (stored as integer with 3 decimal places)
    pub total_profit_usd: Arc<AtomicU64>,
}

lazy_static! {
    /// Global static instance of arbitrage metrics
    pub static ref ARBITRAGE_METRICS: ArbitrageMetrics = {
        ArbitrageMetrics {
            total_results_received: Arc::new(AtomicU64::new(0)),
            total_opportunities_processed: Arc::new(AtomicU64::new(0)),
            total_successful_transactions: Arc::new(AtomicU64::new(0)),
            total_failed_transactions: Arc::new(AtomicU64::new(0)),
            total_profit_usd: Arc::new(AtomicU64::new(0)),
        }
    };
}

// Transaction monitoring metrics
lazy_static! {
    static ref TX_CONFIRMED_COUNTER: Counter<u64> = {
        QTRADE_RUNTIME_METER
            .u64_counter("qtrade.arbitrage.transaction_confirmed")
            .with_description("Number of arbitrage transactions confirmed on-chain")
            .build()
    };

    static ref TX_FAILED_COUNTER: Counter<u64> = {
        QTRADE_RUNTIME_METER
            .u64_counter("qtrade.arbitrage.transaction_failed")
            .with_description("Number of arbitrage transactions that failed on-chain")
            .build()
    };

    static ref TX_TIMEOUT_COUNTER: Counter<u64> = {
        QTRADE_RUNTIME_METER
            .u64_counter("qtrade.arbitrage.transaction_timeout")
            .with_description("Number of arbitrage transactions that timed out waiting for confirmation")
            .build()
    };

    static ref TX_CONFIRMATION_RATE: Histogram<f64> = {
        QTRADE_RUNTIME_METER
            .f64_histogram("qtrade.arbitrage.transaction_confirmation_rate")
            .with_description("Rate of successful transaction confirmations")
            .build()
    };
}

/// Record metrics for an arbitrage result
pub fn record_arbitrage_result_received() {
    ARBITRAGE_METRICS.total_results_received.fetch_add(1, Ordering::SeqCst);
}

/// Record metrics for an arbitrage opportunity being processed
pub fn record_arbitrage_opportunity_processed() {
    ARBITRAGE_METRICS.total_opportunities_processed.fetch_add(1, Ordering::SeqCst);
}

/// Record metrics for a successful arbitrage transaction
pub fn record_successful_arbitrage_transaction(profit_usd: f64) {
    ARBITRAGE_METRICS.total_successful_transactions.fetch_add(1, Ordering::SeqCst);

    // Convert to integer (multiply by 1000 to track with 3 decimal points)
    let profit_int = (profit_usd * 1000.0) as u64;
    ARBITRAGE_METRICS.total_profit_usd.fetch_add(profit_int, Ordering::SeqCst);
}

/// Record metrics for a failed arbitrage transaction
pub fn record_failed_arbitrage_transaction() {
    ARBITRAGE_METRICS.total_failed_transactions.fetch_add(1, Ordering::SeqCst);
}

/// Get the total profit in USD (converted back to float with 3 decimal precision)
pub fn get_total_profit_usd() -> f64 {
    let total_int = ARBITRAGE_METRICS.total_profit_usd.load(Ordering::SeqCst);
    total_int as f64 / 1000.0
}

/// Record metrics for a transaction confirmed on-chain
pub fn record_arbitrage_transaction_confirmed(profit: f64) {
    TX_CONFIRMED_COUNTER.add(1, &[]);
    record_successful_arbitrage_transaction(profit);
}

/// Record metrics for a transaction that failed on-chain
pub fn record_arbitrage_transaction_failed() {
    TX_FAILED_COUNTER.add(1, &[]);
    ARBITRAGE_METRICS.total_failed_transactions.fetch_add(1, Ordering::SeqCst);
}

/// Record metrics for a transaction that timed out waiting for confirmation
pub fn record_arbitrage_transaction_timeout() {
    TX_TIMEOUT_COUNTER.add(1, &[]);
    ARBITRAGE_METRICS.total_failed_transactions.fetch_add(1, Ordering::SeqCst);
}

/// Record metrics for the transaction confirmation rate
pub fn record_arbitrage_transaction_confirmation_rate(rate: f64) {
    TX_CONFIRMATION_RATE.record(rate, &[]);
}
