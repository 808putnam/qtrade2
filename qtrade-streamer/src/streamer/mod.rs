//! This module represents a streamer-parser for a Geyser stream.
//!
//! The functionality includes:
//! - Connecting to a Geyser stream
//! - Parsing and processing streamed data
//! - Providing utilities for handling stream data
//!
//! This module abstracts the complexities of interacting with a Geyser stream and provides
//! a simple interface for streaming and parsing data.

use anyhow::Result;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use std::fs;
use tracing::info;
use yellowstone_vixen::{self as vixen, Pipeline};
use yellowstone_vixen::config::{NullConfig, VixenConfig };

use crate::parser::orca::AccountParser as OrcaAccParser;
use crate::parser::raydium::AccountParser as RaydiumAccParser;
use crate::parser::raydium_clmm::AccountParser as RaydiumClmmAccParser;
use crate::parser::raydium_cpmm::AccountParser as RaydiumCpmmAccParser;

use crate::streamer::handlers::orca_handler::OrcaHandler;
use crate::streamer::handlers::raydium_handler::RaydiumHandler;
use crate::streamer::handlers::raydium_clmm_handler::RaydiumClmmHandler;
use crate::streamer::handlers::raydium_cpmm_handler::RaydiumCpmmHandler;

mod caches;
mod handlers;

pub use caches::mint_cache::*;
pub use caches::pool_cache::*;
pub use caches::pool_config_cache::*;

// For help in naming spans
use crate::QTRADE_STREAMER_TRACER_NAME;
const STREAMER: &str = "streamer";

/// A trait for cache operations
///
/// This trait defines the common operations for various caches in the system.
/// It allows for retrieving, updating, and removing entries from caches.
pub trait Cache<K, V>
where
    V: Clone
{
    // Note: Creation of cache instances is handled outside the trait
    // to maintain object safety

    /// Get all entries in the cache as a Vec
    async fn get_all_entries(&self) -> Vec<(K, V)>;

    /// Get all entries in the cache as a boxed slice for efficient memory usage
    async fn get_all_entries_as_slice(&self) -> Box<[(K, V)]>;

    /// Read a specific entry from the cache by key
    async fn read_cache(&self, key: &K) -> Option<V>;

    /// Update or insert a cache entry
    async fn update_cache(&self, key: K, value: V) -> Option<V>;

    /// Remove a cache entry
    async fn remove_cache(&self, key: K) -> Option<(K, V)>;
}

/// Connects to a Geyser stream and updates the acctsdb cache and pool reserves cache.
///
/// This function establishes a connection to a Geyser stream, listens for updates,
/// and processes the streamed data to update the accounts database (acctsdb) cache
/// and the pool reserves cache.
pub async fn run_streamer(vixon_config_path: &str) -> Result<()> {
    let tracer = global::tracer(QTRADE_STREAMER_TRACER_NAME);
    let span_name = format!("{}::run_streamer", STREAMER);

    let result = tracer.in_span(span_name, |_cx| async move {
        info!("Connecting to Geyser stream...");

        // TODO: Confirm with bare metal geyser if this is still valid
        // Note: Cannot setup multiple filters as the current geyser
        //       we connect to is limited to 1 filter per connection
        //

        let config = read_and_parse_config(vixon_config_path)?;
        let result = vixen::Runtime::builder()
            .account(Pipeline::new(OrcaAccParser, [OrcaHandler::new()]))
            .account(Pipeline::new(RaydiumAccParser, [RaydiumHandler::new()]))
            .account(Pipeline::new(RaydiumClmmAccParser, [RaydiumClmmHandler::new()]))
            .account(Pipeline::new(RaydiumCpmmAccParser, [RaydiumCpmmHandler::new()]))
            .build(config)
            .try_run_async()
            .await
            .map_err(|e| {
                // TODO: Populate anyhow error from yellowstone error
                anyhow::anyhow!("Yellowstone error: {}", e)
            });

        result

        /* See TODO note above
        let config = read_and_parse_config(vixon_config_path)?;
        let orca_acc_parser = vixen::Runtime::builder()
            .account(Pipeline::new(OrcaAccParser, [OrcaHandler::new()]))
            .build(config)
            .try_run_async();

        let config = read_and_parse_config(vixon_config_path)?;
        let raydium_acc_parser = vixen::Runtime::builder()
            .account(Pipeline::new(RaydiumAccParser, [RaydiumHandler]))
            .build(config)
            .try_run_async();

        let config = read_and_parse_config(vixon_config_path)?;
        let raydium_clmm_acc_parser = vixen::Runtime::builder()
            .account(Pipeline::new(RaydiumClmmAccParser, [RaydiumClmmHandler]))
            .build(config)
            .try_run_async();

        let config = read_and_parse_config(vixon_config_path)?;
        let raydium_cpmm_acc_parser = vixen::Runtime::builder()
            .account(Pipeline::new(RaydiumCpmmAccParser, [RaydiumCpmmHandler]))
            .build(config)
            .try_run_async();

        try_join!(
            orca_acc_parser,
            raydium_acc_parser,
            raydium_clmm_acc_parser,
            raydium_cpmm_acc_parser
        )?;

        Ok(())
        */
    }).await;

    result
}

fn read_and_parse_config(path: &str) -> Result<VixenConfig<NullConfig>> {
    let tracer = global::tracer(QTRADE_STREAMER_TRACER_NAME);
    let span_name = format!("{}::read_and_parse_config", STREAMER);

    let result = tracer.in_span(span_name, move |_cx| {
        let config_str = fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Error reading config file: {}", e))?;
        let config = toml::from_str(&config_str)
            .map_err(|e| anyhow::anyhow!("Error parsing config: {}", e))?;
        Ok(config)
    });

    result
}

