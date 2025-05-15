use std::sync::{Arc, Mutex};
use std::collections::{HashMap, VecDeque};
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use solana_sdk::system_instruction;
use solana_client::rpc_client::RpcClient;
use anyhow::{Result, anyhow};
use tracing::{info, warn, error};

/// Represents the tier of a key in the hierarchical key management system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyTier {
    /// HODL keys: Secure cold wallets that are only accessed to fund Bank keys
    Hodl,
    /// Bank keys: Used to fund Explorer keys with SOL
    Bank,
    /// Explorer keys: Used to execute actual transactions on Solana
    Explorer,
}

/// Represents the status of a key in the pool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyStatus {
    /// Key is available for use
    Available,
    /// Key is currently in use
    InUse,
    /// Key has been used and should be retired
    Used,
}

/// Represents a keypair along with its metadata
#[derive(Debug)]
pub struct KeyInfo {
    /// The actual Solana keypair
    keypair: Keypair,
    /// Current status of the keypair
    status: KeyStatus,
    /// When this key was last used (Unix timestamp)
    last_used: Option<u64>,
    /// Number of times this key has been used
    use_count: u32,
    /// Amount of SOL balance this key should maintain
    target_balance: u64,
}

// Manual implementation of Clone for KeyInfo since Keypair doesn't implement Clone
impl Clone for KeyInfo {
    fn clone(&self) -> Self {
        Self {
            keypair: Keypair::from_bytes(&self.keypair.to_bytes()).unwrap(),
            status: self.status,
            last_used: self.last_used,
            use_count: self.use_count,
            target_balance: self.target_balance,
        }
    }
}

impl KeyInfo {
    /// Create a new KeyInfo with the given keypair and target balance
    pub fn new(keypair: Keypair, target_balance: u64) -> Self {
        Self {
            keypair,
            status: KeyStatus::Available,
            last_used: None,
            use_count: 0,
            target_balance,
        }
    }

    /// Mark the key as in use
    pub fn mark_in_use(&mut self, timestamp: u64) {
        self.status = KeyStatus::InUse;
        self.last_used = Some(timestamp);
    }

    /// Mark the key as used (to be retired)
    pub fn mark_used(&mut self, timestamp: u64) {
        self.status = KeyStatus::Used;
        self.last_used = Some(timestamp);
        self.use_count += 1;
    }

    /// Mark the key as available again
    pub fn mark_available(&mut self) {
        self.status = KeyStatus::Available;
    }

    /// Get the keypair
    pub fn keypair(&self) -> &Keypair {
        &self.keypair
    }

    /// Get a clone of the keypair
    pub fn keypair_clone(&self) -> Keypair {
        Keypair::from_bytes(&self.keypair.to_bytes()).unwrap()
    }

    /// Get the public key
    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }

    /// Get the status of the key
    pub fn status(&self) -> KeyStatus {
        self.status
    }
}

/// A thread-safe pool of keys for a specific tier
#[derive(Clone)]
pub struct KeyPool {
    keys: Arc<Mutex<HashMap<Pubkey, KeyInfo>>>,
    available_keys: Arc<Mutex<VecDeque<Pubkey>>>,
    tier: KeyTier,
}

impl KeyPool {
    /// Create a new KeyPool for the specified tier with the given keypairs
    pub fn new(tier: KeyTier, keys: Vec<(Keypair, u64)>) -> Self {
        let mut key_map = HashMap::new();
        let mut available_queue = VecDeque::new();

        for (keypair, target_balance) in keys {
            let pubkey = keypair.pubkey();
            key_map.insert(pubkey, KeyInfo::new(keypair, target_balance));
            available_queue.push_back(pubkey);
        }

        Self {
            keys: Arc::new(Mutex::new(key_map)),
            available_keys: Arc::new(Mutex::new(available_queue)),
            tier,
        }
    }

    /// Get the next available keypair from the pool
    pub fn get_keypair(&self) -> Option<(Pubkey, Keypair)> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut available_keys = match self.available_keys.lock() {
            Ok(guard) => guard,
            Err(_) => return None,
        };

