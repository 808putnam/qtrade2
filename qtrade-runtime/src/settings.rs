//! Configuration and settings management for qtrade-runtime
//!
//! This module centralizes all configuration handling for the qtrade-runtime,
//! providing a flexible configuration system with support for:
//!
//! - Command-line arguments
//! - Environment variables
//! - TOML configuration files
//! - Default values
//!
//! # Configuration Precedence
//!
//! Settings are loaded with the following precedence (highest to lowest):
//!
//! 1. Command-line arguments
//! 2. Environment variables
//! 3. Configuration file (TOML)
//! 4. Default values
//!
//! This means that a setting specified in the command line will override the same setting
//! from any other source, regardless of where it's defined.
//!
//! # Configuration File
//!
//! The configuration file uses TOML format and can be specified with the `--config` flag.
//! If not specified, the system will look for a file named `qtrade.toml` in the current directory.
//!
//! You can generate an example configuration file using `Settings::create_example_config()`.
//!
//! # Environment Variables
//!
//! The following environment variables are recognized:
//!
//! - `BLOXROUTE_API_KEY`
//! - `HELIUS_API_KEY`
//! - `NEXTBLOCK_API_KEY`
//! - `QUICKNODE_API_KEY`
//! - `TEMPORAL_API_KEY`
//! - `QTRADE_NONCE_ACCOUNTS` (comma-separated list)
//! - `QTRADE_NONCE_AUTHORITY_SECRET`

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs, path::Path};
use std::io::Read;

/// Default values for when environment variables are not set
const DEFAULT_VALUE: &str = "";

/// API keys and other settings loaded from environment variables and/or provided by flags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    // API Keys
    pub bloxroute_api_key: String,
    pub helius_api_key: String,
    pub nextblock_api_key: String,
    pub quicknode_api_key: String,
    pub temporal_api_key: String,

    // Nonce account configuration
    pub nonce_accounts: Vec<String>,
    pub nonce_authority_secret: String,

    // File paths provided via command line
    pub vixon_config_path: String,

    // Single wallet mode for testing and debugging
    pub single_wallet: bool,
    pub single_wallet_private_key: Option<String>,

    // Runtime configuration
    pub blockchain: crate::Blockchain,
    pub router: crate::Router,

    // RPC providers to use for transaction submissions
    pub active_rpcs: Vec<crate::RpcProvider>,

    // DEX platforms to use for indexing and routing
    pub active_dexes: Vec<crate::Dex>,

    // Transaction simulation flag
    pub simulate: bool,
}

/// Command-line override flags passed from qtrade-client
///
/// These flags have the highest precedence in the configuration system:
/// CLI arguments > Environment variables > Config file > Default values
///
/// To use a configuration file, specify its path with the `--config` flag.
/// If not specified, the system looks for a file named `qtrade.toml` in the current directory.
///
/// You can generate an example configuration file using:
/// `Settings::create_example_config("path/to/example_config.toml")`
#[derive(Debug, Clone, Default)]
pub struct Flags {
    // Configuration paths
    pub config_file_path: Option<String>, // Path to TOML config file
    pub vixon_config_path: Option<String>,

    // API keys
    pub bloxroute_api_key: Option<String>,
    pub helius_api_key: Option<String>,
    pub nextblock_api_key: Option<String>,
    pub quicknode_api_key: Option<String>,
    pub temporal_api_key: Option<String>,

    // Single wallet mode for testing and debugging
    pub single_wallet: bool,
    pub single_wallet_private_key: Option<String>,

    // Runtime configuration
    pub blockchain: Option<crate::Blockchain>,
    pub router: Option<crate::Router>,

    // RPC providers to use for transaction submissions (comma-separated string representation)
    pub active_rpcs: Option<Vec<String>>,

    // DEX platforms to use for indexing and routing (comma-separated string representation)
    pub active_dexes: Option<Vec<String>>,

    // Transaction simulation flag
    pub simulate: bool,
}

impl Settings {
    /// Default config file location if not specified
    pub const DEFAULT_CONFIG_PATH: &'static str = "./qtrade.toml";

    /// Load settings from a TOML configuration file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        tracing::info!("Loading configuration from file: {}", path.display());

