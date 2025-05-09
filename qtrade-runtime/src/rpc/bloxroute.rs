// References
// https://docs.bloxroute.com/solana/trader-api-v2/api-endpoints/general/submit-signed-transaction
// https://portal.bloxroute.com/details

use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use std::error::Error;

use reqwest::Client;
use serde_json::{json, Value};
use base64;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use tracing::info;

use crate::secrets::BLOXROUTE_API_KEY;
use crate::rpc::solana::MAINNET_RPC_URL;

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;
const BLOXROUTE: &str = "rpc::bloxroute::Bloxroute";

const BLOXROUTE_BASE_URL: &str = "https://ny.solana.dex.blxrbdn.com";
const BLOXROUTE_TIP_WALLET: Pubkey = pubkey!("HWEoBxYs7ssKuudEjzjmpfJVX7Dvi7wescFsVx2L5yoY");
const BLOXROUTE_MIN_TIP_AMOUNT: u64 = 1_000_000; // 0.001 SOL

pub struct Bloxroute {
    rpc_url: String,
    tip_wallet: Pubkey,
    min_tip_amount: u64,
    http_client: Client,
    rpc_client: RpcClient,
}

impl Bloxroute {
    pub fn new() -> Self {
        let rpc_url = BLOXROUTE_BASE_URL.to_string();
        Self {
            rpc_url,
            tip_wallet: BLOXROUTE_TIP_WALLET,
            min_tip_amount: BLOXROUTE_MIN_TIP_AMOUNT,
            http_client: Client::new(),
            rpc_client: RpcClient::new(MAINNET_RPC_URL.to_string()),
        }
    }

    // Note, cannot do trait RpcActions for Nextblock as it has async signature for send_tx
    pub async fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::send_tx", BLOXROUTE);

        let result = tracer.in_span(span_name, |_cx| async move {
            let url = format!("{}/api/v2/submit", self.rpc_url);

            // Add the tip_ix instruction to the instructions
            let tip_ix = system_instruction::transfer(&signer.pubkey(), &self.tip_wallet, self.min_tip_amount);
            ixs.push(tip_ix);

            let blockhash = self.rpc_client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(ixs, Some(&signer.pubkey()), &[signer], blockhash);

            // Serialize the transaction
            let serialized_tx = base64::encode(bincode::serialize(&tx)?);

            let data = json!({
                "transaction": { "content": serialized_tx },
                "frontRunningProtection": false,
                "useStakedRPCs": false,
            });

            info!("Sending request to: {}", url);
            info!("Request body: {}", serde_json::to_string_pretty(&data).unwrap());

            let response = self.http_client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", BLOXROUTE_API_KEY)
                .json(&data)
                .send()
                .await?;

            let status = response.status();
            info!("Response status: {}", status);

            let body = response.json::<Value>().await?;

            Ok(body["signature"].to_string())
        }).await;

        result

    }
}