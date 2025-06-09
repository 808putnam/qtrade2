//! This crate handles building and landing transactions on Solana.
//!
//! The functionality includes:
//! - Constructing transactions
//! - Signing transactions
//! - Submitting transactions to the Solana network
//!
//! This module abstracts the complexities of transaction management and provides
//! a simple interface for building and landing transactions on the Solana blockchain.

use anyhow::Result;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use qtrade_shared_types::ArbitrageResult;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use solana_sdk::pubkey::Pubkey;

// Arbitrage modules
pub mod arbitrage;

// Centralized settings management
pub mod settings;

// DEX-specific modules and traits
pub mod dex;

// For help in naming spans
use crate::constants::QTRADE_RELAYER_TRACER_NAME;
use crate::metrics::arbitrage::{
    record_arbitrage_result_received,
};

pub mod blockhash;
pub mod constants;
pub mod metrics;
pub mod nonce;
pub mod rpc;
pub mod utils;

const RELAYER: &str = "relayer";
const CHECK_INTERVAL: Duration = Duration::from_secs(60);
const MAX_QUEUE_SIZE: usize = 100;

// Global receiver for arbitrage results from router
pub static ARBITRAGE_RECEIVER: Mutex<Option<mpsc::Receiver<ArbitrageResult>>> = Mutex::new(None);

// FIFO queue for storing arbitrage results
pub static ARBITRAGE_QUEUE: Mutex<VecDeque<ArbitrageResult>> = Mutex::new(VecDeque::new());

// Static settings instance to be initialized with run_relayer
static mut RELAYER_SETTINGS: Option<settings::RelayerSettings> = None;

/// Initialize the arbitrage receiver
/// This is called from the router module when it creates the channel
pub fn init_arbitrage_receiver(rx: mpsc::Receiver<ArbitrageResult>) {
    let mut receiver = ARBITRAGE_RECEIVER.lock().unwrap();
    *receiver = Some(rx);
}

/// Add an arbitrage result to the FIFO queue
pub fn enqueue_arbitrage_result(result: ArbitrageResult) -> Result<()> {
    let mut queue = ARBITRAGE_QUEUE.lock().map_err(|e| anyhow::anyhow!("Failed to lock arbitrage queue: {:?}", e))?;

    // If queue is at max capacity, remove the oldest result
    if queue.len() >= MAX_QUEUE_SIZE {
        queue.pop_front();
        warn!("Arbitrage queue reached maximum capacity, dropped oldest result");
    }

    // Add the new result to the queue
    queue.push_back(result);
    debug!("Added arbitrage result to queue, current queue size: {}", queue.len());

    Ok(())
}

/// Get the next arbitrage result from the FIFO queue
pub fn dequeue_arbitrage_result() -> Option<ArbitrageResult> {
    let mut queue = match ARBITRAGE_QUEUE.lock() {
        Ok(queue) => queue,
        Err(e) => {
            error!("Failed to lock arbitrage queue: {:?}", e);
            return None;
        }
    };

    // Remove and return the oldest result from the queue
    let result = queue.pop_front();
    if result.is_some() {
        debug!("Removed arbitrage result from queue, current queue size: {}", queue.len());
    }

    result
}

/// Determine the pool public key from the arbitrage result.
/// This is a placeholder implementation - in a production system, this would retrieve
/// the actual pool pubkey from a registry or derive it from the arbitrage result.
pub fn determine_pool_pubkey(pool_index: usize, _arbitrage_result: &ArbitrageResult) -> Pubkey {
    // In real implementation, this would use a lookup table or other mechanism to get the real pool pubkey
    // For now, we're generating a deterministic pubkey based on the pool index
    let seed = format!("pool_{}", pool_index);
    let hash = solana_sdk::hash::hash(seed.as_bytes());
    Pubkey::new_from_array(hash.to_bytes()[0..32].try_into().unwrap())
}

/// Determine which tokens are being swapped based on the delta values.
/// Returns a tuple of (token_a_index, token_b_index) where:
/// - token_a_index is the index of the token being spent (positive delta)
/// - token_b_index is the index of the token being received (negative delta)
pub fn determine_token_indices(deltas: &[f64]) -> (Option<usize>, Option<usize>) {
    let mut token_a_index = None; // Token we're spending (positive delta)
    let mut token_b_index = None; // Token we're receiving (negative delta)

    for (i, delta) in deltas.iter().enumerate() {
        if *delta > 1e-6 {
            // Positive delta means we're spending this token
            token_a_index = Some(i);
        } else if *delta < -1e-6 {
            // Negative delta means we're receiving this token
            token_b_index = Some(i);
        }
    }

    (token_a_index, token_b_index)
}

