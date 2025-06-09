use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once};
use solana_client::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use solana_sdk::nonce::State;
use solana_sdk::nonce::state::Data;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::instruction::{Instruction, AccountMeta};
use solana_sdk::system_instruction;
use solana_sdk::system_program;
use solana_sdk::sysvar;
use std::collections::VecDeque;
use tracing::{debug, error, info};
use tokio::time::{interval, Duration};
use anyhow::Result;
use std::str::FromStr;
use std::env;


use crate::constants::QTRADE_RELAYER_TRACER_NAME;
use crate::metrics::nonce::{
    record_nonce_acquisition, record_nonce_acquisition_with_latency,
    record_nonce_initialization_attempt, record_nonce_advancement_attempt,
    record_nonce_pool_state, record_nonce_release
};
use opentelemetry::global;
use opentelemetry::trace::Tracer;

const UPDATE_INTERVAL: Duration = Duration::from_secs(5); // Check nonce pool every 5 seconds
const MAX_RETRY_ATTEMPTS: usize = 3;
const NONCE_ACCOUNT_RENT_EXEMPT_LAMPORTS: u64 = 1_000_000; // Approximate, adjust as needed

// Environment variable names
const NONCE_ACCOUNTS_ENV: &str = "QTRADE_NONCE_ACCOUNTS";
const NONCE_AUTHORITY_SECRET_ENV: &str = "QTRADE_NONCE_AUTHORITY_SECRET";

/// Status of a nonce account
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NonceStatus {
    Available,
    InUse,
    NeedsInitialization,
    NeedsAdvance,
}

/// Representation of a nonce account with its current status
#[derive(Debug)]
pub struct NonceAccount {
    pub pubkey: Pubkey,
    pub status: NonceStatus,
    pub current_nonce: Option<Hash>,
    pub last_used: Option<std::time::Instant>,
}

/// Pool of nonce accounts
pub struct NoncePool {
    accounts: Mutex<VecDeque<NonceAccount>>,
    authority: Mutex<Option<Keypair>>,
    is_initialized: AtomicBool,
    is_running: AtomicBool,
    in_use_count: AtomicUsize,
}

/// Global singleton instance of the NoncePool
static mut NONCE_POOL_INSTANCE: Option<Arc<NoncePool>> = None;
static INIT_INSTANCE: Once = Once::new();

impl NoncePool {
    /// Get or initialize the global NoncePool instance
    pub fn instance() -> Arc<NoncePool> {
        unsafe {
            INIT_INSTANCE.call_once(|| {
                NONCE_POOL_INSTANCE = Some(Arc::new(NoncePool {
                    accounts: Mutex::new(VecDeque::new()),
                    authority: Mutex::new(None),
                    is_initialized: AtomicBool::new(false),
                    is_running: AtomicBool::new(false),
                    in_use_count: AtomicUsize::new(0),
                }));
            });
            NONCE_POOL_INSTANCE.clone().unwrap()
        }
    }

    /// Initialize the nonce pool with accounts and authority from environment variables
    pub fn init_from_env(&self) -> Result<()> {
        // Load nonce accounts from environment variable
        let nonce_accounts_str = env::var(NONCE_ACCOUNTS_ENV)
            .map_err(|_| anyhow::anyhow!("Environment variable {} not found", NONCE_ACCOUNTS_ENV))?;

        // Parse the comma-separated list of nonce account public keys
        let nonce_pubkeys_vec: Vec<Pubkey> = nonce_accounts_str
            .split(',')
            .filter_map(|s| {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    match Pubkey::from_str(trimmed) {
                        Ok(pubkey) => Some(pubkey),
                        Err(err) => {
                            error!("Failed to parse nonce pubkey {}: {}", trimmed, err);
                            None
                        }
                    }
                } else {
                    None
                }
            })
            .collect();

        if nonce_pubkeys_vec.is_empty() {
            return Err(anyhow::anyhow!("No valid nonce account pubkeys found in {}", NONCE_ACCOUNTS_ENV));
        }

        let nonce_pubkeys_count = nonce_pubkeys_vec.len();

        // Load nonce authority from environment variable
        let authority_secret = env::var(NONCE_AUTHORITY_SECRET_ENV)
            .map_err(|_| anyhow::anyhow!("Environment variable {} not found", NONCE_AUTHORITY_SECRET_ENV))?;

