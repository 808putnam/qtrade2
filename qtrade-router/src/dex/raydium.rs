// Raydium DEX implementation for quoting
//
// This module provides functionality to get quotes from Raydium AMM pools

use solana_sdk::pubkey::Pubkey;
use anyhow::{Result, anyhow};
use super::DexQuoter;
use super::types::{SwapQuote, PoolReserves};

/// Implementation for Raydium quoting functionality
pub struct RaydiumQuoter;

impl RaydiumQuoter {
    /// Create a new RaydiumQuoter instance
    pub fn new() -> Self {
        Self
    }

    /// Calculate output amount based on Constant Product Market Maker (CPMM) formula
    /// This is the formula used by most AMMs: x * y = k (constant product)
    fn calculate_cpmm_output(
        &self,
        reserve_in: u64,
        reserve_out: u64,
        amount_in: u64,
        fee_rate: u16,
    ) -> Result<u64> {
        // Ensure we have valid reserves
        if reserve_in == 0 || reserve_out == 0 {
            return Err(anyhow!("Invalid reserves for CPMM calculation"));
        }

        // Convert to f64 for calculation
        let reserve_in_f = reserve_in as f64;
        let reserve_out_f = reserve_out as f64;
        let amount_in_f = amount_in as f64;

        // Calculate fee
        // fee_rate is in basis points (1/100 of a percent), e.g., 30 = 0.3%
        let fee_rate_f = fee_rate as f64 / 10000.0;

        // Calculate amount in after fee
        let amount_in_with_fee = amount_in_f * (1.0 - fee_rate_f);

        // Calculate output amount using constant product formula: x * y = k
        // (reserve_in + amount_in_with_fee) * (reserve_out - out) = reserve_in * reserve_out
        let product = reserve_in_f * reserve_out_f;
        let new_reserve_in = reserve_in_f + amount_in_with_fee;
        let new_reserve_out = product / new_reserve_in;

        // Calculate output amount
        let amount_out = reserve_out_f - new_reserve_out;

        // Convert back to u64, ensuring we don't exceed available reserves
        let amount_out_u64 = amount_out.min(reserve_out_f) as u64;

        Ok(amount_out_u64)
    }

    /// Calculate input amount required for desired output using CPMM formula
    fn calculate_cpmm_input_for_output(
        &self,
        reserve_in: u64,
        reserve_out: u64,
        amount_out: u64,
        fee_rate: u16,
    ) -> Result<u64> {
        // Ensure we have valid reserves and the requested output isn't larger than reserves
        if reserve_in == 0 || reserve_out == 0 || amount_out >= reserve_out {
            return Err(anyhow!("Invalid parameters for CPMM calculation"));
        }

        // Convert to f64 for calculation
        let reserve_in_f = reserve_in as f64;
        let reserve_out_f = reserve_out as f64;
        let amount_out_f = amount_out as f64;

        // Calculate fee rate factor
        // fee_rate is in basis points (1/100 of a percent), e.g., 30 = 0.3%
        let fee_rate_f = fee_rate as f64 / 10000.0;
        let fee_factor = 1.0 - fee_rate_f;

        // Calculate new reserve out after removing the desired output
        let new_reserve_out = reserve_out_f - amount_out_f;

        // Calculate required input before fees using constant product formula: x * y = k
        // reserve_in * reserve_out = (reserve_in + amount_in) * new_reserve_out
        let product = reserve_in_f * reserve_out_f;
        let amount_in_before_fee = (product / new_reserve_out) - reserve_in_f;

        // Account for fees to get the actual input amount
        let amount_in = amount_in_before_fee / fee_factor;

        // Convert back to u64 and add 1 to account for potential rounding
        // This ensures we always provide enough input
        let amount_in_u64 = amount_in.ceil() as u64;

        Ok(amount_in_u64)
    }
}

impl DexQuoter for RaydiumQuoter {
    fn get_swap_quote(
        &self,
        _pool_address: &Pubkey,
        pool_reserves: &PoolReserves,
        amount_in: u64,
        is_token_a_to_b: bool,
        slippage_bps: u16,
    ) -> Result<SwapQuote> {
        // For Raydium CPMM, we need token reserves
        let token_a_reserves = pool_reserves.token_a_reserves.ok_or_else(||
            anyhow!("Token A reserves not available for Raydium pool"))?;

        let token_b_reserves = pool_reserves.token_b_reserves.ok_or_else(||
            anyhow!("Token B reserves not available for Raydium pool"))?;

        // Determine which token is being swapped in and which is being swapped out
        let (reserve_in, reserve_out) = if is_token_a_to_b {
            (token_a_reserves, token_b_reserves)
        } else {
            (token_b_reserves, token_a_reserves)
        };

        // Calculate the expected output amount
        let estimated_out = self.calculate_cpmm_output(
            reserve_in,
            reserve_out,
            amount_in,
            pool_reserves.fee_rate,
        )?;

        // Calculate minimum output with slippage
        let slippage_factor = 1.0 - (slippage_bps as f64 / 10000.0);
        let min_out = (estimated_out as f64 * slippage_factor).floor() as u64;

        // Calculate fee amount
        let fee_rate_f = pool_reserves.fee_rate as f64 / 10000.0;
        let fee_amount = (amount_in as f64 * fee_rate_f).ceil() as u64;

        // Calculate price impact
        // In a real implementation, we'd compare with oracle/market prices
        // For now, use a simplified approximation
        let no_impact_rate = reserve_out as f64 / reserve_in as f64;
        let execution_rate = estimated_out as f64 / amount_in as f64;
        let price_impact = (no_impact_rate - execution_rate) / no_impact_rate;
        let price_impact = price_impact.max(0.0);

        Ok(SwapQuote {
            amount_in,
            amount_out: estimated_out,
            min_amount_out: Some(min_out),
            max_amount_in: None,
            fee_amount,
            price_impact,
        })
    }
}
