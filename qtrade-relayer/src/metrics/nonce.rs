//! Metrics for tracking nonce account operations
use crate::constants::QTRADE_RELAYER_METER;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use lazy_static::lazy_static;
use opentelemetry::metrics::{Counter, Histogram};

/// Metrics for tracking nonce account operations
pub struct NonceMetrics {
    /// Total number of nonce accounts in the pool
    pub total_nonce_accounts: Arc<AtomicU64>,
    /// Number of available nonce accounts
    pub available_nonce_accounts: Arc<AtomicU64>,
    /// Number of nonce accounts in use
    pub in_use_nonce_accounts: Arc<AtomicU64>,
    /// Number of nonce accounts needing initialization
    pub needs_init_nonce_accounts: Arc<AtomicU64>,
    /// Number of nonce accounts needing advancement
    pub needs_advance_nonce_accounts: Arc<AtomicU64>,
    /// Total number of nonce account acquisitions
    pub total_nonce_acquisitions: Arc<AtomicU64>,
    /// Total number of nonce account releases
    pub total_nonce_releases: Arc<AtomicU64>,
    /// Total number of nonce account initialization attempts
    pub total_init_attempts: Arc<AtomicU64>,
    /// Total number of successful nonce account initializations
    pub successful_init_attempts: Arc<AtomicU64>,
    /// Total number of nonce account advancement attempts
    pub total_advance_attempts: Arc<AtomicU64>,
    /// Total number of successful nonce account advancements
    pub successful_advance_attempts: Arc<AtomicU64>,
}

lazy_static! {
    /// Global static instance of nonce metrics
    pub static ref NONCE_METRICS: NonceMetrics = {
        NonceMetrics {
            total_nonce_accounts: Arc::new(AtomicU64::new(0)),
            available_nonce_accounts: Arc::new(AtomicU64::new(0)),
            in_use_nonce_accounts: Arc::new(AtomicU64::new(0)),
            needs_init_nonce_accounts: Arc::new(AtomicU64::new(0)),
            needs_advance_nonce_accounts: Arc::new(AtomicU64::new(0)),
            total_nonce_acquisitions: Arc::new(AtomicU64::new(0)),
            total_nonce_releases: Arc::new(AtomicU64::new(0)),
            total_init_attempts: Arc::new(AtomicU64::new(0)),
            successful_init_attempts: Arc::new(AtomicU64::new(0)),
            total_advance_attempts: Arc::new(AtomicU64::new(0)),
            successful_advance_attempts: Arc::new(AtomicU64::new(0)),
        }
    };
}

// Nonce pool metrics for monitoring
lazy_static! {
    static ref NONCE_POOL_TOTAL_GAUGE: opentelemetry::metrics::ObservableGauge<i64> = {
        QTRADE_RELAYER_METER
            .i64_observable_gauge("qtrade.nonce.pool_total")
            .with_description("Total number of nonce accounts in the pool")
            .build()
    };

    static ref NONCE_POOL_AVAILABLE_GAUGE: opentelemetry::metrics::ObservableGauge<i64> = {
        QTRADE_RELAYER_METER
            .i64_observable_gauge("qtrade.nonce.pool_available")
            .with_description("Number of available nonce accounts in the pool")
            .build()
    };

    static ref NONCE_POOL_IN_USE_GAUGE: opentelemetry::metrics::ObservableGauge<i64> = {
        QTRADE_RELAYER_METER
            .i64_observable_gauge("qtrade.nonce.pool_in_use")
            .with_description("Number of nonce accounts currently in use")
            .build()
    };

    static ref NONCE_POOL_NEEDS_INIT_GAUGE: opentelemetry::metrics::ObservableGauge<i64> = {
        QTRADE_RELAYER_METER
            .i64_observable_gauge("qtrade.nonce.pool_needs_init")
            .with_description("Number of nonce accounts needing initialization")
            .build()
    };

    static ref NONCE_POOL_NEEDS_ADVANCE_GAUGE: opentelemetry::metrics::ObservableGauge<i64> = {
        QTRADE_RELAYER_METER
            .i64_observable_gauge("qtrade.nonce.pool_needs_advance")
            .with_description("Number of nonce accounts needing advancement")
            .build()
    };

    static ref NONCE_ACQUISITION_COUNTER: Counter<u64> = {
        QTRADE_RELAYER_METER
            .u64_counter("qtrade.nonce.acquisitions")
            .with_description("Number of nonce accounts acquired from the pool")
            .build()
    };

    static ref NONCE_RELEASE_COUNTER: Counter<u64> = {
        QTRADE_RELAYER_METER
            .u64_counter("qtrade.nonce.releases")
            .with_description("Number of nonce accounts released back to the pool")
            .build()
    };

    static ref NONCE_INITIALIZATION_COUNTER: Counter<u64> = {
        QTRADE_RELAYER_METER
            .u64_counter("qtrade.nonce.initializations")
            .with_description("Number of nonce account initialization attempts")
            .build()
    };

    static ref NONCE_SUCCESSFUL_INITIALIZATION_COUNTER: Counter<u64> = {
        QTRADE_RELAYER_METER
            .u64_counter("qtrade.nonce.successful_initializations")
            .with_description("Number of successful nonce account initializations")
            .build()
    };

    static ref NONCE_ADVANCEMENT_COUNTER: Counter<u64> = {
        QTRADE_RELAYER_METER
            .u64_counter("qtrade.nonce.advancements")
            .with_description("Number of nonce account advancement attempts")
            .build()
    };

    static ref NONCE_SUCCESSFUL_ADVANCEMENT_COUNTER: Counter<u64> = {
        QTRADE_RELAYER_METER
            .u64_counter("qtrade.nonce.successful_advancements")
            .with_description("Number of successful nonce account advancements")
            .build()
    };

    static ref NONCE_ACQUISITION_LATENCY: Histogram<f64> = {
        QTRADE_RELAYER_METER
            .f64_histogram("qtrade.nonce.acquisition_latency")
            .with_description("Time taken to acquire a nonce account from the pool (ms)")
            .build()
    };
}

