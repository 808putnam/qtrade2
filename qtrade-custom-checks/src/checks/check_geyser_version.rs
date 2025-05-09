use anyhow::Result;
use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry::trace::Tracer;
use regex::Regex;
use std::fs;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};

// For help in naming spans
use crate::checks::QTRADE_CUSTOM_CHECKS_TRACER_NAME;
use crate::checks::QTRADE_CUSTOM_CHECKS_METER;

const CHECK_GEYSER_VERSION: &str = "check_geyser_version";
const CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub async fn run_check_geyser_version() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);
    let paths = [
        "/home/ubuntu/dev/yellowstone-grpc/yellowstone-grpc-geyser/Cargo.toml"
    ];

    loop {
        let span_name = format!("{}::run_check_geyser_version", CHECK_GEYSER_VERSION);
        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            info!("Checking geyser version");

            let check_geyser_version_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_geyser_version")
                .with_description("Geyser version")
                .build();

            let mut file_path = None;
            for path in &paths {
                if fs::metadata(path).is_ok() {
                    file_path = Some(path);
                    break;
                }
            }

            if let Some(path) = file_path {
                match fs::read_to_string(path) {
                    Ok(contents) => {
                        let re = Regex::new(r#"name = "yellowstone-grpc-geyser"\nversion = "([\d.]+)""#).unwrap();
                        if let Some(caps) = re.captures(&contents.to_string()) {
                            let version = &caps[1];

                            info!("Geyser version found: {}", version);
                            check_geyser_version_instrument.record(1, &[KeyValue::new("geyser:version", version.to_string())]);

                        } else {
                            error!("Geyser version not found in Cargo.toml");
                            check_geyser_version_instrument.record(-1, &[KeyValue::new("geyser:version", "version not found".to_string())]);
                        }
                    }
                    Err(e) => {
                        error!("Error reading Cargo.toml for Geyser: {:?}", e);
                        check_geyser_version_instrument.record(0, &[KeyValue::new("geyser:version", "command execution error".to_string())]);
                    }
                }
            } else {
                error!("'Cargo.toml' file not found for Geyser. Please make sure it is in the correct path.");
                check_geyser_version_instrument.record(0, &[KeyValue::new("geyser:version", "cargo.toml file not found".to_string())]);
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
