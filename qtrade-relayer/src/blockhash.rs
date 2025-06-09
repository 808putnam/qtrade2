use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, Once};
use solana_client::rpc_client::RpcClient;
use solana_sdk::hash::Hash;
use tokio::time::{interval, Duration};
use std::time::Instant;
use tracing::{debug, error, info, warn};
use crate::constants::QTRADE_RELAYER_TRACER_NAME;
use opentelemetry::global;
use opentelemetry::trace::Tracer;
use anyhow::Result;

const UPDATE_INTERVAL: Duration = Duration::from_secs(1);
const BLOCKHASH_MAX_AGE: Duration = Duration::from_secs(90); // Conservative max age for Solana blockhashes (150 blocks)

/// Structure for caching the latest blockhash
pub struct BlockhashCache {
    blockhash: Mutex<Hash>,
    last_update: Mutex<Instant>,
    is_initialized: AtomicBool,
    is_running: AtomicBool,
}

/// Global singleton instance of the BlockhashCache
static mut BLOCKHASH_CACHE_INSTANCE: Option<Arc<BlockhashCache>> = None;
static INIT_INSTANCE: Once = Once::new();

impl BlockhashCache {
    /// Get or initialize the global BlockhashCache instance
    pub fn instance() -> Arc<BlockhashCache> {
        unsafe {
            INIT_INSTANCE.call_once(|| {
                BLOCKHASH_CACHE_INSTANCE = Some(Arc::new(BlockhashCache {
                    blockhash: Mutex::new(Hash::default()),
                    last_update: Mutex::new(Instant::now()),
                    is_initialized: AtomicBool::new(false),
                    is_running: AtomicBool::new(false),
                }));
            });
            BLOCKHASH_CACHE_INSTANCE.clone().unwrap()
        }
    }

    /// Starts the blockhash update task
    pub async fn start_update_task(&self, rpc_url: &str) -> Result<()> {
        let already_running = self.is_running.swap(true, Ordering::SeqCst);
        if already_running {
            debug!("Blockhash cache update task is already running");
            return Ok(());
        }

        info!("Starting blockhash cache update task");
        let rpc_client = RpcClient::new(rpc_url.to_string());

        // Update once immediately before starting the interval
        self.update_blockhash(&rpc_client)?;

        // Clone Arc for the task
        let cache_ref = Arc::clone(&BlockhashCache::instance());

        // Spawn the update task
        tokio::spawn(async move {
            let tracer = global::tracer(QTRADE_RELAYER_TRACER_NAME);
            let mut update_interval = interval(UPDATE_INTERVAL);

            loop {
                update_interval.tick().await;

                let span_name = format!("{}::update_task", "blockhash_cache");
                let result = tracer.in_span(span_name, |_cx| {
                    if let Err(e) = cache_ref.update_blockhash(&rpc_client) {
                        error!("Failed to update blockhash: {:?}", e);
                    }
                    // Return an empty result since we're in a synchronous closure
                    Ok::<_, anyhow::Error>(())
                });

                // Handle any errors from the span operation itself
                if let Err(e) = result {
                    error!("Error in blockhash cache update task span: {:?}", e);
                }
            }
        });

        Ok(())
    }

    /// Updates the cached blockhash
    fn update_blockhash(&self, rpc_client: &RpcClient) -> Result<()> {
        match rpc_client.get_latest_blockhash() {
            Ok(hash) => {
                // Lock and update the blockhash
                if let Ok(mut blockhash) = self.blockhash.lock() {
                    *blockhash = hash;
                } else {
                    error!("Failed to lock blockhash for update");
                    return Err(anyhow::anyhow!("Failed to lock blockhash for update"));
                }

                // Lock and update the timestamp
                if let Ok(mut last_update) = self.last_update.lock() {
                    *last_update = Instant::now();
                    self.is_initialized.store(true, Ordering::SeqCst);
                    debug!("Updated blockhash cache: {}", hash);
                } else {
                    error!("Failed to lock last_update timestamp");
                    return Err(anyhow::anyhow!("Failed to lock last_update timestamp"));
                }

                Ok(())
            },
            Err(e) => {
                error!("Failed to get latest blockhash from RPC client: {:?}", e);
                Err(anyhow::anyhow!("Failed to get latest blockhash from RPC client: {:?}", e))
            }
        }
    }

    /// Gets the cached blockhash, or fetches a new one if too old
    pub fn get_blockhash(&self, rpc_client: &RpcClient) -> Result<Hash> {
        // Check if cache is initialized
        if !self.is_initialized.load(Ordering::SeqCst) {
            warn!("Blockhash cache not initialized yet, fetching directly");
            return rpc_client.get_latest_blockhash()
                .map_err(|e| anyhow::anyhow!("Failed to get latest blockhash: {:?}", e));
        }

        // Check if cached blockhash is still fresh
        let is_expired = {
            if let Ok(last_update) = self.last_update.lock() {
                last_update.elapsed() > BLOCKHASH_MAX_AGE
            } else {
                // If we can't lock, assume it's expired
                true
            }
        };

        if is_expired {
            warn!("Cached blockhash is expired, fetching new one");
            return rpc_client.get_latest_blockhash()
                .map_err(|e| anyhow::anyhow!("Failed to get latest blockhash: {:?}", e));
        }

        // Return the cached blockhash
        if let Ok(blockhash) = self.blockhash.lock() {
            Ok(*blockhash)
        } else {
            error!("Failed to lock blockhash for reading");
            Err(anyhow::anyhow!("Failed to lock blockhash for reading"))
        }
    }
}
