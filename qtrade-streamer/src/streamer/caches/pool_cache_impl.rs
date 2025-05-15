// Implementation of qtrade_solver::PoolCache for qtrade-runtime's PoolCache
use spl_pod::solana_pubkey::Pubkey;
use crate::streamer::caches::pool_cache::{PoolCache, PoolCacheState};
use std::any::Any;
use tracing::info;

/// Implementation of the PoolCache trait from qtrade-solver for our PoolCache struct
/// This allows our local PoolCache to be used with the solver component
#[async_trait::async_trait]
impl qtrade_solver::PoolCache for PoolCache {
    async fn get_all_entries_as_slice(&self) -> Vec<qtrade_solver::PoolEntry> {
        info!("Getting pool entries for solver");

        // Get entries from our cache using the Cache trait implementation
        // We use get_all_entries instead of get_all_entries_as_slice because we need to
        // transform the data to match qtrade_solver::PoolEntry format
        let entries = <Self as crate::streamer::Cache<Pubkey, PoolCacheState>>::get_all_entries(self).await;

        // Map our cache entries to the format expected by qtrade_solver
        let result = entries
            .into_iter()
            .map(|(key, state)| {
                // Box the state as dyn Any + Send + Sync as required by the solver
                let boxed_state: Box<dyn Any + Send + Sync> = Box::new(state);
                (key, boxed_state)
            })
            .collect::<Vec<qtrade_solver::PoolEntry>>();

        info!("Retrieved {} pool entries for solver", result.len());
        result
    }
}