        let pubkey = match available_keys.pop_front() {
            Some(key) => key,
            None => return None,
        };

        let mut keys = match self.keys.lock() {
            Ok(guard) => guard,
            Err(_) => {
                // Push the key back since we couldn't get the lock
                available_keys.push_front(pubkey);
                return None;
            }
        };

        if let Some(key_info) = keys.get_mut(&pubkey) {
            if key_info.status == KeyStatus::Available {
                key_info.mark_in_use(now);
                return Some((pubkey, key_info.keypair_clone()));
            }
        }

        // If we get here, the key wasn't available after all
        available_keys.push_front(pubkey);
        None
    }

    /// Return a keypair to the pool or mark it as used
    pub fn return_keypair(&self, pubkey: &Pubkey, retire: bool) -> Result<()> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut keys = self.keys.lock().map_err(|e| anyhow!("Failed to lock keys: {:?}", e))?;

        if let Some(key_info) = keys.get_mut(pubkey) {
            if retire {
                // Mark as used so it won't be reused
                key_info.mark_used(now);
            } else {
                // Make available again and put back in the queue
                key_info.mark_available();
                let mut available_keys = self.available_keys.lock().map_err(|e| anyhow!("Failed to lock available_keys: {:?}", e))?;
                available_keys.push_back(*pubkey);
            }
            Ok(())
        } else {
            Err(anyhow!("Keypair not found in pool"))
        }
    }

    /// Check if the pool has any available keys
    pub fn has_available_keys(&self) -> bool {
        let available_keys = match self.available_keys.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        !available_keys.is_empty()
    }

    /// Get all keys in the pool with their status
    pub fn get_all_keys(&self) -> Result<Vec<(Pubkey, KeyStatus)>> {
        let keys = self.keys.lock().map_err(|e| anyhow!("Failed to lock keys: {:?}", e))?;

        let result = keys.iter()
            .map(|(pubkey, info)| (*pubkey, info.status()))
            .collect();

        Ok(result)
    }

    /// Get a reference to a specific key's info
    pub fn get_key_info(&self, pubkey: &Pubkey) -> Result<Option<KeyInfo>> {
        let keys = self.keys.lock().map_err(|e| anyhow!("Failed to lock keys: {:?}", e))?;

        Ok(keys.get(pubkey).cloned())
    }

    /// Add a new keypair to the pool
    pub fn add_keypair(&self, keypair: Keypair, target_balance: u64) -> Result<()> {
        let pubkey = keypair.pubkey();

        let mut keys = self.keys.lock().map_err(|e| anyhow!("Failed to lock keys: {:?}", e))?;
        let mut available_keys = self.available_keys.lock().map_err(|e| anyhow!("Failed to lock available_keys: {:?}", e))?;

        if keys.contains_key(&pubkey) {
            return Err(anyhow!("Keypair already exists in pool"));
        }

        keys.insert(pubkey, KeyInfo::new(keypair, target_balance));
        available_keys.push_back(pubkey);

        Ok(())
    }

    /// Remove a keypair from the pool
    pub fn remove_keypair(&self, pubkey: &Pubkey) -> Result<Option<Keypair>> {
        let mut keys = self.keys.lock().map_err(|e| anyhow!("Failed to lock keys: {:?}", e))?;

        // Remove from available_keys if present
        let mut available_keys = self.available_keys.lock().map_err(|e| anyhow!("Failed to lock available_keys: {:?}", e))?;
        available_keys.retain(|k| k != pubkey);

        // Remove from keys
        Ok(keys.remove(pubkey).map(|info| info.keypair_clone()))
    }

    /// Get the tier of this key pool
    pub fn tier(&self) -> KeyTier {
        self.tier
    }
}

/// Manager for the tiered key system
#[derive(Clone)]
pub struct KeyManager {
    hodl_pool: KeyPool,
    bank_pool: KeyPool,
    explorer_pool: KeyPool,
    rpc_client: Arc<RpcClient>,
    hodl_min_balance: u64,
    bank_min_balance: u64,
    explorer_min_balance: u64,
}