/// Executes an arbitrage opportunity by constructing and submitting a transaction
async fn execute_arbitrage(arbitrage_result: &ArbitrageResult) -> Result<()> {
    // Get the global relayer settings
    let settings = get_relayer_settings();
    // Start a new span for the arbitrage execution
    let tracer = global::tracer(QTRADE_RELAYER_TRACER_NAME);
    let span_name = format!("{}::execute_arbitrage", RELAYER);

    tracer.in_span(span_name, |_cx| async move {
        // Check if we're in simulation mode
        let is_simulation = settings.simulate;
        if is_simulation {
            info!("Running in SIMULATION mode - transactions will not be submitted to the network");
        } else {
            info!("Starting execution of arbitrage opportunity");
        }

        // 1. Validate the arbitrage result using the extracted validation function
        if !crate::arbitrage::prepare::validate_arbitrage_result(arbitrage_result)? {
            // If validation fails, we return early
            return Ok(());
        }

        // 2. Construct swap parameters based on the arbitrage result
        info!("Constructing transaction instructions for arbitrage execution");

        let swap_params_result = crate::arbitrage::prepare::construct_swap_parameters(arbitrage_result)?;

        // If no profitable swap operations were found, return early
        let (swap_params_list, _estimated_profit) = match swap_params_result {
            Some((params, profit)) => (params, profit),
            None => return Ok(()),
        };

        // 3. Get an explorer keypair from our tiered wallet system for transaction signing
        let (explorer_pubkey, explorer_keypair) = crate::arbitrage::prepare::acquire_explorer_keypair()?;

        info!("Using explorer keypair with public key: {}", explorer_pubkey);

        // 4. Create the swap instructions using the explorer keypair
        let instructions = crate::arbitrage::prepare::create_swap_instructions(&swap_params_list, &explorer_pubkey)?;

        // 5. Submit the transaction to multiple RPC providers
        info!("Submitting transaction to multiple RPC providers");
        let rpc_results = crate::arbitrage::submit::submit_transaction(
            &instructions,
            &explorer_keypair,
            settings,
            is_simulation
        ).await?;

        // 6. Analyze results and record metrics
        info!("Analyzing transaction submission results");

        // Check if we're in simulation mode
        if is_simulation {
            // We still want to retire the keypair to prevent reuse
            info!("Retiring explorer keypair after simulation: {}", explorer_pubkey);
            if let Err(e) = crate::arbitrage::prepare::return_explorer_keypair_to_pool(&explorer_pubkey, true) {
                error!("Failed to retire explorer key {}: {:?}", explorer_pubkey, e);
            }
            return Ok(());
        }

        // Log detailed results for monitoring and debugging
        let mut successful_submissions = 0;
        for (provider, success, message) in &rpc_results {
            if *success {
                info!("{}: Successfully submitted ({})", provider, message);
                successful_submissions += 1;
            } else {
                warn!("{}: Failed to submit ({})", provider, message);
            }
        }

        if successful_submissions == 0 {
            error!("Transaction submission failed on all RPC providers");
            crate::metrics::arbitrage::record_failed_arbitrage_transaction();
        } else {
            info!("Transaction successfully submitted to {} RPC providers", successful_submissions);
            // Record successful submission metrics would go here
        }

        // Mark the Explorer key as used so it will be retired
        info!("Retiring explorer keypair after transaction use: {}", explorer_pubkey);
        // We retire the key no matter what happened - success or failure
        // This ensures keys aren't reused even if transactions failed to submit
        if let Err(e) = crate::arbitrage::prepare::return_explorer_keypair_to_pool(&explorer_pubkey, true) {
            error!("Failed to retire explorer key {}: {:?}", explorer_pubkey, e);
        }

        info!("Arbitrage execution complete");
        Ok(())
    }).await
}

/// Get the global relayer settings instance
/// Will panic if called before run_relayer
pub fn get_relayer_settings() -> &'static settings::RelayerSettings {
    unsafe {
        RELAYER_SETTINGS.as_ref().expect("Relayer settings not initialized. Must call run_relayer first.")
    }
}

