//! Module for preparing arbitrage transactions

use anyhow::{Result, anyhow};
use qtrade_shared_types::ArbitrageResult;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::instruction::Instruction;
use tracing::{info, warn, error};
use crate::dex;
use crate::determine_pool_pubkey;
use crate::determine_token_indices;
use crate::metrics::arbitrage::record_failed_arbitrage_transaction;
use qtrade_wallets::{get_explorer_keypair, return_explorer_keypair};

/// Validates an arbitrage result to ensure it's valid for execution
///
/// Returns Ok(true) if the arbitrage result is valid and profitable
/// Returns Ok(false) if the arbitrage result is invalid or not profitable
/// Returns Err if there was an error during validation
pub fn validate_arbitrage_result(arbitrage_result: &ArbitrageResult) -> Result<bool> {
    // 1. Validate the arbitrage result
    if arbitrage_result.status != "optimal" {
        warn!("Skipping arbitrage execution as status is not optimal: {}", arbitrage_result.status);
        return Ok(false);
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
        return Ok(false);
    }

    Ok(true)
}

/// Struct to hold swap parameters for an arbitrage operation
#[derive(Debug, Clone)]
pub struct ArbitrageSwapParams {
    pub pool_index: usize,
    pub dex_type: dex::DexType,
    pub pool_pubkey: Pubkey,
    pub token_a_wallet: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_a_vault: Pubkey,
    pub token_b_wallet: Pubkey,
    pub token_b_mint: Pubkey,
    pub token_b_vault: Pubkey,
    pub amount_in: u64,
    pub min_amount_out: u64,
}

/// Constructs swap parameters based on the arbitrage result
///
/// This function:
/// 1. Processes each pool in the arbitrage result
/// 2. Calculates profit for each pool
/// 3. Constructs swap parameters for each profitable operation
///
/// Returns Ok(Some((swap_params_list, estimated_profit))) if profitable swap operations were found
/// Returns Ok(None) if no profitable swap operations were found
/// Returns Err if there was an error during parameter construction
pub fn construct_swap_parameters(arbitrage_result: &ArbitrageResult) -> Result<Option<(Vec<ArbitrageSwapParams>, f64)>> {
    // Record metrics for processing an arbitrage opportunity
    crate::metrics::arbitrage::record_arbitrage_opportunity_processed();

    // Initialize values for tracking profit
    let mut estimated_profit = 0.0;
    let mut swap_params_list = Vec::new();

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

                swap_params_list.push(swap_params);
                info!("Prepared swap parameters for pool {}", pool_index);
            }
        }
    }

    if swap_params_list.is_empty() {
        info!("No profitable swap operations prepared, skipping execution");
        return Ok(None);
    }

    info!("Prepared {} swap operations with estimated profit: {:.6}",
        swap_params_list.len(), estimated_profit);

    Ok(Some((swap_params_list, estimated_profit)))
}

/// Acquires an explorer keypair from the tiered wallet system for transaction signing
///
/// Returns Ok((pubkey, keypair)) if an explorer keypair is available
/// Returns Err if no explorer keypairs are available
pub fn acquire_explorer_keypair() -> Result<(Pubkey, Keypair)> {
    match get_explorer_keypair() {
        Some(keypair) => {
            info!("Using explorer keypair with public key: {}", keypair.0);
            Ok(keypair)
        },
        None => {
            error!("No explorer keypairs available for transaction signing");
            record_failed_arbitrage_transaction();
            Err(anyhow!("No explorer keypairs available for transaction signing"))
        }
    }
}

/// Returns an explorer keypair to the tiered wallet system
///
/// Call this after transaction execution (successful or not) to prevent keypair reuse
pub fn return_explorer_keypair_to_pool(pubkey: &Pubkey, retire: bool) -> Result<()> {
    if let Err(e) = return_explorer_keypair(pubkey, retire) {
        error!("Failed to return explorer key {}: {:?}", pubkey, e);
        return Err(anyhow!("Failed to return explorer keypair to pool"));
    }

    info!("Returned explorer keypair {} to pool (retired: {})", pubkey, retire);
    Ok(())
}

