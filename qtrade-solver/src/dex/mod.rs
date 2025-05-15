// DEX module for qtrade-solver
//
// This module contains implementations for getting quotes from various DEXes
// for use in arbitrage opportunity calculations.

pub mod orca;
pub mod raydium;
pub mod types;
pub mod mock;

use solana_sdk::pubkey::Pubkey;
use anyhow::Result;
use crate::dex::types::{SwapQuote, PoolReserves, DexType};

/// Trait for DEX quote providers
pub trait DexQuoter {
    /// Get a quote for a swap from the DEX
    fn get_swap_quote(
        &self,
        pool_address: &Pubkey,
        pool_reserves: &PoolReserves,
        amount_in: u64,
        is_token_a_to_b: bool,
        slippage_bps: u16,
    ) -> Result<SwapQuote>;
}

/// Factory function to create a DEX quoter
pub fn create_dex_quoter(dex_type: DexType) -> Box<dyn DexQuoter> {
    match dex_type {
        DexType::Orca => Box::new(orca::OrcaQuoter::new()),
        DexType::Raydium => Box::new(raydium::RaydiumQuoter::new()),
        DexType::RaydiumCpmm => Box::new(raydium::RaydiumQuoter::new()), // Using same implementation for now
        DexType::RaydiumClmm => unimplemented!("Raydium CLMM quoter not yet implemented"),
    }
}

/// Determine DEX type based on pool address
///
/// This function tries to determine the DEX type based on the pool address format or prefix.
pub fn determine_dex_type(_pool_address: &Pubkey) -> DexType {
    // This is a placeholder implementation. In a real implementation, we would check
    // the pool address against known patterns or prefixes for each DEX.
    // For now, default to Orca
    DexType::Orca
}
