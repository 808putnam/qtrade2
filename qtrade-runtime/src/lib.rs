use anyhow::Result;
use once_cell::sync::Lazy;
use opentelemetry::{
    global,
    InstrumentationScope,
    metrics::Meter,
    trace::Tracer};
use rustls::crypto::ring::default_provider;
use rustls::crypto::CryptoProvider;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tokio::try_join;

pub mod settings;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Blockchain {
    Solana,
    Sui,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Router {
    Cvxpy,
    OpenQAOA,
    CFMMRouter,
}

/// Represents available DEX platforms that the system can interact with.
///
/// This enum allows specifying which DEX platforms should be active
/// for indexing, routing, and transaction submission. By default, all DEXes
/// are active, but users can restrict which ones are used via
/// command-line arguments, environment variables, or configuration files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Dex {
    /// Orca DEX platform
    Orca,
    /// Raydium DEX platform (general)
    Raydium,
    /// Raydium Constant Product Market Maker (CPMM)
    RaydiumCpmm,
    /// Raydium Concentrated Liquidity Market Maker (CLMM)
    RaydiumClmm,
}

impl Dex {
    pub fn as_str(&self) -> &'static str {
        match self {
            Dex::Orca => "orca",
            Dex::Raydium => "raydium",
            Dex::RaydiumCpmm => "raydium-cpmm",
            Dex::RaydiumClmm => "raydium-clmm",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "orca" => Some(Dex::Orca),
            "raydium" => Some(Dex::Raydium),
            "raydium-cpmm" => Some(Dex::RaydiumCpmm),
            "raydium_cpmm" => Some(Dex::RaydiumCpmm),
            "raydium-clmm" => Some(Dex::RaydiumClmm),
            "raydium_clmm" => Some(Dex::RaydiumClmm),
            _ => None,
        }
    }
}

/// Represents available RPC providers for transaction submissions.
///
/// This enum allows the system to specify which RPC providers should
/// be active for submitting transactions. By default, all providers
/// are active, but users can restrict which ones are used via
/// command-line arguments, environment variables, or configuration files.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RpcProvider {
    /// Bloxroute RPC provider
    Bloxroute,
    /// Helius RPC provider
    Helius,
    /// Jito RPC provider
    Jito,
    /// Nextblock RPC provider
    Nextblock,
    /// Quicknode RPC provider
    Quicknode,
    /// Solana RPC provider
    Solana,
    /// Temporal RPC provider
    Temporal,
    /// Triton RPC provider
    Triton,
}

impl RpcProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            RpcProvider::Bloxroute => "bloxroute",
            RpcProvider::Helius => "helius",
            RpcProvider::Jito => "jito",
            RpcProvider::Nextblock => "nextblock",
            RpcProvider::Quicknode => "quicknode",
            RpcProvider::Solana => "solana",
            RpcProvider::Temporal => "temporal",
            RpcProvider::Triton => "triton",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bloxroute" => Some(RpcProvider::Bloxroute),
            "helius" => Some(RpcProvider::Helius),
            "jito" => Some(RpcProvider::Jito),
            "nextblock" => Some(RpcProvider::Nextblock),
            "quicknode" => Some(RpcProvider::Quicknode),
            "solana" => Some(RpcProvider::Solana),
            "temporal" => Some(RpcProvider::Temporal),
            "triton" => Some(RpcProvider::Triton),
            _ => None,
        }
    }
}

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

pub async fn run_qtrade(
    flags: settings::Flags,
    cancellation_token: CancellationToken
) -> Result<()> {
    // Load settings using provided flags
    let settings = settings::Settings::load(flags)?;

    // Validate the settings
    settings.validate()?;

    let tracer = global::tracer_with_scope(QTRADE_RUNTIME_SCOPE.clone());
    let span_name = format!("{}::run_qtrade", QTRADE_RUNTIME);

    let result = tracer.in_span(span_name, |_cx| async move {
        tokio::select! {
            _ = cancellation_token.cancelled() => {
                // unsubscribe from geyser
                tracing::info!("Shutting down due to cancellation signal");
            }
            result = async {
                // Use the blockchain and router from settings
                run_qtrade_inner(settings, cancellation_token.clone()).await
            } => {
                result?;
            }
        }

        Ok(())
    }).await;

    result
}

async fn run_qtrade_inner(
    settings: settings::Settings,
    cancellation_token: tokio_util::sync::CancellationToken
) -> Result<()> {
    let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
    let span_name = format!("{}::run_qtrade_inner", QTRADE_RUNTIME);

    // Log the blockchain and router being used
    tracing::info!("Running qtrade with blockchain: {:?} and router: {:?}",
                   settings.blockchain, settings.router);

    let result = tracer.in_span(span_name, |_cx| async move {
        if CryptoProvider::get_default().is_none() {
            default_provider()
                .install_default()
                .map_err(|e| anyhow::anyhow!("Failed to install default CryptoProvider: {:?}", e))?;
        }

        // Initialize database connection for transaction recording
        if let Err(e) = qtrade_relayer::metrics::database::init_database() {
            // Log the error but continue execution - we'll fall back to logging
            tracing::warn!("Failed to initialize database connection: {:?}. Will use log-based recording fallback.", e);
        }

        // Create wallet settings from runtime settings
        let wallet_settings = qtrade_wallets::WalletSettings {
            single_wallet: settings.single_wallet,
            single_wallet_private_key: settings.single_wallet_private_key.clone(),
        };
        // Pass wallet settings to the wallet system
        let wallets_future = qtrade_wallets::run_wallets(wallet_settings);

        // Convert runtime settings to relayer settings
        let relayer_settings = qtrade_relayer::settings::RelayerSettings::new_with_rpcs(
            settings.bloxroute_api_key.clone(),
            settings.helius_api_key.clone(),
            settings.nextblock_api_key.clone(),
            settings.quicknode_api_key.clone(),
            settings.temporal_api_key.clone(),
            settings.active_rpcs.iter().map(|rpc| rpc.as_str().to_string()).collect(),
            settings.simulate,
        );
        // Create a clone of the cancellation token for relayer
        let relayer_token = cancellation_token.clone();
        let relayer_future = qtrade_relayer::run_relayer(Some(relayer_settings), relayer_token);

        // Using the PoolCache from the runtime to pass to the router
        let router_future = qtrade_router::run_router(Arc::clone(&qtrade_indexer::POOL_CACHE));

        // Create indexer settings from runtime settings
        let indexer_settings = qtrade_indexer::settings::IndexerSettings::new_with_config(
            settings.active_dexes.iter().map(|dex| dex.as_str().to_string()).collect(),
            settings.vixon_config_path.clone()
        );

        // Pass indexer settings to the streamer
        let indexer_future = qtrade_indexer::streamer::run_streamer(
            Some(indexer_settings)
        );

        // Run async run_xxx functions concurrently
        try_join!(
            relayer_future,
            router_future,
            indexer_future,
            wallets_future
        )?;

        Ok(())
    }).await;

    result
}