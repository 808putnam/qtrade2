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
use tokio::task::yield_now;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use serde_json::json;

// DEX-specific modules and traits
pub mod dex;

// For RPC providers
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use crate::rpc::{RpcActions, solana::{Solana, SolanaEndpoint}, helius::Helius, temporal::Temporal};
use crate::rpc::jito::JitoJsonRpcSDK;
use crate::rpc::nextblock::Nextblock;
use crate::rpc::bloxroute::Bloxroute;
use crate::rpc::quicknode::Quicknode;

// For tiered wallet system
use qtrade_wallets::{get_explorer_keypair, return_explorer_keypair};

// For help in naming spans
use crate::constants::QTRADE_LANDER_TRACER_NAME;
use crate::metrics::arbitrage::{
    record_arbitrage_result_received,
    record_arbitrage_opportunity_processed,
    record_failed_arbitrage_transaction,
    record_arbitrage_transaction_confirmed,
    record_arbitrage_transaction_failed,
    record_arbitrage_transaction_timeout,
    record_arbitrage_transaction_confirmation_rate,
};
use crate::metrics::database::record_transaction_taxable_event;

pub mod blockhash;
pub mod constants;
pub mod metrics;
pub mod nonce;
pub mod rpc;
pub mod secrets;
pub mod utils;

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

