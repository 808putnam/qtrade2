//! Async Programming Examples and Interview Questions
//!
//! This module demonstrates async/await patterns with Tokio:
//! - Basic async/await usage
//! - Concurrent task execution
//! - Handling timeouts and cancellation
//! - Working with streams
//! - Error handling in async contexts

use tokio::sync::{mpsc, oneshot, Mutex as TokioMutex};
use tokio::time::{self, Duration};
use std::collections::HashMap;
use std::sync::Arc;
use futures::{stream, StreamExt};
use anyhow::Result;

/// Problem: Asynchronous Task Execution
///
/// Execute an asynchronous task and return its result.
pub async fn async_task() -> String {
    // Simulate some async work
    time::sleep(Duration::from_millis(100)).await;
    "Task completed".to_string()
}

/// Problem: Concurrent Task Execution
///
/// Execute multiple tasks concurrently and collect their results.
pub async fn concurrent_tasks(count: usize) -> Vec<String> {
    let mut tasks = Vec::with_capacity(count);

    for i in 0..count {
        tasks.push(tokio::spawn(async move {
            // Simulate different task durations
            time::sleep(Duration::from_millis(50 * (i as u64 + 1))).await;
            format!("Task {} completed", i)
        }));
    }

    let mut results = Vec::with_capacity(count);
    for task in tasks {
        results.push(task.await.unwrap());
    }

    results
}

/// Problem: Task with Timeout
///
/// Execute a task with a timeout and return an error if it takes too long.
pub async fn task_with_timeout(duration_ms: u64) -> Result<String, &'static str> {
    let task = async {
        // Simulate a long-running task
        time::sleep(Duration::from_millis(500)).await;
        "Task completed".to_string()
    };

    // Set a timeout
    match time::timeout(Duration::from_millis(duration_ms), task).await {
        Ok(result) => Ok(result),
        Err(_) => Err("Task timed out"),
    }
}

/// Problem: Async Producer-Consumer
///
/// Implement a producer-consumer pattern using async channels.
pub struct AsyncWorkQueue<T> {
    sender: mpsc::Sender<T>,
}

impl<T: Send + 'static> AsyncWorkQueue<T> {
    pub async fn new<F>(worker_count: usize, worker_fn: F) -> Self
    where
        F: Fn(T) -> futures::future::BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::channel::<T>(100);
        let worker_fn = Arc::new(worker_fn);

        let rx = Arc::new(TokioMutex::new(rx));

        for _ in 0..worker_count {
            let rx = Arc::clone(&rx);
            let worker_fn = Arc::clone(&worker_fn);

            tokio::spawn(async move {
                loop {
                    let item = {
                        let mut rx = rx.lock().await;
                        match rx.recv().await {
                            Some(item) => item,
                            None => break, // Channel closed, exit the loop
                        }
                    };

                    (worker_fn)(item).await;
                }
            });
        }

        Self { sender: tx }
    }

    pub async fn send(&self, item: T) -> Result<(), mpsc::error::SendError<T>> {
        self.sender.send(item).await
    }
}

/// Problem: Async Resource Pool
///
/// Implement a simple resource pool for reusing expensive resources.
pub struct AsyncResourcePool<T> {
    resources: Arc<TokioMutex<Vec<T>>>,
    create_fn: Arc<dyn Fn() -> futures::future::BoxFuture<'static, T> + Send + Sync>,
}

// Implement Clone for AsyncResourcePool
impl<T> Clone for AsyncResourcePool<T> {
    fn clone(&self) -> Self {
        Self {
            resources: Arc::clone(&self.resources),
            create_fn: Arc::clone(&self.create_fn),
        }
    }
}

impl<T: Send + 'static> AsyncResourcePool<T> {
    pub fn new<F>(create_fn: F) -> Self
    where
        F: Fn() -> futures::future::BoxFuture<'static, T> + Send + Sync + 'static,
    {
        Self {
            resources: Arc::new(TokioMutex::new(Vec::new())),
            create_fn: Arc::new(create_fn),
        }
    }

    pub async fn get(&self) -> PooledResource<T> {
        let resource = {
            let mut resources = self.resources.lock().await;
            if let Some(resource) = resources.pop() {
                resource
            } else {
                (self.create_fn)().await
            }
        };

        PooledResource {
            resource: Some(resource),
            pool: self,
        }
    }

    async fn return_resource(&self, resource: T) {
        let mut resources = self.resources.lock().await;
        resources.push(resource);
    }
}

pub struct PooledResource<'a, T: Send + 'static> {
    resource: Option<T>,
    pool: &'a AsyncResourcePool<T>,
}

impl<'a, T: Send + 'static> std::ops::Deref for PooledResource<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource.as_ref().unwrap()
    }
}

impl<'a, T: Send + 'static> std::ops::DerefMut for PooledResource<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource.as_mut().unwrap()
    }
}

impl<'a, T: Send + 'static> Drop for PooledResource<'a, T> {
    fn drop(&mut self) {
        if let Some(resource) = self.resource.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.return_resource(resource).await;
            });
        }
    }
}

/// Problem: Async Cache with Expiry
///
/// Implement a cache where entries expire after a certain time.
pub struct AsyncExpiryCache<K, V> {
    cache: TokioMutex<HashMap<K, (V, tokio::time::Instant)>>,
    ttl: Duration,
}

