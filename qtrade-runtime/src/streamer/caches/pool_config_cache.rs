use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
// qtrade: from raydium_clmm, account_helper.rs
use spl_pod::solana_pubkey::Pubkey;
use opentelemetry::global;
use opentelemetry::trace::Tracer;

use crate::parser::orca::KeyedWhirlpoolsConfig as OrcaKeyedWhirlpoolsConfig;
use crate::streamer::Cache;

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;
const POOL_CONFIG_CACHE: &str = "streamer::caches::PoolConfigCache";

// Notes:
// 1. If you add more enum variants, make sure the enum fields all #[derive Clone]
// 2. The large_enum_variant is a lint from the Clippy tool in Rust. It warns when
//    an enum variant is significantly larger than the other variants. This can lead
//    to inefficient memory usage because the size of an enum is determined by its largest variant.
//    To fix this, you can consider using Box to store large data on the heap or refactor the enum
//    to reduce the size of its variants.
//    Here's an example:
//    ```rust
//    enum Example {
//        SmallVariant,
//        LargeVariant(Box<LargeStruct>), // Use Box to store large data on the heap
//    }
//    ```
// This way, the enum itself remains small, and the large data is stored on the heap.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub enum PoolConfigCacheState {
    OrcaPoolConfigState(OrcaKeyedWhirlpoolsConfig),
}

// Reference:
// https://draft.ryhl.io/blog/shared-mutable-state/
#[derive(Clone)]
pub struct PoolConfigCache {
    inner: Arc<RwLock<PoolConfigCacheInner>>
}

struct PoolConfigCacheInner {
    data: DashMap<Pubkey, PoolConfigCacheState>,
}

impl PoolConfigCache {
    // Keep the constructor, but not as part of the Cache trait
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(PoolConfigCacheInner {
                data: DashMap::new(),
            }))
        }
    }
}

impl Cache<Pubkey, PoolConfigCacheState> for PoolConfigCache {

    async fn get_all_entries(&self) -> Vec<(Pubkey, PoolConfigCacheState)> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::get_all_entries", POOL_CONFIG_CACHE);

        let result = tracer.in_span(span_name, |_cx| async move {
            // We add a block here to:
            // 1. Make sure not to hold RwLockReadGuard across await points
            // 2. Make sure not to hold any reference to dashmap
            let cache_result = {
                let cache_read = self.inner.read().await;
                cache_read.data.iter().map(|entry| (*entry.key(), entry.value().clone())).collect()
            };

            cache_result
        }).await;

        result
    }

    /// The `get_all_entries_as_slice` function is an asynchronous method that retrieves
    /// all entries from the `PoolConfigCache` and returns them as a boxed slice (`Box<[(Pubkey, PoolConfigCacheState)]>`).
    /// It ensures that the `RwLockReadGuard` is not held across await points by limiting
    /// its scope within a block. The function iterates over the entries in the `DashMap`,
    /// clones each key and value, collects them into a `Vec`, and then converts the `Vec`
    /// into a boxed slice.
    ///
    /// Notes:
    /// 1. When a Box goes out of scope, its memory is automatically deallocated.
    /// 2. Therefore, the user of the get_all_entries_as_slice function does not need to manually free the Box. It will be freed when it is no longer needed.
    /// 3. Here is an example of how a user might call the get_all_entries_as_slice function and use the returned Box:
    ///
    /// ```rust
    /// #[tokio::main]
    /// async fn main() {
    ///     let pool_config_cache = PoolConfigCache::new();
    ///
    ///     // Populate the cache with some data (example)
    ///     let pubkey = Pubkey::new_unique();
    ///     let state = PoolConfigCacheState::OrcaPoolState(OrcaKeyedWhirlpoolsConfig { /* fields */ });
    ///     pool_config_cache.inner.write().await.data.insert(pubkey, state);
    ///
    ///     // Get all entries as a slice
    ///     let entries: Box<[(Pubkey, PoolConfigCacheState)]> = pool_config_cache.get_all_entries_as_slice().await;
    ///
    ///     // Use the entries
    ///     for (key, value) in entries.iter() {
    ///         println!("{:?}: {:?}", key, value);
    ///     }
    ///
    ///     // The Box will be automatically freed when it goes out of scope
    /// }
    /// ```
    async fn get_all_entries_as_slice(&self) -> Box<[(Pubkey, PoolConfigCacheState)]> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::get_all_entries_as_slice", POOL_CONFIG_CACHE);

        let result = tracer.in_span(span_name, |_cx| async move {
            // We add a block here to:
            // 1. Make sure not to hold RwLockReadGuard across await points
            // 2. Make sure not to hold any reference to dashmap
            let cache_result = {
                let cache_read = self.inner.read().await;
                let cache_result: Vec<(Pubkey, PoolConfigCacheState)> = cache_read.data.iter().map(|entry| (*entry.key(), entry.value().clone())).collect();
                cache_result.into_boxed_slice()
            };

            cache_result
        }).await;

        result
    }

    async fn read_cache(&self, key: &Pubkey) -> Option<PoolConfigCacheState> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::read_cache", POOL_CONFIG_CACHE);

        let result = tracer.in_span(span_name, |_cx| async move {
            // We add a block here to:
            // 1. Make sure not to hold RwLockReadGuard across await points
            // 2. Make sure not to hold any reference to dashmap
            let cache_result = {
                let cache_read = self.inner.read().await;
                let cache_entry = cache_read.data.get(key);
                match cache_entry {
                    Some(cache_entry) => {
                        let value = cache_entry.value().clone();
                        Some(value)
                    }
                    None => None
                }
            };

            cache_result
        }).await;

        result
    }

    async fn update_cache(&self, key: Pubkey, value: PoolConfigCacheState) -> Option<PoolConfigCacheState> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::update_cache", POOL_CONFIG_CACHE);

        let result = tracer.in_span(span_name, |_cx| async move {
            // We add a block here to:
            // 1. Make sure not to hold RwLockWriteGuard across await points
            // 2. Make sure not to hold any reference to dashmap
            let cache_result = {
                let cache_write = self.inner.write().await;
                cache_write.data.insert(key, value)
            };

            cache_result
        }).await;

        result
    }

    async fn remove_cache(&self, key: Pubkey) -> Option<(Pubkey, PoolConfigCacheState)> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::remove_cache", POOL_CONFIG_CACHE);

        let result = tracer.in_span(span_name, |_cx| async move {
            // We add a block here to:
            // 1. Make sure not to hold RwLockWriteGuard across await points
            // 2. Make sure not to hold any reference to dashmap
            let cache_result = {
                let cache_write = self.inner.write().await;
                cache_write.data.remove(&key)
            };

            cache_result
        }).await;

        result
    }
}