/// Create swap instructions for each swap parameter using the explorer keypair public key
///
/// This function converts the high-level swap parameters into Solana instruction objects
/// by calling the appropriate DEX-specific swap implementation for each parameter set.
///
/// Returns a vector of Solana instructions that can be included in a transaction
pub fn create_swap_instructions(
    swap_params_list: &[ArbitrageSwapParams],
    explorer_pubkey: &Pubkey,
) -> Result<Vec<Instruction>> {
    info!("Creating swap instructions with explorer pubkey: {}", explorer_pubkey);
    let mut instructions: Vec<Instruction> = Vec::new();

    for params in swap_params_list {
        // Create the appropriate DEX swap implementation
        let dex_swap = dex::create_dex_swap(params.dex_type);

        // Create the swap instruction with the explorer keypair as the authority
        let swap_instruction = dex_swap.create_swap_instruction(
            &params.pool_pubkey,
            explorer_pubkey, // Explorer pubkey is used as token authority
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
            anyhow!("Failed to create swap instruction")
        })?;

        instructions.push(swap_instruction);
        info!("Added swap instruction for pool {}", params.pool_index);
    }

    Ok(instructions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_arbitrage_result_optimal() {
        // Create a valid arbitrage result with optimal status and non-zero deltas
        let arbitrage_result = ArbitrageResult {
            status: "optimal".to_string(),
            deltas: vec![vec![0.001, -0.0009]],
            lambdas: vec![vec![0.0, 0.0]],
            a_matrices: vec![vec![vec![0.0]]],
        };

        let result = validate_arbitrage_result(&arbitrage_result).unwrap();
        assert!(result, "Should validate as true for optimal result with non-zero deltas");
    }

    #[test]
    fn test_validate_arbitrage_result_non_optimal() {
        // Create an arbitrage result with non-optimal status
        let arbitrage_result = ArbitrageResult {
            status: "suboptimal".to_string(),
            deltas: vec![vec![0.001, -0.0009]],
            lambdas: vec![vec![0.0, 0.0]],
            a_matrices: vec![vec![vec![0.0]]],
        };

        let result = validate_arbitrage_result(&arbitrage_result).unwrap();
        assert!(!result, "Should validate as false for non-optimal result");
    }

    #[test]
    fn test_validate_arbitrage_result_zero_deltas() {
        // Create an arbitrage result with optimal status but zero deltas
        let arbitrage_result = ArbitrageResult {
            status: "optimal".to_string(),
            deltas: vec![vec![0.0, 0.0]],
            lambdas: vec![vec![0.0, 0.0]],
            a_matrices: vec![vec![vec![0.0]]],
        };

        let result = validate_arbitrage_result(&arbitrage_result).unwrap();
        assert!(!result, "Should validate as false for zero deltas");
    }

    // Note: For this task's focused scope, we're skipping the unit tests for construct_swap_parameters.
    // These tests will require mock implementations of determine_pool_pubkey and determine_dex_type,
    // which would be better implemented using a proper dependency injection pattern.
    // This will be addressed in a future task when we implement more sophisticated testing infrastructure.

    #[test]
    fn test_create_swap_instructions() {
        // Create a mock DEX type and pool pubkey
        let pool_pubkey = Pubkey::new_unique();
        let dex_type = dex::DexType::Orca;

        // Create mock token accounts
        let token_a_wallet = Pubkey::new_unique();
        let token_a_mint = Pubkey::new_unique();
        let token_a_vault = Pubkey::new_unique();
        let token_b_wallet = Pubkey::new_unique();
        let token_b_mint = Pubkey::new_unique();
        let token_b_vault = Pubkey::new_unique();

        // Create mock explorer pubkey
        let explorer_pubkey = Pubkey::new_unique();

        // Create a swap parameter
        let swap_param = ArbitrageSwapParams {
            pool_index: 0,
            dex_type,
            pool_pubkey,
            token_a_wallet,
            token_a_mint,
            token_a_vault,
            token_b_wallet,
            token_b_mint,
            token_b_vault,
            amount_in: 1000,
            min_amount_out: 990,
        };

        // Call the function with a list containing one swap parameter
        let result = create_swap_instructions(&[swap_param], &explorer_pubkey);

        // We can't fully test the instruction creation since it depends on the DEX swap implementation
        // But we can at least check that the function returns a result
        assert!(result.is_ok(), "Should return Ok result");

        // Skip the actual instruction validation for now since we'd need to mock the DEX swap implementations
        // This could be expanded in the future with proper mocking
    }
}
