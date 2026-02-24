// Core synchronous LRU cache implementation
use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex, PoisonError};
use std::fmt;

// Public modules
pub mod eviction;
pub mod sharded;

/// Error types for cache operations
#[derive(Debug, Clone)]
pub enum CacheError {
    /// The mutex protecting the cache was poisoned
    LockPoisoned,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::LockPoisoned => write!(f, "Cache lock was poisoned"),
        }
    }
}

impl std::error::Error for CacheError {}

impl<T> From<PoisonError<T>> for CacheError {
    fn from(_: PoisonError<T>) -> Self {
        CacheError::LockPoisoned
    }
}

// Optional async support (requires tokio feature)
#[cfg(feature = "async")]
pub mod async_cache;

#[cfg(feature = "async")]
pub use async_cache::AsyncLruCache;

pub use sharded::ShardedLruCache;

/// A node in the doubly-linked list that tracks access order
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

/// Internal LRU cache structure protected by a mutex
struct LruCacheInner<K, V> {
    capacity: usize,
    /// Maps keys to indices in the nodes vector
    map: HashMap<K, usize>,
    /// Storage for all nodes (Some for used slots, None for free slots)
    nodes: Vec<Option<Node<K, V>>>,
    /// Index of the most recently used node (head of list)
    head: Option<usize>,
    /// Index of the least recently used node (tail of list)
    tail: Option<usize>,
    /// Stack of free indices for reuse
    free_list: Vec<usize>,
}

impl<K: Clone + Eq + Hash, V: Clone> LruCacheInner<K, V> {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Cache capacity must be greater than 0");
        
        LruCacheInner {
            capacity,
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            head: None,
            tail: None,
            free_list: Vec::with_capacity(capacity),
        }
    }

    /// Move a node to the front (most recently used position)
    fn move_to_front(&mut self, idx: usize) {
        if self.head == Some(idx) {
            // Already at front
            return;
        }

        // Extract the prev and next indices first
        let (prev_idx, next_idx) = if let Some(ref node) = self.nodes[idx] {
            (node.prev, node.next)
        } else {
            return;
        };

        // Update the previous node's next pointer
        if let Some(prev_idx) = prev_idx {
            if let Some(ref mut prev_node) = self.nodes[prev_idx] {
                prev_node.next = next_idx;
            }
        }
        
        // Update the next node's prev pointer
        if let Some(next_idx) = next_idx {
            if let Some(ref mut next_node) = self.nodes[next_idx] {
                next_node.prev = prev_idx;
            }
        } else {
            // This was the tail
            self.tail = prev_idx;
        }

        // Update current node to be at front
        if let Some(ref mut node) = self.nodes[idx] {
            node.prev = None;
            node.next = self.head;
        }

        // Update old head's prev pointer
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

    /// Get a value from the cache, updating its recency
    fn get(&mut self, key: &K) -> Option<V> {
        if let Some(&idx) = self.map.get(key) {
            self.move_to_front(idx);
            self.nodes[idx].as_ref().map(|node| node.value.clone())
        } else {
            None
        }
    }

    /// Insert or update a key-value pair
    fn put(&mut self, key: K, value: V) {
        // Check if key already exists
        if let Some(&idx) = self.map.get(&key) {
            // Update existing entry
            if let Some(ref mut node) = self.nodes[idx] {
                node.value = value;
            }
            self.move_to_front(idx);
            return;
        }

        // Need to insert new entry
        // Check if we need to evict
        if self.map.len() >= self.capacity {
            self.evict_lru();
        }

        // Get an index for the new node
        let idx = if let Some(free_idx) = self.free_list.pop() {
            free_idx
        } else {
            let idx = self.nodes.len();
            self.nodes.push(None);
            idx
        };

        // Create and insert new node
        let new_node = Node {
            key: key.clone(),
            value,
            prev: None,
            next: self.head,
        };

        // Update old head
        if let Some(old_head_idx) = self.head {
            if let Some(ref mut old_head) = self.nodes[old_head_idx] {
                old_head.prev = Some(idx);
            }
        }

        self.nodes[idx] = Some(new_node);
        self.map.insert(key, idx);
        self.head = Some(idx);

        if self.tail.is_none() {
            self.tail = Some(idx);
        }
    }

    /// Evict the least recently used item
    fn evict_lru(&mut self) {
        if let Some(tail_idx) = self.tail {
            if let Some(tail_node) = self.nodes[tail_idx].take() {
                // Remove from map
                self.map.remove(&tail_node.key);
                
                // Update tail pointer
                self.tail = tail_node.prev;
                
                if let Some(new_tail_idx) = tail_node.prev {
                    if let Some(ref mut new_tail) = self.nodes[new_tail_idx] {
                        new_tail.next = None;
                    }
                } else {
                    // List is now empty
                    self.head = None;
                }

                // Add to free list
                self.free_list.push(tail_idx);
            }
        }
    }

    /// Get the current number of items in the cache
    fn len(&self) -> usize {
        self.map.len()
    }

    /// Check if the cache is empty
    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Get the capacity of the cache
    fn capacity(&self) -> usize {
        self.capacity
    }

    /// Clear all items from the cache
    fn clear(&mut self) {
        self.map.clear();
        self.nodes.clear();
        self.head = None;
        self.tail = None;
        self.free_list.clear();
    }
}

