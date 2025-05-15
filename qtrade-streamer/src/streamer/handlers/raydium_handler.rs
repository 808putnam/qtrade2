use std::any::Any;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use opentelemetry::global;
use opentelemetry::metrics::ObservableCounter;
use opentelemetry::trace::Tracer;
use tracing::{debug, warn};
use yellowstone_vixen::{self as vixen};

use crate::parser::raydium::RaydiumProgramState;

// For help in naming spans
use crate::QTRADE_STREAMER_TRACER_NAME;
use crate::QTRADE_STREAMER_METER;
const RAYDIUM_HANDLER: &str = "streamer::handlers::RaydiumHandler";

#[derive(Debug)]
pub struct RaydiumHandler {
    cache_hits: Arc<AtomicU64>,
    cache_hits_instrument: ObservableCounter<u64>
}

impl RaydiumHandler {
    pub fn new() -> Self {
        let cache_hits = Arc::new(AtomicU64::new(0));
        let cache_hits_clone = Arc::clone(&cache_hits);

        let cache_hits_instrument = QTRADE_STREAMER_METER
            .u64_observable_counter("raydium_cache_hits")
            .with_description("Records cache hits for Raydium pool events")
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

        RaydiumHandler {
            cache_hits,
            cache_hits_instrument,
        }
    }
}

impl<V: std::fmt::Debug + Sync + Any> vixen::Handler<V> for RaydiumHandler {
    async fn handle(&self, value: &V) -> vixen::HandlerResult<()> {
        let tracer = global::tracer(QTRADE_STREAMER_TRACER_NAME);
        let span_name = format!("{}::handle", RAYDIUM_HANDLER);

        let result = tracer.in_span(span_name, |_cx| async move {
            debug!(?value);

            if let Some(raydium_program_state) = (value as &dyn Any).downcast_ref::<RaydiumProgramState>() {
                match raydium_program_state {
                    RaydiumProgramState::AmmInfo(keyed_amm_info) => {
                        debug!("Processing AmmInfo: {:?}", keyed_amm_info);
                    }
                    _ => {}
                }
            } else {
                warn!("Value is not a RaydiumProgramState");
            }

            Ok(())
        }).await;

        result
    }
}
