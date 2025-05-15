//! OpenTelemetry integration for wallet metrics

use opentelemetry::global;
use opentelemetry::metrics::{Counter, Histogram, Meter};
use lazy_static::lazy_static;
use super::WALLET_METRICS;
use std::sync::atomic::Ordering;

// Create a static meter provider
pub const QTRADE_WALLETS_METER_NAME: &str = "qtrade-wallets";

lazy_static! {
    static ref QTRADE_WALLETS_METER: Meter = global::meter(QTRADE_WALLETS_METER_NAME);
}

// OpenTelemetry counters for key operations
lazy_static! {
    // Explorer key metrics
    static ref EXPLORER_KEYS_ACQUIRED_COUNTER: Counter<u64> = {
        QTRADE_WALLETS_METER
            .u64_counter("qtrade.wallets.explorer_keys_acquired")
            .with_description("Number of explorer keys acquired for transaction signing")
            .build()
    };

    static ref EXPLORER_KEYS_RETIRED_COUNTER: Counter<u64> = {
        QTRADE_WALLETS_METER
            .u64_counter("qtrade.wallets.explorer_keys_retired")
            .with_description("Number of explorer keys retired after use")
            .build()
    };

    static ref EXPLORER_KEYS_CREATED_COUNTER: Counter<u64> = {
        QTRADE_WALLETS_METER
            .u64_counter("qtrade.wallets.explorer_keys_created")
            .with_description("Number of new explorer keys created")
            .build()
    };

    static ref EXPLORER_KEYS_FUNDS_RECOVERED_COUNTER: Counter<u64> = {
        QTRADE_WALLETS_METER
            .u64_counter("qtrade.wallets.explorer_keys_funds_recovered")
            .with_description("Number of explorer keys with funds recovered")
            .build()
    };

    // Bank key metrics
    static ref BANK_KEYS_FUNDED_COUNTER: Counter<u64> = {
        QTRADE_WALLETS_METER
            .u64_counter("qtrade.wallets.bank_keys_funded")
            .with_description("Number of bank keys funded from HODL keys")
            .build()
    };

    // SOL metrics
    static ref SOL_RECOVERED_COUNTER: Counter<u64> = {
        QTRADE_WALLETS_METER
            .u64_counter("qtrade.wallets.sol_recovered")
            .with_description("Total SOL recovered from explorer keys (lamports * 10^-9)")
            .build()
    };

    // Key pool status metrics
    static ref KEY_POOL_SIZE_GAUGE: Counter<u64> = {
        QTRADE_WALLETS_METER
            .u64_counter("qtrade.wallets.key_pool_size")
            .with_description("Size of each key pool by tier and status")
            .build()
    };

    // Key balance metrics
    static ref KEY_BALANCE_HISTOGRAM: Histogram<f64> = {
        QTRADE_WALLETS_METER
            .f64_histogram("qtrade.wallets.key_balance")
            .with_description("Distribution of key balances by tier")
            .build()
    };
}

/// Initialize OpenTelemetry metrics
pub fn init() {
    // This function is called to ensure the lazy_static initialization happens
    // and sets up the OpenTelemetry meters
}

/// Record OpenTelemetry metrics based on the current wallet metrics
pub fn record_otel_metrics() {
    // Explorer key metrics
    let acquired = WALLET_METRICS.explorer_keys_acquired.load(Ordering::SeqCst);
    EXPLORER_KEYS_ACQUIRED_COUNTER.add(acquired, &[]);

    let retired = WALLET_METRICS.explorer_keys_retired.load(Ordering::SeqCst);
    EXPLORER_KEYS_RETIRED_COUNTER.add(retired, &[]);

    let created = WALLET_METRICS.explorer_keys_created.load(Ordering::SeqCst);
    EXPLORER_KEYS_CREATED_COUNTER.add(created, &[]);

    let recovered = WALLET_METRICS.explorer_keys_funds_recovered.load(Ordering::SeqCst);
    EXPLORER_KEYS_FUNDS_RECOVERED_COUNTER.add(recovered, &[]);

    // Bank key metrics
    let funded = WALLET_METRICS.bank_keys_funded.load(Ordering::SeqCst);
    BANK_KEYS_FUNDED_COUNTER.add(funded, &[]);

    // SOL metrics
    let sol_recovered = WALLET_METRICS.total_sol_recovered.load(Ordering::SeqCst);
    SOL_RECOVERED_COUNTER.add(sol_recovered, &[]);
}

/// Record metrics for key pool sizes in OpenTelemetry
pub fn record_key_pool_sizes(hodl_total: u64, hodl_available: u64,
                           bank_total: u64, bank_available: u64,
                           explorer_total: u64, explorer_available: u64) {
    // Record with appropriate labels for tier and status
    KEY_POOL_SIZE_GAUGE.add(hodl_total, &[opentelemetry::KeyValue::new("tier", "hodl"),
                                        opentelemetry::KeyValue::new("status", "total")]);
    KEY_POOL_SIZE_GAUGE.add(hodl_available, &[opentelemetry::KeyValue::new("tier", "hodl"),
                                           opentelemetry::KeyValue::new("status", "available")]);

    KEY_POOL_SIZE_GAUGE.add(bank_total, &[opentelemetry::KeyValue::new("tier", "bank"),
                                        opentelemetry::KeyValue::new("status", "total")]);
    KEY_POOL_SIZE_GAUGE.add(bank_available, &[opentelemetry::KeyValue::new("tier", "bank"),
                                           opentelemetry::KeyValue::new("status", "available")]);

    KEY_POOL_SIZE_GAUGE.add(explorer_total, &[opentelemetry::KeyValue::new("tier", "explorer"),
                                           opentelemetry::KeyValue::new("status", "total")]);
    KEY_POOL_SIZE_GAUGE.add(explorer_available, &[opentelemetry::KeyValue::new("tier", "explorer"),
                                              opentelemetry::KeyValue::new("status", "available")]);
}

/// Record key balance in OpenTelemetry
pub fn record_key_balance(tier: &str, balance_sol: f64) {
    // Convert &str to String to avoid borrowed data escaping the function
    let tier_owned = tier.to_string();
    KEY_BALANCE_HISTOGRAM.record(balance_sol, &[opentelemetry::KeyValue::new("tier", tier_owned)]);
}
