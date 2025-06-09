use serde::{Deserialize, Serialize};
use spl_pod::solana_pubkey::Pubkey;
use std::any::Any;
use async_trait::async_trait;

/// ArbitrageResult represents the result of the router's optimization process
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ArbitrageResult {
    /// Delta values (tender amounts) for each pool
    pub deltas: Vec<Vec<f64>>,
    /// Lambda values (receive amounts) for each pool
    pub lambdas: Vec<Vec<f64>>,
    /// A-matrix that maps global to local indices
    pub a_matrices: Vec<Vec<Vec<f64>>>,
    /// Status of the optimization problem
    pub status: String,
}

/// Define the PoolEntry type alias for shared use between router and indexer
pub type PoolEntry = (Pubkey, Box<dyn Any + Send + Sync>);

/// Trait for cache implementations used by the router
/// Allows retrieving pool entries for processing by the optimization engine
#[async_trait::async_trait]
pub trait PoolCache: Send + Sync {
    /// Get all entries from the cache as a vector
    /// Returns a vector of (key, boxed state) pairs
    async fn get_all_entries_as_slice(&self) -> Vec<PoolEntry>;
}
