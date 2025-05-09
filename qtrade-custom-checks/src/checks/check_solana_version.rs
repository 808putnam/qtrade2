use anyhow::Result;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use opentelemetry::KeyValue;
use regex::Regex;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::process::Command;
use tokio::time::sleep;
use tracing::{info, error};

// For help in naming spans
use crate::checks::QTRADE_CUSTOM_CHECKS_TRACER_NAME;
use crate::checks::QTRADE_CUSTOM_CHECKS_METER;

const CHECK_SOLANA_VERSION: &str = "check_solana_version";
const CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub async fn run_check_solana_version() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);

    loop {
        let span_name = format!("{}::run_check_solana_version", CHECK_SOLANA_VERSION);
        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            info!("Checking solana version");

            let check_solana_version_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_solana_version")
                .with_description("Solana version")
                .build();

            // Run the solana -V command
            let output = Command::new("solana")
                .arg("-V")
                .output()
                .await?;

            if !output.status.success() {
                error!("Failed to execute solana command");
                return Err(anyhow::anyhow!("Failed to execute solana command"));
            }

            let output_str = String::from_utf8_lossy(&output.stdout);
            let re = Regex::new(r"solana-cli ([\d.]+)").unwrap();
            if let Some(caps) = re.captures(&output_str) {
                let version = caps.get(1).map_or("", |m| m.as_str());
                info!("Solana version: {}", version);
                check_solana_version_instrument.record(1, &[KeyValue::new("solana:version", version.to_string())]);
            } else {
                error!("Failed to parse solana version");
                check_solana_version_instrument.record(-1, &[KeyValue::new("solana:version", "version not found")]);
                return Err(anyhow::anyhow!("Failed to parse solana version"));
            }

            Ok(())
        }).await;

        if let Err(e) = result {
            error!("Error running check: {:?}", e);
        }

        // Wait for specified duration before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}
