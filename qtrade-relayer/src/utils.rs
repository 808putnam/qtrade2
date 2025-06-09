use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::hash::Hash;
use solana_sdk::transaction::Transaction;
use solana_sdk::message::Message;
use solana_client::rpc_client::RpcClient;
use anyhow::Result;
use std::error::Error;
use tracing::{info, warn};

use crate::rpc;
use crate::nonce;

/// Create a transaction with a durable nonce
pub fn create_nonce_tx(
    instructions: &[Instruction],
    payer: Option<&Pubkey>,
    signers: &[&Keypair],
    nonce_hash: Hash,
) -> Transaction {
    let message = Message::new(instructions, payer);
    let mut tx = Transaction::new_unsigned(message);

    // Override the recent blockhash with the nonce value
    tx.message.recent_blockhash = nonce_hash;

    // Sign the transaction - we convert the slice to Vec to satisfy the type system
    let signers_vec: Vec<&dyn Signer> = signers.iter().map(|s| *s as &dyn Signer).collect();
    tx.sign(&signers_vec, nonce_hash);

    tx
}

/// Extension trait for Transaction to add nonce-related methods
pub trait TransactionExt {
    fn new_signed_with_payer_and_nonce(
        instructions: &[Instruction],
        payer: Option<&Pubkey>,
        signers: &[&dyn Signer],
        nonce_hash: Hash,
    ) -> Transaction;
}

impl TransactionExt for Transaction {
    fn new_signed_with_payer_and_nonce(
        instructions: &[Instruction],
        payer: Option<&Pubkey>,
        signers: &[&dyn Signer],
        nonce_hash: Hash,
    ) -> Transaction {
        // Create a transaction with nonce
        let message = Message::new(instructions, payer);
        let mut tx = Transaction::new_unsigned(message);

        // Override the recent blockhash with the nonce value
        tx.message.recent_blockhash = nonce_hash;

        // Sign the transaction
        tx.sign(signers, nonce_hash);

        tx
    }
}

/// Attempt to send a transaction using a nonce account if available
pub async fn try_with_nonce_or_blockhash<T: rpc::RpcActions>(
    rpc_provider: &T,
    instructions: &mut Vec<Instruction>,
    signer: &Keypair,
    rpc_name: &str,
    nonce_pool: &nonce::NoncePool,
    rpc_client: &RpcClient,
    rpc_results: &mut Vec<(String, bool, String)>,  // Changed from &str to String for first element
) -> Result<()> {
    // Try to use nonce if available
    match nonce_pool.acquire_nonce(rpc_client) {
        Ok((nonce_pubkey, nonce_hash)) => {
            match nonce_pool.get_authority() {
                Ok(nonce_authority) => {
                    info!("Using nonce account {} with hash {} for {}", nonce_pubkey, nonce_hash, rpc_name);

                    // Create nonce info for transactions
                    let nonce_info = rpc::NonceInfo {
                        nonce_pubkey: &nonce_pubkey,
                        nonce_authority: &nonce_authority,
                        nonce_hash,
                    };

                    // Send with nonce
                    let mut nonce_instructions = instructions.clone();
                    match rpc_provider.send_nonce_tx(&mut nonce_instructions, signer, nonce_info) {
                        Ok(signature) => {
                            info!("Transaction submitted successfully via {} with nonce: {}", rpc_name, signature);
                            let provider_name = format!("{} (nonce)", rpc_name);
                            rpc_results.push((provider_name, true, signature.to_string()));
                        },
                        Err(e) => {
                            warn!("Failed to submit transaction via {} with nonce: {}", rpc_name, e);
                            let provider_name = format!("{} (nonce)", rpc_name);
                            rpc_results.push((provider_name, false, e.to_string()));
                        }
                    }

                    // Release the nonce account back to the pool
                    if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                        warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                    }

                    return Ok(());
                },
                Err(e) => {
                    warn!("Failed to get nonce authority: {}, falling back to blockhash", e);
                }
            }
        },
        Err(e) => {
            warn!("No nonce accounts available for {}: {}, using blockhash instead", rpc_name, e);
        }
    }

    // Fall back to blockhash
    let mut regular_instructions = instructions.clone();
    match rpc_provider.send_tx(&mut regular_instructions, signer) {
        Ok(signature) => {
            info!("Transaction submitted successfully via {}: {}", rpc_name, signature);
            rpc_results.push((rpc_name.to_string(), true, signature.to_string()));
        },
        Err(e) => {
            warn!("Failed to submit transaction via {}: {}", rpc_name, e);
            rpc_results.push((rpc_name.to_string(), false, e.to_string()));
        }
    }

    Ok(())
}

