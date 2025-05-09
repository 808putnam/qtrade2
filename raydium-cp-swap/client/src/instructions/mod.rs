pub mod amm_instructions;
pub mod events_instructions_parse;
pub mod rpc;
pub mod token_instructions;
pub mod utils;

// qtrade: migrating ClientConfig to instructions for dual bin/lib build
use anyhow::{format_err, Result};
use configparser::ini::Ini;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
pub struct ClientConfig {
    pub http_url: String,
    pub ws_url: String,
    pub payer_path: String,
    pub admin_path: String,
    pub raydium_cp_program: Pubkey,
    pub slippage: f64,
}

pub fn load_cfg(client_config: &String) -> Result<ClientConfig> {
    let mut config = Ini::new();
    let _map = config.load(client_config).unwrap();
    let http_url = config.get("Global", "http_url").unwrap();
    if http_url.is_empty() {
        panic!("http_url must not be empty");
    }
    let ws_url = config.get("Global", "ws_url").unwrap();
    if ws_url.is_empty() {
        panic!("ws_url must not be empty");
    }
    let payer_path = config.get("Global", "payer_path").unwrap();
    if payer_path.is_empty() {
        panic!("payer_path must not be empty");
    }
    let admin_path = config.get("Global", "admin_path").unwrap();
    if admin_path.is_empty() {
        panic!("admin_path must not be empty");
    }

    let raydium_cp_program_str = config.get("Global", "raydium_cp_program").unwrap();
    if raydium_cp_program_str.is_empty() {
        panic!("raydium_cp_program must not be empty");
    }
    let raydium_cp_program = Pubkey::from_str(&raydium_cp_program_str).unwrap();
    let slippage = config.getfloat("Global", "slippage").unwrap().unwrap();

    Ok(ClientConfig {
        http_url,
        ws_url,
        payer_path,
        admin_path,
        raydium_cp_program,
        slippage,
    })
}

pub fn read_keypair_file(s: &str) -> Result<Keypair> {
    solana_sdk::signature::read_keypair_file(s)
        .map_err(|_| format_err!("failed to read keypair from {}", s))
}
