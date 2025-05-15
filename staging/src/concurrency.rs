//! Concurrency Examples and Interview Questions
//!
//! This module covers standard library concurrency patterns:
//! - Threads and thread safety
//! - Mutex, RwLock, and Arc
//! - Channels for message passing
//! - Thread pools
//! - Atomics and memory ordering

use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Problem: Parallel Map Operation
///
/// Apply a function to each element of a vector in parallel and collect the results.
pub fn parallel_map<T, U, F>(input: Vec<T>, f: F) -> Vec<U>
where
    T: Send + 'static,
    U: Send + 'static,
    F: Fn(T) -> U + Send + Sync + 'static,
{
    let f = Arc::new(f);
    let mut handles = vec![];
    let (tx, rx) = mpsc::channel();

    for (i, item) in input.into_iter().enumerate() {
        let tx = tx.clone();
        let f = Arc::clone(&f);

        let handle = thread::spawn(move || {
            let result = f(item);
            tx.send((i, result)).expect("Channel send failed");
        });

        handles.push(handle);
    }

    drop(tx); // Drop the original sender to close the channel when all threads are done

    // Collect results in the original order
    let mut results: Vec<_> = rx.into_iter().collect();
    results.sort_by_key(|&(idx, _)| idx);

    results.into_iter().map(|(_, item)| item).collect()
}

/// Problem: Thread-safe Counter
///
/// Implement a counter that can be safely incremented from multiple threads.
pub struct ThreadSafeCounter {
    count: Mutex<usize>,
}

impl ThreadSafeCounter {
    pub fn new() -> Self {
        Self {
            count: Mutex::new(0),
        }
    }

    pub fn increment(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
    }

    pub fn get_count(&self) -> usize {
        *self.count.lock().unwrap()
    }
}

/// Problem: Thread-safe Counter with Atomic
///
/// Implement a counter using atomics for better performance.
pub struct AtomicCounter {
    count: AtomicUsize,
}

impl AtomicCounter {
    pub fn new() -> Self {
        Self {
            count: AtomicUsize::new(0),
        }
    }

    pub fn increment(&self) {
        self.count.fetch_add(1, Ordering::SeqCst);
    }

    pub fn get_count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

/// Problem: Producer-Consumer Pattern
///
/// Implement a producer-consumer pattern using channels.
pub struct WorkQueue<T> {
    sender: mpsc::Sender<T>,
}

impl<T: Send + 'static> WorkQueue<T> {
    pub fn new<F>(worker_count: usize, worker_fn: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        let (tx, rx) = mpsc::channel::<T>();
        let rx = Arc::new(Mutex::new(rx));
        let worker_fn = Arc::new(worker_fn);

        for _ in 0..worker_count {
            let rx = Arc::clone(&rx);
            let worker_fn = Arc::clone(&worker_fn);

            thread::spawn(move || loop {
                let item = {
                    let rx_guard = rx.lock().unwrap();
                    match rx_guard.recv() {
                        Ok(item) => item,
                        Err(_) => break, // Channel closed, exit the loop
                    }
                };

                worker_fn(item);
            });
        }

        Self { sender: tx }
    }

    pub fn send(&self, item: T) -> Result<(), mpsc::SendError<T>> {
        self.sender.send(item)
    }
}

/// Problem: Thread-safe Cache
///
/// Implement a thread-safe cache that allows multiple readers but exclusive writers.
pub struct ThreadSafeCache<K, V> {
    cache: RwLock<HashMap<K, V>>,
}

impl<K, V> ThreadSafeCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let cache = self.cache.read().unwrap();
        cache.get(key).cloned()
    }

    pub fn insert(&self, key: K, value: V) {
        let mut cache = self.cache.write().unwrap();
        cache.insert(key, value);
    }

    pub fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().unwrap();
        cache.remove(key)
    }
}

use std::collections::HashMap;