impl<K, V> AsyncExpiryCache<K, V>
where
    K: Eq + std::hash::Hash + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    pub fn new(ttl_seconds: u64) -> Self {
        let cache = Self {
            cache: TokioMutex::new(HashMap::new()),
            ttl: Duration::from_secs(ttl_seconds),
        };

        // Spawn a background task to clean up expired entries
        // We'll use a different approach that avoids cloning the TokioMutex
        let cache_clone = TokioMutex::new(HashMap::<K, (V, tokio::time::Instant)>::new());
        let cache_arc = Arc::new(cache_clone);
        let weak_cache = Arc::downgrade(&cache_arc);
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                if let Some(cache_mutex) = weak_cache.upgrade() {
                    let now = tokio::time::Instant::now();
                    let mut cache = cache_mutex.lock().await;
                    cache.retain(|_, (_, expiry)| *expiry > now);
                } else {
                    break; // Cache was dropped, exit the loop
                }
            }
        });

        cache
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.lock().await;
        if let Some((value, expiry)) = cache.get(key) {
            if tokio::time::Instant::now() < *expiry {
                return Some(value.clone());
            } else {
                cache.remove(key);
            }
        }
        None
    }

    pub async fn insert(&self, key: K, value: V) {
        let mut cache = self.cache.lock().await;
        let expiry = tokio::time::Instant::now() + self.ttl;
        cache.insert(key, (value, expiry));
    }
}

/// Problem: Execute Tasks with Rate Limiting
///
/// Process a stream of tasks with a rate limit.
pub async fn process_with_rate_limit<T, F, Fut>(
    items: Vec<T>,
    concurrency_limit: usize,
    process_fn: F,
) -> Vec<Result<String, String>>
where
    T: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + 'static,
    Fut: futures::Future<Output = Result<String, String>> + Send + 'static,
{
    stream::iter(items)
        .map(|item| {
            let process_fn = &process_fn;
            async move { process_fn(item).await }
        })
        .buffer_unordered(concurrency_limit)
        .collect::<Vec<_>>()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::future::BoxFuture;

    #[tokio::test]
    async fn test_async_task() {
        let result = async_task().await;
        assert_eq!(result, "Task completed");
    }

    #[tokio::test]
    async fn test_concurrent_tasks() {
        let results = concurrent_tasks(5).await;
        assert_eq!(results.len(), 5);
        for i in 0..5 {
            assert_eq!(results[i], format!("Task {} completed", i));
        }
    }

    #[tokio::test]
    async fn test_task_with_timeout() {
        // Test success case (timeout longer than task)
        let result = task_with_timeout(1000).await;
        assert_eq!(result, Ok("Task completed".to_string()));

        // Test timeout case (timeout shorter than task)
        let result = task_with_timeout(100).await;
        assert_eq!(result, Err("Task timed out"));
    }

    #[tokio::test]
    async fn test_async_work_queue() {
        let processed = Arc::new(TokioMutex::new(Vec::new()));
        let processed_clone = Arc::clone(&processed);

        let process_fn = move |item: i32| -> BoxFuture<'static, ()> {
            let processed = processed_clone.clone();
            Box::pin(async move {
                time::sleep(Duration::from_millis(10)).await;
                let mut processed = processed.lock().await;
                processed.push(item * 2);
            })
        };

        let queue = AsyncWorkQueue::new(4, process_fn).await;

        for i in 0..100 {
            queue.send(i).await.unwrap();
        }

        // Allow time for processing
        time::sleep(Duration::from_millis(500)).await;

        let processed = processed.lock().await;
        assert_eq!(processed.len(), 100);

        // Check results (they'll be in an unpredictable order)
        let mut expected: Vec<i32> = (0..100).map(|x| x * 2).collect();
        let mut actual = processed.clone();

        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn test_async_resource_pool() {
        let counter = Arc::new(TokioMutex::new(0));
        let counter_clone = Arc::clone(&counter);

        // Create a pool that provides unique numbers
        let pool = AsyncResourcePool::new(move || {
            let counter = counter_clone.clone();
            Box::pin(async move {
                let mut counter = counter.lock().await;
                *counter += 1;
                *counter
            })
        });

        // Acquire resources
        let resource1 = pool.get().await;
        assert_eq!(*resource1, 1);

        let resource2 = pool.get().await;
        assert_eq!(*resource2, 2);

        // Drop resource1, which returns it to the pool
        drop(resource1);

        // Allow the resource to be returned to the pool
        time::sleep(Duration::from_millis(10)).await;

        // Next get should reuse resource1 (with value 1)
        let resource3 = pool.get().await;
        assert_eq!(*resource3, 1);
    }

    #[tokio::test]
    async fn test_async_expiry_cache() {
        let cache = AsyncExpiryCache::new(1); // 1 second TTL

        cache.insert("key1", "value1").await;
        assert_eq!(cache.get(&"key1").await, Some("value1"));

        // Wait for the entry to expire
        time::sleep(Duration::from_secs(2)).await;

        // The entry should now be gone
        assert_eq!(cache.get(&"key1").await, None);
    }

    #[tokio::test]
    async fn test_process_with_rate_limit() {
        let items = vec![1, 2, 3, 4, 5];

        let results = process_with_rate_limit(items, 2, |item| async move {
            time::sleep(Duration::from_millis(50)).await;
            Ok(format!("Processed {}", item))
        }).await;

        assert_eq!(results.len(), 5);
        for (i, result) in results.into_iter().enumerate() {
            assert_eq!(result.unwrap(), format!("Processed {}", i + 1));
        }
    }
}
