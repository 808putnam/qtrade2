//! Configuration and settings management for qtrade-relayer
//!
//! This module centralizes all configuration handling for the qtrade-relayer,
//! provides a RelayerSettings struct to centralize API keys and other configuration.
//! It can load settings either from environment variables or from qtrade-runtime's settings.

use std::env;

/// API keys and other settings for relayer operations
#[derive(Debug, Clone)]
pub struct RelayerSettings {
    // API Keys
    pub bloxroute_api_key: String,
    pub helius_api_key: String,
    pub nextblock_api_key: String,
    pub quicknode_api_key: String,
    pub temporal_api_key: String,

    /// List of RPC providers to use for transaction submissions.
    ///
    /// This controls which RPC providers will be used when submitting transactions.
    /// By default, all providers are active. The strings should match the lowercase
    /// names of the RPC providers: "bloxroute", "helius", "jito", "nextblock",
    /// "quicknode", "solana", "temporal", "triton".
    pub active_rpcs: Vec<String>,

    // Transaction simulation flag
    pub simulate: bool,
}

impl RelayerSettings {
    /// Create a new RelayerSettings instance from environment variables
    pub fn from_env() -> Self {
        let bloxroute_api_key = env::var("BLOXROUTE_API_KEY")
            .unwrap_or_else(|_| "".to_string());

        let helius_api_key = env::var("HELIUS_API_KEY")
            .unwrap_or_else(|_| "".to_string());

        let nextblock_api_key = env::var("NEXTBLOCK_API_KEY")
            .unwrap_or_else(|_| "".to_string());

        let quicknode_api_key = env::var("QUICKNODE_API_KEY")
            .unwrap_or_else(|_| "".to_string());

        let temporal_api_key = env::var("TEMPORAL_API_KEY")
            .unwrap_or_else(|_| "".to_string());

        let simulate = env::var("SIMULATE")
            .map(|v| v == "true")
            .unwrap_or(false);

        // Parse active RPCs from environment variable if available
        let active_rpcs = match env::var("QTRADE_ACTIVE_RPCS") {
            Ok(rpcs_str) if !rpcs_str.is_empty() => {
                rpcs_str.split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            },
            _ => vec![
                "bloxroute".to_string(),
                "helius".to_string(),
                "jito".to_string(),
                "nextblock".to_string(),
                "quicknode".to_string(),
                "solana".to_string(),
                "temporal".to_string()
            ] // Default to all RPC providers
        };

        Self {
            bloxroute_api_key,
            helius_api_key,
            nextblock_api_key,
            quicknode_api_key,
            temporal_api_key,
            active_rpcs,
            simulate,
        }
    }

    /// Create a new RelayerSettings instance with specific values
    pub fn new(
        bloxroute_api_key: String,
        helius_api_key: String,
        nextblock_api_key: String,
        quicknode_api_key: String,
        temporal_api_key: String,
        simulate: bool,
    ) -> Self {
        // Default to all RPC providers
        let active_rpcs = vec![
            "bloxroute".to_string(),
            "helius".to_string(),
            "jito".to_string(),
            "nextblock".to_string(),
            "quicknode".to_string(),
            "solana".to_string(),
            "temporal".to_string()
        ];

        Self {
            bloxroute_api_key,
            helius_api_key,
            nextblock_api_key,
            quicknode_api_key,
            temporal_api_key,
            active_rpcs,
            simulate,
        }
    }

    /// Create a new RelayerSettings instance with specific values including active RPCs
    pub fn new_with_rpcs(
        bloxroute_api_key: String,
        helius_api_key: String,
        nextblock_api_key: String,
        quicknode_api_key: String,
        temporal_api_key: String,
        active_rpcs: Vec<String>,
        simulate: bool,
    ) -> Self {
        Self {
            bloxroute_api_key,
            helius_api_key,
            nextblock_api_key,
            quicknode_api_key,
            temporal_api_key,
            active_rpcs,
            simulate,
        }
    }

    // Getter methods for API keys
    pub fn get_bloxroute_api_key(&self) -> &str {
        &self.bloxroute_api_key
    }

    pub fn get_helius_api_key(&self) -> &str {
        &self.helius_api_key
    }

    pub fn get_nextblock_api_key(&self) -> &str {
        &self.nextblock_api_key
    }

    pub fn get_quicknode_api_key(&self) -> &str {
        &self.quicknode_api_key
    }

    pub fn get_temporal_api_key(&self) -> &str {
        &self.temporal_api_key
    }

    pub fn is_simulate(&self) -> bool {
        self.simulate
    }
}

// For tests and examples, provide a way to create RelayerSettings with default values
#[cfg(test)]
impl Default for RelayerSettings {
    fn default() -> Self {
        Self {
            bloxroute_api_key: "".to_string(),
            helius_api_key: "".to_string(),
            nextblock_api_key: "".to_string(),
            quicknode_api_key: "".to_string(),
            temporal_api_key: "".to_string(),
            active_rpcs: vec![
                "bloxroute".to_string(),
                "helius".to_string(),
                "jito".to_string(),
                "nextblock".to_string(),
                "quicknode".to_string(),
                "solana".to_string(),
                "temporal".to_string()
            ],
            simulate: false,
        }
    }
}