/// Problem: Parallel Download
///
/// Simulate downloading multiple files in parallel and wait for all to complete.
pub fn parallel_download<F>(urls: Vec<String>, download_fn: F) -> Vec<Result<String, String>>
where
    F: Fn(&str) -> Result<String, String> + Send + Sync + 'static,
{
    let download_fn = Arc::new(download_fn);
    let results = Arc::new(Mutex::new(vec![None; urls.len()]));
    let mut handles = vec![];

    for (i, url) in urls.into_iter().enumerate() {
        let download_fn = Arc::clone(&download_fn);
        let results = Arc::clone(&results);

        let handle = thread::spawn(move || {
            let result = download_fn(&url);
            let mut results = results.lock().unwrap();
            results[i] = Some(result);
        });

        handles.push(handle);
    }

    // Wait for all threads to finish
    for handle in handles {
        handle.join().unwrap();
    }

    // Collect results
    let results = Arc::try_unwrap(results)
        .unwrap()
        .into_inner()
        .unwrap()
        .into_iter()
        .map(Option::unwrap)
        .collect();

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_map() {
        let input = vec![1, 2, 3, 4, 5];
        let result = parallel_map(input, |x| x * x);
        assert_eq!(result, vec![1, 4, 9, 16, 25]);
    }

    #[test]
    fn test_thread_safe_counter() {
        let counter = Arc::new(ThreadSafeCounter::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    counter.increment();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get_count(), 1000);
    }

    #[test]
    fn test_atomic_counter() {
        let counter = Arc::new(AtomicCounter::new());
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    counter.increment();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(counter.get_count(), 1000);
    }

    #[test]
    fn test_work_queue() {
        let processed = Arc::new(Mutex::new(Vec::new()));
        let processed_clone = Arc::clone(&processed);

        let queue = WorkQueue::new(4, move |item: i32| {
            thread::sleep(Duration::from_millis(10));
            let mut processed = processed_clone.lock().unwrap();
            processed.push(item * 2);
        });

        for i in 0..100 {
            queue.send(i).unwrap();
        }

        // Allow time for processing
        thread::sleep(Duration::from_millis(500));

        let processed = processed.lock().unwrap();
        assert_eq!(processed.len(), 100);

        // Check results (they'll be in an unpredictable order)
        let mut expected: Vec<i32> = (0..100).map(|x| x * 2).collect();
        let mut actual = processed.clone();

        expected.sort();
        actual.sort();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_thread_safe_cache() {
        let cache = Arc::new(ThreadSafeCache::new());

        // Test basic functionality
        cache.insert("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));

        // Test concurrent reads
        let mut handles = vec![];
        for _ in 0..10 {
            let cache = Arc::clone(&cache);
            let handle = thread::spawn(move || {
                assert_eq!(cache.get(&"key1"), Some("value1"));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Test concurrent writes
        let mut handles = vec![];
        for i in 0..10 {
            let cache = Arc::clone(&cache);
            let key = format!("key{}", i + 2);
            let value = format!("value{}", i + 2);
            let handle = thread::spawn(move || {
                cache.insert(key, value);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        // Verify all writes happened
        for i in 0..10 {
            let key = format!("key{}", i + 2);
            let expected = format!("value{}", i + 2);
            assert_eq!(cache.get(&key), Some(expected));
        }
    }

    #[test]
    fn test_parallel_download() {
        let urls = vec![
            "https://example.com/1".to_string(),
            "https://example.com/2".to_string(),
            "https://example.com/3".to_string(),
            "https://example.com/fail".to_string(),
        ];

        let results = parallel_download(urls, |url| {
            thread::sleep(Duration::from_millis(10));
            if url.ends_with("fail") {
                Err(format!("Failed to download {}", url))
            } else {
                Ok(format!("Content of {}", url))
            }
        });

        assert_eq!(results.len(), 4);
        assert!(matches!(results[0], Ok(_)));
        assert!(matches!(results[1], Ok(_)));
        assert!(matches!(results[2], Ok(_)));
        assert!(matches!(results[3], Err(_)));
    }
}