/// Listens to the relayer queue and handles transaction submissions.
///
/// This function performs the following tasks:
/// - Listens to the relayer queue for transaction submissions
/// - Handles submissions to the relayer queue
/// - Calls appropriate DEX module APIs to construct transactions
/// - Calls appropriate RPC module APIs to land transactions
/// - Records metrics for the transactions
pub async fn run_relayer(
    settings: Option<settings::RelayerSettings>,
    cancellation_token: tokio_util::sync::CancellationToken,
) -> Result<()> {
    let tracer = global::tracer(QTRADE_RELAYER_TRACER_NAME);

    // Initialize relayer settings
    if let Some(provided_settings) = settings {
        // Initialize from provided settings
        unsafe {
            RELAYER_SETTINGS = Some(provided_settings);
        }
        info!("Initialized relayer settings from provided settings");
    } else {
        // Initialize from environment variables for backward compatibility
        unsafe {
            RELAYER_SETTINGS = Some(settings::RelayerSettings::from_env());
        }
        info!("Initialized relayer settings from environment variables");
    }

    // Initialize and start the blockhash cache update task
    let blockhash_cache = crate::blockhash::BlockhashCache::instance();
    if let Err(e) = blockhash_cache.start_update_task(rpc::solana::MAINNET_RPC_URL).await {
        error!("Failed to start blockhash cache update task: {:?}", e);
    }

    // Initialize the nonce pool
    info!("Initializing nonce pool from environment variables");
    let nonce_pool = crate::nonce::NoncePool::instance();
    match nonce_pool.init_from_env() {
        Ok(_) => {
            info!("Nonce pool initialized successfully");
            // Start the nonce pool maintenance task
            if let Err(e) = nonce_pool.start_maintenance_task(rpc::solana::MAINNET_RPC_URL).await {
                error!("Failed to start nonce pool maintenance task: {:?}", e);
            } else {
                info!("Nonce pool maintenance task started");
            }
        },
        Err(e) => {
            warn!("Failed to initialize nonce pool: {:?}. Continuing with blockhash only.", e);
        }
    }

    loop  {
        // Check if we've been asked to cancel
        if cancellation_token.is_cancelled() {
            info!("Cancellation token activated, shutting down relayer");
            return Ok(());
        }

        let span_name = format!("{}::run_relayer", RELAYER);

        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            // Listen to relayer queue for transaction submissions
            info!("Listening to relayer queue for transaction submissions...");

            // Step 1: Check the channel for new arbitrage results and add them to the queue
            {
                let mut receiver_guard = ARBITRAGE_RECEIVER.lock().map_err(|e| anyhow::anyhow!("Failed to lock arbitrage receiver: {:?}", e))?;
                if let Some(ref mut rx) = *receiver_guard {
                    // Try to receive all available arbitrage results without blocking
                    loop {
                        match rx.try_recv() {
                            Ok(arbitrage_result) => {
                                info!("Received arbitrage result with status: {}", arbitrage_result.status);

                                // Record metrics for received arbitrage result
                                record_arbitrage_result_received();

                                // Add the result to our FIFO queue
                                if let Err(e) = enqueue_arbitrage_result(arbitrage_result) {
                                    error!("Failed to enqueue arbitrage result: {:?}", e);
                                }
                            },
                            Err(mpsc::error::TryRecvError::Empty) => {
                                // No more arbitrage results in the channel, break the loop
                                debug!("No more arbitrage results in the channel");
                                break;
                            },
                            Err(mpsc::error::TryRecvError::Disconnected) => {
                                // Channel is disconnected, log an error and break the loop
                                error!("Arbitrage channel disconnected");
                                break;
                            }
                        }
                    }
                }
            }

            // Step 2: Process the next arbitrage result from the queue if available
            if let Some(arbitrage_result) = dequeue_arbitrage_result() {
                info!("Processing arbitrage result from queue with status: {}", arbitrage_result.status);

                // Log information about the arbitrage result
                info!("Arbitrage result contains {} delta entries, {} lambda entries, and {} A-matrices",
                    arbitrage_result.deltas.len(),
                    arbitrage_result.lambdas.len(),
                    arbitrage_result.a_matrices.len()
                );

                // Execute the arbitrage opportunity
                if let Err(e) = execute_arbitrage(&arbitrage_result).await {
                    error!("Failed to execute arbitrage: {:?}", e);
                }
            } else {
                debug!("No arbitrage results in the queue to process");
            }

            Ok(())
        }).await;

        // Handle result
        if let Err(e) = result {
            error!("Error running relayer: {:?}", e);
        }

        // Wait for specified duration before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}


