/// Sharded cache implementation for improved concurrency
/// 
/// This module provides a sharded cache that splits the keyspace across
/// multiple independent cache shards. This reduces lock contention by
/// allowing concurrent access to different shards.
/// 
/// Trade-off: Provides approximate LRU semantics (per-shard LRU) rather
/// than global LRU, in exchange for better concurrent performance.
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};
use std::collections::hash_map::DefaultHasher;

#[derive(Clone)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

struct LruCacheInner<K, V> {
    capacity: usize,
    map: HashMap<K, usize>,
    nodes: Vec<Option<Node<K, V>>>,
    head: Option<usize>,
    tail: Option<usize>,
    free_list: Vec<usize>,
}

impl<K: Clone + Eq + Hash, V: Clone> LruCacheInner<K, V> {
    fn new(capacity: usize) -> Self {
        LruCacheInner {
            capacity,
            map: HashMap::new(),
            nodes: Vec::new(),
            head: None,
            tail: None,
            free_list: Vec::new(),
        }
    }

    fn move_to_front(&mut self, idx: usize) {
        if self.head == Some(idx) {
            return;
        }

        let (prev, next) = {
            let node = self.nodes[idx].as_ref().unwrap();
            (node.prev, node.next)
        };

        if let Some(prev_idx) = prev {
            if let Some(ref mut prev_node) = self.nodes[prev_idx] {
                prev_node.next = next;
            }
        }

        if let Some(next_idx) = next {
            if let Some(ref mut next_node) = self.nodes[next_idx] {
                next_node.prev = prev;
            }
        }

        if self.tail == Some(idx) {
            self.tail = prev;
        }

        if let Some(old_head_idx) = self.head {
            if let Some(ref mut old_head) = self.nodes[old_head_idx] {
                old_head.prev = Some(idx);
            }
        }

        if let Some(ref mut node) = self.nodes[idx] {
            node.prev = None;
            node.next = self.head;
        }

        self.head = Some(idx);

        if self.tail.is_none() {
            self.tail = Some(idx);
        }
    }

    fn get(&mut self, key: &K) -> Option<V> {
        if let Some(&idx) = self.map.get(key) {
            self.move_to_front(idx);
            self.nodes[idx].as_ref().map(|node| node.value.clone())
        } else {
            None
        }
    }

    fn put(&mut self, key: K, value: V) {
        if let Some(&idx) = self.map.get(&key) {
            if let Some(ref mut node) = self.nodes[idx] {
                node.value = value;
            }
            self.move_to_front(idx);
            return;
        }

        if self.map.len() >= self.capacity {
            self.evict_lru();
        }

        let idx = if let Some(free_idx) = self.free_list.pop() {
            free_idx
        } else {
            let new_idx = self.nodes.len();
            self.nodes.push(None);
            new_idx
        };

        let new_node = Node {
            key: key.clone(),
            value,
            prev: None,
            next: self.head,
        };

        self.nodes[idx] = Some(new_node);
        self.map.insert(key, idx);

        if let Some(old_head_idx) = self.head {
            if let Some(ref mut old_head) = self.nodes[old_head_idx] {
                old_head.prev = Some(idx);
            }
        }

        self.head = Some(idx);

        if self.tail.is_none() {
            self.tail = Some(idx);
        }
    }

