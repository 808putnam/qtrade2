//! Configuration and settings management for qtrade-indexer
//!
//! This module centralizes all configuration handling for the qtrade-indexer,
//! providing a structured way to pass settings to the indexer components.

use serde::{Deserialize, Serialize};

/// Configuration settings for the qtrade-indexer
///
/// This struct holds settings that control the behavior of the indexer,
/// such as which DEX platforms to index and any other configuration
/// parameters that might be needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexerSettings {
    /// DEX platforms to use for indexing
    ///
    /// This controls which DEX platforms will be monitored and indexed
    /// by the streamer. By default, all supported DEXes are active,
    /// but this can be restricted to specific platforms if needed.
    pub active_dexes: Vec<String>, // Using strings to avoid dependency on qtrade-runtime

    /// Path to the vixen configuration file
    ///
    /// This file contains configuration for the yellowstone-vixen streamer,
    /// such as RPC endpoints and other stream-related settings.
    pub vixen_config_path: String,
}

impl IndexerSettings {
    /// Create a new IndexerSettings instance with default values
    pub fn new() -> Self {
        Self {
            active_dexes: vec![
                "orca".to_string(),
                "raydium".to_string(),
                "raydium-cpmm".to_string(),
                "raydium-clmm".to_string(),
            ],
            vixen_config_path: "default_vixon_config.toml".to_string(),
        }
    }

    /// Create a new IndexerSettings instance with specific active DEXes
    pub fn new_with_dexes(active_dexes: Vec<String>) -> Self {
        Self {
            active_dexes,
            vixen_config_path: "default_vixon_config.toml".to_string(),
        }
    }

    /// Create a new IndexerSettings instance with specific active DEXes and vixen config path
    pub fn new_with_config(active_dexes: Vec<String>, vixen_config_path: String) -> Self {
        Self {
            active_dexes,
            vixen_config_path,
        }
    }

    /// Check if a specific DEX platform is active
    pub fn is_dex_active(&self, dex_name: &str) -> bool {
        self.active_dexes.iter().any(|d| d.eq_ignore_ascii_case(dex_name))
    }
}

impl Default for IndexerSettings {
    fn default() -> Self {
        Self::new()
    }
}
