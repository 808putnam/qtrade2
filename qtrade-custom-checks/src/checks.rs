use anyhow::Result;
use once_cell::sync::Lazy;
use opentelemetry::{
    global,
    InstrumentationScope,
    metrics::Meter,
    trace::Tracer};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use tokio_util::sync::CancellationToken;
use tokio::try_join;

pub mod check_geyser;
pub mod check_geyser_version;
pub mod check_solana;
pub mod check_solana_fw;
pub mod check_solana_last_known_block;
pub mod check_solana_port_conn;
pub mod check_solana_port_conn_count;
pub mod check_solana_version_distribution;
pub mod check_solana_version;
pub mod check_systemd_healthchecks;

// Our one global named tracer we will use throughout the runtime
const QTRADE_CUSTOM_CHECKS_TRACER_NAME: &str = "qtrade_custom_checks";
const QTRADE_CUSTOM_CHECKS: &str = "qtrade_custom_checks";

pub static QTRADE_CUSTOM_CHECKS_SCOPE: Lazy<InstrumentationScope> = Lazy::new(|| {
    InstrumentationScope::builder(QTRADE_CUSTOM_CHECKS_TRACER_NAME)
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url("https://opentelemetry.io/schemas/1.17.0")
        .build()
});

pub static QTRADE_CUSTOM_CHECKS_METER: Lazy<Meter> = Lazy::new(|| {
    global::meter(QTRADE_CUSTOM_CHECKS)
});

pub async fn run_custom_checks(
    cancellation_token: CancellationToken
) -> Result<()> {
    let tracer = global::tracer_with_scope(QTRADE_CUSTOM_CHECKS_SCOPE.clone());
    let span_name = format!("{}::run_custom_checks", QTRADE_CUSTOM_CHECKS);

    let result = tracer.in_span(span_name, |_cx| async move {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                // unsubscribe from geyser
            }
            result = async {
                run_custom_checks_inner().await
            } => {
                result?;
            }
        }

        Ok(())
    }).await;

    result
}

async fn run_custom_checks_inner() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);
    let span_name = format!("{}::run_custom_checks_inner", QTRADE_CUSTOM_CHECKS);

    let result = tracer.in_span(span_name, |_cx| async move {
        if CryptoProvider::get_default().is_none() {
            default_provider()
                .install_default()
                .map_err(|e| anyhow::anyhow!("Failed to install default CryptoProvider: {:?}", e))?;
        }

        let check_geyser_future = check_geyser::run_check_geyser();
        let check_geyser_version_future = check_geyser_version::run_check_geyser_version();
        let check_solana_future = check_solana::run_check_solana();
        let check_solana_fw_future = check_solana_fw::run_check_solana_fw();
        let check_solana_last_known_block_future = check_solana_last_known_block::run_check_solana_last_known_block();
        let check_solana_port_conn_future = check_solana_port_conn::run_check_solana_port_conn();
        let check_solana_port_conn_count_future = check_solana_port_conn_count::run_check_solana_port_conn_count();
        let check_solana_version_distribution_future = check_solana_version_distribution::run_solana_version_distribution();
        let check_solana_version_future = check_solana_version::run_check_solana_version();
        let check_systemd_healthchecks_future = check_systemd_healthchecks::run_check_systemd_healthchecks();

        // Run async run_xxx functions concurrently
        try_join!(
            check_geyser_future,
            check_geyser_version_future,
            check_solana_future,
            check_solana_fw_future,
            check_solana_last_known_block_future,
            check_solana_port_conn_future,
            check_solana_port_conn_count_future,
            check_solana_version_distribution_future,
            check_solana_version_future,
            check_systemd_healthchecks_future
        )?;

        Ok(())
    }).await;

    result
}