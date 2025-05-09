use anyhow::Result;
use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry::trace::Tracer;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::sleep;
use tracing::{info, error};

// For help in naming spans
use crate::checks::QTRADE_CUSTOM_CHECKS_TRACER_NAME;
use crate::checks::QTRADE_CUSTOM_CHECKS_METER;

const CHECK_SOLANA_PORT_CONN: &str = "check_solana_port_conn";
const CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub async fn run_check_solana_port_conn() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);

    loop {
        let span_name = format!("{}::run_check_solana_port_conn", CHECK_SOLANA_PORT_CONN);
        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            info!("Checking solana port conn");
    
            let check_solana_port_conn_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_solana_port_conn")
                .with_description("Solana port connections")
                .build();

            let ports_tags = vec![
                (6677, "port.6677_geyser"),
                (8899, "port.8899_rpc"),
                (9000, "port.9000_tpu"),
            ];

            for (port, tag) in ports_tags {
                match check_port_connections(port).await {
                    Ok(count) => {
                        info!("Active connections for {}: {}", tag, count);
                        check_solana_port_conn_instrument.record(count as i64, &[KeyValue::new("port", tag.to_string())]);
                    }
                    Err(e) => {
                        error!("Error checking port {}: {:?}", port, e);
                    }
                }
            }

            Ok(())
        }).await;

        if let Err(e) = result {
            error!("Error running check: {:?}", e);
        }

        // Wait for spe before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}

async fn check_port_connections(port: u16) -> Result<usize> {
    let output = Command::new("netstat")
        .arg("-na")
        .output()
        .await?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to run netstat command"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let count = stdout
        .lines()
        .filter(|line| line.contains("EST") && line.contains(&port.to_string()))
        .count();

    Ok(count)
}