    fn evict_lru(&mut self) {
        if let Some(tail_idx) = self.tail {
            if let Some(tail_node) = self.nodes[tail_idx].take() {
                self.map.remove(&tail_node.key);
                self.tail = tail_node.prev;

                if let Some(new_tail_idx) = tail_node.prev {
                    if let Some(ref mut new_tail) = self.nodes[new_tail_idx] {
                        new_tail.next = None;
                    }
                } else {
                    self.head = None;
                }

                self.free_list.push(tail_idx);
            }
        }
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn clear(&mut self) {
        self.map.clear();
        self.nodes.clear();
        self.head = None;
        self.tail = None;
        self.free_list.clear();
    }
}

/// A sharded LRU cache for improved concurrent performance
/// 
/// The cache is divided into multiple shards, each with its own lock.
/// Keys are distributed across shards using a hash function. This allows
/// concurrent access to different shards without contention.
/// 
/// # Trade-offs
/// 
/// - **Better concurrency**: Multiple threads can access different shards simultaneously
/// - **Approximate LRU**: Each shard maintains its own LRU order, not global LRU
/// - **Memory overhead**: More complex structure with multiple shard instances
/// 
/// # Examples
/// 
/// ```
/// use lru_cache::ShardedLruCache;
/// 
/// // Create cache with 1000 total capacity across 16 shards
/// let cache = ShardedLruCache::new(1000, 16);
/// 
/// cache.put(1, "one");
/// cache.put(2, "two");
/// 
/// assert_eq!(cache.get(&1), Some("one"));
/// ```
#[derive(Clone)]
pub struct ShardedLruCache<K, V> {
    shards: Arc<Vec<Mutex<LruCacheInner<K, V>>>>,
    shard_count: usize,
}

impl<K: Clone + Eq + Hash, V: Clone> ShardedLruCache<K, V> {
    /// Creates a new sharded LRU cache
    /// 
    /// # Arguments
    /// 
    /// * `total_capacity` - Total capacity across all shards
    /// * `shard_count` - Number of shards to create (typically power of 2)
    /// 
    /// # Panics
    /// 
    /// Panics if total_capacity or shard_count is 0
    pub fn new(total_capacity: usize, shard_count: usize) -> Self {
        assert!(total_capacity > 0, "Total capacity must be greater than 0");
        assert!(shard_count > 0, "Shard count must be greater than 0");
        
        let capacity_per_shard = total_capacity.div_ceil(shard_count);
        
        let shards: Vec<Mutex<LruCacheInner<K, V>>> = (0..shard_count)
            .map(|_| Mutex::new(LruCacheInner::new(capacity_per_shard)))
            .collect();
        
        ShardedLruCache {
            shards: Arc::new(shards),
            shard_count,
        }
    }
    
    /// Get the shard index for a given key
    fn shard_index(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % self.shard_count
    }
    
    /// Gets a value from the cache
    /// 
    /// If the key exists in its shard, it is marked as recently used and its value is returned.
    pub fn get(&self, key: &K) -> Option<V> {
        let shard_idx = self.shard_index(key);
        let mut shard = self.shards[shard_idx].lock().unwrap();
        shard.get(key)
    }
    
    /// Inserts a key-value pair into the cache
    /// 
    /// The key is hashed to determine which shard it belongs to.
    /// If the shard is at capacity, the least recently used item in that shard is evicted.
    pub fn put(&self, key: K, value: V) {
        let shard_idx = self.shard_index(&key);
        let mut shard = self.shards[shard_idx].lock().unwrap();
        shard.put(key, value);
    }
    
    /// Returns the approximate number of items in the cache
    /// 
    /// Note: This requires locking all shards, so it's relatively expensive
    pub fn len(&self) -> usize {
        self.shards.iter()
            .map(|shard| shard.lock().unwrap().len())
            .sum()
    }
    
    /// Returns true if the cache contains no items
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Returns the number of shards
    pub fn shard_count(&self) -> usize {
        self.shard_count
    }
    
    /// Removes all items from all shards
    pub fn clear(&self) {
        for shard in self.shards.iter() {
            shard.lock().unwrap().clear();
        }
    }
}

// Implement Send and Sync for thread safety
unsafe impl<K: Send, V: Send> Send for ShardedLruCache<K, V> {}
unsafe impl<K: Send, V: Send> Sync for ShardedLruCache<K, V> {}

/// Statistics for sharded cache performance analysis
#[derive(Debug, Clone)]
pub struct ShardStats {
    pub shard_index: usize,
    pub size: usize,
    pub capacity: usize,
    pub utilization: f64,
}

impl<K: Clone + Eq + Hash, V: Clone> ShardedLruCache<K, V> {
    /// Get statistics for all shards
    /// 
    /// Useful for analyzing shard distribution and load balancing
    pub fn shard_stats(&self) -> Vec<ShardStats> {
        self.shards.iter()
            .enumerate()
            .map(|(idx, shard)| {
                let inner = shard.lock().unwrap();
                let size = inner.len();
                let capacity = inner.capacity;
                ShardStats {
                    shard_index: idx,
                    size,
                    capacity,
                    utilization: size as f64 / capacity as f64,
                }
            })
            .collect()
    }
}


