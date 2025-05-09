use anyhow::Result;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tracing::{info, error};

// For help in naming spans
use crate::checks::QTRADE_CUSTOM_CHECKS_TRACER_NAME;
use crate::checks::QTRADE_CUSTOM_CHECKS_METER;

const CHECK_GEYSER: &str = "check_geyser";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub async fn run_check_geyser() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);

    loop {
        let span_name = format!("{}::run_check_geyser", CHECK_GEYSER);
        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            info!("Checking geyser");

            let check_geyser_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_geyser")
                .with_description("Status of geyser")
                .build();
    
            let address = "127.0.0.1:6677";
            let geyser_status = match tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(address)).await {
                Ok(Ok(_)) => 0, // Connection successful
                _ => -1,        // Connection failed
            };

            info!("Geyser status: {}", geyser_status);

            check_geyser_instrument.record(geyser_status, &[]);

            Ok(())
        }).await;

        if let Err(e) = result {
            error!("Error running check: {:?}", e);
        }

        // Wait for spe before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}
