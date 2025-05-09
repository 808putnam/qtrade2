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
use opentelemetry::trace::{Span, Tracer};

use crate::rpc::RpcActions;
use crate::secrets::HELIUS_API_KEY;

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;
const HELIUS: &str = "rpc::helius::Helius";

const HELIUS_BASE_URL: &str = "https://mainnet.helius-rpc.com/?api-key=";

pub struct Helius {
    rpc_url: String,
    rpc_client: RpcClient,
}

impl Helius {
    pub fn new() -> Self {
        let rpc_url = format!("{}{}", HELIUS_BASE_URL, HELIUS_API_KEY);
        Self {
            rpc_client: RpcClient::new(rpc_url.clone()),
            rpc_url,
        }
    }
}

impl RpcActions for Helius {
    fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::send_tx", HELIUS);

        let result = tracer.in_span(span_name, move |_cx| {
            let blockhash = self.rpc_client.get_latest_blockhash()?;
            let tx = Transaction::new_signed_with_payer(ixs, Some(&signer.pubkey()), &[signer], blockhash);

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