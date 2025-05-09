//! This module handles building and landing transactions on Solana.
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
use qtrade_solver::ArbitrageResult;
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::yield_now;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use serde_json::json;

// For RPC providers
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use crate::rpc::{RpcActions, solana::{Solana, SolanaEndpoint}, helius::Helius, temporal::Temporal};
use crate::rpc::jito::JitoJsonRpcSDK;
use crate::rpc::nextblock::Nextblock;
use crate::rpc::bloxroute::Bloxroute;
use crate::rpc::quicknode::Quicknode;

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;
use crate::metrics::arbitrage::{
    record_arbitrage_result_received,
    record_arbitrage_opportunity_processed,
    record_failed_arbitrage_transaction,
    record_arbitrage_transaction_confirmed,
    record_arbitrage_transaction_failed,
    record_arbitrage_transaction_timeout
};
use crate::metrics::database::record_transaction_taxable_event;

const LANDER: &str = "lander";
const CHECK_INTERVAL: Duration = Duration::from_secs(60);
const MAX_QUEUE_SIZE: usize = 100;

// Global receiver for arbitrage results from solver
pub static ARBITRAGE_RECEIVER: Mutex<Option<mpsc::Receiver<ArbitrageResult>>> = Mutex::new(None);

// FIFO queue for storing arbitrage results
pub static ARBITRAGE_QUEUE: Mutex<VecDeque<ArbitrageResult>> = Mutex::new(VecDeque::new());

/// Initialize the arbitrage receiver
/// This is called from the solver module when it creates the channel
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

