use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::Keypair;
use solana_sdk::pubkey::Pubkey;
use std::error::Error;

pub mod bloxroute;
pub mod helius;
pub mod jito;
pub mod nextblock;
pub mod quicknode;
pub mod solana;
pub mod temporal;
pub mod triton;

pub trait RpcActions {
    fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>>;
    fn rpc_client(&self) -> &RpcClient;
    fn rpc_url(&self) -> &str;
    fn tip_wallet(&self) -> Option<&Pubkey>;
    fn min_tip_amount(&self) -> Option<u64>;
}