/// Helper function to convert a String signature to String to ensure type consistency
fn to_string_or_pass(signature: String) -> String {
    signature
}

/// Attempt to send an async transaction using a nonce account if available
pub async fn try_with_nonce_or_blockhash_async<F, G, T>(
    rpc_name: &str,
    instructions: &mut Vec<Instruction>,
    signer: &Keypair,
    nonce_pool: &nonce::NoncePool,
    rpc_client: &RpcClient,
    rpc_results: &mut Vec<(String, bool, String)>,
    send_tx_fn: F,
    send_nonce_tx_fn: G,
) -> Result<()>
where
    F: FnOnce(&mut Vec<Instruction>, &Keypair) -> Result<String, Box<dyn Error>> + Send,
    G: FnOnce(&mut Vec<Instruction>, &Keypair, rpc::NonceInfo<'_>) -> Result<String, Box<dyn Error>> + Send,
{
    // Try to use nonce if available
    match nonce_pool.acquire_nonce(rpc_client) {
        Ok((nonce_pubkey, nonce_hash)) => {
            match nonce_pool.get_authority() {
                Ok(nonce_authority) => {
                    info!("Using nonce account {} with hash {} for {}", nonce_pubkey, nonce_hash, rpc_name);

                    // Create nonce info for transactions
                    let nonce_info = rpc::NonceInfo {
                        nonce_pubkey: &nonce_pubkey,
                        nonce_authority: &nonce_authority,
                        nonce_hash,
                    };

                    // Send with nonce
                    let mut nonce_instructions = instructions.clone();
                    match send_nonce_tx_fn(&mut nonce_instructions, signer, nonce_info) {
                        Ok(signature) => {
                            info!("Transaction submitted successfully via {} with nonce: {}", rpc_name, signature);
                            let provider_name = format!("{} (nonce)", rpc_name);
                            rpc_results.push((provider_name, true, to_string_or_pass(signature)));
                        },
                        Err(e) => {
                            warn!("Failed to submit transaction via {} with nonce: {}", rpc_name, e);
                            let provider_name = format!("{} (nonce)", rpc_name);
                            rpc_results.push((provider_name, false, e.to_string()));
                        }
                    }

                    // Release the nonce account back to the pool
                    if let Err(e) = nonce_pool.release_nonce(&nonce_pubkey) {
                        warn!("Failed to release nonce account {}: {}", nonce_pubkey, e);
                    }

                    return Ok(());
                },
                Err(e) => {
                    warn!("Failed to get nonce authority: {}, falling back to blockhash", e);
                }
            }
        },
        Err(e) => {
            warn!("No nonce accounts available for {}: {}, using blockhash instead", rpc_name, e);
        }
    }

    // Fall back to blockhash
    let mut regular_instructions = instructions.clone();
    match send_tx_fn(&mut regular_instructions, signer) {
        Ok(signature) => {
            info!("Transaction submitted successfully via {}: {}", rpc_name, signature);
            rpc_results.push((rpc_name.to_string(), true, to_string_or_pass(signature)));
        },
        Err(e) => {
            warn!("Failed to submit transaction via {}: {}", rpc_name, e);
            rpc_results.push((rpc_name.to_string(), false, e.to_string()));
        }
    }

    Ok(())
}
