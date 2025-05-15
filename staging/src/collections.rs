//! Rust Collection Examples and Interview Questions
//!
//! This module covers common collection-related questions:
//! - Vec, HashMap, HashSet operations
//! - Custom data structures
//! - Collection algorithms and patterns

use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Reverse;

/// Problem: Two Sum
///
/// Given an array of integers and a target, return the indices of the two numbers that add up to the target.
/// Assume there is exactly one solution, and you may not use the same element twice.
pub fn two_sum(nums: &[i32], target: i32) -> Option<(usize, usize)> {
    let mut map = HashMap::new();

    for (i, &num) in nums.iter().enumerate() {
        let complement = target - num;

        if let Some(&j) = map.get(&complement) {
            return Some((j, i));
        }

        map.insert(num, i);
    }

    None
}

/// Problem: Find duplicate elements in an array
///
/// Return a vector containing all elements that appear more than once in the input vector.
pub fn find_duplicates<T: Eq + std::hash::Hash + Clone>(nums: &[T]) -> Vec<T> {
    let mut seen = HashSet::new();
    let mut duplicates = HashSet::new();

    for num in nums {
        if !seen.insert(num) {
            duplicates.insert(num.clone());
        }
    }

    duplicates.into_iter().collect()
}

/// Problem: Merge two sorted vectors
///
/// Given two sorted vectors, return a new sorted vector containing all elements from both.
pub fn merge_sorted_vecs<T: Ord + Clone>(a: &[T], b: &[T]) -> Vec<T> {
    let mut result = Vec::with_capacity(a.len() + b.len());
    let mut i = 0;
    let mut j = 0;

    while i < a.len() && j < b.len() {
        if a[i] <= b[j] {
            result.push(a[i].clone());
            i += 1;
        } else {
            result.push(b[j].clone());
            j += 1;
        }
    }

    // Add remaining elements from either vector
    result.extend_from_slice(&a[i..]);
    result.extend_from_slice(&b[j..]);

    result
}

/// Problem: Group Anagrams
///
/// Given an array of strings, group the anagrams together.
pub fn group_anagrams(strs: &[String]) -> Vec<Vec<String>> {
    let mut groups: HashMap<String, Vec<String>> = HashMap::new();

    for s in strs {
        // Sort characters to create a key for the anagram group
        let mut chars: Vec<char> = s.chars().collect();
        chars.sort_unstable();
        let key: String = chars.into_iter().collect();

        groups.entry(key).or_insert_with(Vec::new).push(s.clone());
    }

    groups.into_values().collect()
}

/// Problem: Top K Frequent Elements
///
/// Given a vector of integers, return the k most frequent elements.
pub fn top_k_frequent(nums: &[i32], k: usize) -> Vec<i32> {
    let mut freq_map = HashMap::new();
    for &num in nums {
        *freq_map.entry(num).or_insert(0) += 1;
    }

    let mut heap = BinaryHeap::new();
    for (&num, &count) in &freq_map {
        heap.push((count, num));
        if heap.len() > k {
            heap.pop();
        }
    }

    heap.into_iter()
        .map(|(_, num)| num)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

/// Problem: LRU Cache implementation
///
/// Implement a data structure for a Least Recently Used (LRU) cache.
pub struct LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    capacity: usize,
    cache: HashMap<K, (V, usize)>,  // (value, timestamp)
    time: usize,
}

impl<K, V> LRUCache<K, V>
where
    K: Eq + std::hash::Hash + Clone,
    V: Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::with_capacity(capacity),
            time: 0,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some((value, timestamp)) = self.cache.get_mut(key) {
            self.time += 1;
            *timestamp = self.time;
            Some(value.clone())
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        self.time += 1;

        // If key exists, update and return
        if self.cache.contains_key(&key) {
            self.cache.insert(key, (value, self.time));
            return;
        }

        // If at capacity, remove least recently used
        if self.cache.len() >= self.capacity {
            let lru_key = self.cache
                .iter()
                .min_by_key(|(_, (_, timestamp))| timestamp)
                .map(|(k, _)| k.clone())
                .unwrap();

            self.cache.remove(&lru_key);
        }

        // Insert new entry
        self.cache.insert(key, (value, self.time));
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_sum() {
        let nums = vec![2, 7, 11, 15];
        let target = 9;
        assert_eq!(two_sum(&nums, target), Some((0, 1)));

        let nums = vec![3, 2, 4];
        let target = 6;
        assert_eq!(two_sum(&nums, target), Some((1, 2)));
    }

    #[test]
    fn test_find_duplicates() {
        let nums = vec![1, 2, 3, 1, 3, 5];
        let mut result = find_duplicates(&nums);
        result.sort_unstable(); // Sort for comparison
        assert_eq!(result, vec![1, 3]);

        let nums = vec!["a", "b", "c", "a", "d", "c"];
        let mut result = find_duplicates(&nums);
        result.sort_unstable(); // Sort for comparison
        assert_eq!(result, vec!["a", "c"]);
    }

    #[test]
    fn test_merge_sorted_vecs() {
        let a = vec![1, 3, 5, 7];
        let b = vec![2, 4, 6, 8];
        assert_eq!(merge_sorted_vecs(&a, &b), vec![1, 2, 3, 4, 5, 6, 7, 8]);

        let a = vec![1, 2, 3];
        let b = vec![4, 5, 6];
        assert_eq!(merge_sorted_vecs(&a, &b), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn test_group_anagrams() {
        let strs = vec![
            "eat".to_string(),
            "tea".to_string(),
            "tan".to_string(),
            "ate".to_string(),
            "nat".to_string(),
            "bat".to_string(),
        ];

        let mut result = group_anagrams(&strs);

        // Sort each group for comparison
        for group in &mut result {
            group.sort();
        }

        // Sort groups by first element for stable comparison
        result.sort_by_key(|group| group[0].clone());

        let expected = vec![
            vec!["ate".to_string(), "eat".to_string(), "tea".to_string()],
            vec!["bat".to_string()],
            vec!["nat".to_string(), "tan".to_string()],
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_top_k_frequent() {
        let nums = vec![1, 1, 1, 2, 2, 3];
        let k = 2;
        let result = top_k_frequent(&nums, k);
        assert_eq!(result, vec![1, 2]);

        let nums = vec![1, 1, 1, 2, 2, 3, 3, 3, 3];
        let k = 2;
        let result = top_k_frequent(&nums, k);
        assert_eq!(result, vec![3, 1]);
    }

    #[test]
    fn test_lru_cache() {
        let mut cache = LRUCache::new(2);

        cache.put(1, 1);
        cache.put(2, 2);
        assert_eq!(cache.get(&1), Some(1));

        // This should evict key 2
        cache.put(3, 3);
        assert_eq!(cache.get(&2), None);

        // This should evict key 1
        cache.put(4, 4);
        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&3), Some(3));
        assert_eq!(cache.get(&4), Some(4));
    }
}
