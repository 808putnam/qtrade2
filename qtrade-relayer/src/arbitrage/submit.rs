//! Module for submitting arbitrage transactions via multiple RPC providers

use anyhow::{Result, anyhow};
use solana_sdk::{instruction::Instruction, signature::{Keypair, Signer}, transaction::Transaction};
use serde_json::json;
use tracing::{info, warn};
use bincode;

use crate::rpc::{RpcActions, NonceInfo};
use crate::rpc::solana::{Solana, SolanaEndpoint};
use crate::rpc::helius::Helius;
use crate::rpc::temporal::Temporal;
use crate::rpc::jito::JitoJsonRpcSDK;
use crate::rpc::nextblock::Nextblock;
use crate::rpc::bloxroute::Bloxroute;
use crate::rpc::quicknode::Quicknode;
use crate::metrics::arbitrage::record_failed_arbitrage_transaction;
use crate::nonce::NoncePool;
use crate::settings::RelayerSettings;

/// Result of transaction submission to an RPC provider
pub type RpcSubmissionResult = (String, bool, String);

/// Submits transactions via multiple RPC providers
///
/// Attempts to send the transaction through various RPC providers for redundancy
/// Uses nonce accounts when available, falling back to recent blockhashes
///
/// Returns a vector of (provider name, success flag, signature/error message) tuples
pub async fn submit_transaction(
    instructions: &[Instruction],
    explorer_keypair: &Keypair,
    settings: &RelayerSettings,
    is_simulation: bool,
) -> Result<Vec<RpcSubmissionResult>> {
    let mut rpc_results: Vec<RpcSubmissionResult> = Vec::new();

    if is_simulation {
        info!("SIMULATION MODE: Simulating transaction instead of submitting");

        // Create RPC providers with our settings
        let (_, helius, nextblock, _, _) = create_rpc_with_settings(settings);

        // Solana RPC (preferred simulation provider)
        if is_rpc_active(settings, "solana") {
            let solana_rpc = Solana::new(SolanaEndpoint::Mainnet);
            let solana_instructions = instructions.to_vec();

            match solana_rpc.simulate_tx(&mut solana_instructions.clone(), explorer_keypair) {
                Ok(simulation_result) => {
                    info!("Transaction simulation result from Solana RPC:");
                    info!("{}", simulation_result);
                    rpc_results.push(("Solana RPC (simulation)".to_string(), true, simulation_result));
                },
                Err(e) => {
                    warn!("Failed to simulate transaction with Solana RPC: {}", e);
                    rpc_results.push(("Solana RPC (simulation)".to_string(), false, e.to_string()));
                }
            }
        } else {
            info!("Skipping Solana RPC simulation (not in active RPCs list)");
        }

        // Helius RPC simulation
        if is_rpc_active(settings, "helius") {
            let helius_instructions = instructions.to_vec();
            match helius.simulate_tx(&mut helius_instructions.clone(), explorer_keypair) {
                Ok(simulation_result) => {
                    info!("Transaction simulation result from Helius:");
                    info!("{}", simulation_result);
                    rpc_results.push(("Helius (simulation)".to_string(), true, simulation_result));
                },
                Err(e) => {
                    warn!("Failed to simulate transaction with Helius: {}", e);
                    rpc_results.push(("Helius (simulation)".to_string(), false, e.to_string()));
                }
            }
        } else {
            info!("Skipping Helius simulation (not in active RPCs list)");
        }

        // Nextblock RPC simulation (async)
        if is_rpc_active(settings, "nextblock") {
            let nextblock_instructions = instructions.to_vec();
            match nextblock.simulate_tx(&mut nextblock_instructions.clone(), explorer_keypair).await {
                Ok(simulation_result) => {
                    info!("Transaction simulation result from Nextblock:");
                    info!("{}", simulation_result);
                    rpc_results.push(("Nextblock (simulation)".to_string(), true, simulation_result));
                },
                Err(e) => {
                    warn!("Failed to simulate transaction with Nextblock: {}", e);
                    rpc_results.push(("Nextblock (simulation)".to_string(), false, e.to_string()));
                }
            }
        } else {
            info!("Skipping Nextblock simulation (not in active RPCs list)");
        }

        // Check if all simulations failed
        if !rpc_results.iter().any(|(_, success, _)| *success) && !rpc_results.is_empty() {
            record_failed_arbitrage_transaction();
            warn!("All transaction simulations failed.");
        } else if rpc_results.is_empty() {
            warn!("No simulations were run as no active RPCs were configured for simulation");
        }

        // Log detailed simulation results
        info!("Transaction simulation complete with results:");
        for (provider, success, message) in &rpc_results {
            if *success {
                info!("{}: Simulation successful", provider);
            } else {
                warn!("{}: Simulation failed ({})", provider, message);
            }
        }

        return Ok(rpc_results);
    }

    // Regular submission mode
    info!("Submitting transaction to multiple RPC providers");

    // Create RPC providers with our settings
    let (bloxroute, helius, nextblock, quicknode, temporal) = create_rpc_with_settings(settings);

    // Setup nonce pool and Solana RPC client for nonce operations
    let solana_rpc = Solana::new(SolanaEndpoint::Mainnet);
    let solana_rpc_client = solana_rpc.rpc_client();
    let nonce_pool = NoncePool::instance();

    // -- Solana RPC --
    if is_rpc_active(settings, "solana") {
        info!("Attempting submission via Solana RPC");
        let mut solana_instructions = instructions.to_vec();

        // Try to use nonce if available
        let mut solana_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Solana RPC", nonce_pubkey, nonce_hash);

                        let nonce_info = NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.to_vec();
                        match solana_rpc.send_nonce_tx(&mut nonce_instructions, explorer_keypair, nonce_info) {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via Solana RPC with nonce: {}", signature);
                                rpc_results.push(("Solana RPC (nonce)".to_string(), true, signature));
                                solana_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via Solana RPC with nonce: {}", e);
                                rpc_results.push(("Solana RPC (nonce)".to_string(), false, e.to_string()));
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
            match solana_rpc.send_tx(&mut solana_instructions, explorer_keypair) {
                Ok(signature) => {
                    info!("Transaction submitted successfully via Solana RPC: {}", signature);
                    rpc_results.push(("Solana RPC".to_string(), true, signature));
                },
                Err(e) => {
                    warn!("Failed to submit transaction via Solana RPC: {}", e);
                    rpc_results.push(("Solana RPC".to_string(), false, e.to_string()));
                }
            }
        }
    }

    // -- Helius RPC --
    if is_rpc_active(settings, "helius") {
        info!("Attempting submission via Helius");
        let mut helius_instructions = instructions.to_vec();

        // Try to use nonce if available
        let mut helius_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Helius", nonce_pubkey, nonce_hash);

                        let nonce_info = NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.to_vec();
                        match helius.send_nonce_tx(&mut nonce_instructions, explorer_keypair, nonce_info) {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via Helius with nonce: {}", signature);
                                rpc_results.push(("Helius (nonce)".to_string(), true, signature));
                                helius_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via Helius with nonce: {}", e);
                                rpc_results.push(("Helius (nonce)".to_string(), false, e.to_string()));
                            }
                        }

                        // Release the nonce account back to the pool
                        if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                            warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get nonce authority for Helius: {}, falling back to blockhash", e);
                    }
                }
            },
            Err(e) => {
                warn!("No nonce accounts available for Helius: {}, using blockhash instead", e);
            }
        }

        // If nonce wasn't used, fall back to blockhash
        if !helius_used_nonce {
            match helius.send_tx(&mut helius_instructions, explorer_keypair) {
                Ok(signature) => {
                    info!("Transaction submitted successfully via Helius: {}", signature);
                    rpc_results.push(("Helius".to_string(), true, signature));
                },
                Err(e) => {
                    warn!("Failed to submit transaction via Helius: {}", e);
                    rpc_results.push(("Helius".to_string(), false, e.to_string()));
                }
            }
        }
    }

    // -- QuickNode RPC --
    if is_rpc_active(settings, "quicknode") {
        info!("Attempting submission via QuickNode");
        let mut quicknode_instructions = instructions.to_vec();

        // Try to use nonce if available
        let mut quicknode_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for QuickNode", nonce_pubkey, nonce_hash);

                        let nonce_info = NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.to_vec();
                        match quicknode.send_nonce_tx(&mut nonce_instructions, explorer_keypair, nonce_info) {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via QuickNode with nonce: {}", signature);
                                rpc_results.push(("QuickNode (nonce)".to_string(), true, signature));
                                quicknode_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via QuickNode with nonce: {}", e);
                                rpc_results.push(("QuickNode (nonce)".to_string(), false, e.to_string()));
                            }
                        }

                        // Release the nonce account back to the pool
                        if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                            warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get nonce authority for QuickNode: {}, falling back to blockhash", e);
                    }
                }
            },
            Err(e) => {
                warn!("No nonce accounts available for QuickNode: {}, using blockhash instead", e);
            }
        }

        // If nonce wasn't used, fall back to blockhash
        if !quicknode_used_nonce {
            match quicknode.send_tx(&mut quicknode_instructions, explorer_keypair) {
                Ok(signature) => {
                    info!("Transaction submitted successfully via QuickNode: {}", signature);
                    rpc_results.push(("QuickNode".to_string(), true, signature));
                },
                Err(e) => {
                    warn!("Failed to submit transaction via QuickNode: {}", e);
                    rpc_results.push(("QuickNode".to_string(), false, e.to_string()));
                }
            }
        }
    }

    // -- Temporal RPC --
    if is_rpc_active(settings, "temporal") {
        info!("Attempting submission via Temporal");
        let mut temporal_instructions = instructions.to_vec();

        // Try to use nonce if available
        let mut temporal_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Temporal", nonce_pubkey, nonce_hash);

                        let nonce_info = NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.to_vec();
                        match temporal.send_nonce_tx(&mut nonce_instructions, explorer_keypair, nonce_info) {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via Temporal with nonce: {}", signature);
                                rpc_results.push(("Temporal (nonce)".to_string(), true, signature));
                                temporal_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via Temporal with nonce: {}", e);
                                rpc_results.push(("Temporal (nonce)".to_string(), false, e.to_string()));
                            }
                        }

                        // Release the nonce account back to the pool
                        if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                            warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                        }
                    },
                    Err(e) => {
                        warn!("Failed to get nonce authority for Temporal: {}, falling back to blockhash", e);
                    }
                }
            },
            Err(e) => {
                warn!("No nonce accounts available for Temporal: {}, using blockhash instead", e);
            }
        }

        // If nonce wasn't used, fall back to blockhash
        if !temporal_used_nonce {
            match temporal.send_tx(&mut temporal_instructions, explorer_keypair) {
                Ok(signature) => {
                    info!("Transaction submitted successfully via Temporal: {}", signature);
                    rpc_results.push(("Temporal".to_string(), true, signature));
                },
                Err(e) => {
                    warn!("Failed to submit transaction via Temporal: {}", e);
                    rpc_results.push(("Temporal".to_string(), false, e.to_string()));
                }
            }
        }
    }

    // -- Jito RPC (async) --
    if is_rpc_active(settings, "jito") {
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

                        // Create nonce instruction
                        let advance_nonce_instruction = solana_sdk::system_instruction::advance_nonce_account(
                            &nonce_pubkey,
                            &nonce_authority.pubkey(),
                        );

                        // Create full instruction set
                        let mut jito_instructions = vec![advance_nonce_instruction];
                        jito_instructions.extend_from_slice(instructions);

                        // Create transaction
                        let tx = Transaction::new_signed_with_payer(
                            &jito_instructions,
                            Some(&explorer_keypair.pubkey()),
                            &[explorer_keypair, &nonce_authority],
                            nonce_hash,
                        );

                        serialized_tx = match bincode::serialize(&tx) {
                            Ok(data) => {
                                // Use the new way to encode base64
                                use base64::Engine;
                                tx_created = true;
                                base64::engine::general_purpose::STANDARD.encode(data)
                            },
                            Err(e) => {
                                warn!("Failed to serialize nonce transaction for Jito: {}", e);
                                String::new()
                            }
                        };

                        // Release the nonce account back to the pool
                        if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                            warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
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
            let blockhash = {
                // Try to get from blockhash cache first
                if let Ok(cached_blockhash) = crate::blockhash::BlockhashCache::instance().get_blockhash(&solana_rpc_client) {
                    cached_blockhash
                } else {
                    // Otherwise get from RPC
                    match solana_rpc_client.get_latest_blockhash() {
                        Ok(bh) => bh,
                        Err(e) => {
                            warn!("Failed to get blockhash for Jito submission: {}", e);
                            return Err(anyhow!("Failed to get blockhash for Jito submission: {}", e));
                        }
                    }
                }
            };

            let tx = Transaction::new_signed_with_payer(
                instructions,
                Some(&explorer_keypair.pubkey()),
                &[explorer_keypair],
                blockhash
            );

            serialized_tx = match bincode::serialize(&tx) {
                Ok(data) => {
                    // Use the engine to encode base64
                    use base64::Engine;
                    base64::engine::general_purpose::STANDARD.encode(data)
                },
                Err(e) => {
                    warn!("Failed to serialize transaction for Jito: {}", e);
                    return Err(anyhow!("Failed to serialize transaction for Jito: {}", e));
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
                rpc_results.push(("Jito".to_string(), true, format!("{:?}", response)));
            },
            Err(e) => {
                warn!("Failed to submit transaction via Jito: {}", e);
                rpc_results.push(("Jito".to_string(), false, e.to_string()));
            }
        }
    }

    // -- Nextblock RPC (async) --
    if is_rpc_active(settings, "nextblock") {
        info!("Attempting submission via Nextblock");
        let mut nextblock_instructions = instructions.to_vec();

        // Try to use nonce if available
        let mut nextblock_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Nextblock", nonce_pubkey, nonce_hash);

                        let nonce_info = NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.to_vec();
                        match nextblock.send_nonce_tx(&mut nonce_instructions, explorer_keypair, nonce_info).await {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via Nextblock with nonce: {}", signature);
                                rpc_results.push(("Nextblock (nonce)".to_string(), true, signature));
                                nextblock_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via Nextblock with nonce: {}", e);
                                rpc_results.push(("Nextblock (nonce)".to_string(), false, e.to_string()));
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
        match nextblock.send_tx(&mut nextblock_instructions, explorer_keypair).await {
            Ok(signature) => {
                info!("Transaction submitted successfully via Nextblock: {}", signature);
                rpc_results.push(("Nextblock".to_string(), true, signature));
            },
            Err(e) => {
                warn!("Failed to submit transaction via Nextblock: {}", e);
                rpc_results.push(("Nextblock".to_string(), false, e.to_string()));
            }
        }
    }
}

    // -- Bloxroute RPC (async) --
    if is_rpc_active(settings, "bloxroute") {
        info!("Attempting submission via Bloxroute");
        let mut bloxroute_instructions = instructions.to_vec();

        // Try to use nonce if available
        let mut bloxroute_used_nonce = false;
        match nonce_pool.acquire_nonce(&solana_rpc_client) {
            Ok((nonce_pubkey, nonce_hash)) => {
                match nonce_pool.get_authority() {
                    Ok(nonce_authority) => {
                        info!("Using nonce account {} with hash {} for Bloxroute", nonce_pubkey, nonce_hash);

                        let nonce_info = NonceInfo {
                            nonce_pubkey: &nonce_pubkey,
                            nonce_authority: &nonce_authority,
                            nonce_hash,
                        };

                        // Send with nonce
                        let mut nonce_instructions = instructions.to_vec();
                        match bloxroute.send_nonce_tx(&mut nonce_instructions, explorer_keypair, nonce_info).await {
                            Ok(signature) => {
                                info!("Transaction submitted successfully via Bloxroute with nonce: {}", signature);
                                rpc_results.push(("Bloxroute (nonce)".to_string(), true, signature));
                                bloxroute_used_nonce = true;
                            },
                            Err(e) => {
                                warn!("Failed to submit transaction via Bloxroute with nonce: {}", e);
                                rpc_results.push(("Bloxroute (nonce)".to_string(), false, e.to_string()));
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
        match bloxroute.send_tx(&mut bloxroute_instructions, explorer_keypair).await {
            Ok(signature) => {
                info!("Transaction submitted successfully via Bloxroute: {}", signature);
                rpc_results.push(("Bloxroute".to_string(), true, signature));
            },
            Err(e) => {
                warn!("Failed to submit transaction via Bloxroute: {}", e);
                rpc_results.push(("Bloxroute".to_string(), false, e.to_string()));
            }
        }
    }
}

    // Check circuit breakers - if multiple providers report the same critical error
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
        warn!("Detected critical submission errors across multiple providers; transaction likely invalid");
        record_failed_arbitrage_transaction();
    }

    info!("Completed transaction submission to all RPC providers");

    // Return the results of all submission attempts
    Ok(rpc_results)
}

/// Helper function: Create RPC service instances with the provided settings
pub fn create_rpc_with_settings(settings: &RelayerSettings) -> (Bloxroute, Helius, Nextblock, Quicknode, Temporal) {
    let bloxroute = Bloxroute::with_settings(settings);
    let helius = Helius::with_settings(settings);
    let nextblock = Nextblock::with_settings(settings);
    let quicknode = Quicknode::with_settings(settings);
    let temporal = Temporal::with_settings(settings);

    (bloxroute, helius, nextblock, quicknode, temporal)
}

/// Checks if a specific RPC provider is active in the settings.
///
/// This function determines whether a given RPC provider should be used
/// for transaction submission based on the active_rpcs configuration.
///
/// # Arguments
///
/// * `settings` - The relayer settings containing the active_rpcs configuration
/// * `rpc_name` - The name of the RPC provider to check (case-insensitive)
///
/// # Returns
///
/// * `true` if the RPC provider should be used
/// * `false` if the RPC provider should be skipped
///
/// # Example
///
/// ```
/// use qtrade_relayer::settings::RelayerSettings;
/// use qtrade_relayer::arbitrage::submit::is_rpc_active;
///
/// let settings = RelayerSettings::new_with_rpcs(
///     "".to_string(), // bloxroute_api_key
///     "".to_string(), // helius_api_key
///     "".to_string(), // nextblock_api_key
///     "".to_string(), // quicknode_api_key
///     "".to_string(), // temporal_api_key
///     vec!["solana".to_string(), "jito".to_string()], // only use Solana and Jito RPCs
///     false // simulate
/// );
///
/// assert!(is_rpc_active(&settings, "solana"));
/// assert!(is_rpc_active(&settings, "jito"));
/// assert!(!is_rpc_active(&settings, "helius"));
/// ```
pub fn is_rpc_active(settings: &RelayerSettings, rpc_name: &str) -> bool {
    settings.active_rpcs.iter().any(|name| name.to_lowercase() == rpc_name.to_lowercase())
}
