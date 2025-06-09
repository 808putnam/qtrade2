// qtrade: spl_pod changed namespacing from 0.3.0 to 0.50
use spl_pod::solana_pubkey::Pubkey;

mod account_helpers;
mod account_parser;

pub const RADIUM_ADDRESS: &str = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
pub const RADIUM_PROGRAM_ID: Pubkey = Pubkey::from_str_const(RADIUM_ADDRESS);

pub use account_helpers::*;
pub use account_parser::*;
