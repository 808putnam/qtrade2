use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use opentelemetry::global;
use opentelemetry::metrics::ObservableCounter;
use opentelemetry::trace::Tracer;
use tracing::{debug, warn};

use yellowstone_vixen::{self as vixen};

use crate::POOL_CACHE;
use crate::parser::orca::OrcaProgramState;
use crate::streamer::Cache;
use crate::streamer::PoolCacheState;

// For help in naming spans
use crate::QTRADE_STREAMER_TRACER_NAME;
use crate::QTRADE_STREAMER_METER;
const ORCA_HANDLER: &str = "streamer::handlers::OrcaHandler";

#[derive(Debug)]
pub struct OrcaHandler {
    cache_hits: Arc<AtomicU64>,
    cache_hits_instrument: ObservableCounter<u64>
}

impl OrcaHandler {
    pub fn new() -> Self {
        let cache_hits = Arc::new(AtomicU64::new(0));
        let cache_hits_clone = Arc::clone(&cache_hits);

        let cache_hits_instrument = QTRADE_STREAMER_METER
            .u64_observable_counter("orca_cache_hits")
            .with_description("Records cache hits for Orca pool events")
            .with_unit("hits/minute")
            .with_callback(move |observer| {
                // Load the current value of cache_hits
                let hits = cache_hits_clone.load(Ordering::Relaxed);
                // Observe the current value
                observer.observe(hits, &[]);
                // Reset cache_hits to 0
                cache_hits_clone.store(0, Ordering::Relaxed);
            })
            .build();

        OrcaHandler {
            cache_hits,
            cache_hits_instrument,
        }
    }
}

impl<V: std::fmt::Debug + Sync + Any> vixen::Handler<V> for OrcaHandler {
    async fn handle(&self, value: &V) -> vixen::HandlerResult<()> {
        let tracer = global::tracer(QTRADE_STREAMER_TRACER_NAME);
        let span_name = format!("{}::handle", ORCA_HANDLER);

        let result = tracer.in_span(span_name, |_cx| async move {
            debug!(?value);

            if let Some(orca_program_state) = (value as &dyn Any).downcast_ref::<OrcaProgramState>() {
                match orca_program_state {
                    OrcaProgramState::Whirlpool(keyed_whirlpool) => {
                        // No need to clone since POOL_CACHE is already an Arc
                        // Use the Cache trait method directly
                        let pool_cache_state = PoolCacheState::OrcaPoolState(keyed_whirlpool.clone());
                        POOL_CACHE.update_cache(keyed_whirlpool.pubkey, pool_cache_state).await;

                        // Increment cache_hits safely
                        self.cache_hits.fetch_add(1, Ordering::Relaxed);
                    },
                    OrcaProgramState::WhirlpoolsConfig(whirlpools_config) => {
                        debug!("Processing whirlpool config: {:?}", whirlpools_config);
                    },
                    OrcaProgramState::FeeTier(fee_tier) => {},
                    OrcaProgramState::Position(position) => {},
                    OrcaProgramState::TickArray(tick_array) => {},
                }
            } else {
                warn!("Value is not a OrcaProgramState");
            }

            Ok(())
        }).await;

        result
    }
}