impl KeyManager {
    /// Create a new KeyManager
    pub fn new(
        hodl_keys: Vec<(Keypair, u64)>,
        bank_keys: Vec<(Keypair, u64)>,
        explorer_keys: Vec<(Keypair, u64)>,
        rpc_url: &str,
        hodl_min_balance: u64,
        bank_min_balance: u64,
        explorer_min_balance: u64,
    ) -> Self {
        let hodl_pool = KeyPool::new(KeyTier::Hodl, hodl_keys);
        let bank_pool = KeyPool::new(KeyTier::Bank, bank_keys);
        let explorer_pool = KeyPool::new(KeyTier::Explorer, explorer_keys);

        let rpc_client = Arc::new(RpcClient::new(rpc_url.to_string()));

        Self {
            hodl_pool,
            bank_pool,
            explorer_pool,
            rpc_client,
            hodl_min_balance,
            bank_min_balance,
            explorer_min_balance,
        }
    }

    /// Get a reference to the HODL key pool
    pub fn hodl_pool(&self) -> &KeyPool {
        &self.hodl_pool
    }

    /// Get a reference to the Bank key pool
    pub fn bank_pool(&self) -> &KeyPool {
        &self.bank_pool
    }

    /// Get a reference to the Explorer key pool
    pub fn explorer_pool(&self) -> &KeyPool {
        &self.explorer_pool
    }

    /// Get an available Explorer keypair for transaction signing
    pub fn get_explorer_keypair(&self) -> Option<(Pubkey, Keypair)> {
        let result = self.explorer_pool.get_keypair();

        if result.is_some() {
            // Record metric for explorer key acquisition
            crate::wallet_metrics::record_explorer_key_acquired();
        }

        result
    }

    /// Return an Explorer keypair to the pool or retire it
    pub fn return_explorer_keypair(&self, pubkey: &Pubkey, retire: bool) -> Result<()> {
        let result = self.explorer_pool.return_keypair(pubkey, retire);

        if result.is_ok() && retire {
            // Record metric for explorer key retirement
            crate::wallet_metrics::record_explorer_key_retired();
        }

        result
    }

    /// Create new Explorer keys and fund them from Bank keys
    pub async fn create_and_fund_explorer_keys(&self, count: usize, lamports_per_key: u64) -> Result<Vec<Pubkey>> {
        let mut new_explorer_pubkeys = Vec::new();

        for _ in 0..count {
            // Get a bank keypair to fund the new explorer
            let (bank_pubkey, bank_keypair) = match self.bank_pool.get_keypair() {
                Some(kp) => kp,
                None => return Err(anyhow!("No available bank keypairs")),
            };

            // Create a new explorer keypair
            let explorer_keypair = Keypair::new();
            let explorer_pubkey = explorer_keypair.pubkey();

            // Fund the explorer from the bank
            let result = self.transfer_sol(
                &bank_keypair,
                &explorer_pubkey,
                lamports_per_key,
            ).await;

            // Return the bank keypair to the pool
            self.bank_pool.return_keypair(&bank_pubkey, false)?;

            // Check if the funding was successful
            match result {
                Ok(_) => {
                    // Add the new explorer keypair to the pool
                    self.explorer_pool.add_keypair(explorer_keypair, lamports_per_key)?;
                    new_explorer_pubkeys.push(explorer_pubkey);
                    info!("Created and funded new explorer key: {}", explorer_pubkey);
                },
                Err(e) => {
                    error!("Failed to fund explorer key {}: {}", explorer_pubkey, e);
                }
            }
        }

        // Record metrics for new explorer keys created
        let created_count = new_explorer_pubkeys.len() as u64;
        if created_count > 0 {
            info!("Created {} new explorer keys", created_count);
            crate::wallet_metrics::record_explorer_keys_created(created_count);

            // Record balance metrics for the new keys
            for pubkey in &new_explorer_pubkeys {
                if let Ok(balance) = self.rpc_client.get_balance(pubkey) {
                    let balance_sol = balance as f64 / 1_000_000_000.0;
                    crate::wallet_metrics::record_key_balance("explorer", balance_sol);
                }
            }
        }

        Ok(new_explorer_pubkeys)
    }