/// A thread-safe LRU (Least Recently Used) cache
/// 
/// This cache automatically evicts the least recently used items when capacity is reached.
/// All operations are O(1) on average and are safe for concurrent access from multiple threads.
/// 
/// # Examples
/// 
/// ```
/// use lru_cache::LruCache;
/// use std::thread;
/// 
/// let cache = LruCache::new(2);
/// 
/// cache.put(1, "one").unwrap();
/// cache.put(2, "two").unwrap();
/// 
/// assert_eq!(cache.get(&1).unwrap(), Some("one"));
/// 
/// // This will evict key 2 (least recently used)
/// cache.put(3, "three").unwrap();
/// assert_eq!(cache.get(&2).unwrap(), None);
/// ```
#[derive(Clone)]
pub struct LruCache<K, V> {
    inner: Arc<Mutex<LruCacheInner<K, V>>>,
}

impl<K: Clone + Eq + Hash, V: Clone> LruCache<K, V> {
    /// Creates a new LRU cache with the specified capacity
    /// 
    /// # Panics
    /// 
    /// Panics if capacity is 0
    pub fn new(capacity: usize) -> Self {
        LruCache {
            inner: Arc::new(Mutex::new(LruCacheInner::new(capacity))),
        }
    }

    /// Gets a value from the cache
    /// 
    /// If the key exists, it is marked as recently used and its value is returned.
    /// Returns `None` if the key is not in the cache.
    /// 
    /// # Time Complexity
    /// 
    /// O(1) average case
    /// 
    /// # Errors
    /// 
    /// Returns `CacheError::LockPoisoned` if the mutex is poisoned
    pub fn get(&self, key: &K) -> Result<Option<V>, CacheError> {
        let mut inner = self.inner.lock()?;
        Ok(inner.get(key))
    }

    /// Inserts a key-value pair into the cache
    /// 
    /// If the key already exists, its value is updated and it's marked as recently used.
    /// If the cache is at capacity, the least recently used item is evicted.
    /// 
    /// # Time Complexity
    /// 
    /// O(1) average case
    /// 
    /// # Errors
    /// 
    /// Returns `CacheError::LockPoisoned` if the mutex is poisoned
    pub fn put(&self, key: K, value: V) -> Result<(), CacheError> {
        let mut inner = self.inner.lock()?;
        inner.put(key, value);
        Ok(())
    }

    /// Returns the number of items currently in the cache
    /// 
    /// # Errors
    /// 
    /// Returns `CacheError::LockPoisoned` if the mutex is poisoned
    pub fn len(&self) -> Result<usize, CacheError> {
        let inner = self.inner.lock()?;
        Ok(inner.len())
    }

    /// Returns true if the cache contains no items
    /// 
    /// # Errors
    /// 
    /// Returns `CacheError::LockPoisoned` if the mutex is poisoned
    pub fn is_empty(&self) -> Result<bool, CacheError> {
        let inner = self.inner.lock()?;
        Ok(inner.is_empty())
    }

    /// Returns the maximum capacity of the cache
    /// 
    /// # Errors
    /// 
    /// Returns `CacheError::LockPoisoned` if the mutex is poisoned
    pub fn capacity(&self) -> Result<usize, CacheError> {
        let inner = self.inner.lock()?;
        Ok(inner.capacity())
    }

    /// Removes all items from the cache
    /// 
    /// # Errors
    /// 
    /// Returns `CacheError::LockPoisoned` if the mutex is poisoned
    pub fn clear(&self) -> Result<(), CacheError> {
        let mut inner = self.inner.lock()?;
        inner.clear();
        Ok(())
    }
}

// Implement Send and Sync explicitly to ensure thread safety
unsafe impl<K: Send, V: Send> Send for LruCache<K, V> {}
unsafe impl<K: Send, V: Send> Sync for LruCache<K, V> {}

#[cfg(test)]
mod tests;

