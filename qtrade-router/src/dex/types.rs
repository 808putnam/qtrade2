// DEX types for qtrade-router
//
// This module contains shared types for DEX quote providers.

/// Identifies the DEX type for quote retrieval
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DexType {
    Orca,
    Raydium,
    RaydiumCpmm,
    RaydiumClmm,
}

/// Represents pool reserves and state for quote calculation
#[derive(Debug, Clone)]
pub struct PoolReserves {
    /// Current square root price as a u128 (for Orca, other DEXs might use differently)
    pub sqrt_price: u128,

    /// Current tick index (for Orca and other AMMs that use ticks)
    pub tick_current_index: i32,

    /// Current liquidity in the pool
    pub liquidity: u128,

    /// Fee rate in basis points (e.g., 3000 for 0.3%)
    pub fee_rate: u16,

    /// Tick spacing for the pool
    pub tick_spacing: u16,

    /// Token A reserves (for CPMM-style AMMs)
    pub token_a_reserves: Option<u64>,

    /// Token B reserves (for CPMM-style AMMs)
    pub token_b_reserves: Option<u64>,
}

impl Default for PoolReserves {
    fn default() -> Self {
        Self {
            sqrt_price: 0,
            tick_current_index: 0,
            liquidity: 0,
            fee_rate: 0,
            tick_spacing: 0,
            token_a_reserves: None,
            token_b_reserves: None,
        }
    }
}

/// Quote result from a DEX for a potential swap
#[derive(Debug, Clone)]
pub struct SwapQuote {
    /// The input token amount
    pub amount_in: u64,

    /// The estimated output token amount
    pub amount_out: u64,

    /// The minimum expected output amount after slippage
    pub min_amount_out: Option<u64>,

    /// The maximum input amount to spend after slippage
    pub max_amount_in: Option<u64>,

    /// Fee amount in the input token
    pub fee_amount: u64,

    /// Price impact calculated as percentage (0.01 = 1%)
    pub price_impact: f64,
}

/// Provides context for Orca Whirlpool quoting
#[derive(Debug, Clone)]
pub struct OrcaWhirlpoolContext {
    /// References to tick arrays needed for swap calculation
    pub tick_arrays: Vec<Vec<u8>>, // Placeholder for actual tick array data structure
}

/// Provides context for Raydium CLMM quoting
#[derive(Debug, Clone)]
pub struct RaydiumClmmContext {
    // Add Raydium CLMM specific fields here
}

/// Provides context for Raydium CPMM quoting
#[derive(Debug, Clone)]
pub struct RaydiumCpmmContext {
    // Add Raydium CPMM specific fields here
}