    /// Fund Bank keys from HODL keys
    pub async fn fund_bank_keys(&self, lamports_per_key: u64) -> Result<usize> {
        let mut funded_count = 0;

        // Get all bank keys that need funding
        let bank_keys = self.bank_pool.get_all_keys()?;
        for (bank_pubkey, status) in bank_keys {
            if status != KeyStatus::Available {
                continue;
            }

            // Check current balance
            let balance = match self.rpc_client.get_balance(&bank_pubkey) {
                Ok(b) => b,
                Err(_) => continue,
            };

            if balance >= lamports_per_key {
                continue; // Already funded enough
            }

            // Get a hodl keypair to fund this bank key
            let (hodl_pubkey, hodl_keypair) = match self.hodl_pool.get_keypair() {
                Some(kp) => kp,
                None => {
                    break; // No more HODL keys available
                }
            };

            // Calculate how much to transfer
            let amount_to_transfer = lamports_per_key.saturating_sub(balance);

            // Fund the bank from the hodl
            let result = self.transfer_sol(
                &hodl_keypair,
                &bank_pubkey,
                amount_to_transfer,
            ).await;

            // Return the hodl keypair to the pool
            self.hodl_pool.return_keypair(&hodl_pubkey, false)?;

            // Check if the funding was successful
            match result {
                Ok(_) => {
                    info!("Funded bank key {} with {} SOL from HODL key",
                          bank_pubkey, amount_to_transfer as f64 / 1_000_000_000.0);
                    funded_count += 1;

                    // Record key balance metric
                    let bank_balance_sol = (balance + amount_to_transfer) as f64 / 1_000_000_000.0;
                    crate::wallet_metrics::record_key_balance("bank", bank_balance_sol);
                },
                Err(e) => {
                    error!("Failed to fund bank key {}: {}", bank_pubkey, e);
                }
            }
        }

        // Record metrics for bank keys funded
        if funded_count > 0 {
            crate::wallet_metrics::record_bank_keys_funded(funded_count as u64);
        }

        Ok(funded_count)
    }

