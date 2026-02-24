// Library tests module

use crate::*;
use crate::eviction::*;
use crate::sharded::*;

#[cfg(feature = "async")]
use crate::async_cache::*;

use std::thread;

// ============================================================================
// Basic LRU Cache Tests
// ============================================================================

#[test]
fn test_basic_operations() {
    let cache = LruCache::new(2);
    
    cache.put(1, "one");
    cache.put(2, "two");
    
    assert_eq!(cache.get(&1), Some("one"));
    assert_eq!(cache.get(&2), Some("two"));
    assert_eq!(cache.len(), 2);
}

#[test]
fn test_lru_eviction() {
    let cache = LruCache::new(2);
    
    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three"); // Should evict 1
    
    assert_eq!(cache.get(&1), None);
    assert_eq!(cache.get(&2), Some("two"));
    assert_eq!(cache.get(&3), Some("three"));
}

#[test]
fn test_update_existing() {
    let cache = LruCache::new(2);
    
    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(1, "ONE"); // Update existing
    cache.put(3, "three"); // Should evict 2, not 1
    
    assert_eq!(cache.get(&1), Some("ONE"));
    assert_eq!(cache.get(&2), None);
    assert_eq!(cache.get(&3), Some("three"));
}

#[test]
fn test_get_updates_recency() {
    let cache = LruCache::new(2);
    
    cache.put(1, "one");
    cache.put(2, "two");
    cache.get(&1); // Mark 1 as recently used
    cache.put(3, "three"); // Should evict 2
    
    assert_eq!(cache.get(&1), Some("one"));
    assert_eq!(cache.get(&2), None);
    assert_eq!(cache.get(&3), Some("three"));
}

#[test]
fn test_clear() {
    let cache = LruCache::new(3);
    
    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three");
    
    assert_eq!(cache.len(), 3);
    cache.clear();
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.get(&1), None);
}

#[test]
fn test_concurrent_access() {
    let cache = LruCache::new(100);
    let mut handles = vec![];

    // Spawn multiple writer threads
    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = thread::spawn(move || {
            for j in 0..100 {
                cache_clone.put(i * 100 + j, format!("value-{i}-{j}"));
            }
        });
        handles.push(handle);
    }

    // Spawn multiple reader threads
    for _ in 0..10 {
        let cache_clone = cache.clone();
        let handle = thread::spawn(move || {
            for i in 0..1000 {
                let _ = cache_clone.get(&i);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Cache should not exceed capacity
    assert!(cache.len() <= 100);
}

#[test]
fn test_capacity_boundary() {
    let cache = LruCache::new(1);
    
    cache.put(1, "one");
    assert_eq!(cache.len(), 1);
    
    cache.put(2, "two");
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get(&1), None);
    assert_eq!(cache.get(&2), Some("two"));
}

#[test]
fn test_reinsert_evicted() {
    let cache = LruCache::new(2);
    
    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three"); // Evicts 1
    
    assert_eq!(cache.get(&1), None);
    
    cache.put(1, "NEW ONE"); // Reinsert 1
    
    assert_eq!(cache.get(&1), Some("NEW ONE"));
    assert_eq!(cache.len(), 2);
}

#[test]
#[should_panic(expected = "Cache capacity must be greater than 0")]
fn test_zero_capacity_panics() {
    let _cache: LruCache<i32, i32> = LruCache::new(0);
}

// ============================================================================
// Eviction Policy Tests
// ============================================================================

#[test]
fn test_lru_policy() {
    let cache = ThreadSafeEvictableCache::new(2, LruPolicy::new());
    
    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three"); // Evicts 1
    
    assert_eq!(cache.get(&1), None);
    assert_eq!(cache.get(&2), Some("two"));
    assert_eq!(cache.get(&3), Some("three"));
}

#[test]
fn test_lfu_policy() {
    let cache = ThreadSafeEvictableCache::new(2, LfuPolicy::new());
    
    cache.put(1, "one");
    cache.put(2, "two");
    cache.get(&1); // Access 1 twice
    cache.get(&1);
    cache.put(3, "three"); // Should evict 2 (less frequently used)
    
    assert_eq!(cache.get(&1), Some("one"));
    assert_eq!(cache.get(&2), None);
    assert_eq!(cache.get(&3), Some("three"));
}

#[test]
fn test_fifo_policy() {
    let cache = ThreadSafeEvictableCache::new(2, FifoPolicy::new());
    
    cache.put(1, "one");
    cache.put(2, "two");
    cache.get(&1); // Access doesn't matter for FIFO
    cache.put(3, "three"); // Evicts 1 (first in)
    
    assert_eq!(cache.get(&1), None);
    assert_eq!(cache.get(&2), Some("two"));
    assert_eq!(cache.get(&3), Some("three"));
}

// ============================================================================
// Sharded Cache Tests
// ============================================================================

#[test]
fn test_sharded_basic_operations() {
    let cache = ShardedLruCache::new(100, 4);
    
    cache.put(1, "one");
    cache.put(2, "two");
    
    assert_eq!(cache.get(&1), Some("one"));
    assert_eq!(cache.get(&2), Some("two"));
}

#[test]
fn test_sharded_eviction() {
    let cache = ShardedLruCache::new(4, 2); // 2 per shard
    
    // Fill cache
    cache.put(1, "one");
    cache.put(2, "two");
    cache.put(3, "three");
    cache.put(4, "four");
    
    // These should cause evictions
    cache.put(5, "five");
    cache.put(6, "six");
    
    // Total should not exceed capacity
    assert!(cache.len() <= 4);
}

#[test]
fn test_sharded_concurrent_access() {
    let cache = ShardedLruCache::new(1000, 16);
    let mut handles = vec![];

    // Spawn multiple writer threads
    for i in 0..20 {
        let cache_clone = cache.clone();
        let handle = thread::spawn(move || {
            for j in 0..100 {
                cache_clone.put(i * 100 + j, format!("value-{i}-{j}"));
            }
        });
        handles.push(handle);
    }

    // Spawn multiple reader threads
    for _ in 0..20 {
        let cache_clone = cache.clone();
        let handle = thread::spawn(move || {
            for i in 0..2000 {
                let _ = cache_clone.get(&i);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Cache should not significantly exceed capacity
    assert!(cache.len() <= 1200); // Allow some overhead
}

#[test]
fn test_shard_stats() {
    let cache = ShardedLruCache::new(100, 4);
    
    for i in 0..50 {
        cache.put(i, format!("value-{i}"));
    }
    
    let stats = cache.shard_stats();
    assert_eq!(stats.len(), 4);
    
    let total_size: usize = stats.iter().map(|s| s.size).sum();
    assert!(total_size <= 100);
    
    for stat in stats {
        println!("Shard {}: {}/{} items ({:.1}% full)",
            stat.shard_index, stat.size, stat.capacity,
            stat.utilization * 100.0);
    }
}

#[test]
fn test_sharded_clear() {
    let cache = ShardedLruCache::new(100, 4);
    
    for i in 0..50 {
        cache.put(i, format!("value-{i}"));
    }
    
    assert!(!cache.is_empty());
    cache.clear();
    assert!(cache.is_empty());
}

// ============================================================================
// Async Cache Tests
// ============================================================================

#[cfg(feature = "async")]
mod async_tests {
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
