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

const CHECK_SOLANA_PORT_CONN_COUNT: &str = "check_solana_port_conn_count";
const CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub async fn run_check_solana_port_conn_count() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);

    loop {
        let span_name = format!("{}::run_check_solana_port_conn_count", CHECK_SOLANA_PORT_CONN_COUNT);
        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            info!("Checking Solana port conn count");

            let check_solana_port_conn_count_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_solana_port_conn_count")
                .with_description("Solana port connection counts")
                .build();

            let ports_tags = vec![
                ("8899", "solana_port_8899"),
                ("6677", "solana_port_6677"),
            ];

            for (port, tag) in ports_tags {
                let command = format!("netstat -na | grep EST | grep {} | awk '{{print $5}}' | cut -d: -f1 | sort | uniq -c | sort -nr", port);
                match Command::new("sh").arg("-c").arg(&command).output() {
                    Ok(output) => {
                        if !output.status.success() {
                            error!("Error running command: {}", command);
                            continue;
                        }

                        let output_str = String::from_utf8_lossy(&output.stdout);
                        for line in output_str.lines() {
                            let parts: Vec<&str> = line.trim().split_whitespace().collect();
                            if parts.len() == 2 {
                                if let Ok(client_connection_count) = parts[0].parse::<i32>() {
                                    let clientip = parts[1];
                                    info!(
                                        "solana_node_conn_count: {} tags: type:{}, clientip:{}, client_connection_count:{}",
                                        client_connection_count, tag, clientip, client_connection_count
                                    );
                                    check_solana_port_conn_count_instrument.record(client_connection_count as i64, &[KeyValue::new("port", tag.to_string())]);

                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error running SolanaNodeConnCountCheck on port {}: {:?}", port, e);
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
