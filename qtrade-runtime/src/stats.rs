use anyhow::Result;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;

const STATS: &str = "stats";
const CHECK_INTERVAL: Duration = Duration::from_secs(60);


pub async fn run_stats() -> Result<()> {
    let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
    
    loop {
        let span_name = format!("{}::run_stats", STATS);

        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            // Setup timer for periodic stats management
            info!("Setting up timer for periodic stats management...");

            // Periodically check and manage stats
            info!("Checking and managing stats...");

            Ok(())
        }).await;

        // result
        if let Err(e) = result {
            error!("Error running check: {:?}", e);
        }

        // Wait for specified duration before running the check again
        sleep(CHECK_INTERVAL).await; 
    }
}
