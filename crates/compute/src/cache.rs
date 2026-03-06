//! MathCore Cache - Multi-level caching system for computation optimization
//!
//! This module provides:
//! - L1/L2 caching for computation results
//! - Expression caching for symbolic computation
//! - AST caching for parsed expressions
//! - Precomputation system for common values

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::symbolic::Expr;

/// Cache error types
#[derive(Debug, thiserror::Error)]
pub enum CacheError {
    #[error("Cache miss: {0}")]
    Miss(String),

    #[error("Invalid key: {0}")]
    InvalidKey(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// L1 Memory Cache - Fast in-memory LRU cache
pub struct L1Cache<K, V> {
    data: HashMap<K, CacheEntry<V>>,
    capacity: usize,
    ttl: Duration,
    hits: AtomicU64,
    misses: AtomicU64,
}

struct CacheEntry<V> {
    value: V,
    created: Instant,
    access_count: AtomicU64,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> L1Cache<K, V> {
    /// Create a new L1 cache
    pub fn new(capacity: usize, ttl_secs: u64) -> Self {
        Self {
            data: HashMap::new(),
            capacity,
            ttl: Duration::from_secs(ttl_secs),
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    /// Get a value from cache
    pub fn get<Q>(&self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if let Some(entry) = self.data.get(key) {
            if entry.created.elapsed() < self.ttl {
                self.hits.fetch_add(1, Ordering::Relaxed);
                // Update access count
                entry.access_count.fetch_add(1, Ordering::Relaxed);
                return Some(entry.value.clone());
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    /// Insert a value into cache
    pub fn insert(&mut self, key: K, value: V) {
        // Evict if at capacity
        if self.data.len() >= self.capacity {
            self.evict_lru();
        }

        self.data.insert(
            key,
            CacheEntry {
                value,
                created: Instant::now(),
                access_count: AtomicU64::new(0),
            },
        );
    }

    /// Check if key exists
    pub fn contains(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }

    /// Remove a key
    pub fn remove(&mut self, key: &K) -> Option<V> {
        self.data.remove(key).map(|e| e.value)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            hits as f64 / total as f64
        } else {
            0.0
        };

        CacheStats {
            size: self.data.len(),
            capacity: self.capacity,
            hits,
            misses,
            hit_rate,
        }
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some(lru_key) = self
            .data
            .iter()
            .min_by(|(_, e1), (_, e2)| {
                // First compare access count (lower first)
                let count1 = e1.access_count.load(Ordering::Relaxed);
                let count2 = e2.access_count.load(Ordering::Relaxed);
                let count_cmp = count1.cmp(&count2);
                if count_cmp != std::cmp::Ordering::Equal {
                    return count_cmp;
                }
                // If same count, evict older entry
                e1.created.cmp(&e2.created)
            })
            .map(|(k, _)| k.clone())
        {
            self.data.remove(&lru_key);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub capacity: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
}

/// Expression cache for symbolic computation
pub struct ExpressionCache {
    // Cache for parsed expressions (string -> AST)
    parsed: parking_lot::RwLock<L1Cache<String, Expr>>,
    // Cache for simplified expressions
    simplified: parking_lot::RwLock<L1Cache<String, String>>,
    // Cache for derivative results
    derivatives: parking_lot::RwLock<L1Cache<DerivativeKey, Expr>>,
    // Cache for integral results
    integrals: parking_lot::RwLock<L1Cache<DerivativeKey, Expr>>,
}

/// Key for derivative cache
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct DerivativeKey {
    expression: String,
    variable: String,
}

impl ExpressionCache {
    /// Create a new expression cache
    pub fn new(parsed_capacity: usize, simplified_capacity: usize, ttl_secs: u64) -> Self {
        Self {
            parsed: parking_lot::RwLock::new(L1Cache::new(parsed_capacity, ttl_secs)),
            simplified: parking_lot::RwLock::new(L1Cache::new(simplified_capacity, ttl_secs)),
            derivatives: parking_lot::RwLock::new(L1Cache::new(simplified_capacity * 2, ttl_secs)),
            integrals: parking_lot::RwLock::new(L1Cache::new(simplified_capacity * 2, ttl_secs)),
        }
    }

    /// Get cached parsed expression
    pub fn get_parsed(&self, expr: &str) -> Option<Expr> {
        self.parsed.read().get(expr)
    }

    /// Cache parsed expression
    pub fn cache_parsed(&self, expr: &str, ast: Expr) {
        self.parsed.write().insert(expr.to_string(), ast);
    }

    /// Get cached simplified expression
    pub fn get_simplified(&self, expr: &str) -> Option<String> {
        self.simplified.read().get(expr)
    }

    /// Cache simplified expression
    pub fn cache_simplified(&self, expr: &str, simplified: String) {
        self.simplified.write().insert(expr.to_string(), simplified);
    }

    /// Get cached derivative
    pub fn get_derivative(&self, expr: &str, var: &str) -> Option<Expr> {
        let key = DerivativeKey {
            expression: expr.to_string(),
            variable: var.to_string(),
        };
        self.derivatives.read().get(&key)
    }

    /// Cache derivative result
    pub fn cache_derivative(&self, expr: &str, var: &str, deriv: Expr) {
        let key = DerivativeKey {
            expression: expr.to_string(),
            variable: var.to_string(),
        };
        self.derivatives.write().insert(key, deriv);
    }

    /// Get cached integral
    pub fn get_integral(&self, expr: &str, var: &str) -> Option<Expr> {
        let key = DerivativeKey {
            expression: expr.to_string(),
            variable: var.to_string(),
        };
        self.integrals.read().get(&key)
    }

    /// Cache integral result
    pub fn cache_integral(&self, expr: &str, var: &str, integral: Expr) {
        let key = DerivativeKey {
            expression: expr.to_string(),
            variable: var.to_string(),
        };
        self.integrals.write().insert(key, integral);
    }

    /// Clear all caches
    pub fn clear(&self) {
        self.parsed.write().clear();
        self.simplified.write().clear();
        self.derivatives.write().clear();
        self.integrals.write().clear();
    }

    /// Get statistics
    pub fn stats(&self) -> ExpressionCacheStats {
        ExpressionCacheStats {
            parsed: self.parsed.read().stats(),
            simplified: self.simplified.read().stats(),
            derivatives: self.derivatives.read().stats(),
            integrals: self.integrals.read().stats(),
        }
    }
}

/// Expression cache statistics
#[derive(Debug, Clone)]
pub struct ExpressionCacheStats {
    pub parsed: CacheStats,
    pub simplified: CacheStats,
    pub derivatives: CacheStats,
    pub integrals: CacheStats,
}

/// Precomputation cache for expensive operations
pub struct PrecomputeCache<V> {
    cache: Arc<parking_lot::RwLock<HashMap<String, V>>>,
}

impl<V: Clone> PrecomputeCache<V> {
    /// Create a new precompute cache
    pub fn new() -> Self {
        Self {
            cache: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        }
    }

    /// Get a cached value
    pub fn get(&self, key: &str) -> Option<V> {
        self.cache.read().get(key).cloned()
    }

    /// Insert a value
    pub fn insert(&self, key: String, value: V) {
        self.cache.write().insert(key, value);
    }

    /// Check if key exists
    pub fn contains(&self, key: &str) -> bool {
        self.cache.read().contains_key(key)
    }

    /// Clear the cache
    pub fn clear(&self) {
        self.cache.write().clear();
    }

    /// Get cache size
    pub fn len(&self) -> usize {
        self.cache.read().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.cache.read().is_empty()
    }

    /// Get or compute
    pub fn get_or_insert_with<F>(&self, key: &str, f: F) -> V
    where
        F: FnOnce() -> V,
    {
        if let Some(v) = self.get(key) {
            return v;
        }
        let v = f();
        self.insert(key.to_string(), v.clone());
        v
    }
}

impl<V: Clone> Default for PrecomputeCache<V> {
    fn default() -> Self {
        Self::new()
    }
}

/// Thread-safe computation cache with atomic operations
pub struct AtomicCache<V> {
    data: Arc<parking_lot::RwLock<HashMap<String, V>>>,
    generation: AtomicU64,
}

impl<V: Clone> AtomicCache<V> {
    /// Create a new atomic cache
    pub fn new() -> Self {
        Self {
            data: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            generation: AtomicU64::new(0),
        }
    }

    /// Get a value
    pub fn get(&self, key: &str) -> Option<V> {
        self.data.read().get(key).cloned()
    }

    /// Insert a value
    pub fn insert(&self, key: String, value: V) {
        self.data.write().insert(key, value);
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Invalidate cache (increment generation)
    pub fn invalidate(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current generation
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    /// Clear cache
    pub fn clear(&self) {
        self.data.write().clear();
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

impl<V: Clone> Default for AtomicCache<V> {
    fn default() -> Self {
        Self::new()
    }
}

///展开缓存 - 存储表达式展开结果
pub struct ExpansionCache {
    cache: PrecomputeCache<ExpansionResult>,
}

/// 展开结果
#[derive(Debug, Clone)]
pub struct ExpansionResult {
    pub expanded: String,
    pub terms: usize,
    pub operations: usize,
}

impl ExpansionCache {
    /// Create a new expansion cache
    pub fn new() -> Self {
        Self {
            cache: PrecomputeCache::new(),
        }
    }

    /// Get cached expansion
    pub fn get(&self, expr: &str) -> Option<ExpansionResult> {
        self.cache.get(&Self::key(expr))
    }

    /// Cache expansion result
    pub fn insert(&self, expr: &str, result: ExpansionResult) {
        self.cache.insert(Self::key(expr), result);
    }

    /// Generate cache key
    fn key(expr: &str) -> String {
        format!("expand:{}", expr)
    }

    /// Clear cache
    pub fn clear(&self) {
        self.cache.clear();
    }
}

impl Default for ExpansionCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Global cache instances
lazy_static::lazy_static! {
    /// Global expression cache
    pub static ref EXPRESSION_CACHE: ExpressionCache = ExpressionCache::new(1000, 500, 300);

    /// Global precompute cache
    pub static ref PRECOMPUTE_CACHE: PrecomputeCache<f64> = PrecomputeCache::<f64>::new();

    /// Global expansion cache
    pub static ref EXPANSION_CACHE: ExpansionCache = ExpansionCache::new();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_l1_cache_basic() {
        let mut cache: L1Cache<String, i32> = L1Cache::new(10, 60);
        cache.insert("key1".to_string(), 42);
        assert_eq!(cache.get(&"key1".to_string()), Some(42));
    }

    #[test]
    fn test_l1_cache_eviction() {
        let mut cache: L1Cache<String, i32> = L1Cache::new(2, 60);
        cache.insert("key1".to_string(), 1);
        cache.insert("key2".to_string(), 2);
        cache.insert("key3".to_string(), 3); // Should evict key1

        assert!(cache.get(&"key1".to_string()).is_none());
        assert!(cache.get(&"key2".to_string()).is_some());
        assert!(cache.get(&"key3".to_string()).is_some());
    }

    #[test]
    fn test_l1_cache_stats() {
        let mut cache: L1Cache<String, i32> = L1Cache::new(10, 60);
        cache.insert("key1".to_string(), 42);
        cache.get(&"key1".to_string());
        cache.get(&"missing".to_string());

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_expression_cache() {
        let mut cache = ExpressionCache::new(10, 10, 60);

        // Test parsed cache
        let expr = "x^2 + 2*x + 1";
        let ast = crate::symbolic::parse(expr).unwrap();
        cache.cache_parsed(expr, ast.clone());

        let cached = cache.get_parsed(expr);
        assert!(cached.is_some());
    }

    #[test]
    fn test_precompute_cache() {
        let cache: PrecomputeCache<i32> = PrecomputeCache::new();
        cache.insert("factorial_5".to_string(), 120);

        assert_eq!(cache.get(&"factorial_5".to_string()), Some(120));
    }

    #[test]
    fn test_atomic_cache() {
        let cache: AtomicCache<i32> = AtomicCache::new();

        cache.insert("key1".to_string(), 42);
        assert_eq!(cache.get(&"key1".to_string()), Some(42));

        let gen = cache.generation();
        assert!(gen > 0);
    }
}
