use anyhow::Result;
use once_cell::sync::Lazy;
use opentelemetry::{
    global,
    InstrumentationScope,
    metrics::Meter,
    trace::Tracer};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tokio::try_join;

use crate::streamer::MintCache;
use crate::streamer::PoolCache;
use crate::streamer::PoolConfigCache;

pub mod acctsdb;
pub mod dex;
pub mod lander;
pub mod metrics;
pub mod oracle;
pub mod parser;
pub mod rpc;
pub mod secrets;
pub mod solana;
pub mod solver;
pub mod stats;
pub mod streamer;
pub mod wallets;

// Our one global named tracer we will use throughout the runtime
const QTRADE_RUNTIME_TRACER_NAME: &str = "qtrade_runtime";
const QTRADE_RUNTIME: &str = "qtrade_runtime";

pub static QTRADE_RUNTIME_SCOPE: Lazy<InstrumentationScope> = Lazy::new(|| {
    InstrumentationScope::builder(QTRADE_RUNTIME_TRACER_NAME)
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url("https://opentelemetry.io/schemas/1.17.0")
        .build()
});

pub static QTRADE_RUNTIME_METER: Lazy<Meter> = Lazy::new(|| {
    global::meter(QTRADE_RUNTIME)
});

/// `xxx_CACHE` are global static variables of type `Lazy<Arc<ConcreteCache>>`, which are initialized using the
/// `once_cell` crate to ensure they are created only once and shared across the entire crate.
///
/// `Arc` stands for "Atomic Reference Counted" and is a thread-safe reference-counting pointer.
/// It allows multiple ownership of the same data by keeping track of the number of references to the data.
///
/// Use `let pool_cache = Arc::clone(&POOL_CACHE)` to create a new `Arc` pointer that points to the same `Cache` instance
/// as the global `POOL_CACHE`.
/// This does not create a new instance but rather increments the reference count,
/// allowing safe shared access to the same instance.
///
/// The local variable `pool_cache` now holds an `Arc` pointer to the same `Cache` instance,
/// which can be passed to other components or tasks that need access to the cache.
///
/// This approach ensures that the cache instances are shared and accessible in a thread-safe
/// manner across different parts of the application.
pub static MINT_CACHE: Lazy<Arc<MintCache>> = Lazy::new(|| {
    Arc::new(MintCache::new())
});
pub static POOL_CACHE: Lazy<Arc<PoolCache>> = Lazy::new(|| {
    Arc::new(PoolCache::new())
});
pub static POOL_CONFIG_CACHE: Lazy<Arc<PoolConfigCache>> = Lazy::new(|| {
    Arc::new(PoolConfigCache::new())
});

pub enum Blockchain {
    Solana,
    Sui,
}

pub enum Solver {
    Cvxpy,
    OpenQAOA,
    CFMMRouter,
}

pub async fn run_qtrade(
    wallet_config_path: &str,
    vixon_config_path: &str,
    blockchain: Blockchain,
    solver: Solver,
    cancellation_token: CancellationToken
) -> Result<()> {

    let tracer = global::tracer_with_scope(QTRADE_RUNTIME_SCOPE.clone());
    let span_name = format!("{}::run_qtrade", QTRADE_RUNTIME);

    let result = tracer.in_span(span_name, |_cx| async move {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                // unsubscribe from geyser
            }
            result = async {
                run_qtrade_inner(
                    wallet_config_path,
                    vixon_config_path,
                    blockchain,
                    solver).await
            } => {
                result?;
            }
        }

        Ok(())
    }).await;

    result
}

async fn run_qtrade_inner(
    wallet_config_path: &str,
    vixon_config_path: &str,
    blockchain: Blockchain,
    solver: Solver
) -> Result<()> {
    let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
    let span_name = format!("{}::run_qtrade_inner", QTRADE_RUNTIME);

    let result = tracer.in_span(span_name, |_cx| async move {
        if CryptoProvider::get_default().is_none() {
            default_provider()
                .install_default()
                .map_err(|e| anyhow::anyhow!("Failed to install default CryptoProvider: {:?}", e))?;
        }

        // Initialize database connection for transaction recording
        if let Err(e) = metrics::database::init_database() {
            // Log the error but continue execution - we'll fall back to logging
            tracing::warn!("Failed to initialize database connection: {:?}. Will use log-based recording fallback.", e);
        }

        let stats_future = stats::run_stats();

        // 1. Now get our defined set of wallets setup
        let wallets_future = wallets::run_wallets(wallet_config_path);

        // 2. Next, we get an initial set of pool reserves
        let acctsdb_future = acctsdb::run_acctsdb();

        // 3. Now we get the oracle setup which will be used by solver
        // TODO: Are we migrating to USDC pools for our oracles?
        // oracle::run_oracle();

        // 4. Next we get the lander reader to land transactions
        //    from the lander queue - populated by streamer
        let lander_future = lander::run_lander();

        // 5. Now we get the solver setup - initialized from acctsdb and
        //    ready to receive inputs from the streamer
        let solver_future = solver::run_solver();

        // 6. Finally, we get the streamer setup to stream data
        //    and submit to solver
        // TODO: Enable when bare metal geyser is available and uncomment in try_join! below as well
        // let streamer_future = streamer::run_streamer(vixon_config_path);

        // Run async run_xxx functions concurrently
        try_join!(
            stats_future,
            acctsdb_future,
            lander_future,
            solver_future,
            // streamer_future,
            wallets_future
        )?;

        Ok(())
    }).await;

    result
}