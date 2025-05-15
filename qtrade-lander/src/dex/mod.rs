// DEX module for qtrade-lander
//
// This module contains the implementations for various DEXes swap instructions.
// Current supported DEXes:
// - Orca (Whirlpool)
//
// Planned support:
// - Raydium
// - Raydium CPMM
// - Raydium CLMM

pub mod orca;
pub mod raydium;
pub mod raydium_cpmm;
pub mod raydium_clmm;

use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use anyhow::Result;

/// Trait for DEX implementations
pub trait DexSwap {
    /// Create a swap instruction for the DEX
    fn create_swap_instruction(&self,
        pool_address: &Pubkey,
        token_authority: &Pubkey,
        token_a_address: &Pubkey,
        token_a_mint: &Pubkey,
        token_a_vault: &Pubkey,
        token_b_address: &Pubkey,
        token_b_mint: &Pubkey,
        token_b_vault: &Pubkey,
        amount: u64,
        amount_threshold: u64,
        is_token_a_to_b: bool,
        is_exact_input: bool,
    ) -> Result<Instruction>;
}

/// Identifies the DEX type for swap instruction creation
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DexType {
    Orca,
    Raydium,
    RaydiumCpmm,
    RaydiumClmm,
}

/// Factory function to create a DEX swap implementation
pub fn create_dex_swap(dex_type: DexType) -> Box<dyn DexSwap> {
    match dex_type {
        DexType::Orca => Box::new(orca::OrcaSwap::new()),
        DexType::Raydium => Box::new(raydium::RaydiumSwap::new()),
        DexType::RaydiumCpmm => Box::new(raydium_cpmm::RaydiumCpmmSwap::new()),
        DexType::RaydiumClmm => Box::new(raydium_clmm::RaydiumClmmSwap::new()),
    }
}

/// Determine DEX type based on pool address
///
/// This function tries to determine the DEX type based on the pool address format or prefix.
/// Currently it's a placeholder, to be implemented with actual DEX detection logic.
pub fn determine_dex_type(pool_address: &Pubkey) -> DexType {
    // For now, default to Orca
    // In the future, we'll implement proper detection based on the pool address
    DexType::Orca
}
