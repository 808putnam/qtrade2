use anyhow::Result;
use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry::trace::Tracer;
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};

// For help in naming spans
use crate::checks::QTRADE_CUSTOM_CHECKS_TRACER_NAME;
use crate::checks::QTRADE_CUSTOM_CHECKS_METER;

const CHECK_SOLANA_VERSION_DISTRIBUTION: &str = "check_solana_version_distribution";
const CHECK_INTERVAL: Duration = Duration::from_secs(3600); // 1 hour

pub async fn run_solana_version_distribution() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);

    loop {
        let span_name = format!("{}::run_check_solana_version_distribution", CHECK_SOLANA_VERSION_DISTRIBUTION);
        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            info!("Checking solana version distribution");

            let check_solana_version_distribution_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_solana_version_distribution")
                .with_description("Solana version distribution")
                .build();

            // The Solana CLI command to execute
            let command = "solana gossip | cut -d '|' -f7 | sort -n | uniq -c | sort -nr | grep -v 'unknown' | head -10";

            // Execute the command
            let output = Command::new("sh")
                .arg("-c")
                .arg(command)
                .output()
                .expect("Failed to execute command");

            // Capture the output and error
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if !stderr.is_empty() {
                error!("Command Error for {}: {}", CHECK_SOLANA_VERSION_DISTRIBUTION, stderr);
                return Ok(());
            }

            for line in stdout.lines() {
                let parts: Vec<&str> = line.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    let count: i32 = parts[0].parse().unwrap_or(0);
                    let version = parts[1];
                    info!("Version: {}, Count: {}", version, count);
                    check_solana_version_distribution_instrument.record(count as i64, &[KeyValue::new("solana:version", version.to_string())]);
                }
            }

            Ok(())
        }).await;

        if let Err(e) = result {
            error!("Error running check {}: {:?}", CHECK_SOLANA_VERSION_DISTRIBUTION, e);
        }

        // Wait for spe before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}
