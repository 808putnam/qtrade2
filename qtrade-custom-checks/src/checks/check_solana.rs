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

const CHECK_SOLANA: &str = "check_solana";
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
const CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub async fn run_check_solana() -> Result<()> {
    let tracer = global::tracer(QTRADE_CUSTOM_CHECKS_TRACER_NAME);

    loop {
        let span_name = format!("{}::run_check_solana", CHECK_SOLANA);
        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            info!("Checking solana");

            let check_solana_instrument = QTRADE_CUSTOM_CHECKS_METER
                .i64_gauge("check_solana")
                .with_description("Solana sync status")
                .build();

            let client = Client::builder()
                .timeout(CLIENT_TIMEOUT) 
                .build()?;

            let local_response = client.post("http://localhost:8899")
                .header("Content-Type", "application/json")
                .body(r#"{"jsonrpc":"2.0","id":1,"method":"getEpochInfo"}"#)
                .send()
                .await?
                .text()
                .await?;

            let mainnet_response = client.post("https://api.mainnet-beta.solana.com/")
                .header("Content-Type", "application/json")
                .body(r#"{"method":"getEpochInfo","jsonrpc":"2.0","params":[],"id":"61f06520-d9e8-4040-973b-84f8e57e3aca"}"#)
                .send()
                .await?
                .text()
                .await?;

            let local_block_height: i64 = serde_json::from_str::<Value>(&local_response)?
                .get("result")
                .and_then(|v| v.get("blockHeight"))
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Failed to parse local block height"))?;

            let mainnet_block_height: i64 = serde_json::from_str::<Value>(&mainnet_response)?
                .get("result")
                .and_then(|v| v.get("blockHeight"))
                .and_then(|v| v.as_i64())
                .ok_or_else(|| anyhow::anyhow!("Failed to parse mainnet block height"))?;

            let sync_status = if (mainnet_block_height - 5) <= local_block_height {
                0
            } else {
                mainnet_block_height - local_block_height
            };

            info!("Solana sync status: {}", sync_status);
            check_solana_instrument.record(sync_status, &[]);

            Ok(())
        }).await;

        if let Err(e) = result {
            error!("Error running check: {:?}", e);
        }

        // Wait for specified duration before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}