/// Record the current state of the nonce pool
pub fn record_nonce_pool_state(
    total: usize,
    available: usize,
    in_use: usize,
    needs_init: usize,
    needs_advance: usize,
) {
    // Update atomic counters
    NONCE_METRICS.total_nonce_accounts.store(total as u64, Ordering::Relaxed);
    NONCE_METRICS.available_nonce_accounts.store(available as u64, Ordering::Relaxed);
    NONCE_METRICS.in_use_nonce_accounts.store(in_use as u64, Ordering::Relaxed);
    NONCE_METRICS.needs_init_nonce_accounts.store(needs_init as u64, Ordering::Relaxed);
    NONCE_METRICS.needs_advance_nonce_accounts.store(needs_advance as u64, Ordering::Relaxed);

    // These values will be collected by the OpenTelemetry metrics system
    // and sent to the monitoring backend (like DataDog)
}

/// Record a nonce acquisition event
pub fn record_nonce_acquisition() {
    NONCE_METRICS.total_nonce_acquisitions.fetch_add(1, Ordering::Relaxed);
    NONCE_ACQUISITION_COUNTER.add(1, &[]);
}

/// Record a nonce acquisition with latency information
pub fn record_nonce_acquisition_with_latency(latency_ms: f64) {
    record_nonce_acquisition();
    NONCE_ACQUISITION_LATENCY.record(latency_ms, &[]);
}

/// Record a nonce release event
pub fn record_nonce_release() {
    NONCE_METRICS.total_nonce_releases.fetch_add(1, Ordering::Relaxed);
    NONCE_RELEASE_COUNTER.add(1, &[]);
}

/// Record a nonce initialization attempt
pub fn record_nonce_initialization_attempt(success: bool) {
    NONCE_METRICS.total_init_attempts.fetch_add(1, Ordering::Relaxed);
    NONCE_INITIALIZATION_COUNTER.add(1, &[]);

    if success {
        NONCE_METRICS.successful_init_attempts.fetch_add(1, Ordering::Relaxed);
        NONCE_SUCCESSFUL_INITIALIZATION_COUNTER.add(1, &[]);
    }
}

/// Record a nonce advancement attempt
pub fn record_nonce_advancement_attempt(success: bool) {
    NONCE_METRICS.total_advance_attempts.fetch_add(1, Ordering::Relaxed);
    NONCE_ADVANCEMENT_COUNTER.add(1, &[]);

    if success {
        NONCE_METRICS.successful_advance_attempts.fetch_add(1, Ordering::Relaxed);
        NONCE_SUCCESSFUL_ADVANCEMENT_COUNTER.add(1, &[]);
    }
}
