// References
// https://www.quicknode.com/docs/solana
// https://dashboard.quicknode.com/

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
use crate::secrets::QUICKNODE_API_KEY;

// For help in naming spans
use crate::constants::QTRADE_LANDER_TRACER_NAME;
const QUICKNODE: &str = "rpc::quicknode::Quicknode";

const QUICKNODE_BASE_URL: &str = "https://broken-blue-shadow.solana-mainnet.quiknode.pro/";

pub struct Quicknode {
    rpc_url: String,
    rpc_client: RpcClient,
}

impl Quicknode {
    pub fn new() -> Self {
        let rpc_url = format!("{}{}", QUICKNODE_BASE_URL, QUICKNODE_API_KEY);
        Self {
            rpc_client: RpcClient::new(rpc_url.clone()),
            rpc_url,
        }
    }
}

impl RpcActions for Quicknode {
    fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::send_request", QUICKNODE);

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
        let tracer = global::tracer(QTRADE_LANDER_TRACER_NAME);
        let span_name = format!("{}::send_nonce_tx", QUICKNODE);

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
}
