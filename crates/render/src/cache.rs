//! Cache module - Multi-level caching system
//!
//! Provides L1 (memory), L2 (disk), and precompute caching.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// L1 Memory Cache - in-memory LRU cache
pub struct L1Cache<K, V> {
    data: HashMap<K, (V, Instant)>,
    capacity: usize,
    ttl_secs: u64,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> L1Cache<K, V> {
    pub fn new(capacity: usize, ttl_secs: u64) -> Self {
        Self {
            data: HashMap::new(),
            capacity,
            ttl_secs,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        self.data.get(key).and_then(|(v, time)| {
            if time.elapsed().as_secs() < self.ttl_secs {
                Some(v.clone())
            } else {
                None
            }
        })
    }

    pub fn insert(&mut self, key: K, value: V) {
        if self.data.len() >= self.capacity {
            if let Some(oldest_key) = self
                .data
                .iter()
                .min_by_key(|(_, (_, t))| t.elapsed())
                .map(|(k, _)| k.clone())
            {
                self.data.remove(&oldest_key);
            }
        }
        self.data.insert(key, (value, Instant::now()));
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Precompute Cache - stores expensive computation results
pub struct PrecomputeCache<V> {
    cache: Arc<RwLock<HashMap<String, V>>>,
}

impl<V: Clone> PrecomputeCache<V> {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get(&self, key: &str) -> Option<V> {
        self.cache.read().ok()?.get(key).cloned()
    }

    pub fn insert(&self, key: String, value: V) {
        if let Ok(mut c) = self.cache.write() {
            c.insert(key, value);
        }
    }

    pub fn clear(&self) {
        if let Ok(mut c) = self.cache.write() {
            c.clear();
        }
    }
}

impl<V: Clone> Default for PrecomputeCache<V> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l1_cache() {
        let mut cache: L1Cache<String, i32> = L1Cache::new(10, 60);
        cache.insert("key1".to_string(), 42);
        assert_eq!(cache.get(&"key1".to_string()), Some(42));
    }

    #[test]
    fn test_precompute_cache() {
        let cache: PrecomputeCache<i32> = PrecomputeCache::new();
        cache.insert("factorial_5".to_string(), 120);
        assert_eq!(cache.get(&"factorial_5".to_string()), Some(120));
    }
}