        // Parse the authority secret key
        let authority_keypair = match decode_keypair(&authority_secret) {
            Ok(keypair) => keypair,
            Err(err) => {
                return Err(anyhow::anyhow!("Failed to decode nonce authority keypair: {}", err));
            }
        };

        // Initialize the pool with nonce accounts
        let mut accounts_queue = VecDeque::new();
        for pubkey in &nonce_pubkeys_vec {
            accounts_queue.push_back(NonceAccount {
                pubkey: *pubkey,
                status: NonceStatus::NeedsInitialization, // Will be updated during refresh
                current_nonce: None,
                last_used: None,
            });
        }

        // Store accounts and authority in the pool
        {
            let mut accounts = self.accounts.lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock nonce accounts mutex"))?;
            *accounts = accounts_queue;
        }

        {
            let mut authority = self.authority.lock()
                .map_err(|_| anyhow::anyhow!("Failed to lock nonce authority mutex"))?;
            *authority = Some(authority_keypair);
        }

        self.is_initialized.store(true, Ordering::SeqCst);
        info!("Nonce pool initialized with {} accounts", nonce_pubkeys_count);

        Ok(())
    }

    /// Start the nonce pool maintenance task
    pub async fn start_maintenance_task(&self, rpc_url: &str) -> Result<()> {
        if !self.is_initialized.load(Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Nonce pool not initialized"));
        }

        let already_running = self.is_running.swap(true, Ordering::SeqCst);
        if already_running {
            debug!("Nonce pool maintenance task is already running");
            return Ok(());
        }

        info!("Starting nonce pool maintenance task");
        let rpc_client = RpcClient::new(rpc_url.to_string());

        // Refresh nonce accounts once immediately before starting the interval
        self.refresh_nonce_accounts(&rpc_client)?;

        // Clone Arc for the task
        let nonce_pool = Arc::clone(&NoncePool::instance());
        let rpc_url_owned = rpc_url.to_string();

        // Spawn the maintenance task
        tokio::spawn(async move {
            let tracer = global::tracer(QTRADE_RELAYER_TRACER_NAME);
            let mut update_interval = interval(UPDATE_INTERVAL);
            let rpc_client = RpcClient::new(rpc_url_owned);

            loop {
                update_interval.tick().await;

                let span_name = format!("{}::maintenance_task", "nonce_pool");
                let result = tracer.in_span(span_name, |_cx| {
                    if let Err(e) = nonce_pool.refresh_nonce_accounts(&rpc_client) {
                        error!("Failed to refresh nonce accounts: {:?}", e);
                    }
                    // Return an empty result since we're in a synchronous closure
                    Ok::<_, anyhow::Error>(())
                });

                // Handle any errors from the span operation itself
                if let Err(e) = result {
                    error!("Error in nonce pool maintenance task span: {:?}", e);
                }
            }
        });

        Ok(())
    }

    /// Refresh all nonce accounts in the pool
    fn refresh_nonce_accounts(&self, rpc_client: &RpcClient) -> Result<()> {
        let mut accounts = self.accounts.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock nonce accounts mutex"))?;

        // Authority is needed for initialization and advancing
        let authority = self.authority.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock nonce authority mutex"))?;

        if authority.is_none() {
            return Err(anyhow::anyhow!("Nonce authority not set"));
        }
        let authority = authority.as_ref().unwrap();

        // Check and update each nonce account
        for i in 0..accounts.len() {
            // Skip accounts that are currently in use
            if accounts[i].status == NonceStatus::InUse {
                continue;
            }

            let account = &mut accounts[i];
            let pubkey = account.pubkey;

            // Try to get the nonce account data
            match get_nonce_account_data(rpc_client, &pubkey) {
                Ok(Some(nonce_data)) => {
                    // Nonce account exists and has valid data
                    account.current_nonce = Some(nonce_data.blockhash());
                    account.status = NonceStatus::Available;
                    debug!("Nonce account {} is available with blockhash {}", pubkey, nonce_data.blockhash());
                },
                Ok(None) => {
                    // Nonce account exists but needs initialization
                    debug!("Nonce account {} needs initialization", pubkey);
                    account.status = NonceStatus::NeedsInitialization;
                    account.current_nonce = None;

                    // Try to initialize the nonce account
                    if let Err(e) = initialize_nonce_account(rpc_client, &pubkey, authority) {
                        error!("Failed to initialize nonce account {}: {}", pubkey, e);
                    } else {
                        // Initialization successful, update status in the next refresh cycle
                        info!("Successfully initialized nonce account {}", pubkey);
                    }
                },
                Err(e) => {
                    // Error checking nonce account
                    error!("Error checking nonce account {}: {}", pubkey, e);
                }
            }
        }

        // Log pool status
        let available_count = accounts.iter().filter(|a| a.status == NonceStatus::Available).count();
        let in_use_count = accounts.iter().filter(|a| a.status == NonceStatus::InUse).count();
        let init_needed = accounts.iter().filter(|a| a.status == NonceStatus::NeedsInitialization).count();
        let advance_needed = accounts.iter().filter(|a| a.status == NonceStatus::NeedsAdvance).count();
        let total_count = accounts.len();

        // Record metrics for monitoring
        record_nonce_pool_state(
            total_count,
            available_count,
            in_use_count,
            init_needed,
            advance_needed
        );

        info!(
            "Nonce pool status: {} available, {} in use, {} need initialization, {} need advance",
            available_count, in_use_count, init_needed, advance_needed
        );

        Ok(())
    }

    /// Acquire a nonce account from the pool
    pub fn acquire_nonce(&self, _rpc_client: &RpcClient) -> Result<(Pubkey, Hash)> {
        if !self.is_initialized.load(Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Nonce pool not initialized"));
        }

        let start_time = std::time::Instant::now();

        // Lock the accounts mutex
        let mut accounts = self.accounts.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock nonce accounts mutex"))?;

        // Find an available nonce account
        for i in 0..accounts.len() {
            if accounts[i].status == NonceStatus::Available {
                let account = &mut accounts[i];
                let pubkey = account.pubkey;
                let nonce = account.current_nonce
                    .ok_or_else(|| anyhow::anyhow!("Nonce value missing for available account {}", pubkey))?;

                // Mark as in use
                account.status = NonceStatus::InUse;
                account.last_used = Some(std::time::Instant::now());
                self.in_use_count.fetch_add(1, Ordering::SeqCst);

                // Calculate and record acquisition latency
                let elapsed_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                record_nonce_acquisition_with_latency(elapsed_ms);

                info!("Acquired nonce account {} with hash {}", pubkey, nonce);
                return Ok((pubkey, nonce));
            }
        }

        // No available nonce account found
        Err(anyhow::anyhow!("No available nonce accounts in the pool"))
    }

    /// Release a nonce account back to the pool
    pub fn release_nonce(&self, nonce_pubkey: &Pubkey) -> Result<()> {
        if !self.is_initialized.load(Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Nonce pool not initialized"));
        }

        // Lock the accounts mutex
        let mut accounts = self.accounts.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock nonce accounts mutex"))?;

        // Find the nonce account
        for account in accounts.iter_mut() {
            if &account.pubkey == nonce_pubkey {
                if account.status == NonceStatus::InUse {
                    // Mark as needing advance (after use, the nonce needs to be advanced for reuse)
                    account.status = NonceStatus::NeedsAdvance;
                    self.in_use_count.fetch_sub(1, Ordering::SeqCst);

                    // Record metric for nonce release
                    record_nonce_release();

                    info!("Released nonce account {} (needs advance)", nonce_pubkey);
                    return Ok(());
                } else {
                    return Err(anyhow::anyhow!("Nonce account {} not marked as in use", nonce_pubkey));
                }
            }
        }

        // Nonce account not found in the pool
        Err(anyhow::anyhow!("Nonce account {} not found in the pool", nonce_pubkey))
    }

    /// Get the nonce authority keypair
    pub fn get_authority(&self) -> Result<Keypair> {
        if !self.is_initialized.load(Ordering::SeqCst) {
            return Err(anyhow::anyhow!("Nonce pool not initialized"));
        }

        let authority = self.authority.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock nonce authority mutex"))?;

        if let Some(ref keypair) = *authority {
            Ok(keypair.insecure_clone())
        } else {
            Err(anyhow::anyhow!("Nonce authority not set"))
        }
    }

    /// Get nonce usage statistics
    pub fn get_stats(&self) -> Result<(usize, usize)> {
        let accounts = self.accounts.lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock nonce accounts mutex"))?;

        let total = accounts.len();
        let in_use = self.in_use_count.load(Ordering::SeqCst);

        Ok((total, in_use))
    }
}

