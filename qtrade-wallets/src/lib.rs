mod key_pool;
pub mod metrics;

use anyhow::Result;
use solana_sdk::signature::{Keypair, Signer};
use std::env;
use tracing::{info, warn, error};

// Re-export metrics module
pub use crate::metrics as wallet_metrics;

pub use key_pool::{
    KeyTier, KeyStatus, KeyInfo, KeyPool, KeyManager
};

/// Wallet configuration settings for controlling wallet behavior
#[derive(Debug, Clone)]
pub struct WalletSettings {
    /// Whether to use a single wallet instead of the multi-tiered wallet system
    pub single_wallet: bool,

    /// Private key for the single wallet mode (if enabled)
    pub single_wallet_private_key: Option<String>,
}

// Constants for key balancing
const MIN_EXPLORER_KEYS: usize = 5;
const EXPLORER_KEYS_TO_CREATE: usize = 3;
const LAMPORTS_PER_EXPLORER: u64 = 10_000_000; // 0.01 SOL
const LAMPORTS_PER_BANK: u64 = 100_000_000;    // 0.1 SOL

// Our global key manager instance
static mut KEY_MANAGER: Option<KeyManager> = None;

/// Run the wallet management service
///
/// This function initializes the wallet system and then periodically manages wallet balances.
/// It periodically checks and manages wallet balances based on a timer.
pub async fn run_wallets(settings: WalletSettings) -> Result<()> {
    use opentelemetry::global;
    use opentelemetry::trace::Tracer;
    use std::time::Duration;
    use tokio::time::sleep;
    use tracing::{info, error};

    const WALLETS: &str = "wallets";
    const CHECK_INTERVAL: Duration = Duration::from_secs(60);

    let tracer_name = "qtrade_wallets";
    let tracer = global::tracer(tracer_name);

    // First, initialize the wallet system
    initialize_wallet_system(&settings).await?;

    loop {
        let span_name = format!("{}::run_wallets", WALLETS);

        let result: Result<(), anyhow::Error> = tracer.in_span(span_name, |_cx| async move {
            // Periodically check and manage wallet balances
            info!("Checking and managing wallet balances...");

            // Call the balancer to:
            // 1. Clean up used explorer keys and recover funds
            // 2. Fund bank keys from HODL keys if needed
            // 3. Create new explorer keys if needed
            if let Err(e) = balancer().await {
                error!("Error in wallet balancer: {:?}", e);
            }

            Ok(())
        }).await;

        // result
        if let Err(e) = result {
            error!("Error running wallet management: {:?}", e);
        }

        // Wait for specified duration before running the check again
        sleep(CHECK_INTERVAL).await;
    }
}

/// Initialize the key manager with keys from environment variables
pub fn init() -> Result<()> {
    // Get RPC URL from environment
    let rpc_url = env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string());

    // Load HODL keys from environment (comma-separated private keys)
    let hodl_keys_str = env::var("HODL_KEYS").unwrap_or_else(|_| "".to_string());
    let hodl_keys = load_keypairs_from_str(&hodl_keys_str, 1_000_000_000); // 1 SOL target balance

    // Load bank keys from environment
    let bank_keys_str = env::var("BANK_KEYS").unwrap_or_else(|_| "".to_string());
    let bank_keys = load_keypairs_from_str(&bank_keys_str, LAMPORTS_PER_BANK);

    // Load explorer keys from environment or create new ones if none provided
    let explorer_keys_str = env::var("EXPLORER_KEYS").unwrap_or_else(|_| "".to_string());
    let explorer_keys = if explorer_keys_str.is_empty() {
        // Create some initial explorer keys if none provided
        (0..MIN_EXPLORER_KEYS).map(|_| {
            (Keypair::new(), LAMPORTS_PER_EXPLORER)
        }).collect()
    } else {
        load_keypairs_from_str(&explorer_keys_str, LAMPORTS_PER_EXPLORER)
    };

    // Log key counts before creating the key manager
    let hodl_count = hodl_keys.len();
    let bank_count = bank_keys.len();
    let explorer_count = explorer_keys.len();

    // Create the key manager
    let key_manager = KeyManager::new(
        hodl_keys,
        bank_keys,
        explorer_keys,
        &rpc_url,
        500_000_000,  // 0.5 SOL min for HODL
        50_000_000,   // 0.05 SOL min for Bank
        5_000_000,    // 0.005 SOL min for Explorer
    );

    // Store the key manager in our global static
    unsafe {
        KEY_MANAGER = Some(key_manager);
    }

    info!("Key manager initialized with {} HODL keys, {} Bank keys, and {} Explorer keys",
        hodl_count, bank_count, explorer_count);

    // Initialize metrics
    wallet_metrics::init();

    Ok(())
}