/// Executes an arbitrage opportunity by constructing and submitting a transaction
async fn execute_arbitrage(arbitrage_result: &ArbitrageResult) -> Result<()> {
    // Start a new span for the arbitrage execution
    let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
    let span_name = format!("{}::execute_arbitrage", LANDER);

    tracer.in_span(span_name, |_cx| async move {
        info!("Starting execution of arbitrage opportunity");

        // 1. Validate the arbitrage result
        if arbitrage_result.status != "optimal" {
            warn!("Skipping arbitrage execution as status is not optimal: {}", arbitrage_result.status);
            return Ok(());
        }

        // Check for at least one pool with non-zero deltas
        let mut has_profitable_pools = false;
        for deltas in &arbitrage_result.deltas {
            if deltas.iter().any(|&d| d.abs() > 1e-6) {
                has_profitable_pools = true;
                break;
            }
        }

        if !has_profitable_pools {
            info!("No pools with significant deltas found, skipping execution");
            return Ok(());
        }

        // 2. Construct the transaction instructions
        info!("Constructing transaction instructions for arbitrage execution");

        // Record metrics for processing an arbitrage opportunity
        record_arbitrage_opportunity_processed();

        // Initialize values for tracking profit
        let mut estimated_profit = 0.0;
        let mut instructions: Vec<Instruction> = Vec::new();

        // Create a more structured approach to creating swap instructions based on deltas and lambdas
        for (pool_index, (deltas, lambdas)) in arbitrage_result.deltas.iter()
            .zip(arbitrage_result.lambdas.iter())
            .enumerate()
        {
            // Skip pools with no significant deltas
            let has_nonzero_deltas = deltas.iter().any(|&d| d.abs() > 1e-6);
            if !has_nonzero_deltas {
                continue;
            }

            info!("Processing pool {} with deltas: {:?} and lambdas: {:?}", pool_index, deltas, lambdas);

            // Map global token indices to local pool indices using a_matrices
            // This helps us understand which tokens are involved in this pool
            if pool_index < arbitrage_result.a_matrices.len() {
                // We would use a_matrix to map global token indices to local indices
                // For now, we'll just use the deltas directly
                let token_count = deltas.len();

                // Calculate profit for this pool
                let mut pool_profit = 0.0;
                for i in 0..token_count {
                    // Positive delta means we're spending this token, negative lambda means we're receiving
                    if deltas[i] > 0.0 && i < lambdas.len() && lambdas[i] < 0.0 {
                        // Simple profit calculation: what we receive minus what we spend
                        pool_profit += lambdas[i].abs() - deltas[i];
                    }
                }

                if pool_profit > 0.0 {
                    info!("Pool {} estimated profit: {:.6}", pool_index, pool_profit);
                    estimated_profit += pool_profit;

                    // Here we would determine the DEX type and create appropriate instructions
                    // For example, if pool_index corresponds to an Orca pool:

                    // For now, create a dummy instruction that would be replaced with actual DEX instructions
                    let program_id = Pubkey::new_unique(); // In prod: actual DEX program ID

                    // Generate pseudo-meaningful data for the instruction
                    // In production, this would be actual swap data with proper encoding
                    let mut swap_data = Vec::new();
                    swap_data.extend_from_slice(&[0, 1]); // Instruction discriminator

                    // Add delta values as bytes (simplified)
                    for delta in deltas {
                        let value = (delta.abs() * 1_000_000.0) as u64; // Convert to lamports-like value
                        swap_data.extend_from_slice(&value.to_le_bytes());
                    }

                    // Create instruction
                    let swap_instruction = Instruction {
                        program_id,
                        accounts: Vec::new(), // Would include token accounts, pool accounts, etc.
                        data: swap_data,
                    };

                    instructions.push(swap_instruction);
                    info!("Added swap instruction for pool {}", pool_index);
                }
            }
        }

        if instructions.is_empty() {
            info!("No profitable swap instructions generated, skipping execution");
            return Ok(());
        }

        info!("Generated {} swap instructions with estimated profit: {:.6}",
            instructions.len(), estimated_profit);

        // 3. Create a signer keypair
        // In production, this would load from a secure keystore
        let signer = Keypair::new();
        info!("Using keypair with public key: {}", signer.pubkey());

        // 4. Submit the transaction to multiple RPC providers
        info!("Submitting arbitrage transaction to RPC providers");

        // Shuffle RPC providers for unpredictability
        // In production, consider provider performance, reliability, and costs
        let mut rpc_results = Vec::new();

        // -- Solana RPC --
        info!("Attempting submission via Solana RPC");
        let solana_rpc = Solana::new(SolanaEndpoint::Mainnet);
        let mut solana_instructions = instructions.clone();
        match solana_rpc.send_tx(&mut solana_instructions, &signer) {
            Ok(signature) => {
                info!("Transaction submitted successfully via Solana RPC: {}", signature);
                rpc_results.push(("Solana RPC", true, signature));
            },
            Err(e) => {
                warn!("Failed to submit transaction via Solana RPC: {}", e);
                rpc_results.push(("Solana RPC", false, e.to_string()));
            }
        }

        // -- Helius RPC --
        info!("Attempting submission via Helius");
        let helius_rpc = Helius::new();
        let mut helius_instructions = instructions.clone();
        match helius_rpc.send_tx(&mut helius_instructions, &signer) {
            Ok(signature) => {
                info!("Transaction submitted successfully via Helius: {}", signature);
                rpc_results.push(("Helius", true, signature));
            },
            Err(e) => {
                warn!("Failed to submit transaction via Helius: {}", e);
                rpc_results.push(("Helius", false, e.to_string()));
            }
        }

        // -- QuickNode RPC --
        info!("Attempting submission via QuickNode");
        let quicknode_rpc = Quicknode::new();
        let mut quicknode_instructions = instructions.clone();
        match quicknode_rpc.send_tx(&mut quicknode_instructions, &signer) {
            Ok(signature) => {
                info!("Transaction submitted successfully via QuickNode: {}", signature);
                rpc_results.push(("QuickNode", true, signature));
            },
            Err(e) => {
                warn!("Failed to submit transaction via QuickNode: {}", e);
                rpc_results.push(("QuickNode", false, e.to_string()));
            }
        }

        // -- Temporal RPC --
        info!("Attempting submission via Temporal");
        let temporal_rpc = Temporal::new();
        let mut temporal_instructions = instructions.clone();
        match temporal_rpc.send_tx(&mut temporal_instructions, &signer) {
            Ok(signature) => {
                info!("Transaction submitted successfully via Temporal: {}", signature);
                rpc_results.push(("Temporal", true, signature));
            },
            Err(e) => {
                warn!("Failed to submit transaction via Temporal: {}", e);
                rpc_results.push(("Temporal", false, e.to_string()));
            }
        }

        // -- Jito RPC (async) --
        info!("Attempting submission via Jito");
        let jito_sdk = JitoJsonRpcSDK::new("https://mainnet.block-engine.jito.wtf/api/v1/bundles", None);

        // For Jito, we need to create a transaction object first because it uses a different API
        let solana_rpc = Solana::new(SolanaEndpoint::Mainnet);
        let solana_rpc_client = solana_rpc.rpc_client();
        let blockhash = match solana_rpc_client.get_latest_blockhash() {
            Ok(bh) => bh,
            Err(e) => {
                warn!("Failed to get blockhash for Jito submission: {}", e);
                return Ok(());
            }
        };

        let tx = Transaction::new_signed_with_payer(&instructions, Some(&signer.pubkey()), &[&signer], blockhash);
        let serialized_tx = match bincode::serialize(&tx) {
            Ok(data) => {
                // Use the new way to encode base64
                use base64::Engine;
                base64::engine::general_purpose::STANDARD.encode(data)
            },
            Err(e) => {
                warn!("Failed to serialize transaction for Jito: {}", e);
                return Ok(());
            }
        };

        // Prepare Jito transaction parameters
        let params = json!({
            "tx": serialized_tx,
            "skipPreflight": true
        });

        match jito_sdk.send_txn(Some(params), false).await {
            Ok(response) => {
                info!("Transaction submitted successfully via Jito");
                rpc_results.push(("Jito", true, format!("{:?}", response)));
            },
            Err(e) => {
                warn!("Failed to submit transaction via Jito: {}", e);
                rpc_results.push(("Jito", false, e.to_string()));
            }
        }

        // -- Nextblock (async) --
        info!("Attempting submission via Nextblock");
        let nextblock_client = Nextblock::new();
        let mut nextblock_instructions = instructions.clone();
        match nextblock_client.send_tx(&mut nextblock_instructions, &signer).await {
            Ok(signature) => {
                info!("Transaction submitted successfully via Nextblock: {}", signature);
                rpc_results.push(("Nextblock", true, signature));
            },
            Err(e) => {
                warn!("Failed to submit transaction via Nextblock: {}", e);
                rpc_results.push(("Nextblock", false, e.to_string()));
            }
        }

        // -- Bloxroute (async) --
        info!("Attempting submission via Bloxroute");
        let bloxroute = Bloxroute::new();
        let mut bloxroute_instructions = instructions.clone();
        match bloxroute.send_tx(&mut bloxroute_instructions, &signer).await {
            Ok(signature) => {
                info!("Transaction submitted successfully via Bloxroute: {}", signature);
                rpc_results.push(("Bloxroute", true, signature));
            },
            Err(e) => {
                warn!("Failed to submit transaction via Bloxroute: {}", e);
                rpc_results.push(("Bloxroute", false, e.to_string()));
            }
        }

        // Add circuit breaker mechanisms - if multiple providers consistently report the same error type
        // that indicates a transaction would be invalid (e.g., insufficient funds), stop trying
        let sim_error_types = ["InsufficientFundsForFee", "InvalidAccount", "AccountNotFound"];
        let mut fatal_simulation_errors = 0;

        // Group errors by type to detect systemic issues
        let mut error_count_by_type = std::collections::HashMap::new();
        for (_, success, message) in &rpc_results {
            if !success {
                for error_type in &sim_error_types {
                    if message.contains(error_type) {
                        let count = error_count_by_type.entry(error_type.to_string()).or_insert(0);
                        *count += 1;
                        if *count >= 2 {
                            fatal_simulation_errors += 1;
                            break;
                        }
                    }
                }
            }
        }

        if fatal_simulation_errors > 0 {
            warn!("Detected critical simulation errors across multiple providers; transaction likely invalid");
            record_failed_arbitrage_transaction();
            return Ok(());
        }

        // 5. Analyze results and record metrics
        info!("Analyzing transaction submission results");

        // Check if at least one submission was successful
        let successful_submissions = rpc_results.iter().filter(|(_, success, _)| *success).count();

        if successful_submissions > 0 {
            info!("Transaction successfully submitted to {} out of {} RPC providers",
                successful_submissions, rpc_results.len());
                 // Collect transaction signatures from successful submissions to monitor their status
        info!("Monitoring transaction signatures for confirmation on-chain");
        let tx_signatures: Vec<(String, String)> = rpc_results.iter()
            .filter(|(_, success, _)| *success)
            .map(|(provider, _, signature)| (provider.to_string(), signature.to_string()))
            .collect();

        // Set up monitoring timeout
        let monitor_start = std::time::Instant::now();
        let monitor_timeout = std::time::Duration::from_secs(30); // 30 second timeout

        // Use the Solana RPC client for monitoring all signatures
        let solana_rpc = Solana::new(SolanaEndpoint::Mainnet);
        let rpc_client = solana_rpc.rpc_client();

        // Track which signatures confirmed successfully
        let mut pending_signatures = tx_signatures.clone();
        let mut confirmed_signatures = Vec::new();
        let mut failed_signatures = Vec::new();

        // Poll until timeout or all signatures are confirmed/failed
        while !pending_signatures.is_empty() && monitor_start.elapsed() < monitor_timeout {
            // Create a new pending list for the next iteration
            let mut new_pending = Vec::new();

            // Check each pending signature
            for (provider, signature) in &pending_signatures {
                // Parse the signature string to a Signature
                match signature.parse::<solana_sdk::signature::Signature>() {
                    Ok(sig) => {                                match rpc_client.get_signature_status(&sig) {
                                    Ok(Some(status)) => {
                                        if status.is_ok() {
                                            info!("Transaction from {} confirmed: {}", provider, signature);
                                            confirmed_signatures.push((provider.clone(), signature.clone()));

                                            // Record confirmation metrics
                                            record_arbitrage_transaction_confirmed(estimated_profit);

                                            // Record the confirmed transaction as a taxable event
                                            if let Err(e) = record_transaction_taxable_event(provider, signature, estimated_profit) {
                                                error!("Failed to record taxable event: {:?}", e);
                                                // Continue execution even if database recording fails
                                            }
                                        } else {
                                            warn!("Transaction from {} failed with status: {:?}", provider, status);
                                            failed_signatures.push((provider.clone(), signature.clone()));

                                            // Record failure metrics
                                            record_arbitrage_transaction_failed();
                                        }
                            },
                            Ok(None) => {
                                // Still pending
                                new_pending.push((provider.clone(), signature.clone()));
                            },
                            Err(e) => {
                                warn!("Error checking signature status for {}: {}", signature, e);
                                new_pending.push((provider.clone(), signature.clone()));
                            }
                        }
                    },
                    Err(e) => {
                        warn!("Failed to parse signature from {}: {}", provider, e);
                        failed_signatures.push((provider.clone(), signature.clone()));
                    }
                }
            }

            // Update pending signatures for the next iteration
            pending_signatures = new_pending;

            // If there are still pending signatures, wait before polling again
            if !pending_signatures.is_empty() {
                sleep(Duration::from_millis(500)).await;
            }
        }

        // Handle any remaining pending signatures as timeouts
        let timeout_count = pending_signatures.len();
        for (provider, signature) in pending_signatures {
            warn!("Transaction from {} timed out waiting for confirmation: {}", provider, signature);
            failed_signatures.push((provider, signature.clone()));
            record_arbitrage_transaction_timeout();
        }

            // Report on confirmation status
            info!(
                "Transaction monitoring complete: {} confirmed, {} failed, {} still pending after timeout",
                confirmed_signatures.len(),
                failed_signatures.len() - timeout_count,
                timeout_count
            );

            // Record metrics based on confirmation status
            if !confirmed_signatures.is_empty() {
                info!("At least one transaction confirmed on-chain");
                // Note: We record transaction confirmations individually during monitoring
                // and already call record_successful_arbitrage_transaction there
                info!("Successful arbitrage transaction with estimated profit: {:.6} USD", estimated_profit);

                // Record confirmation rate metrics
                let confirmation_rate = confirmed_signatures.len() as f64 / tx_signatures.len() as f64;
                use crate::metrics::arbitrage::record_arbitrage_transaction_confirmation_rate;
                record_arbitrage_transaction_confirmation_rate(confirmation_rate);
            } else {
                warn!("No transactions were confirmed on-chain within the timeout period");
                // We've already recorded individual failures during monitoring
            }
        } else {
            error!("Transaction submission failed on all {} RPC providers", rpc_results.len());
            record_failed_arbitrage_transaction();
        }

        // Log detailed results for monitoring and debugging
        for (provider, success, message) in &rpc_results {
            if *success {
                info!("{}: Successfully submitted ({})", provider, message);
            } else {
                warn!("{}: Failed to submit ({})", provider, message);
            }
        }

        // We've already logged the confirmation status and recorded metrics above

        info!("Arbitrage execution complete");
        Ok(())
    }).await
}

/// Listens to the lander queue and handles transaction submissions.
///
/// This function performs the following tasks:
/// - Listens to the lander queue for transaction submissions
/// - Handles submissions to the lander queue
/// - Calls appropriate DEX module APIs to construct transactions
/// - Calls appropriate RPC module APIs to land transactions
/// - Records metrics for the transactions
pub async fn run_lander() -> Result<()> {
    let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);

    loop  {
        let span_name = format!("{}::run_lander", LANDER);

        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            // Listen to lander queue for transaction submissions
            info!("Listening to lander queue for transaction submissions...");

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

            // Simulate an async operation with yield_now
            yield_now().await;

            Ok(())
        }).await;

        // result
        if let Err(e) = result {
            error!("Error running lander: {:?}", e);
        }

        // Wait for specified duration before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}