/// Get nonce account data
fn get_nonce_account_data(rpc_client: &RpcClient, pubkey: &Pubkey) -> Result<Option<Data>> {
    // Try to get the account
    match rpc_client.get_account(pubkey) {
        Ok(account) => {
            // Use the recommended solana_client::nonce_utils::data_from_account function
            match solana_client::nonce_utils::data_from_account(&account) {
                Ok(data) => {
                    // Nonce account is initialized
                    Ok(Some(data))
                }
                Err(err) => {
                    // Check if the error indicates the nonce is uninitialized
                    if err.to_string().contains("invalid account data") {
                        // Account exists but nonce is not initialized
                        Ok(None)
                    } else {
                        // Other error, likely not a valid nonce account
                        Err(anyhow::anyhow!("Not a valid nonce account: {}", err))
                    }
                }
            }
        }
        Err(_) => {
            // Account doesn't exist or other RPC error
            Ok(None)
        }
    }
}

/// Initialize a nonce account
fn initialize_nonce_account(rpc_client: &RpcClient, nonce_pubkey: &Pubkey, authority: &Keypair) -> Result<()> {
    // Create instruction to initialize nonce account
    let instruction = system_instruction::create_nonce_account(
        &authority.pubkey(),
        nonce_pubkey,
        &authority.pubkey(), // Authority for the nonce account
        NONCE_ACCOUNT_RENT_EXEMPT_LAMPORTS,
    );

    // Create and send transaction
    let blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instruction,
        Some(&authority.pubkey()),
        &[authority],
        blockhash,
    );

    // Record initialization attempt in metrics
    let result = rpc_client.send_and_confirm_transaction(&transaction);
    let success = result.is_ok();
    record_nonce_initialization_attempt(success);

    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(anyhow::anyhow!("Failed to initialize nonce account: {}", err)),
    }
}