/// Helper function to load keypairs from a comma-separated string
fn load_keypairs_from_str(keys_str: &str, target_balance: u64) -> Vec<(Keypair, u64)> {
    if keys_str.is_empty() {
        return Vec::new();
    }

    keys_str.split(',')
        .filter_map(|key_str| {
            // Try to parse as base58 private key
            match bs58::decode(key_str.trim()).into_vec() {
                Ok(bytes) => {
                    if let Ok(keypair) = Keypair::from_bytes(&bytes) {
                        Some((keypair, target_balance))
                    } else {
                        warn!("Failed to create keypair from bytes");
                        None
                    }
                },
                Err(_) => {
                    warn!("Failed to decode base58 key");
                    None
                }
            }
        })
        .collect()
}

/// Get an instance of the key manager
pub fn get_key_manager() -> Option<KeyManager> {
    unsafe {
        KEY_MANAGER.clone()
    }
}

// Global flag to track if we're in single wallet mode
static mut SINGLE_WALLET_MODE: bool = false;

/// Balance the key pools, ensuring adequate funding and key availability
pub async fn balancer() -> Result<()> {
    // Skip balancing in single wallet mode
    if unsafe { SINGLE_WALLET_MODE } {
        info!("Skipping key pool balancing in single wallet mode");
        return Ok(());
    }

    match get_key_manager() {
        Some(key_manager) => {
            info!("Running key pool balancer...");

            // Run the balancer
            key_manager.balance(
                MIN_EXPLORER_KEYS,
                EXPLORER_KEYS_TO_CREATE,
                LAMPORTS_PER_EXPLORER,
                LAMPORTS_PER_BANK
            ).await?;

            // After balancing, update metrics about pool sizes
            let hodl_keys = key_manager.hodl_pool().get_all_keys()?;
            let bank_keys = key_manager.bank_pool().get_all_keys()?;
            let explorer_keys = key_manager.explorer_pool().get_all_keys()?;

            let hodl_available = hodl_keys.iter().filter(|(_, status)| *status == key_pool::KeyStatus::Available).count() as u64;
            let bank_available = bank_keys.iter().filter(|(_, status)| *status == key_pool::KeyStatus::Available).count() as u64;
            let explorer_available = explorer_keys.iter().filter(|(_, status)| *status == key_pool::KeyStatus::Available).count() as u64;

            // Record metrics with OpenTelemetry
            wallet_metrics::otel::record_key_pool_sizes(
                hodl_keys.len() as u64, hodl_available,
                bank_keys.len() as u64, bank_available,
                explorer_keys.len() as u64, explorer_available
            );

            // Record metrics to OpenTelemetry
            wallet_metrics::otel::record_otel_metrics();

            info!("Key pool balancing complete");
            Ok(())
        },
        None => {
            error!("Key manager not initialized");
            Err(anyhow::anyhow!("Key manager not initialized"))
        }
    }
}

/// Get an explorer keypair for transaction signing
pub fn get_explorer_keypair() -> Option<(solana_sdk::pubkey::Pubkey, Keypair)> {
    // Special handling for single wallet mode
    if unsafe { SINGLE_WALLET_MODE } {
        if let Some(key_manager) = get_key_manager() {
            // In single wallet mode, we don't care about normal explorer key management
            // We'll always return the same key
            let result = key_manager.get_explorer_keypair();
            if result.is_some() {
                wallet_metrics::record_explorer_key_acquired();
                info!("Single wallet mode: returning the dedicated signing wallet");
            }
            return result;
        }
    }

    // Normal multi-tier wallet behavior
    match get_key_manager() {
        Some(key_manager) => {
            let result = key_manager.get_explorer_keypair();
            if result.is_some() {
                // Record an explorer key acquisition
                wallet_metrics::record_explorer_key_acquired();
            }
            result
        },
        None => {
            error!("Key manager not initialized");
            None
        }
    }
}

/// Return an explorer keypair to the pool or mark it as used
pub fn return_explorer_keypair(pubkey: &solana_sdk::pubkey::Pubkey, retire: bool) -> Result<()> {
    // In single wallet mode, we don't actually retire keys
    if unsafe { SINGLE_WALLET_MODE } {
        // Just pretend we returned it successfully
        info!("Single wallet mode: ignoring key retirement request (key will be reused)");
        return Ok(());
    }

    // Normal multi-tier wallet behavior
    match get_key_manager() {
        Some(key_manager) => {
            let result = key_manager.return_explorer_keypair(pubkey, retire);
            if result.is_ok() && retire {
                // Record explorer key retirement
                wallet_metrics::record_explorer_key_retired();
            }
            result
        },
        None => {
            error!("Key manager not initialized");
            Err(anyhow::anyhow!("Key manager not initialized"))
        }
    }
}