        let mut file = fs::File::open(path)
            .with_context(|| format!("Failed to open config file: {}", path.display()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let config: Settings = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse TOML from config file: {}", path.display()))?;

        tracing::debug!("Successfully loaded configuration from file");
        Ok(config)
    }

    /// Load settings with precedence: CLI args > Env vars > Config file > Default values
    ///
    /// This follows the precedence order:
    /// 1. Command line arguments (highest priority)
    /// 2. Environment variables
    /// 3. Configuration file
    /// 4. Default values (lowest priority)
    pub fn load(flags: Flags) -> Result<Self> {
        // Start with default settings
        let mut settings = Self::default();

        // Try to load from config file if provided, otherwise try default location
        let config_path = flags.config_file_path.as_ref().map(|p| p.as_str())
            .unwrap_or(Self::DEFAULT_CONFIG_PATH);

        // Load from config file if it exists (silently skip if not found)
        if Path::new(config_path).exists() {
            if let Ok(file_settings) = Self::load_from_file(config_path) {
                settings = file_settings;
                tracing::info!("Loaded configuration from file: {}", config_path);
            } else {
                tracing::warn!("Failed to load configuration from file: {}", config_path);
            }
        } else {
            tracing::debug!("No configuration file found at: {}", config_path);
        }

        // Override with environment variables
        settings.bloxroute_api_key = env::var("BLOXROUTE_API_KEY")
            .ok()
            .unwrap_or(settings.bloxroute_api_key);

        settings.helius_api_key = env::var("HELIUS_API_KEY")
            .ok()
            .unwrap_or(settings.helius_api_key);

        settings.nextblock_api_key = env::var("NEXTBLOCK_API_KEY")
            .ok()
            .unwrap_or(settings.nextblock_api_key);

        settings.quicknode_api_key = env::var("QUICKNODE_API_KEY")
            .ok()
            .unwrap_or(settings.quicknode_api_key);

        settings.temporal_api_key = env::var("TEMPORAL_API_KEY")
            .ok()
            .unwrap_or(settings.temporal_api_key);

        // Special handling for nonce accounts which are comma-separated
        if let Ok(accounts_str) = env::var("QTRADE_NONCE_ACCOUNTS") {
            settings.nonce_accounts = accounts_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        settings.nonce_authority_secret = env::var("QTRADE_NONCE_AUTHORITY_SECRET")
            .ok()
            .unwrap_or(settings.nonce_authority_secret);

        // Finally override with CLI flags (highest precedence)
        if let Some(api_key) = flags.bloxroute_api_key {
            settings.bloxroute_api_key = api_key;
        }

        if let Some(api_key) = flags.helius_api_key {
            settings.helius_api_key = api_key;
        }

        if let Some(api_key) = flags.nextblock_api_key {
            settings.nextblock_api_key = api_key;
        }

        if let Some(api_key) = flags.quicknode_api_key {
            settings.quicknode_api_key = api_key;
        }

        if let Some(api_key) = flags.temporal_api_key {
            settings.temporal_api_key = api_key;
        }

        // Required file paths - these must be provided either from flags, config file, or defaults
        settings.vixon_config_path = flags.vixon_config_path
            .unwrap_or(settings.vixon_config_path);

        // Parse active RPCs from string array to RpcProvider enum array
        let mut rpcs_from_flags = false;
        if let Some(active_rpcs_strs) = &flags.active_rpcs {
            let mut parsed_rpcs = Vec::new();
            for rpc_str in active_rpcs_strs {
                if let Some(rpc_provider) = crate::RpcProvider::from_str(rpc_str) {
                    parsed_rpcs.push(rpc_provider);
                } else {
                    tracing::warn!("Unknown RPC provider: {}", rpc_str);
                }
            }

            if !parsed_rpcs.is_empty() {
                settings.active_rpcs = parsed_rpcs;
                rpcs_from_flags = true;
            } else {
                tracing::warn!("No valid RPC providers found in active-rpcs flag, using defaults");
            }
        }

        // Also check environment variable for active RPC providers (comma-separated)
        if !rpcs_from_flags {
            if let Ok(active_rpcs_str) = env::var("QTRADE_ACTIVE_RPCS") {
                if !active_rpcs_str.is_empty() {
                    let mut parsed_rpcs = Vec::new();
                    for rpc_str in active_rpcs_str.split(',').map(|s| s.trim()) {
                        if let Some(rpc_provider) = crate::RpcProvider::from_str(rpc_str) {
                            parsed_rpcs.push(rpc_provider);
                        } else {
                            tracing::warn!("Unknown RPC provider in environment variable: {}", rpc_str);
                        }
                    }

                    if !parsed_rpcs.is_empty() {
                        // Use environment variable if flags didn't already set it
                        settings.active_rpcs = parsed_rpcs;
                    }
                }
            }
        }

        // Parse active DEXes from string array to Dex enum array
        let mut dexes_from_flags = false;
        if let Some(active_dexes_strs) = &flags.active_dexes {
            let mut parsed_dexes = Vec::new();
            for dex_str in active_dexes_strs {
                if let Some(dex) = crate::Dex::from_str(dex_str) {
                    parsed_dexes.push(dex);
                } else {
                    tracing::warn!("Unknown DEX platform: {}", dex_str);
                }
            }

            if !parsed_dexes.is_empty() {
                settings.active_dexes = parsed_dexes;
                dexes_from_flags = true;
            } else {
                tracing::warn!("No valid DEX platforms found in active-dexes flag, using defaults");
            }
        }

        // Also check environment variable for active DEX platforms (comma-separated)
        if !dexes_from_flags {
            if let Ok(active_dexes_str) = env::var("QTRADE_ACTIVE_DEXES") {
                if !active_dexes_str.is_empty() {
                    let mut parsed_dexes = Vec::new();
                    for dex_str in active_dexes_str.split(',').map(|s| s.trim()) {
                        if let Some(dex) = crate::Dex::from_str(dex_str) {
                            parsed_dexes.push(dex);
                        } else {
                            tracing::warn!("Unknown DEX platform in environment variable: {}", dex_str);
                        }
                    }

                    if !parsed_dexes.is_empty() {
                        // Use environment variable if flags didn't already set it
                        settings.active_dexes = parsed_dexes;
                    } else {
                        tracing::warn!("No valid DEX platforms found in QTRADE_ACTIVE_DEXES, using defaults");
                    }
                }
            }
        }

        // Validate required configurations
        if settings.vixon_config_path.is_empty() {
            return Err(anyhow::anyhow!("Vixon config path must be provided"));
        }

        // Required runtime configuration - must be provided from flags or config
        settings.blockchain = flags.blockchain.unwrap_or(settings.blockchain);
        settings.router = flags.router.unwrap_or(settings.router);

        // Boolean flags (flags directly override config)
        if flags.single_wallet {
            settings.single_wallet = true;
        }

        if flags.simulate {
            settings.simulate = true;
        }

        // Single wallet private key (flag overrides config)
        if let Some(key) = flags.single_wallet_private_key {
            settings.single_wallet_private_key = Some(key);
        }

        // Log single wallet mode status
        if settings.single_wallet {
            tracing::info!("Single wallet mode is enabled");
            if settings.single_wallet_private_key.is_some() {
                tracing::info!("Using provided private key for single wallet");
            } else {
                tracing::warn!("Single wallet mode enabled but no private key provided");
            }
        }

        Ok(settings)
    }

    /// Save the current settings to a TOML configuration file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        tracing::info!("Saving configuration to file: {}", path.display());

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        // Serialize settings to TOML string
        let toml_string = toml::to_string(self)
            .context("Failed to serialize settings to TOML")?;

        // Write to file
        fs::write(path, toml_string)
            .with_context(|| format!("Failed to write configuration to file: {}", path.display()))?;

        tracing::info!("Successfully saved configuration to file");
        Ok(())
    }

    /// Generate a template configuration file with the current settings
    pub fn save_template<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.save_to_file(path)
    }

