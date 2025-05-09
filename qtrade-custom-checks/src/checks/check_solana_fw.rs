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

const CHECK_SOLANA_FW: &str = "check_solana_fw";
// const CHECK_INTERVAL: Duration = Duration::from_secs(86400); // 1 day
const CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub async fn run_check_solana_fw() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);

    loop {
        let span_name = format!("{}::run_check_solana_fw", CHECK_SOLANA_FW);
        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            info!("Checking solana fw");

            let check_solana_fw_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_solana_fw")
                .with_description("Solana firewall")
                .build();

            /* leaving this here for reference
            let commands = vec![
                ("anywhere", "sudo ufw status | grep Anywhere | wc -l"),
                ("SolanaSpecificRules", "sudo ufw status | grep \"Solana Specific Rules\" | wc -l"),
                ("GenericUFWRules", "sudo ufw status | grep \"Generic UFW Rules\" | wc -l"),
                ("other", "sudo ufw status | grep ALLOW | grep -v \"Generic UFW Rules\" | grep -v \"Solana Specific Rules\" | wc -l"),
            ];
            */
            let commands = vec![
                ("anywhere", "sudo ufw status | grep Anywhere | wc -l"),
                ("SolanaSpecificRules", "sudo ufw status | grep \"Solana Specific Rules\" | wc -l"),
                ("SolanaUDPSpecificRules", "sudo ufw status | grep \"Solana udp Specific Rules\" | wc -l"),
                ("SolanaTCPSpecificRules", "sudo ufw status | grep \"Solana tcp Specific Rules\" | wc -l"),
                ("other", "sudo ufw status | grep ALLOW | grep -v \"Solana udp Specific Rules\" | grep -v \"Solana tcp Specific Rules\" | grep -v \"Solana Specific Rules\" | wc -l"),
            ];

            for (rule_type, command) in commands {
                match Command::new("sh").arg("-c").arg(command).output() {
                    Ok(output) => {
                        if !output.stderr.is_empty() {
                            error!("Command Error for {}: {}", rule_type, String::from_utf8_lossy(&output.stderr));
                            continue;
                        }

                        let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        match output_str.parse::<i32>() {
                            Ok(rule_count) => {
                                info!("UFW rule count for {}: {}", rule_type, rule_count);
                                check_solana_fw_instrument.record(rule_count as i64, &[KeyValue::new("solana.firewall", rule_type.to_string())]);

                                // Here you would send the gauge metric, e.g., using a telemetry library
                                // self.gauge(format!("solana.firewall.{}", rule_type.to_lowercase()), rule_count);
                            }
                            Err(_) => {
                                error!("Failed to parse output to integer for {}: {}", rule_type, output_str);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error running UFW command for {}: {:?}", rule_type, e);
                    }
                }
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
