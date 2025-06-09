use once_cell::sync::Lazy;
use opentelemetry::{
    global,
    InstrumentationScope,
    metrics::Meter};
use std::sync::Arc;

use crate::streamer::MintCache;
use crate::streamer::PoolCache;
use crate::streamer::PoolConfigCache;

pub mod parser;
pub mod settings;
pub mod streamer;

// Our one global named tracer we will use throughout the indexer
const QTRADE_INDEXER_TRACER_NAME: &str = "qtrade_indexer";
const QTRADE_INDEXER: &str = "qtrade_indexer";

pub static QTRADE_INDEXER_SCOPE: Lazy<InstrumentationScope> = Lazy::new(|| {
    InstrumentationScope::builder(QTRADE_INDEXER_TRACER_NAME)
        .with_version(env!("CARGO_PKG_VERSION"))
        .with_schema_url("https://opentelemetry.io/schemas/1.17.0")
        .build()
});

pub static QTRADE_INDEXER_METER: Lazy<Meter> = Lazy::new(|| {
    global::meter(QTRADE_INDEXER)
});

/// `xxx_CACHE` are global static variables of type `Lazy<Arc<ConcreteCache>>`, which are initialized using the
/// `once_cell` crate to ensure they are created only once and shared across the entire crate.
///
/// `Arc` stands for "Atomic Reference Counted" and is a thread-safe reference-counting pointer.
/// It allows multiple ownership of the same data by keeping track of the number of references to the data.
///
/// Use `let pool_cache = Arc::clone(&POOL_CACHE)` to create a new `Arc` pointer that points to the same `Cache` instance
/// as the global `POOL_CACHE`.
/// This does not create a new instance but rather increments the reference count,
/// allowing safe shared access to the same instance.
///
/// The local variable `pool_cache` now holds an `Arc` pointer to the same `Cache` instance,
/// which can be passed to other components or tasks that need access to the cache.
///
/// This approach ensures that the cache instances are shared and accessible in a thread-safe
/// manner across different parts of the application.
pub static MINT_CACHE: Lazy<Arc<MintCache>> = Lazy::new(|| {
    Arc::new(MintCache::new())
});
pub static POOL_CACHE: Lazy<Arc<PoolCache>> = Lazy::new(|| {
    Arc::new(PoolCache::new())
});
pub static POOL_CONFIG_CACHE: Lazy<Arc<PoolConfigCache>> = Lazy::new(|| {
    Arc::new(PoolConfigCache::new())
});