    /// Validate that all required settings are present
    pub fn validate(&self) -> Result<()> {
        // Check if required API keys are set
        if self.bloxroute_api_key.is_empty() {
            tracing::warn!("BLOXROUTE_API_KEY is not set. Some functionality may be limited.");
        }

        if self.helius_api_key.is_empty() {
            tracing::warn!("HELIUS_API_KEY is not set. Some functionality may be limited.");
        }

        if self.nextblock_api_key.is_empty() {
            tracing::warn!("NEXTBLOCK_API_KEY is not set. Some functionality may be limited.");
        }

        if self.quicknode_api_key.is_empty() {
            tracing::warn!("QUICKNODE_API_KEY is not set. Some functionality may be limited.");
        }

        if self.temporal_api_key.is_empty() {
            tracing::warn!("TEMPORAL_API_KEY is not set. Some functionality may be limited.");
        }

        // Validate required file paths
        if self.vixon_config_path.is_empty() {
            return Err(anyhow::anyhow!("Vixon config path must be provided"));
        }

        // Note: We don't validate nonce account settings as they might be optional

        Ok(())
    }

    /// Get API keys and secrets for use in other modules
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

    pub fn get_nonce_accounts(&self) -> &[String] {
        &self.nonce_accounts
    }

    pub fn get_nonce_authority_secret(&self) -> &str {
        &self.nonce_authority_secret
    }

