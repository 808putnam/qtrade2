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

pub mod secrets;

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
        if let Err(e) = qtrade_lander::metrics::database::init_database() {
            // Log the error but continue execution - we'll fall back to logging
            tracing::warn!("Failed to initialize database connection: {:?}. Will use log-based recording fallback.", e);
        }

        let wallets_future = qtrade_wallets::run_wallets(wallet_config_path);
        let lander_future = qtrade_lander::run_lander();
        // Using the PoolCache from the runtime to pass to the solver
        let solver_future = qtrade_solver::run_solver(Arc::clone(&qtrade_streamer::POOL_CACHE));

        let streamer_future = qtrade_streamer::streamer::run_streamer(vixon_config_path);

        // Run async run_xxx functions concurrently
        try_join!(
            lander_future,
            solver_future,
            streamer_future,
            wallets_future
        )?;

        Ok(())
    }).await;

    result
}