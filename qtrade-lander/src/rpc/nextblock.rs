// References
// https://docs.nextblock.io/getting-started/quickstart
// https://docs.nextblock.io/pricing-and-rate-limits
// https://nextblock.io/app/dashboard

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
use base64::Engine;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use tracing::{info, warn};

use crate::secrets::NEXTBLOCK_API_KEY;
use crate::rpc::solana::MAINNET_RPC_URL;

// For help in naming spans
use crate::constants::QTRADE_LANDER_TRACER_NAME;
const NEXTBLOCK: &str = "rpc::nextblock::Nextblock";

const NEXTBLOCK_BASE_URL: &str = "https://ny.nextblock.io";
const NEXTBLOCK_TIP_WALLET: Pubkey = pubkey!("nextBLoCkPMgmG8ZgJtABeScP35qLa2AMCNKntAP7Xc");
const NEXTBLOCK_MIN_TIP_AMOUNT: u64 = 1_000_000; // 0.001 SOL

pub struct Nextblock {
    rpc_url: String,
    tip_wallet: Pubkey,
    min_tip_amount: u64,
    http_client: Client,
    rpc_client: RpcClient,
}

impl Nextblock {
    pub fn new() -> Self {
        let rpc_url = NEXTBLOCK_BASE_URL.to_string();
        Self {
            rpc_url,
            tip_wallet: NEXTBLOCK_TIP_WALLET,
            min_tip_amount: NEXTBLOCK_MIN_TIP_AMOUNT,
            http_client: Client::new(),
            rpc_client: RpcClient::new(MAINNET_RPC_URL.to_string()),

        }
    }
}

// Note, cannot do trait RpcActions for Nextblock as it has async signature for send_tx
impl Nextblock {
    pub async fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::send_tx", NEXTBLOCK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let url = format!("{}/api/v2/submit", self.rpc_url);

            let tip_ix = system_instruction::transfer(&signer.pubkey(), &self.tip_wallet, self.min_tip_amount);
            ixs.push(tip_ix);

            let blockhash_cache = crate::blockhash::BlockhashCache::instance();
            let blockhash = match blockhash_cache.get_blockhash(&self.rpc_client) {
                Ok(hash) => hash,
                Err(e) => {
                    // Fall back to direct RPC call if cache fails
                    warn!("Failed to get cached blockhash: {}, falling back to direct RPC", e);
                    self.rpc_client.get_latest_blockhash()?
                }
            };
            let tx = Transaction::new_signed_with_payer(ixs, Some(&signer.pubkey()), &[signer], blockhash);

            // Serialize the transaction
            let serialized_tx = base64::engine::general_purpose::STANDARD.encode(bincode::serialize(&tx)?);

            let data = json!({
                "tx": serialized_tx,
            });

            info!("Sending request to: {}", url);
            info!("Request body: {}", serde_json::to_string_pretty(&data).unwrap());

            let response = self.http_client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", NEXTBLOCK_API_KEY))
                .json(&data)
                .send()
                .await?;

            let body: Value = response.json().await?;

            Ok(body["signature"].to_string())
        }).await;

        result
    }

    pub async fn send_nonce_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair, nonce_info: crate::rpc::NonceInfo<'_>) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::send_nonce_tx", NEXTBLOCK);

        let result = tracer.in_span(span_name, |_cx| async move {
            let url = format!("{}/api/v2/submit", self.rpc_url);

            // Add tip instruction
            let tip_ix = system_instruction::transfer(&signer.pubkey(), &self.tip_wallet, self.min_tip_amount);
            ixs.push(tip_ix);

            // Create the nonce advance instruction
            let nonce_advance_ix = crate::nonce::create_nonce_instruction(
                nonce_info.nonce_pubkey,
                &nonce_info.nonce_authority.pubkey()
            );

            // Add nonce instruction as the first instruction
            let mut all_ixs = vec![nonce_advance_ix];
            all_ixs.append(ixs);

            // Create and sign the transaction using the nonce
            use crate::utils::TransactionExt;
            let tx = Transaction::new_signed_with_payer_and_nonce(
                &all_ixs,
                Some(&signer.pubkey()),
                &[signer, nonce_info.nonce_authority],
                nonce_info.nonce_hash,
            );

            // Serialize the transaction
            let serialized_tx = base64::engine::general_purpose::STANDARD.encode(bincode::serialize(&tx)?);

            let data = json!({
                "tx": serialized_tx,
            });

            info!("Sending request to: {} (with nonce)", url);
            info!("Request body: {}", serde_json::to_string_pretty(&data).unwrap());

            let response = self.http_client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", NEXTBLOCK_API_KEY))
                .json(&data)
                .send()
                .await?;

            let body: Value = response.json().await?;

            Ok(body["signature"].to_string())
        }).await;

        result
    }

    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    pub fn tip_wallet(&self) -> Option<&Pubkey> {
        Some(&self.tip_wallet)
    }

    pub fn min_tip_amount(&self) -> Option<u64> {
        Some(self.min_tip_amount)
    }
}
