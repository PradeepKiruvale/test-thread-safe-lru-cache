/// Async-compatible LRU cache implementation
/// 
/// This module provides an async version of the LRU cache that works with
/// async runtimes like Tokio. It uses tokio::sync::Mutex instead of std::sync::Mutex
/// to avoid blocking async tasks.

use std::collections::HashMap;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,
    next: Option<usize>,
}

struct AsyncLruCacheInner<K, V> {
    capacity: usize,
    map: HashMap<K, usize>,
    nodes: Vec<Option<Node<K, V>>>,
    head: Option<usize>,
    tail: Option<usize>,
    free_list: Vec<usize>,
}

impl<K: Clone + Eq + Hash, V: Clone> AsyncLruCacheInner<K, V> {
    fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "Cache capacity must be greater than 0");
        AsyncLruCacheInner {
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

    fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn clear(&mut self) {
        self.map.clear();
        self.nodes.clear();
        self.head = None;
        self.tail = None;
        self.free_list.clear();
    }
}

/// An async-compatible thread-safe LRU cache
/// 
/// This cache uses tokio::sync::Mutex which allows async operations
/// without blocking the executor thread pool.
/// 
/// # Examples
/// 
/// ```no_run
/// use lru_cache::AsyncLruCache;
/// 
/// #[tokio::main]
/// async fn main() {
///     let cache = AsyncLruCache::new(100);
///     
///     cache.put(1, "one").await;
///     cache.put(2, "two").await;
///     
///     let value = cache.get(&1).await;
///     assert_eq!(value, Some("one"));
/// }
/// ```
#[derive(Clone)]
pub struct AsyncLruCache<K, V> {
    inner: Arc<Mutex<AsyncLruCacheInner<K, V>>>,
}

impl<K: Clone + Eq + Hash, V: Clone> AsyncLruCache<K, V> {
    /// Creates a new async LRU cache with the specified capacity
    pub fn new(capacity: usize) -> Self {
        AsyncLruCache {
            inner: Arc::new(Mutex::new(AsyncLruCacheInner::new(capacity))),
        }
    }

    /// Gets a value from the cache asynchronously
    /// 
    /// Returns None if the key is not in the cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut inner = self.inner.lock().await;
        inner.get(key)
    }

    /// Inserts a key-value pair into the cache asynchronously
    pub async fn put(&self, key: K, value: V) {
        let mut inner = self.inner.lock().await;
        inner.put(key, value);
    }

    /// Returns the number of items currently in the cache
    pub async fn len(&self) -> usize {
        let inner = self.inner.lock().await;
        inner.len()
    }

    /// Returns true if the cache contains no items
    pub async fn is_empty(&self) -> bool {
        let inner = self.inner.lock().await;
        inner.is_empty()
    }

    /// Returns the maximum capacity of the cache
    pub async fn capacity(&self) -> usize {
        let inner = self.inner.lock().await;
        inner.capacity()
    }

    /// Removes all items from the cache
    pub async fn clear(&self) {
        let mut inner = self.inner.lock().await;
        inner.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_basic_operations() {
        let cache = AsyncLruCache::new(2);
        
        cache.put(1, "one").await;
        cache.put(2, "two").await;
        
        assert_eq!(cache.get(&1).await, Some("one"));
        assert_eq!(cache.get(&2).await, Some("two"));
        assert_eq!(cache.len().await, 2);
    }

    #[tokio::test]
    async fn test_async_eviction() {
        let cache = AsyncLruCache::new(2);
        
        cache.put(1, "one").await;
        cache.put(2, "two").await;
        cache.put(3, "three").await;
        
        assert_eq!(cache.get(&1).await, None);
        assert_eq!(cache.get(&2).await, Some("two"));
        assert_eq!(cache.get(&3).await, Some("three"));
    }

    #[tokio::test]
    async fn test_async_concurrent() {
        let cache = AsyncLruCache::new(100);
        let mut handles = vec![];

        for i in 0..10 {
            let cache_clone = cache.clone();
            let handle = tokio::spawn(async move {
                for j in 0..100 {
                    cache_clone.put(i * 100 + j, format!("value-{}-{}", i, j)).await;
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert!(cache.len().await <= 100);
    }
}
