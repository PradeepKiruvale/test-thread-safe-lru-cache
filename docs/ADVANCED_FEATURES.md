# Advanced Features Guide

This guide covers the advanced features of the LRU cache library, including async support, configurable eviction policies, sharded implementations, and performance optimizations.

## Table of Contents

1. [Async-Compatible Cache](#async-compatible-cache)
2. [Configurable Eviction Policies](#configurable-eviction-policies)
3. [Sharded Cache](#sharded-cache)
4. [Performance Comparison](#performance-comparison)
5. [Choosing the Right Implementation](#choosing-the-right-implementation)

## Async-Compatible Cache

### Overview

The async-compatible cache uses `tokio::sync::Mutex` instead of `std::sync::Mutex`, making it suitable for async runtimes without blocking the executor thread pool.

### When to Use

- **Async/await codebases**: When your application uses Tokio or other async runtimes
- **I/O-bound operations**: When cache operations are interspersed with async I/O
- **Web servers**: For caching in async web frameworks (Axum, Actix-web, etc.)

### Basic Usage

```rust
use lru_cache::AsyncLruCache;

#[tokio::main]
async fn main() {
    let cache = AsyncLruCache::new(100);
    
    cache.put(1, "one").await;
    cache.put(2, "two").await;
    
    let value = cache.get(&1).await;
    assert_eq!(value, Some("one"));
}
```

### Concurrent Async Operations

```rust
use lru_cache::AsyncLruCache;
use tokio::task;

#[tokio::main]
async fn main() {
    let cache = AsyncLruCache::new(1000);
    
    // Spawn multiple concurrent tasks
    let mut handles = vec![];
    
    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = task::spawn(async move {
            for j in 0..100 {
                cache_clone.put(i * 100 + j, format!("value-{}", j)).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }
}
```

### Trade-offs

**Advantages:**
- Non-blocking: Doesn't block async executor threads
- Integrates with async ecosystem
- Better for I/O-bound workloads

**Disadvantages:**
- Slightly higher overhead than sync mutex
- Requires tokio runtime
- Still serializes operations (no concurrent reads)

## Configurable Eviction Policies

### Overview

The eviction policy module provides different strategies for selecting which item to evict when the cache reaches capacity.

### Available Policies

#### 1. LRU (Least Recently Used)

Evicts the item that hasn't been accessed for the longest time.

```rust
use lru_cache::eviction::{ThreadSafeEvictableCache, LruPolicy};

let cache = ThreadSafeEvictableCache::new(100, LruPolicy::new());
```

**Best for:**
- Temporal locality patterns
- Recently accessed items are likely to be accessed again
- Most general-purpose use cases

#### 2. LFU (Least Frequently Used)

Evicts the item with the lowest access count.

```rust
use lru_cache::eviction::{ThreadSafeEvictableCache, LfuPolicy};

let cache = ThreadSafeEvictableCache::new(100, LfuPolicy::new());
```

**Best for:**
- Frequency-based patterns
- Popular items should stay cached
- Workloads with clear "hot" and "cold" data

#### 3. FIFO (First In First Out)

Evicts the oldest item regardless of access patterns.

```rust
use lru_cache::eviction::{ThreadSafeEvictableCache, FifoPolicy};

let cache = ThreadSafeEvictableCache::new(100, FifoPolicy::new());
```

**Best for:**
- Simple, predictable behavior
- When access patterns don't matter
- Time-based data (e.g., event logs)

#### 4. Random

Evicts a random item.

```rust
use lru_cache::eviction::{ThreadSafeEvictableCache, RandomPolicy};

let cache = ThreadSafeEvictableCache::new(100, RandomPolicy::new());
```

**Best for:**
- When eviction strategy doesn't significantly impact performance
- Lowest overhead implementation
- Avoiding worst-case scenarios of other policies

### Comparison Example

```rust
use lru_cache::eviction::*;

// Scenario: Caching web pages
let lru = ThreadSafeEvictableCache::new(3, LruPolicy::new());
let lfu = ThreadSafeEvictableCache::new(3, LfuPolicy::new());
let fifo = ThreadSafeEvictableCache::new(3, FifoPolicy::new());

// Add pages
for cache in [&lru, &lfu, &fifo] {
    cache.put("home", "Home Page");
    cache.put("product", "Product Page");
    cache.put("about", "About Page");
}

// Simulate access pattern
// Homepage: accessed 10 times
for _ in 0..10 {
    lru.get(&"home");
    lfu.get(&"home");
    fifo.get(&"home");
}

// Add new page (triggers eviction)
lru.put("contact", "Contact Page");
lfu.put("contact", "Contact Page");
fifo.put("contact", "Contact Page");

// Results:
// LRU:  evicts "about" (least recently used)
// LFU:  evicts "about" (least frequently used)
// FIFO: evicts "home" (first in, despite high usage!)
```

### Custom Eviction Policy

You can implement your own eviction policy:

```rust
use lru_cache::eviction::EvictionPolicy;

struct CustomPolicy {
    // Your state here
}

impl<K: Clone + Eq> EvictionPolicy<K> for CustomPolicy {
    fn access(&mut self, key: &K) {
        // Track access
    }
    
    fn insert(&mut self, key: K) {
        // Track insertion
    }
    
    fn remove(&mut self, key: &K) {
        // Remove from tracking
    }
    
    fn evict(&mut self) -> Option<K> {
        // Select item to evict
        None
    }
    
    fn clear(&mut self) {
        // Clear all tracking
    }
}
```

## Sharded Cache

### Overview

The sharded cache splits the keyspace across multiple independent cache instances (shards), reducing lock contention by allowing concurrent access to different shards.

### Architecture

```
ShardedLruCache
├── Shard 0 (Mutex<LruCacheInner>)  ← Keys with hash % N == 0
├── Shard 1 (Mutex<LruCacheInner>)  ← Keys with hash % N == 1
├── Shard 2 (Mutex<LruCacheInner>)  ← Keys with hash % N == 2
└── Shard N (Mutex<LruCacheInner>)  ← Keys with hash % N == N
```

### Basic Usage

```rust
use lru_cache::ShardedLruCache;

// Create cache with 1000 total capacity across 16 shards
let cache = ShardedLruCache::new(1000, 16);

cache.put(1, "one");
cache.put(2, "two");

assert_eq!(cache.get(&1), Some("one"));
```

### Shard Count Selection

```rust
use std::thread;

// Rule of thumb: shard_count ≈ number of CPU cores
let num_cores = thread::available_parallelism()
    .map(|n| n.get())
    .unwrap_or(8);

let cache = ShardedLruCache::new(10_000, num_cores);
```

### Monitoring Shard Distribution

```rust
use lru_cache::ShardedLruCache;

let cache = ShardedLruCache::new(1000, 8);

// Add items...
for i in 0..500 {
    cache.put(i, format!("value-{}", i));
}

// Check shard statistics
let stats = cache.shard_stats();
for stat in stats {
    println!("Shard {}: {}/{} items ({:.1}% full)",
        stat.shard_index,
        stat.size,
        stat.capacity,
        stat.utilization * 100.0
    );
}
```

### Performance Characteristics

| Shard Count | Concurrency | LRU Accuracy | Memory Overhead |
|-------------|-------------|--------------|-----------------|
| 1           | Low         | Perfect      | Minimal         |
| 4           | Good        | Good         | Low             |
| 8           | Better      | Good         | Medium          |
| 16          | Best        | Fair         | Higher          |
| 32+         | Best        | Fair         | Significant     |

### Trade-offs

**Advantages:**
- **Higher throughput**: Multiple threads can access different shards simultaneously
- **Reduced contention**: Lock contention scales with O(1/N) where N is shard count
- **Better CPU utilization**: Threads spend less time waiting for locks

**Disadvantages:**
- **Approximate LRU**: Each shard maintains its own LRU order, not global
- **Memory overhead**: More complex structure with multiple instances
- **Uneven distribution**: Hash-based sharding may not distribute evenly
- **More complex**: Harder to reason about than single-lock design

### When to Use Sharded Cache

**Use sharded cache when:**
- High thread count (>4 threads)
- High contention measured via profiling
- Approximate LRU is acceptable
- Throughput is critical

**Stick with standard cache when:**
- Low thread count (≤4 threads)
- Global LRU semantics required
- Simplicity preferred
- Memory is constrained

## Performance Comparison

### Running Benchmarks

```bash
# Run all examples
cargo run --example performance_comparison --release

# Run specific example
cargo run --example sharded_cache_example --release
cargo run --example async_cache_example --features async
cargo run --example eviction_policies_example
```

### Expected Results

On a typical 8-core system with 100,000 operations across 8 threads:

| Implementation           | Throughput (ops/sec) | Speedup | Hit Rate |
|-------------------------|---------------------|---------|----------|
| Standard LRU            | ~1,200,000          | 1.0x    | 45%      |
| Sharded LRU (4 shards)  | ~2,400,000          | 2.0x    | 44%      |
| Sharded LRU (8 shards)  | ~3,600,000          | 3.0x    | 44%      |
| Sharded LRU (16 shards) | ~4,200,000          | 3.5x    | 43%      |
| LFU Policy              | ~1,100,000          | 0.9x    | 48%      |
| FIFO Policy             | ~1,300,000          | 1.1x    | 38%      |

*Note: Results vary based on hardware, workload, and access patterns.*

### Profiling Your Application

```rust
use std::time::Instant;

let cache = lru_cache::ShardedLruCache::new(10_000, 8);

let start = Instant::now();

// Your workload here
for i in 0..1_000_000 {
    if i % 2 == 0 {
        cache.put(i, format!("value-{}", i));
    } else {
        cache.get(&(i - 1));
    }
}

let duration = start.elapsed();
println!("Throughput: {:.0} ops/sec", 
    1_000_000.0 / duration.as_secs_f64());
```

## Choosing the Right Implementation

### Decision Tree

```
Are you using async/await?
├─ Yes → AsyncLruCache
└─ No → Continue

Do you need >1M ops/sec with >4 threads?
├─ Yes → Consider ShardedLruCache
└─ No → Continue

Do you need custom eviction logic?
├─ Yes → ThreadSafeEvictableCache with custom policy
└─ No → Standard LruCache

Is LRU not optimal for your access pattern?
├─ Yes (frequency matters) → LfuPolicy
├─ Yes (age matters) → FifoPolicy
└─ No → LruPolicy
```

### Feature Matrix

| Feature | LruCache | AsyncLruCache | ShardedLruCache | EvictableCache |
|---------|----------|---------------|-----------------|----------------|
| Thread-safe | ✓ | ✓ | ✓ | ✓ |
| Async-compatible | ✗ | ✓ | ✗ | ✗ |
| Global LRU | ✓ | ✓ | ✗ | Configurable |
| High concurrency | ✗ | ✗ | ✓ | ✗ |
| Custom policies | ✗ | ✗ | ✗ | ✓ |
| O(1) operations | ✓ | ✓ | ✓ | ✓ |

### Real-World Use Cases

**Web API Cache (Async)**
```rust
use lru_cache::AsyncLruCache;
use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let cache = AsyncLruCache::new(10_000);
    // Use in async handlers
}
```

**High-Throughput Service (Sharded)**
```rust
use lru_cache::ShardedLruCache;

fn main() {
    let cache = ShardedLruCache::new(100_000, 16);
    // Use in multi-threaded service
}
```

**CDN with Popularity Tracking (LFU)**
```rust
use lru_cache::eviction::{ThreadSafeEvictableCache, LfuPolicy};

fn main() {
    let cache = ThreadSafeEvictableCache::new(50_000, LfuPolicy::new());
    // Popular content stays cached
}
```

**Event Stream Buffer (FIFO)**
```rust
use lru_cache::eviction::{ThreadSafeEvictableCache, FifoPolicy};

fn main() {
    let cache = ThreadSafeEvictableCache::new(1_000, FifoPolicy::new());
    // Recent events only
}
```

## Best Practices

### 1. Choose Capacity Wisely

```rust
// Too small: High miss rate
let cache = LruCache::new(10);

// Too large: Memory waste
let cache = LruCache::new(1_000_000);

// Just right: Based on profiling
let cache = LruCache::new(10_000);
```

### 2. Monitor Hit Rate

```rust
use std::sync::atomic::{AtomicUsize, Ordering};

let hits = AtomicUsize::new(0);
let misses = AtomicUsize::new(0);

if cache.get(&key).is_some() {
    hits.fetch_add(1, Ordering::Relaxed);
} else {
    misses.fetch_add(1, Ordering::Relaxed);
}

// Log periodically
let hit_rate = hits.load(Ordering::Relaxed) as f64 
    / (hits.load(Ordering::Relaxed) + misses.load(Ordering::Relaxed)) as f64;
println!("Hit rate: {:.1}%", hit_rate * 100.0);
```

### 3. Profile Before Optimizing

```bash
# Use cargo-flamegraph to identify bottlenecks
cargo install flamegraph
cargo flamegraph --example your_example
```

### 4. Test Under Load

```rust
use std::thread;

let cache = ShardedLruCache::new(10_000, 8);

// Stress test
let handles: Vec<_> = (0..16)
    .map(|i| {
        let cache = cache.clone();
        thread::spawn(move || {
            for j in 0..100_000 {
                cache.put(i * 100_000 + j, j);
            }
        })
    })
    .collect();

for handle in handles {
    handle.join().unwrap();
}
```

## Summary

- **AsyncLruCache**: For async/await applications
- **ShardedLruCache**: For high-concurrency scenarios (>4 threads, >1M ops/sec)
- **EvictableCache**: For custom eviction policies (LRU, LFU, FIFO, Random)
- **Standard LruCache**: For most general-purpose use cases

Choose based on your specific requirements, and always profile to validate assumptions!
