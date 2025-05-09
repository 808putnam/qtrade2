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

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;
const SOLANA: &str = "rpc::solana::Solana";

pub const MAINNET_RPC_URL: &str = "https://api.mainnet-beta.solana.com";
const TESTNET_RPC_URL: &str = "https://api.testnet.solana.com";
const DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";
const LOCAL_RPC_URL: &str = "http://127.0.0.1:8899";

pub enum SolanaEndpoint {
    Mainnet,
    Testnet,
    Devnet,
    Local,
}

pub struct Solana {
    rpc_url: String,
    rpc_client: RpcClient,
}

impl Solana {
    pub fn new(endpoint: SolanaEndpoint) -> Self {
        let rpc_url = match endpoint {
            SolanaEndpoint::Mainnet => MAINNET_RPC_URL.to_string(),
            SolanaEndpoint::Testnet => TESTNET_RPC_URL.to_string(),
            SolanaEndpoint::Devnet => DEVNET_RPC_URL.to_string(),
            SolanaEndpoint::Local => LOCAL_RPC_URL.to_string(),
        };
        Self {
            rpc_client: RpcClient::new(rpc_url.clone()),
            rpc_url,
        }
    }
}

impl RpcActions for Solana {
    fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::send_tx", SOLANA);

        let result = tracer.in_span(span_name, move|_cx| {
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
