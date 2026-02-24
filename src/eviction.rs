/// Configurable eviction policies for caches
/// 
/// This module provides different eviction strategies:
/// - LRU (Least Recently Used)
/// - LFU (Least Frequently Used)
/// - FIFO (First In First Out)
/// - Random
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use crate::CacheError;

/// Eviction policy trait
pub trait EvictionPolicy<K>: Send + Sync {
    /// Record an access to a key
    fn access(&mut self, key: &K);
    
    /// Record an insertion of a key
    fn insert(&mut self, key: K);
    
    /// Remove a key from tracking
    fn remove(&mut self, key: &K);
    
    /// Select a key to evict
    fn evict(&mut self) -> Option<K>;
    
    /// Clear all tracked data
    fn clear(&mut self);
}

/// LRU (Least Recently Used) eviction policy
pub struct LruPolicy<K> {
    access_order: Vec<K>,
}

impl<K: Clone + Eq> Default for LruPolicy<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Clone + Eq> LruPolicy<K> {
    pub fn new() -> Self {
        LruPolicy {
            access_order: Vec::new(),
        }
    }
}

impl<K: Clone + Eq + Send + Sync> EvictionPolicy<K> for LruPolicy<K> {
    fn access(&mut self, key: &K) {
        if let Some(pos) = self.access_order.iter().position(|k| k == key) {
            self.access_order.remove(pos);
        }
        self.access_order.push(key.clone());
    }
    
    fn insert(&mut self, key: K) {
        self.access_order.push(key);
    }
    
    fn remove(&mut self, key: &K) {
        if let Some(pos) = self.access_order.iter().position(|k| k == key) {
            self.access_order.remove(pos);
        }
    }
    
    fn evict(&mut self) -> Option<K> {
        if !self.access_order.is_empty() {
            Some(self.access_order.remove(0))
        } else {
            None
        }
    }
    
    fn clear(&mut self) {
        self.access_order.clear();
    }
}

/// LFU (Least Frequently Used) eviction policy
pub struct LfuPolicy<K> {
    frequency: HashMap<K, usize>,
    insertion_order: Vec<K>,
}

impl<K: Clone + Eq + Hash> Default for LfuPolicy<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Clone + Eq + Hash> LfuPolicy<K> {
    pub fn new() -> Self {
        LfuPolicy {
            frequency: HashMap::new(),
            insertion_order: Vec::new(),
        }
    }
}

impl<K: Clone + Eq + Hash + Send + Sync> EvictionPolicy<K> for LfuPolicy<K> {
    fn access(&mut self, key: &K) {
        *self.frequency.entry(key.clone()).or_insert(0) += 1;
    }
    
    fn insert(&mut self, key: K) {
        self.frequency.insert(key.clone(), 1);
        self.insertion_order.push(key);
    }
    
    fn remove(&mut self, key: &K) {
        self.frequency.remove(key);
        if let Some(pos) = self.insertion_order.iter().position(|k| k == key) {
            self.insertion_order.remove(pos);
        }
    }
    
    fn evict(&mut self) -> Option<K> {
        if self.frequency.is_empty() {
            return None;
        }
        
        // Find key with minimum frequency, using insertion order as tiebreaker
        let min_key = self.insertion_order
            .iter()
            .filter(|k| self.frequency.contains_key(k))
            .min_by_key(|k| self.frequency.get(k).unwrap_or(&0))
            .cloned();
        
        if let Some(ref key) = min_key {
            self.remove(key);
        }
        
        min_key
    }
    
    fn clear(&mut self) {
        self.frequency.clear();
        self.insertion_order.clear();
    }
}

/// FIFO (First In First Out) eviction policy
pub struct FifoPolicy<K> {
    insertion_order: Vec<K>,
}

impl<K: Clone + Eq> Default for FifoPolicy<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Clone + Eq> FifoPolicy<K> {
    pub fn new() -> Self {
        FifoPolicy {
            insertion_order: Vec::new(),
        }
    }
}

impl<K: Clone + Eq + Send + Sync> EvictionPolicy<K> for FifoPolicy<K> {
    fn access(&mut self, _key: &K) {
        // FIFO doesn't care about access patterns
    }
    
    fn insert(&mut self, key: K) {
        self.insertion_order.push(key);
    }
    
    fn remove(&mut self, key: &K) {
        if let Some(pos) = self.insertion_order.iter().position(|k| k == key) {
            self.insertion_order.remove(pos);
        }
    }
    
    fn evict(&mut self) -> Option<K> {
        if !self.insertion_order.is_empty() {
            Some(self.insertion_order.remove(0))
        } else {
            None
        }
    }
    
