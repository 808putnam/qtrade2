// qtrade: spl_pod changed namespacing from 0.3.0 to 0.50
use spl_pod::solana_pubkey::Pubkey;

mod account_helpers;
mod account_parser;

pub const RADIUM_CPMM_ADDRESS: &str = "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C";
pub const RADIUM_CPMM_PROGRAM_ID: Pubkey = Pubkey::from_str_const(RADIUM_CPMM_ADDRESS);

pub use account_helpers::*;
pub use account_parser::*;
