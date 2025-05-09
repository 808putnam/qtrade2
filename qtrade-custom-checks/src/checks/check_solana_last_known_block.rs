use anyhow::Result;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{info, error};

// For help in naming spans
use crate::checks::QTRADE_CUSTOM_CHECKS_TRACER_NAME;
use crate::checks::QTRADE_CUSTOM_CHECKS_METER;

const CHECK_SOLANA_LAST_KNOWN_BLOCK: &str = "check_solana_last_known_block";
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
const CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub async fn run_check_solana_last_known_block() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);

    loop {
        let span_name = format!("{}::run_check_solana_last_known_block", CHECK_SOLANA_LAST_KNOWN_BLOCK);
        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            info!("Checking solana last known block");

            let check_solana_last_known_block_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_solana_last_known_block")
                .with_description("Solana last known block")
                .build();

            let client = Client::builder()
                .timeout(CLIENT_TIMEOUT)
                .build()?;

            let local_response = client.post("http://localhost:8899")
                .header("Content-Type", "application/json")
                .body(r#"{"jsonrpc":"2.0","id":1,"method":"getSlot"}"#)
                .send()
                .await?
                .text()
                .await?;

            let local_slot: i64 = serde_json::from_str::<Value>(&local_response)?
                .get("result")
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Failed to parse local slot"))?;

            info!("Solana last known slot: {}", local_slot);
            check_solana_last_known_block_instrument.record(local_slot, &[]);

            Ok(())
        }).await;

        if let Err(e) = result {
            error!("Error running check: {:?}", e);
        }

        // Wait for spe before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}
