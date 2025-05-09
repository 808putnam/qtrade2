use std::any::Any;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use tracing::{debug, warn};
use yellowstone_vixen::{self as vixen};
use yellowstone_vixen_parser::token_program::TokenProgramState;

// For help in naming spans
use crate::QTRADE_RUNTIME_TRACER_NAME;
const TOKEN_HANDLER: &str = "streamer::handlers::TokenHandler";

#[derive(Debug)]
pub struct TokenHandler;

impl<V: std::fmt::Debug + Sync + Any> vixen::Handler<V> for TokenHandler {
    async fn handle(&self, value: &V) -> vixen::HandlerResult<()> {
        let tracer = global::tracer(QTRADE_RUNTIME_TRACER_NAME);
        let span_name = format!("{}::handle", TOKEN_HANDLER);

        let result = tracer.in_span(span_name, |_cx| async move {
            debug!(?value);

            if let Some(token_program_state) = (value as &dyn Any).downcast_ref::<TokenProgramState>() {
                match token_program_state {
                    TokenProgramState::TokenAccount(account) => {
                        debug!("Processing TokenAccount: {:?}", account);
                    }
                    TokenProgramState::Mint(mint) => {
                        debug!("Processing Mint: {:?}", mint);
                    }
                    TokenProgramState::Multisig(multisig) => {
                        debug!("Processing Multisig: {:?}", multisig);
                    }
                }            
            } else {
                warn!("Value is not a TokenProgramState");
            }

            Ok(())
        }).await;

        result
    }
}
