use solana_client::rpc_client::RpcClient;
use solana_sdk::instruction::Instruction;
use solana_sdk::signature::Keypair;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::hash::Hash;
use std::error::Error;

pub mod bloxroute;
pub mod helius;
pub mod jito;
pub mod nextblock;
pub mod quicknode;
pub mod solana;
pub mod temporal;
pub mod triton;

/// Optional nonce transaction details for use with durable nonces
pub struct NonceInfo<'a> {
    /// The nonce account public key
    pub nonce_pubkey: &'a Pubkey,
    /// The nonce authority keypair for signing
    pub nonce_authority: &'a Keypair,
    /// The nonce value (blockhash) stored in the nonce account
    pub nonce_hash: Hash,
}

pub trait RpcActions {
    /// Send a transaction with either a blockhash or nonce
    fn send_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair) -> Result<String, Box<dyn Error>>;

    /// Send a transaction using a durable nonce account
    fn send_nonce_tx(&self, ixs: &mut Vec<Instruction>, signer: &Keypair, nonce_info: NonceInfo) -> Result<String, Box<dyn Error>> {
        // Default implementation falls back to regular send_tx
        // This allows gradual implementation in all RPC providers
        self.send_tx(ixs, signer)
    }

    fn rpc_client(&self) -> &RpcClient;
    fn rpc_url(&self) -> &str;
    fn tip_wallet(&self) -> Option<&Pubkey>;
    fn min_tip_amount(&self) -> Option<u64>;
}
