// References:
// https://use.temporal.xyz/nozomi/transaction-submission

use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use solana_sdk::system_instruction;
use solana_sdk::transaction::Transaction;
use std::error::Error;
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};

use crate::rpc::RpcActions;
use crate::secrets::TEMPORAL_API_KEY;

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;
const TEMPORAL: &str = "rpc::temporal::Temporal";

const TEMPORAL_BASE_URL: &str = "http://nozomi-preview-pit.temporal.xyz/?c=";
const TEMPORAL_TIP_WALLET: Pubkey = pubkey!("TEMPaMeCRFAS9EKF53Jd6KpHxgL47uWLcpFArU1Fanq");
const TEMPORAL_MIN_TIP_AMOUNT: u64 = 1_000_000; // 0.001 SOL

pub struct Temporal {
    rpc_url: String,
    tip_wallet: Pubkey,
    min_tip_amount: u64,
    rpc_client: RpcClient,
}

impl Temporal {
    pub fn new() -> Self {
        let rpc_url = format!("{}{}", TEMPORAL_BASE_URL, TEMPORAL_API_KEY);
        Self {
            rpc_client: RpcClient::new(rpc_url.clone()),
            rpc_url,
            tip_wallet: TEMPORAL_TIP_WALLET,
            min_tip_amount: TEMPORAL_MIN_TIP_AMOUNT,
        }
    }
}

impl RpcActions for Temporal {
    fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::send_tx", TEMPORAL);

        let result = tracer.in_span(span_name, move |_cx| {
            let tip_ix = system_instruction::transfer(&signer.pubkey(), &self.tip_wallet, self.min_tip_amount);
            ixs.push(tip_ix);

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
        Some(&self.tip_wallet)
    }

    fn min_tip_amount(&self) -> Option<u64> {
        Some(self.min_tip_amount)
    }
}