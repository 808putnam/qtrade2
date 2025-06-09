// References
// https://docs.helius.dev/
// https://docs.helius.dev/solana-rpc-nodes/helius-rpcs-overview
// https://docs.helius.dev/welcome/pricing-and-rate-limits
// https://dashboard.helius.dev/dashboard?projectId=6ba16f26-1234-42ee-a17c-735645fc1995

use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use solana_sdk::transaction::Transaction;
use std::error::Error;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use tracing::warn;

use crate::rpc::RpcActions;
use crate::settings::RelayerSettings;

// For help in naming spans
use crate::constants::QTRADE_RELAYER_TRACER_NAME;
const HELIUS: &str = "rpc::helius::Helius";

const HELIUS_BASE_URL: &str = "https://mainnet.helius-rpc.com/?api-key=";

pub struct Helius {
    rpc_url: String,
    rpc_client: RpcClient,
}

impl Helius {
    pub fn new() -> Self {
        // For backward compatibility
        Self::with_settings(&RelayerSettings::from_env())
    }

    pub fn with_settings(settings: &RelayerSettings) -> Self {
        let rpc_url = format!("{}{}", HELIUS_BASE_URL, settings.get_helius_api_key());
        Self {
            rpc_client: RpcClient::new(rpc_url.clone()),
            rpc_url,
        }
    }

    pub fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }
}

impl RpcActions for Helius {
    fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_RELAYER_TRACER_NAME);
        let span_name = format!("{}::send_tx", HELIUS);

        let result = tracer.in_span(span_name, move |_cx| {
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

            let signature = self.rpc_client.send_transaction(&tx)?;

            Ok(signature.to_string())
        });

        result
    }

    fn send_nonce_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair, nonce_info: crate::rpc::NonceInfo) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_RELAYER_TRACER_NAME);
        let span_name = format!("{}::send_nonce_tx", HELIUS);

        let result = tracer.in_span(span_name, move|_cx| {
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

            let signature = self.rpc_client.send_transaction(&tx)?;
            Ok(signature.to_string())
        });

        result
    }

    fn simulate_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_RELAYER_TRACER_NAME);
        let span_name = format!("{}::simulate_tx", HELIUS);

        let result = tracer.in_span(span_name, move|_cx| {
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

            // Use the Helius RPC client to simulate the transaction
            use solana_client::rpc_request::RpcRequest;
            use solana_sdk::commitment_config::CommitmentConfig;

            // Serialize and encode the transaction for RPC
            let serialized_encoded = bs58::encode(bincode::serialize(&tx).unwrap()).into_string();

            // Send the simulation request
            let simulation_result: serde_json::Value = self.rpc_client.send(
                RpcRequest::SimulateTransaction,
                serde_json::json!([serialized_encoded, {
                    "sigVerify": true,
                    "commitment": CommitmentConfig::confirmed().commitment,
                    "encoding": "jsonParsed",
                }]),
            )?;

            // Format the simulation result as a JSON string
            let pretty_result = serde_json::to_string_pretty(&simulation_result)?;
            Ok(pretty_result)
        });

        result
    }

    fn rpc_client(&self) -> &RpcClient {
        &self.rpc_client
    }

    fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    fn tip_wallet(&self) -> Option<&Pubkey> {
        None
    }

    fn min_tip_amount(&self) -> Option<u64> {
        None
    }

    fn get_api_key(&self) -> &str {
        // Extract the API key from the RPC URL
        self.rpc_url.strip_prefix(HELIUS_BASE_URL).unwrap_or_default()
    }
}
