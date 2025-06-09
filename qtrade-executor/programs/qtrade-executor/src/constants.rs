use anchor_lang::prelude::*;

/// Constants module - equivalent to Solidity contract constants
/// Maps to constants defined in OdosRouterV2.sol

/// The zero address equivalent in Solana (used to represent native SOL)
/// Equivalent to OdosRouterV2._ETH = address(0)
pub const NATIVE_SOL: Pubkey = Pubkey::new_from_array([0; 32]);

/// Maximum additional fee a referral can set (2% = 200 basis points)
/// Simplified from OdosRouterV2.FEE_DENOM / 50 using basis points
pub const MAX_REFERRAL_FEE: u16 = 200; // 2% in basis points

/// Maximum swapMultiFee that can be set (0.5% = 50 basis points)
/// Simplified from OdosRouterV2.FEE_DENOM / 200 using basis points
pub const MAX_SWAP_MULTI_FEE: u16 = 50; // 0.5% in basis points

/// Default swap multi fee (0.05% = 5 basis points)
/// Equivalent to OdosRouterV2 constructor value, simplified to basis points
pub const DEFAULT_SWAP_MULTI_FEE: u16 = 5; // 0.05% in basis points

/// Fee denominator using basis points (10000 = 100%)
/// Simplified from OdosRouterV2.FEE_DENOM (1e18) for easier calculations on Solana
pub const FEE_DENOMINATOR: u16 = 10000;

/// Referral threshold for fees
/// Equivalent to OdosRouterV2.REFERRAL_WITH_FEE_THRESHOLD = 1 << 31
pub const REFERRAL_WITH_FEE_THRESHOLD: u32 = 1u32 << 31;

/// Seeds for PDA generation
pub const STATE_SEED: &[u8] = b"state";
pub const REFERRAL_SEED: &[u8] = b"referral";

/// Account space calculations for rent-exempt storage
pub const PROGRAM_STATE_SIZE: usize = 8 + 32 + 2 + 1; // discriminator + owner + swap_multi_fee + bump
pub const REFERRAL_INFO_SIZE: usize = 8 + 2 + 32 + 1 + 1; // discriminator + referral_fee + beneficiary + registered + bump