    /// Create an example configuration file at the specified path
    pub fn create_example_config<P: AsRef<Path>>(path: P) -> Result<()> {
        let path = path.as_ref();
        tracing::info!("Creating example configuration file at: {}", path.display());

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }

        // Include the example config from a static file
        const EXAMPLE_CONFIG: &str = include_str!("example_config.toml");

        // Write to file
        fs::write(path, EXAMPLE_CONFIG)
            .with_context(|| format!("Failed to write example configuration to file: {}", path.display()))?;

        tracing::info!("Successfully created example configuration file");
        Ok(())
    }

    /// Create an example configuration file based on the current settings
    pub fn create_example_config_from_current<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        tracing::info!("Creating example configuration file from current settings at: {}", path.display());

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
            }
        }

        // Generate example config content from current settings
        let example_config = self.to_example_toml()?;

        // Write to file
        fs::write(path, example_config)
            .with_context(|| format!("Failed to write example configuration to file: {}", path.display()))?;

        tracing::info!("Successfully created example configuration file from current settings");
        Ok(())
    }

    /// Generate example TOML config content from the current settings
    pub fn to_example_toml(&self) -> Result<String> {
        let toml_string = toml::to_string_pretty(self)
            .context("Failed to serialize settings to TOML")?;

        // Add comments to explain each section
        let commented_toml = format!(
            "# QTrade Configuration File\n\
            # This is an example configuration file for the qtrade application.\n\
            # Settings follow this precedence order:\n\
            # CLI arguments > Environment variables > Config file > Default values\n\n\
            # API Keys\n\
            # These are used for various external services\n\
            {}\n\
            # Available blockchain options: Solana, Sui\n\
            # Available router options: Cvxpy, OpenQAOA, CFMMRouter\n",
            toml_string
        );

        Ok(commented_toml)
    }
}

// For tests and examples, provide a way to create Settings with default values
// Implement Default for Settings, useful for building settings incrementally
// and for providing sensible defaults
impl Default for Settings {
    fn default() -> Self {
        Settings {
            bloxroute_api_key: DEFAULT_VALUE.to_string(),
            helius_api_key: DEFAULT_VALUE.to_string(),
            nextblock_api_key: DEFAULT_VALUE.to_string(),
            quicknode_api_key: DEFAULT_VALUE.to_string(),
            temporal_api_key: DEFAULT_VALUE.to_string(),
            nonce_accounts: vec![],
            nonce_authority_secret: DEFAULT_VALUE.to_string(),
            vixon_config_path: "default_vixon_config.json".to_string(),
            single_wallet: false,
            single_wallet_private_key: None,
            blockchain: crate::Blockchain::Solana, // Default to Solana
            router: crate::Router::Cvxpy,     // Default to Cvxpy
            active_rpcs: vec![
                crate::RpcProvider::Bloxroute,
                crate::RpcProvider::Helius,
                crate::RpcProvider::Jito,
                crate::RpcProvider::Nextblock,
                crate::RpcProvider::Quicknode,
                crate::RpcProvider::Solana,
                crate::RpcProvider::Temporal,
            ],                                    // By default, enable all RPCs
            active_dexes: vec![
                crate::Dex::Orca,
                crate::Dex::Raydium,
                crate::Dex::RaydiumCpmm,
                crate::Dex::RaydiumClmm,
            ],                                    // By default, enable all DEXes
            simulate: false,                      // Default simulate to false
        }
    }
}