    /// Transfer SOL from one account to another
    async fn transfer_sol(&self, from_keypair: &Keypair, to_pubkey: &Pubkey, lamports: u64) -> Result<String> {
        // Create a transfer instruction
        let instruction = system_instruction::transfer(
            &from_keypair.pubkey(),
            to_pubkey,
            lamports,
        );

        // Get a recent blockhash
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;

        // Create and sign the transaction
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&from_keypair.pubkey()),
            &[from_keypair],
            recent_blockhash,
        );

        // Send the transaction
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;

        Ok(signature.to_string())
    }

    /// Clean up used Explorer keys and recover funds
    ///
    /// This function does the following:
    /// 1. Finds all Explorer keys marked as Used
    /// 2. Retrieves any remaining SOL from these accounts
    /// 3. Transfers the funds back to a Bank key
    /// 4. Removes the Explorer key from the pool
    ///
    /// This ensures we don't leave funds sitting in unused Explorer accounts
    /// and properly clean up after using keys for transactions.
    pub async fn cleanup_explorer_keys(&self) -> Result<usize> {
        let mut cleaned_count = 0;
        let mut total_lamports_recovered = 0u64;

        let bank_pubkey = match self.bank_pool.get_keypair() {
            Some((pubkey, _)) => pubkey,
            None => return Err(anyhow!("No available bank keypairs for receiving recovered funds")),
        };

        // Get all explorer keys that are marked as used
        let explorer_keys = self.explorer_pool.get_all_keys()?;
        for (explorer_pubkey, status) in explorer_keys {
            if status != KeyStatus::Used {
                continue;
            }

            info!("Cleaning up used Explorer key: {}", explorer_pubkey);

            // Get the keypair for this explorer key
            let explorer_keypair = match self.explorer_pool.get_key_info(&explorer_pubkey)? {
                Some(info) if info.status() == KeyStatus::Used => info.keypair_clone(),
                _ => continue,
            };

            // Check current balance
            let balance = match self.rpc_client.get_balance(&explorer_pubkey) {
                Ok(b) => b,
                Err(e) => {
                    warn!("Failed to check balance of explorer key {}: {}", explorer_pubkey, e);
                    continue;
                }
            };

            info!("Explorer key {} has {} SOL remaining",
                  explorer_pubkey,
                  balance as f64 / 1_000_000_000.0);

            // If balance is very low, just remove the key without attempting transfer
            if balance < 10000 { // Minimum amount worth transferring (0.00001 SOL)
                info!("Balance too low to recover, removing key from pool");
                self.explorer_pool.remove_keypair(&explorer_pubkey)?;
                cleaned_count += 1;
                continue;
            }

            // Calculate transaction fee conservatively
            let estimated_fee = 5000; // 0.000005 SOL

            // Transfer funds back to a bank key, keeping enough for fee
            let lamports_to_transfer = balance.saturating_sub(estimated_fee);

            if lamports_to_transfer > 0 {
                info!("Attempting to recover {} SOL from explorer key to bank key",
                     lamports_to_transfer as f64 / 1_000_000_000.0);

                // Transfer funds back to bank
                match self.transfer_sol(
                    &explorer_keypair,
                    &bank_pubkey,
                    lamports_to_transfer
                ).await {
                    Ok(signature) => {
                        info!("Successfully recovered {} SOL from explorer key {} to bank (tx: {})",
                              lamports_to_transfer as f64 / 1_000_000_000.0,
                              explorer_pubkey,
                              signature);

                        // Track the recovered lamports
                        total_lamports_recovered += lamports_to_transfer;

                        // Remove the explorer key from the pool
                        self.explorer_pool.remove_keypair(&explorer_pubkey)?;
                        cleaned_count += 1;
                    },
                    Err(e) => {
                        error!("Failed to recover funds from explorer key {}: {}", explorer_pubkey, e);

                        // Even if transfer fails, remove the key since it's marked as used
                        // This prevents us from continuously attempting to recover from the same key
                        warn!("Removing explorer key {} from pool despite transfer failure", explorer_pubkey);
                        if let Err(remove_err) = self.explorer_pool.remove_keypair(&explorer_pubkey) {
                            error!("Additionally failed to remove key from pool: {}", remove_err);
                        } else {
                            cleaned_count += 1;
                        }
                    }
                }
            } else {
                // No funds to transfer but still remove the key
                info!("No funds to transfer after accounting for fees, removing key from pool");
                self.explorer_pool.remove_keypair(&explorer_pubkey)?;
                cleaned_count += 1;
            }
        }

        // Return the bank keypair to the pool
        self.bank_pool.return_keypair(&bank_pubkey, false)?;

        if cleaned_count > 0 {
            info!("Cleaned up {} used Explorer keys, recovered {} SOL",
                cleaned_count,
                total_lamports_recovered as f64 / 1_000_000_000.0);

            // Record metrics for explorer key fund recovery with actual lamports recovered
            crate::wallet_metrics::record_explorer_keys_funds_recovered(cleaned_count as u64, total_lamports_recovered);
        }

        Ok(cleaned_count)
    }

    /// Check if we need to create more explorer keys
    pub fn need_more_explorer_keys(&self, min_available: usize) -> bool {
        let available_count = match self.explorer_pool.get_all_keys() {
            Ok(keys) => keys.iter().filter(|(_, status)| *status == KeyStatus::Available).count(),
            Err(_) => 0,
        };

        available_count < min_available
    }

    /// Balance the key pools, ensuring adequate funding and key availability
    ///
    /// This function performs the key maintenance tasks in our tiered key structure:
    /// 1. Clean up used Explorer keys and recover their funds to Bank keys
    /// 2. Fund Bank keys from HODL keys if their balance is low
    /// 3. Create new Explorer keys and fund them from Bank keys if we need more
    pub async fn balance(&self,
        min_explorer_keys: usize,
        explorer_keys_to_create: usize,
        lamports_per_explorer: u64,
        lamports_per_bank: u64
    ) -> Result<()> {
        info!("Starting key pool balancing...");

        // Step 1: Clean up used explorer keys and recover their funds
        // This implements the requirement to close out explorer keys and retrieve their funds
        info!("Step 1: Cleaning up used Explorer keys and recovering funds");
        let cleaned_count = self.cleanup_explorer_keys().await?;
        info!("Cleaned up {} Explorer keys", cleaned_count);

        // Step 2: Fund bank keys from HODL keys if needed
        info!("Step 2: Funding Bank keys from HODL keys if needed");
        let funded_banks = self.fund_bank_keys(lamports_per_bank).await?;
        if funded_banks > 0 {
            info!("Funded {} Bank keys from HODL keys", funded_banks);
        } else {
            info!("No Bank keys needed funding at this time");
        }

        // Step 3: Create new explorer keys if needed and fund them from Bank keys
        info!("Step 3: Creating new Explorer keys if needed");
        let need_more = self.need_more_explorer_keys(min_explorer_keys);
        if need_more {
            info!("Need more Explorer keys, creating {} new ones", explorer_keys_to_create);
            let new_explorers = self.create_and_fund_explorer_keys(
                explorer_keys_to_create,
                lamports_per_explorer
            ).await?;

            if !new_explorers.is_empty() {
                info!("Successfully created and funded {} new Explorer keys", new_explorers.len());
            }
        } else {
            info!("Have sufficient Explorer keys available, no need to create more");
        }

        // Report on current key pool status
        let hodl_keys = self.hodl_pool.get_all_keys()?;
        let bank_keys = self.bank_pool.get_all_keys()?;
        let explorer_keys = self.explorer_pool.get_all_keys()?;

        let hodl_available = hodl_keys.iter().filter(|(_, status)| *status == KeyStatus::Available).count();
        let bank_available = bank_keys.iter().filter(|(_, status)| *status == KeyStatus::Available).count();
        let explorer_available = explorer_keys.iter().filter(|(_, status)| *status == KeyStatus::Available).count();

        info!("Key pool status after balancing: HODL ({} available/{} total), Bank ({} available/{} total), Explorer ({} available/{} total)",
            hodl_available, hodl_keys.len(),
            bank_available, bank_keys.len(),
            explorer_available, explorer_keys.len());

        // Record metrics about key pool sizes
        crate::wallet_metrics::record_key_pool_sizes(
            hodl_keys.len() as u64,
            hodl_available as u64,
            bank_keys.len() as u64,
            bank_available as u64,
            explorer_keys.len() as u64,
            explorer_available as u64
        );

        // Record balance metrics for each key
        self.record_key_balances().await?;

        Ok(())
    }

    /// Record balance metrics for all keys in the pool
    async fn record_key_balances(&self) -> Result<()> {
        // Sample a subset of keys from each pool to avoid too many RPC calls
        let max_keys_to_sample = 5;

        // Sample HODL keys
        let hodl_keys = self.hodl_pool.get_all_keys()?;
        for (i, (pubkey, _)) in hodl_keys.iter().enumerate() {
            if i >= max_keys_to_sample {
                break;
            }

            if let Ok(balance) = self.rpc_client.get_balance(pubkey) {
                let balance_sol = balance as f64 / 1_000_000_000.0;
                crate::wallet_metrics::record_key_balance("hodl", balance_sol);
            }
        }

        // Sample Bank keys
        let bank_keys = self.bank_pool.get_all_keys()?;
        for (i, (pubkey, _)) in bank_keys.iter().enumerate() {
            if i >= max_keys_to_sample {
                break;
            }

            if let Ok(balance) = self.rpc_client.get_balance(pubkey) {
                let balance_sol = balance as f64 / 1_000_000_000.0;
                crate::wallet_metrics::record_key_balance("bank", balance_sol);
            }
        }

        // Sample Explorer keys
        let explorer_keys = self.explorer_pool.get_all_keys()?;
        for (i, (pubkey, _)) in explorer_keys.iter().enumerate() {
            if i >= max_keys_to_sample {
                break;
            }

            if let Ok(balance) = self.rpc_client.get_balance(pubkey) {
                let balance_sol = balance as f64 / 1_000_000_000.0;
                crate::wallet_metrics::record_key_balance("explorer", balance_sol);
            }
        }

        Ok(())
    }
}