/// Advance a nonce account to get a new value
pub fn advance_nonce_account(rpc_client: &RpcClient, nonce_pubkey: &Pubkey, authority: &Keypair) -> Result<Hash> {
    // Create instruction to advance nonce
    let instruction = system_instruction::advance_nonce_account(
        nonce_pubkey,
        &authority.pubkey(),
    );

    // Create and send transaction
    let blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&authority.pubkey()),
        &[authority],
        blockhash,
    );

    // Send and confirm transaction
    let result = rpc_client.send_and_confirm_transaction(&transaction);
    let success = result.is_ok();
    record_nonce_advancement_attempt(success);

    // If there was an error, propagate it
    result?;

    // Get the new nonce value
    let nonce_account = rpc_client.get_account(nonce_pubkey)?;

    // Use data_from_account from solana_client::nonce_utils to deserialize the nonce account data
    let nonce_data = solana_client::nonce_utils::data_from_account(&nonce_account)
        .map_err(|err| anyhow::anyhow!("Failed to get nonce data: {}", err))?;

    Ok(nonce_data.blockhash())
}

/// Create instructions to use a nonce account in a transaction
pub fn create_nonce_instruction(nonce_pubkey: &Pubkey, authority_pubkey: &Pubkey) -> Instruction {
    let nonce_account_pubkey = nonce_pubkey;
    let instruction_accounts = vec![
        AccountMeta::new(*nonce_account_pubkey, false),
        AccountMeta::new_readonly(sysvar::recent_blockhashes::id(), false), // Ignoring deprecation for now
        AccountMeta::new_readonly(*authority_pubkey, true), // Authority must sign
    ];

    Instruction::new_with_bincode(
        system_program::id(),
        &system_instruction::SystemInstruction::AdvanceNonceAccount,
        instruction_accounts,
    )
}

/// Helper function to decode a base58-encoded keypair
fn decode_keypair(encoded: &str) -> Result<Keypair> {
    use bs58;

    let bytes = bs58::decode(encoded)
        .into_vec()
        .map_err(|e| anyhow::anyhow!("Failed to decode base58 string: {}", e))?;

    Keypair::from_bytes(&bytes)
        .map_err(|e| anyhow::anyhow!("Failed to create keypair from bytes: {}", e))
}

// Extension trait for keypair to allow cloning for the authority
trait KeypairExt {
    fn insecure_clone(&self) -> Keypair;
}

impl KeypairExt for Keypair {
    fn insecure_clone(&self) -> Keypair {
        Keypair::from_bytes(&self.to_bytes()).expect("Failed to clone keypair")
    }
}
