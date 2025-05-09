use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use opentelemetry::global;
use opentelemetry::metrics::ObservableCounter;
use opentelemetry::trace::Tracer;
use tracing::{debug, warn};
use yellowstone_vixen::{self as vixen};

use crate::parser::raydium_cpmm::RaydiumProgramState as RaydiumCpmmProgramState;

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;
use crate::QTRADE_RUNTIME_METER;
const RAYDIUM_CPMM_HANDLER: &str = "streamer::handlers::RaydiumCpmmHandler";

#[derive(Debug)]
pub struct RaydiumCpmmHandler {
    cache_hits: Arc<AtomicU64>,
    cache_hits_instrument: ObservableCounter<u64>
}

impl RaydiumCpmmHandler {
    pub fn new() -> Self {
        let cache_hits = Arc::new(AtomicU64::new(0));
        let cache_hits_clone = Arc::clone(&cache_hits);

        let cache_hits_instrument = QTRADE_RUNTIME_METER
            .u64_observable_counter("raydium_cpmm_cache_hits")
            .with_description("Records cache hits for Raydium CPMM pool events")
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

        RaydiumCpmmHandler {
            cache_hits,
            cache_hits_instrument,
        }
    }
}

impl<V: std::fmt::Debug + Sync + Any> vixen::Handler<V> for RaydiumCpmmHandler {
    async fn handle(&self, value: &V) -> vixen::HandlerResult<()> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::handle", RAYDIUM_CPMM_HANDLER);

        let result = tracer.in_span(span_name, |_cx| async move {
            debug!(?value);

            if let Some(raydium_program_state) = (value as &dyn Any).downcast_ref::<RaydiumCpmmProgramState>() {
                match raydium_program_state {
                    RaydiumCpmmProgramState::AmmConfig(amm_config) => {
                        debug!("Processing AmmConfig: {:?}", amm_config);
                    }
                    RaydiumCpmmProgramState::ObservationState(observation_state) => {
                        debug!("Processing ObservationState: {:?}", observation_state);
                    }
                    RaydiumCpmmProgramState::PoolState(keyed_pool_state) => {
                        debug!("Processing PoolState: {:?}", keyed_pool_state);
                    }
                }
            } else {
                warn!("Value is not a RaydiumCpmmProgramState");
            }

            Ok(())
        }).await;

        result
    }
}