/// Initialize a single wallet with a provided private key
fn init_single_wallet(private_key: &str) -> Result<()> {
    // Set the global flag for single wallet mode
    unsafe { SINGLE_WALLET_MODE = true; }

    // Decode the private key from base58
    let key_bytes = bs58::decode(private_key.trim())
        .into_vec()
        .map_err(|e| anyhow::anyhow!("Failed to decode base58 private key: {:?}", e))?;

    // Create a keypair from the bytes
    let keypair = Keypair::from_bytes(&key_bytes)
        .map_err(|e| anyhow::anyhow!("Failed to create keypair from bytes: {:?}", e))?;

    // Get the public key for logging
    let pubkey = keypair.pubkey();

    info!("Initialized single wallet with public key: {}", pubkey);

    // Create our single wallet key manager
    let key_manager = KeyManager::new(
        // HODL keys (empty for single wallet mode)
        vec![],
        // Bank keys (empty for single wallet mode)
        vec![],
        // Explorer keys - just our single wallet
        vec![(keypair, LAMPORTS_PER_EXPLORER)],
        // RPC URL
        &env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        // Min balances - don't matter for single wallet mode
        0, 0, 0
    );

    // Store the key manager in our global static
    unsafe {
        KEY_MANAGER = Some(key_manager);
    }

    // Initialize metrics
    wallet_metrics::init();

    Ok(())
}

/// Initialize a single wallet with a randomly generated key
fn init_single_wallet_with_generated_key() -> Result<()> {
    // Set the global flag for single wallet mode
    unsafe { SINGLE_WALLET_MODE = true; }

    // Generate new keypair
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey();

    info!("Initialized single wallet with GENERATED public key: {}", pubkey);
    warn!("Using generated keypair for single wallet - this key will not persist across restarts!");

    // Create our single wallet key manager
    let key_manager = KeyManager::new(
        // HODL keys (empty for single wallet mode)
        vec![],
        // Bank keys (empty for single wallet mode)
        vec![],
        // Explorer keys - just our single wallet
        vec![(keypair, LAMPORTS_PER_EXPLORER)],
        // RPC URL
        &env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
        // Min balances - don't matter for single wallet mode
        0, 0, 0
    );

    // Store the key manager in our global static
    unsafe {
        KEY_MANAGER = Some(key_manager);
    }

    // Initialize metrics
    wallet_metrics::init();

    Ok(())
}

/// Initialize the tiered wallet system with the wallet configuration.
///
/// This function initializes our three-tier wallet system:
/// - HODL keys (cold storage)
/// - Bank keys (funding wallets)
/// - Explorer keys (transaction signing wallets)
///
/// Or if single wallet mode is enabled, initializes a single wallet.
async fn initialize_wallet_system(settings: &WalletSettings) -> Result<()> {
    use opentelemetry::global;
    use opentelemetry::trace::Tracer;
    use tracing::{info, error};

    // Use the runtime's tracer name for consistency
    let tracer_name = "qtrade_wallets";
    let tracer = global::tracer(tracer_name);

    let span_name = format!("{}::initialize_wallet_system", "wallets");

    tracer.in_span(span_name, |_cx| async move {
        // Check for single wallet mode
        if settings.single_wallet {
            info!("Initializing wallet system in SINGLE WALLET MODE");

            if let Some(private_key) = &settings.single_wallet_private_key {
                // Initialize a single wallet with the provided private key
                if let Err(e) = init_single_wallet(private_key) {
                    error!("Failed to initialize single wallet: {:?}", e);
                    return Err(anyhow::anyhow!("Failed to initialize single wallet: {:?}", e));
                }
            } else {
                warn!("Single wallet mode enabled but no private key provided. Using a generated key.");
                if let Err(e) = init_single_wallet_with_generated_key() {
                    error!("Failed to initialize single wallet with generated key: {:?}", e);
                    return Err(anyhow::anyhow!("Failed to initialize single wallet with generated key: {:?}", e));
                }
            }
        } else {
            // For now, we'll rely on environment variables for keys
            // Initialize the key manager with environment-provided keys
            if let Err(e) = init() {
                error!("Failed to initialize wallet system: {:?}", e);
                return Err(anyhow::anyhow!("Failed to initialize wallet system: {:?}", e));
            }
        }

        info!("Tiered wallet system initialized successfully");

        Ok(())
    }).await
}

