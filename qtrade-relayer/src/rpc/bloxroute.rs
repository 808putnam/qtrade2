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
use base64::Engine;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use tracing::{info, warn};

use crate::settings::RelayerSettings;
use crate::rpc::solana::MAINNET_RPC_URL;
use crate::rpc::RpcActions;

// For help in naming spans
use crate::constants::QTRADE_RELAYER_TRACER_NAME;
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
    api_key: String,
}

impl Bloxroute {
    pub fn new() -> Self {
        // For backward compatibility, load from environment
        let api_key = RelayerSettings::from_env().get_bloxroute_api_key().to_string();
        Self::with_settings(&RelayerSettings::from_env())
    }

    pub fn with_settings(settings: &RelayerSettings) -> Self {
        let rpc_url = BLOXROUTE_BASE_URL.to_string();
        Self {
            rpc_url,
            tip_wallet: BLOXROUTE_TIP_WALLET,
            min_tip_amount: BLOXROUTE_MIN_TIP_AMOUNT,
            http_client: Client::new(),
            rpc_client: RpcClient::new(MAINNET_RPC_URL.to_string()),
            api_key: settings.get_bloxroute_api_key().to_string(),
        }
    }

    // Note, cannot do trait RpcActions for Bloxroute as it has async signature for send_tx
    pub async fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_RELAYER_TRACER_NAME);
        let span_name = format!("{}::send_tx", BLOXROUTE);

        let result = tracer.in_span(span_name, |_cx| async move {
            let url = format!("{}/api/v2/submit", self.rpc_url);

            // Add the tip_ix instruction to the instructions
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
                "useStakedRPCs": false,
            });

            info!("Sending request to: {}", url);
            info!("Request body: {}", serde_json::to_string_pretty(&data).unwrap());

            let response = self.http_client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
                .json(&data)
                .send()
                .await?;

            let body: Value = response.json().await?;

            Ok(body["signature"].to_string())
        }).await;

        result
    }

    pub async fn send_nonce_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair, nonce_info: crate::rpc::NonceInfo<'_>) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_RELAYER_TRACER_NAME);
        let span_name = format!("{}::send_nonce_tx", BLOXROUTE);

        let result = tracer.in_span(span_name, |_cx| async move {
            let url = format!("{}/api/v2/submit", self.rpc_url);

            // Add the tip_ix instruction to the instructions
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
                "useStakedRPCs": false,
            });

            info!("Sending request to: {} (with nonce)", url);
            info!("Request body: {}", serde_json::to_string_pretty(&data).unwrap());

            let response = self.http_client
                .post(&url)
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", self.api_key))
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

// Implement RpcActions trait for Bloxroute
impl RpcActions for Bloxroute {
    fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        // Note: This method can't be part of the trait implementation due to the async signature
        // So we'll return an error instructing to use the async version
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Unsupported,
            "Bloxroute requires the async send_tx method. Use that instead.")))
    }

    fn send_nonce_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair, nonce_info: crate::rpc::NonceInfo) -> Result<String, Box<dyn Error>> {
        // Note: This method can't be part of the trait implementation due to the async signature
        // So we'll return an error instructing to use the async version
        Err(Box::new(std::io::Error::new(std::io::ErrorKind::Unsupported,
            "Bloxroute requires the async send_nonce_tx method. Use that instead.")))
    }

    fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    fn tip_wallet(&self) -> Option<&Pubkey> {
        Some(&self.tip_wallet)
    }

    fn min_tip_amount(&self) -> Option<u64> {
        Some(self.min_tip_amount)
    }

    fn get_api_key(&self) -> &str {
        &self.api_key
    }
}
