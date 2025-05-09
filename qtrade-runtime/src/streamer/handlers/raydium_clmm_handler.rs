use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use opentelemetry::global;
use opentelemetry::metrics::ObservableCounter;
use opentelemetry::trace::Tracer;
use tracing::{debug, warn};
use yellowstone_vixen::{self as vixen};

use crate::parser::raydium_clmm::RaydiumProgramState as RaydiumClmmProgramState;

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;
use crate::QTRADE_RUNTIME_METER;
const RAYDIUM_CLMM_HANDLER: &str = "streamer::handlers::RaydiumClmmHandler";


#[derive(Debug)]
pub struct RaydiumClmmHandler {
    cache_hits: Arc<AtomicU64>,
    cache_hits_instrument: ObservableCounter<u64>
}

impl RaydiumClmmHandler {
    pub fn new() -> Self {
        let cache_hits = Arc::new(AtomicU64::new(0));
        let cache_hits_clone = Arc::clone(&cache_hits);

        let cache_hits_instrument = QTRADE_RUNTIME_METER
            .u64_observable_counter("raydium__clmm_cache_hits")
            .with_description("Records cache hits for Raydium CLMM pool events")
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

        RaydiumClmmHandler {
            cache_hits,
            cache_hits_instrument,
        }
    }
}

impl<V: std::fmt::Debug + Sync + Any> vixen::Handler<V> for RaydiumClmmHandler {
    async fn handle(&self, value: &V) -> vixen::HandlerResult<()> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::handle", RAYDIUM_CLMM_HANDLER);

        let result = tracer.in_span(span_name, |_cx| async move {
            debug!(?value);

            if let Some(raydium_program_state) = (value as &dyn Any).downcast_ref::<RaydiumClmmProgramState>() {
                match raydium_program_state {
                    RaydiumClmmProgramState::AmmConfig(amm_config) => {
                        debug!("Processing AmmConfig: {:?}", amm_config);
                    }
                    RaydiumClmmProgramState::OperationState(operation_state) => {
                        debug!("Processing OperationState: {:?}", operation_state);
                    }
                    RaydiumClmmProgramState::ObservationState(observation_state) => {
                        debug!("Processing ObservationState: {:?}", observation_state);
                    }
                    RaydiumClmmProgramState::PersonalPositionState(personal_position_state) => {
                        debug!("Processing PersonalPositionState: {:?}", personal_position_state);
                    }
                    RaydiumClmmProgramState::PoolState(keyed_pool_state) => {
                        debug!("Processing PoolState: {:?}", keyed_pool_state);
                    }
                    RaydiumClmmProgramState::ProtocolPositionState(protocol_position_state) => {
                        debug!("Processing ProtocolPositionState: {:?}", protocol_position_state);
                    }
                    RaydiumClmmProgramState::TickArrayState(tick_array_state) => {
                        debug!("Processing TickArrayState: {:?}", tick_array_state);
                    }
                    RaydiumClmmProgramState::TickArrayBitmapExtension(tick_array_bitmap_extension) => {
                        debug!("Processing TickArrayBitmapExtension: {:?}", tick_array_bitmap_extension);
                    }
                }
            } else {
                warn!("Value is not a RaydiumClmmProgramState");
            }

            Ok(())
        }).await;

        result
    }
}
