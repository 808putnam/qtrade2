//! This module contains setup and maintenance for a defined set of wallets.
//!
//! The functionality includes:
//! - Initializing and configuring wallets
//! - Managing the set of wallets
//! - Providing utilities for wallet operations
//!
//! This module abstracts the complexities of wallet management and provides
//! a simple interface for interacting with multiple wallets.

use anyhow::Result;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use serde_json::from_slice;
use solana_sdk::signer::keypair::Keypair;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;

const WALLETS: &str = "wallets";
const CHECK_INTERVAL: Duration = Duration::from_secs(60);


/// Reads a `Keypair` from an environment variable
pub fn read_keypair_from_env(var_name: &str) -> Result<Keypair> {
    let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
    let span_name = format!("{}::read_keypair_from_env", WALLETS);

    let result = tracer.in_span(span_name, move |_cx| {
        let keypair_json = env::var(var_name)?;
        let keypair_bytes = keypair_json.as_bytes();
        let keypair_vec: Vec<u8> = from_slice(keypair_bytes)?;
        let keypair = Keypair::from_bytes(&keypair_vec)?;
        Ok(keypair)
    });

    result
}

/// Periodically manages wallet balances based on a timer.
///
/// This function sets up a timer to periodically check and manage
/// the balances of the defined set of wallets. It ensures that
/// wallet balances are maintained and any necessary actions are taken.
pub async fn run_wallets(wallet_config_path: &str) -> Result<()> {
    let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);

    loop {
        let span_name = format!("{}::run_wallets", WALLETS);

        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            // Use wallet_config_path to configure wallets
            // Setup defined set of wallets based on config file
            // See raydium and vixen for examples of config file setups

            let pool_config =
                raydium_amm_v3_client::instructions::load_cfg(&wallet_config_path.to_string())
                    .map_err(|e| anyhow::anyhow!("Failed to load wallet config: {:?}", e))?;
            /*
            let payer =
                raydium_amm_v3_client::instructions::read_keypair_file(&pool_config.payer_path)
                    .map_err(|e| anyhow::anyhow!("Failed to load pool_config.payer_path: {:?}", e))?;
            */
            let payer =
                read_keypair_from_env(&pool_config.payer_path)
                    .map_err(|e| anyhow::anyhow!("Failed to load pool_config.payer_path: {:?}", e))?;

            // Setup timer for periodic wallet management
            info!("Setting up timer for periodic wallet management...");

            // Periodically check and manage wallet balances
            info!("Checking and managing wallet balances...");

            Ok(())
        }).await;

        // result
        if let Err(e) = result {
            error!("Error running wallet management: {:?}", e);
        }

        // Wait for specified duration before running the check again
        sleep(CHECK_INTERVAL).await;
    }

}
