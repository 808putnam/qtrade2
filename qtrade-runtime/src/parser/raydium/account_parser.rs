use borsh::BorshDeserialize;
// qtrade: spl_pod changed namespacing from 0.3.0 to 0.50
// use spl_pod::solana_program::program_error::ProgramError;
use spl_pod::solana_program_error::ProgramError;
use yellowstone_vixen_core::{ParseError, ParseResult, Parser, Prefilter, ProgramParser};
use opentelemetry::global;
use opentelemetry::trace::{Span, Tracer};

// qtrade: from account_helpers.rs
use spl_pod::solana_pubkey::Pubkey;

use super::account_helpers::{ AmmEmpty, AmmInfo, KeyedAmmInfo };
use crate::parser::raydium::RADIUM_PROGRAM_ID;

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;
const RAYDIUM_PROGRAM_STATE: &str = "raydium::RaydiumProgramState";
const RAYDIUM_ACCOUNT_PARSER: &str = "raydium::AccountParser";

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum RaydiumProgramState {
    AmmEmpty(AmmEmpty),
    AmmInfo(KeyedAmmInfo),
}

impl RaydiumProgramState {
    pub fn try_unpack(pubkey_bytes: [u8; 32], data_bytes: &[u8]) -> ParseResult<Self> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::try_unpack", RAYDIUM_PROGRAM_STATE);

        let result = tracer.in_span(span_name, move |_cx| {
            // qtrade
            let pubkey = Pubkey::new_from_array(pubkey_bytes);

            let data_len = data_bytes.len();
            // No anchor discriminator for original raydium program
            // let data_bytes = &data_bytes[ACC_DISCRIMINATOR_SIZE..];

            match data_len {
                AmmInfo::LEN => {
                    let amm_info = AmmInfo::try_from_slice(data_bytes)?;
                    Ok(RaydiumProgramState::AmmInfo(KeyedAmmInfo {
                        pubkey: pubkey,
                        amm_info: amm_info,
                    }))
                },
                // Hack allows us to ignore all other account type updates when parsing
                // without returning an error
                _ => Ok(RaydiumProgramState::AmmEmpty(AmmEmpty{}))
            }
        });

        result
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AccountParser;

impl Parser for AccountParser {
    type Input = yellowstone_vixen_core::AccountUpdate;
    type Output = RaydiumProgramState;

    fn id(&self) -> std::borrow::Cow<str> {
        "raydium::AccountParser".into()
    }

    fn prefilter(&self) -> Prefilter {
        Prefilter::builder()
            .account_owners([RADIUM_PROGRAM_ID])
            .build()
            .unwrap()
    }

    async fn parse(
        &self,
        acct: &yellowstone_vixen_core::AccountUpdate,
    ) -> ParseResult<Self::Output> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::parse", RAYDIUM_ACCOUNT_PARSER);

        let result = tracer.in_span(span_name, |_cx| async move {
            let inner = acct.account.as_ref().ok_or(ProgramError::InvalidArgument)?;

            // qtrade
            let pubkey_bytes: [u8; 32] = inner.pubkey.clone().try_into().map_err(|_| ProgramError::InvalidArgument)?;

            RaydiumProgramState::try_unpack(pubkey_bytes, &inner.data)
        }).await;

        result
    }
}

impl ProgramParser for AccountParser {
    #[inline]
    fn program_id(&self) -> yellowstone_vixen_core::Pubkey {
        RADIUM_PROGRAM_ID.to_bytes().into()
    }
}