/// Determine the pool public key from the arbitrage result.
/// This is a placeholder implementation - in a production system, this would retrieve
/// the actual pool pubkey from a registry or derive it from the arbitrage result.
fn determine_pool_pubkey(pool_index: usize, _arbitrage_result: &ArbitrageResult) -> Pubkey {
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
fn determine_token_indices(deltas: &[f64]) -> (Option<usize>, Option<usize>) {
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
    // Start a new span for the arbitrage execution
    let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
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

        // Define the SwapParams struct outside of any other structs to avoid name conflicts
        #[derive(Debug)]
        struct ArbitrageSwapParams {
            pool_index: usize,
            dex_type: dex::DexType,
            pool_pubkey: Pubkey,
            token_a_wallet: Pubkey,
            token_a_mint: Pubkey,
            token_a_vault: Pubkey,
            token_b_wallet: Pubkey,
            token_b_mint: Pubkey,
            token_b_vault: Pubkey,
            amount_in: u64,
            min_amount_out: u64,
        }

        // We'll use this to store swap parameters to be processed after getting the explorer keypair
        let mut swap_params_list: Option<Vec<ArbitrageSwapParams>> = None;

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

                    // Store the necessary parameters for this swap operation
                    // We'll create the actual instruction after obtaining the explorer keypair

                    // Determine the DEX type based on the pool
                    let pool_pubkey = determine_pool_pubkey(pool_index, &arbitrage_result);
                    let dex_type = dex::determine_dex_type(&pool_pubkey);
                    info!("Determined DEX type: {:?} for pool {}", dex_type, pool_index);

                    // Determine token parameters based on deltas
                    // Deltas > 0 means we're spending this token, < 0 means we're receiving
                    let (token_a_index, token_b_index) = determine_token_indices(deltas);

                    if token_a_index.is_none() || token_b_index.is_none() {
                        warn!("Could not determine token indices for pool {}. Skipping.", pool_index);
                        continue;
                    }

                    let token_a_index = token_a_index.unwrap();
                    let token_b_index = token_b_index.unwrap();

                    // In a real implementation, we would retrieve these from our token registry
                    // For now, creating placeholders
                    let token_a_mint = Pubkey::new_unique(); // Token A mint
                    let token_b_mint = Pubkey::new_unique(); // Token B mint

                    let token_a_wallet = Pubkey::new_unique(); // User's token A account
                    let token_b_wallet = Pubkey::new_unique(); // User's token B account

                    let token_a_vault = Pubkey::new_unique(); // Pool's token A vault
                    let token_b_vault = Pubkey::new_unique(); // Pool's token B vault

                    // Calculate the swap amounts
                    let amount_in = (deltas[token_a_index].abs() * 1_000_000.0) as u64;
                    let min_amount_out = (deltas[token_b_index].abs() * 0.99 * 1_000_000.0) as u64; // 1% slippage

                    // Create and store the swap parameters
                    let swap_params = ArbitrageSwapParams {
                        pool_index,
                        dex_type,
                        pool_pubkey,
                        token_a_wallet,
                        token_a_mint,
                        token_a_vault,
                        token_b_wallet,
                        token_b_mint,
                        token_b_vault,
                        amount_in,
                        min_amount_out,
                    };

                    // We'll store all the swap parameters in a Vec for later processing
                    if swap_params_list.is_none() {
                        swap_params_list = Some(Vec::new());
                    }

                    swap_params_list.as_mut().unwrap().push(swap_params);
                    info!("Prepared swap parameters for pool {}", pool_index);
                }
            }
        }

        if swap_params_list.is_none() || swap_params_list.as_ref().unwrap().is_empty() {
            info!("No profitable swap operations prepared, skipping execution");
            return Ok(());
        }

        info!("Prepared {} swap operations with estimated profit: {:.6}",
            swap_params_list.as_ref().unwrap().len(), estimated_profit);

        // 3. Get an explorer keypair from our tiered wallet system for transaction signing
        let (explorer_pubkey, explorer_keypair) = match get_explorer_keypair() {
            Some(keypair) => keypair,
            None => {
                error!("No explorer keypairs available for transaction signing");
                record_failed_arbitrage_transaction();
                return Err(anyhow::anyhow!("No explorer keypairs available for transaction signing"));
            }
        };

        info!("Using explorer keypair with public key: {}", explorer_pubkey);

        // Now create the actual instructions using the explorer keypair
        info!("Creating swap instructions with explorer pubkey: {}", explorer_pubkey);
        let mut instructions: Vec<Instruction> = Vec::new();

        for params in swap_params_list.unwrap() {
            // Create the appropriate DEX swap implementation
            let dex_swap = dex::create_dex_swap(params.dex_type);

            // Create the swap instruction with the explorer keypair as the authority
            let swap_instruction = dex_swap.create_swap_instruction(
                &params.pool_pubkey,
                &explorer_pubkey, // Now we have the explorer pubkey to use as token authority
                &params.token_a_wallet,
                &params.token_a_mint,
                &params.token_a_vault,
                &params.token_b_wallet,
                &params.token_b_mint,
                &params.token_b_vault,
                params.amount_in,
                params.min_amount_out,
                true, // Direction A to B
                true, // Exact input
            ).map_err(|e| {
                warn!("Failed to create swap instruction for pool {}: {}", params.pool_index, e);
                anyhow::anyhow!("Failed to create swap instruction")
            })?;

            instructions.push(swap_instruction);
            info!("Added swap instruction for pool {}", params.pool_index);
        }

        // 4. Submit the transaction to multiple RPC providers
        info!("Submitting arbitrage transaction to RPC providers");

        // Shuffle RPC providers for unpredictability
        // In production, consider provider performance, reliability, and costs
        let mut rpc_results = Vec::new();

        // -- Solana RPC --
        info!("Attempting submission via Solana RPC");
        let solana_rpc = Solana::new(SolanaEndpoint::Mainnet);
        let solana_rpc_client = solana_rpc.rpc_client();
        let nonce_pool = crate::nonce::NoncePool::instance();

        // Use a direct approach - call directly with provider information
        let mut solana_instructions = instructions.clone();

        // Try to use nonce if available
        let mut solana_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Solana RPC", nonce_pubkey, nonce_hash);

                        let nonce_info = rpc::NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.clone();
                        match solana_rpc.send_nonce_tx(&mut nonce_instructions, &explorer_keypair, nonce_info) {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via Solana RPC with nonce: {}", signature);
                                rpc_results.push(("Solana RPC (nonce)", true, signature));
                                solana_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via Solana RPC with nonce: {}", e);
                                rpc_results.push(("Solana RPC (nonce)", false, e.to_string()));
                            }
                        }

                        // Release the nonce account back to the pool
                        if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                            warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get nonce authority: {}, falling back to blockhash", e);
                    }
                }
            },
            Err(e) => {
                warn!("No nonce accounts available for Solana RPC: {}, using blockhash instead", e);
            }
        }

        // If nonce wasn't used, fall back to blockhash
        if !solana_used_nonce {
            match solana_rpc.send_tx(&mut solana_instructions, &explorer_keypair) {
                Ok(signature) => {
                    info!("Transaction submitted successfully via Solana RPC: {}", signature);
                    rpc_results.push(("Solana RPC", true, signature));
                },
                Err(e) => {
                    warn!("Failed to submit transaction via Solana RPC: {}", e);
                    rpc_results.push(("Solana RPC", false, e.to_string()));
                }
            }
        }

        // -- Helius RPC --
        info!("Attempting submission via Helius");
        let helius_rpc = Helius::new();

        // Use a direct approach - call directly with provider information
        let mut helius_instructions = instructions.clone();

        // Try to use nonce if available
        let mut helius_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Helius", nonce_pubkey, nonce_hash);

                        let nonce_info = rpc::NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.clone();
                        match helius_rpc.send_nonce_tx(&mut nonce_instructions, &explorer_keypair, nonce_info) {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via Helius with nonce: {}", signature);
                                rpc_results.push(("Helius (nonce)", true, signature));
                                helius_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via Helius with nonce: {}", e);
                                rpc_results.push(("Helius (nonce)", false, e.to_string()));
                            }
                        }

                        // Release the nonce account back to the pool
                        if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                            warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get nonce authority: {}, falling back to blockhash", e);
                    }
                }
            },
            Err(e) => {
                warn!("No nonce accounts available for Helius: {}, using blockhash instead", e);
            }
        }

        // If nonce wasn't used, fall back to blockhash
        if !helius_used_nonce {
            match helius_rpc.send_tx(&mut helius_instructions, &explorer_keypair) {
                Ok(signature) => {
                    info!("Transaction submitted successfully via Helius: {}", signature);
                    rpc_results.push(("Helius", true, signature));
                },
                Err(e) => {
                    warn!("Failed to submit transaction via Helius: {}", e);
                    rpc_results.push(("Helius", false, e.to_string()));
                }
            }
        }

        // -- QuickNode RPC --
        info!("Attempting submission via QuickNode");
        let quicknode_rpc = Quicknode::new();

        // Use a direct approach - call directly with provider information
        let mut quicknode_instructions = instructions.clone();

        // Try to use nonce if available
        let mut quicknode_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for QuickNode", nonce_pubkey, nonce_hash);

                        let nonce_info = rpc::NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.clone();
                        match quicknode_rpc.send_nonce_tx(&mut nonce_instructions, &explorer_keypair, nonce_info) {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via QuickNode with nonce: {}", signature);
                                rpc_results.push(("QuickNode (nonce)", true, signature));
                                quicknode_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via QuickNode with nonce: {}", e);
                                rpc_results.push(("QuickNode (nonce)", false, e.to_string()));
                            }
                        }

                        // Release the nonce account back to the pool
                        if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                            warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get nonce authority: {}, falling back to blockhash", e);
                    }
                }
            },
            Err(e) => {
                warn!("No nonce accounts available for QuickNode: {}, using blockhash instead", e);
            }
        }

        // If nonce wasn't used, fall back to blockhash
        if !quicknode_used_nonce {
            match quicknode_rpc.send_tx(&mut quicknode_instructions, &explorer_keypair) {
                Ok(signature) => {
                    info!("Transaction submitted successfully via QuickNode: {}", signature);
                    rpc_results.push(("QuickNode", true, signature));
                },
                Err(e) => {
                    warn!("Failed to submit transaction via QuickNode: {}", e);
                    rpc_results.push(("QuickNode", false, e.to_string()));
                }
            }
        }

        // -- Temporal RPC --
        info!("Attempting submission via Temporal");
        let temporal_rpc = Temporal::new();

        // Use a direct approach - call directly with provider information
        let mut temporal_instructions = instructions.clone();

        // Try to use nonce if available
        let mut temporal_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Temporal", nonce_pubkey, nonce_hash);

                        let nonce_info = rpc::NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.clone();
                        match temporal_rpc.send_nonce_tx(&mut nonce_instructions, &explorer_keypair, nonce_info) {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via Temporal with nonce: {}", signature);
                                rpc_results.push(("Temporal (nonce)", true, signature));
                                temporal_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via Temporal with nonce: {}", e);
                                rpc_results.push(("Temporal (nonce)", false, e.to_string()));
                            }
                        }

                        // Release the nonce account back to the pool
                        if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                            warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get nonce authority: {}, falling back to blockhash", e);
                    }
                }
            },
            Err(e) => {
                warn!("No nonce accounts available for Temporal: {}, using blockhash instead", e);
            }
        }

        // If nonce wasn't used, fall back to blockhash
        if !temporal_used_nonce {
            match temporal_rpc.send_tx(&mut temporal_instructions, &explorer_keypair) {
                Ok(signature) => {
                    info!("Transaction submitted successfully via Temporal: {}", signature);
                    rpc_results.push(("Temporal", true, signature));
                },
                Err(e) => {
                    warn!("Failed to submit transaction via Temporal: {}", e);
                    rpc_results.push(("Temporal", false, e.to_string()));
                }
            }
        }

        // -- Jito RPC (async) --
        info!("Attempting submission via Jito");
        let jito_sdk = JitoJsonRpcSDK::new("https://mainnet.block-engine.jito.wtf/api/v1/bundles", None);

        // Try to use nonce for Jito if available
        let mut tx_created = false;
        let mut serialized_tx = String::new();

        // Try to use nonce if available
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Jito", nonce_pubkey, nonce_hash);

                        // Create nonce advance instruction as the first instruction
                        let nonce_advance_ix = crate::nonce::create_nonce_instruction(&nonce_pubkey, &nonce_authority.pubkey());
                        let mut nonce_instructions = vec![nonce_advance_ix];
                        nonce_instructions.extend_from_slice(&instructions);

                        // Build transaction with nonce hash instead of recent blockhash
                        let message = solana_sdk::message::Message::new(&nonce_instructions, Some(&explorer_keypair.pubkey()));
                        let mut tx = Transaction::new_unsigned(message);
                        tx.message.recent_blockhash = nonce_hash;

                        // Sign the transaction with both keypairs
                        tx.sign(&[&explorer_keypair, &nonce_authority], nonce_hash);

                        // Serialize the transaction
                        if let Ok(data) = bincode::serialize(&tx) {
                            // Use the new way to encode base64
                            use base64::Engine;
                            serialized_tx = base64::engine::general_purpose::STANDARD.encode(data);
                            tx_created = true;

                            // Release the nonce account back to the pool after the transaction is sent
                            if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                                warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                            }
                        } else {
                            warn!("Failed to serialize nonce transaction for Jito, falling back to blockhash");
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get nonce authority for Jito: {}, falling back to blockhash", e);
                    }
                }
            },
            Err(e) => {
                warn!("No nonce accounts available for Jito: {}, using blockhash instead", e);
            }
        }

        // Fall back to blockhash if nonce transaction creation failed
        if !tx_created {
            // Use the cached blockhash
            let blockhash_cache = crate::blockhash::BlockhashCache::instance();
            let blockhash = match blockhash_cache.get_blockhash(solana_rpc_client) {
                Ok(hash) => hash,
                Err(e) => {
                    // Fall back to direct RPC call if cache fails
                    warn!("Failed to get cached blockhash: {}, falling back to direct RPC", e);
                    match solana_rpc_client.get_latest_blockhash() {
                        Ok(bh) => bh,
                        Err(e) => {
                            warn!("Failed to get blockhash for Jito submission: {}", e);
                            return Ok(());
                        }
                    }
                }
            };

            let tx = Transaction::new_signed_with_payer(&instructions, Some(&explorer_keypair.pubkey()), &[&explorer_keypair], blockhash);
            serialized_tx = match bincode::serialize(&tx) {
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
        }

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

        // For Nextblock, we'll handle this directly since it's async
        let mut nextblock_instructions = instructions.clone();

        // Try to use nonce if available
        let mut nextblock_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Nextblock", nonce_pubkey, nonce_hash);

                        let nonce_info = rpc::NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.clone();
                        match nextblock_client.send_nonce_tx(&mut nonce_instructions, &explorer_keypair, nonce_info).await {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via Nextblock with nonce: {}", signature);
                                rpc_results.push(("Nextblock (nonce)", true, signature));
                                nextblock_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via Nextblock with nonce: {}", e);
                                rpc_results.push(("Nextblock (nonce)", false, e.to_string()));
                            }
                        }

                        // Release the nonce account back to the pool
                        if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                            warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get nonce authority for Nextblock: {}, falling back to blockhash", e);
                    }
                }
            },
            Err(e) => {
                warn!("No nonce accounts available for Nextblock: {}, using blockhash instead", e);
            }
        }

        // If nonce wasn't used, fall back to blockhash
        if !nextblock_used_nonce {
            match nextblock_client.send_tx(&mut nextblock_instructions, &explorer_keypair).await {
                Ok(signature) => {
                    info!("Transaction submitted successfully via Nextblock: {}", signature);
                    rpc_results.push(("Nextblock", true, signature));
                },
                Err(e) => {
                    warn!("Failed to submit transaction via Nextblock: {}", e);
                    rpc_results.push(("Nextblock", false, e.to_string()));
                }
            }
        }

        // -- Bloxroute (async) --
        info!("Attempting submission via Bloxroute");
        let bloxroute = Bloxroute::new();

        // For Bloxroute, we'll handle this directly since it's async
        let mut bloxroute_instructions = instructions.clone();

        // Try to use nonce if available
        let mut bloxroute_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Bloxroute", nonce_pubkey, nonce_hash);

                        let nonce_info = rpc::NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.clone();
                        match bloxroute.send_nonce_tx(&mut nonce_instructions, &explorer_keypair, nonce_info).await {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via Bloxroute with nonce: {}", signature);
                                rpc_results.push(("Bloxroute (nonce)", true, signature));
                                bloxroute_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via Bloxroute with nonce: {}", e);
                                rpc_results.push(("Bloxroute (nonce)", false, e.to_string()));
                            }
                        }

                        // Release the nonce account back to the pool
                        if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                            warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get nonce authority for Bloxroute: {}, falling back to blockhash", e);
                    }
                }
            },
            Err(e) => {
                warn!("No nonce accounts available for Bloxroute: {}, using blockhash instead", e);
            }
        }

        // If nonce wasn't used, fall back to blockhash
        if !bloxroute_used_nonce {
            match bloxroute.send_tx(&mut bloxroute_instructions, &explorer_keypair).await {
                Ok(signature) => {
                    info!("Transaction submitted successfully via Bloxroute: {}", signature);
                    rpc_results.push(("Bloxroute", true, signature));
                },
                Err(e) => {
                    warn!("Failed to submit transaction via Bloxroute: {}", e);
                    rpc_results.push(("Bloxroute", false, e.to_string()));
                }
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
                    Ok(sig) => {
                        match rpc_client.get_signature_status(&sig) {
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
            info!("Successful arbitrage transaction with estimated profit: {:.6} USD", estimated_profit);

            // Record confirmation rate metrics
            let confirmation_rate = confirmed_signatures.len() as f64 / tx_signatures.len() as f64;
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

        // Mark the Explorer key as used so it will be retired
        info!("Retiring explorer keypair after transaction use: {}", explorer_pubkey);
        // We retire the key no matter what happened - success or failure
        // This ensures keys aren't reused even if transactions failed to submit
        if let Err(e) = return_explorer_keypair(&explorer_pubkey, true) {
            error!("Failed to retire explorer key {}: {:?}", explorer_pubkey, e);
        }

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
    let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);

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

            Ok(())
        }).await;

        // Handle result
        if let Err(e) = result {
            error!("Error running lander: {:?}", e);
        }

        // Wait for specified duration before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}