    fn clear(&mut self) {
        self.insertion_order.clear();
    }
}

/// Random eviction policy
pub struct RandomPolicy<K> {
    keys: Vec<K>,
}

impl<K: Clone + Eq> Default for RandomPolicy<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Clone + Eq> RandomPolicy<K> {
    pub fn new() -> Self {
        RandomPolicy {
            keys: Vec::new(),
        }
    }
}

impl<K: Clone + Eq + Send + Sync> EvictionPolicy<K> for RandomPolicy<K> {
    fn access(&mut self, _key: &K) {
        // Random policy doesn't care about access patterns
    }
    
    fn insert(&mut self, key: K) {
        self.keys.push(key);
    }
    
    fn remove(&mut self, key: &K) {
        if let Some(pos) = self.keys.iter().position(|k| k == key) {
            self.keys.remove(pos);
        }
    }
    
    fn evict(&mut self) -> Option<K> {
        if self.keys.is_empty() {
            return None;
        }
        
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;
        
        let mut hasher = DefaultHasher::new();
        hasher.write_usize(self.keys.len());
        let random_idx = (hasher.finish() as usize) % self.keys.len();
        
        Some(self.keys.remove(random_idx))
    }
    
    fn clear(&mut self) {
        self.keys.clear();
    }
}

/// Generic cache with configurable eviction policy
pub struct EvictableCache<K, V, P>
where
    K: Clone + Eq + Hash,
    V: Clone,
    P: EvictionPolicy<K>,
{
    capacity: usize,
    data: HashMap<K, V>,
    policy: P,
}

impl<K, V, P> EvictableCache<K, V, P>
where
    K: Clone + Eq + Hash,
    V: Clone,
    P: EvictionPolicy<K>,
{
    pub fn new(capacity: usize, policy: P) -> Self {
        assert!(capacity > 0, "Cache capacity must be greater than 0");
        EvictableCache {
            capacity,
            data: HashMap::new(),
            policy,
        }
    }
    
    pub fn get(&mut self, key: &K) -> Option<V> {
        if let Some(value) = self.data.get(key) {
            self.policy.access(key);
            Some(value.clone())
        } else {
            None
        }
    }
    
    pub fn put(&mut self, key: K, value: V) {
        if self.data.contains_key(&key) {
            self.data.insert(key.clone(), value);
            self.policy.access(&key);
        } else {
            if self.data.len() >= self.capacity {
                if let Some(evict_key) = self.policy.evict() {
                    self.data.remove(&evict_key);
                }
            }
            
            self.data.insert(key.clone(), value);
            self.policy.insert(key);
        }
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
        self.policy.clear();
    }
}

/// Thread-safe wrapper for evictable cache
pub struct ThreadSafeEvictableCache<K, V, P>
where
    K: Clone + Eq + Hash,
    V: Clone,
    P: EvictionPolicy<K>,
{
    inner: Arc<Mutex<EvictableCache<K, V, P>>>,
}

impl<K, V, P> Clone for ThreadSafeEvictableCache<K, V, P>
where
    K: Clone + Eq + Hash,
    V: Clone,
    P: EvictionPolicy<K>,
{
    fn clone(&self) -> Self {
        ThreadSafeEvictableCache {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<K, V, P> ThreadSafeEvictableCache<K, V, P>
where
    K: Clone + Eq + Hash,
    V: Clone,
    P: EvictionPolicy<K>,
{
    pub fn new(capacity: usize, policy: P) -> Self {
        ThreadSafeEvictableCache {
            inner: Arc::new(Mutex::new(EvictableCache::new(capacity, policy))),
        }
    }
    
    pub fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        let mut inner = self.inner.lock()?;
        Ok(inner.get(key))
    }
    
    pub fn put(&self, key: K, value: V) -> Result<(), CacheError> {
        let mut inner = self.inner.lock()?;
        inner.put(key, value);
        Ok(())
    }
    
    pub fn len(&self) -> Result<usize, CacheError> {
        let inner = self.inner.lock()?;
        Ok(inner.len())
    }
    
    pub fn is_empty(&self) -> Result<bool, CacheError> {
        let inner = self.inner.lock()?;
        Ok(inner.is_empty())
    }
    
    pub fn capacity(&self) -> Result<usize, CacheError> {
        let inner = self.inner.lock()?;
        Ok(inner.capacity())
    }
    
    pub fn clear(&self) -> Result<(), CacheError> {
        let mut inner = self.inner.lock()?;
        inner.clear();
        Ok(())
    }
}


