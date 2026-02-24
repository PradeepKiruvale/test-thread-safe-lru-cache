# Thread-Safe LRU Cache

A high-performance, thread-safe Least Recently Used (LRU) cache implementation in Rust.

## Features

- **Thread-Safe**: Safe for concurrent access from multiple threads using `Arc<Mutex<_>>`
- **O(1) Operations**: Average O(1) time complexity for both `get` and `put` operations
- **Automatic Eviction**: Automatically evicts the least recently used item when capacity is reached
- **Memory Bounded**: Guarantees memory usage stays within configured capacity
- **Safe Rust**: No unsafe code, no data races, no undefined behavior
- **Clone Support**: Implements `Clone` for easy sharing across threads

## Design Decisions

### Concurrency Strategy

- Uses a single `Mutex` to protect the entire cache structure
- Ensures correctness and prevents deadlocks (single lock = no lock ordering issues)
- All operations acquire a single lock, perform their work, and release
- Trade-off: Simplicity and correctness over maximum concurrency
- Alternative considered: Fine-grained locking adds complexity with minimal benefit for this use case

### Data Structure

- **HashMap**: Provides O(1) key lookups
- **Doubly-Linked List**: Tracks access order (most recent at head, least recent at tail)
- **Index-Based Links**: Uses `Vec<Option<Node>>` with indices instead of raw pointers for safety
- **Free List**: Recycles freed indices to avoid unbounded vector growth

### Performance Characteristics

- **get()**: O(1) average - HashMap lookup + list reordering
- **put()**: O(1) average - HashMap insert + list operations
- **Memory**: O(capacity) - bounded by configured capacity
- **Contention**: All operations hold lock briefly (microseconds)

## Usage

### Basic Example

```rust
use lru_cache::LruCache;

fn main() {
    let cache = LruCache::new(2);
    
    cache.put(1, "one");
    cache.put(2, "two");
    
    assert_eq!(cache.get(&1), Some("one"));
    
    // This will evict key 2 (least recently used)
    cache.put(3, "three");
    assert_eq!(cache.get(&2), None);
}
```

### Concurrent Access

```rust
use lru_cache::LruCache;
use std::sync::Arc;
use std::thread;

fn main() {
    let cache = Arc::new(LruCache::new(100));
    let mut handles = vec![];
    
    // Spawn multiple threads
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for j in 0..100 {
                cache_clone.put(i * 100 + j, format!("value-{}-{}", i, j));
            }
        });
        handles.push(handle);
    }
    
    // Wait for completion
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Cache size: {}", cache.len());
}
```

## API Reference

### `LruCache::new(capacity: usize) -> Self`

Creates a new LRU cache with the specified capacity.

**Panics** if capacity is 0.

### `get(&self, key: &K) -> Option<V>`

Gets a value from the cache. If the key exists, it's marked as recently used.

Returns `None` if the key is not in the cache.

**Time Complexity**: O(1) average

### `put(&self, key: K, value: V)`

Inserts a key-value pair. If the key exists, updates its value. If the cache is full, evicts the least recently used item.

**Time Complexity**: O(1) average

### `len(&self) -> usize`

Returns the number of items currently in the cache.

### `is_empty(&self) -> bool`

Returns `true` if the cache contains no items.

### `capacity(&self) -> usize`

Returns the maximum capacity of the cache.

### `clear(&self)`

Removes all items from the cache.

## Running Examples

```bash
# Basic usage example
cargo run --example basic_usage

# Concurrent stress test
cargo run --example concurrent_stress_test --release
```

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_concurrent_access
```

## Performance Considerations

1. **Single Lock Design**: While this limits maximum concurrency, it:
   - Prevents deadlocks
   - Ensures correctness under all conditions
   - Simplifies reasoning about thread safety
   - Performs well for typical cache usage patterns

2. **Memory Efficiency**: 
   - Uses a free list to recycle indices
   - Avoids repeated allocations
   - Memory usage bounded by capacity

3. **Lock Granularity**:
   - Fine enough for most use cases
   - Lock is held for microseconds per operation
   - Consider alternative designs only if profiling shows lock contention is a bottleneck

## Thread Safety Guarantees

- ✅ Safe for concurrent reads and writes
- ✅ No data races
- ✅ No deadlocks (single lock)
- ✅ Maintains LRU invariant under contention
- ✅ Bounded memory usage
- ✅ Implements `Send` and `Sync`

## Limitations and Future Improvements

### Current Limitations

1. **Read/Write Symmetry**: Even `get()` requires exclusive access (Mutex write lock) because it updates the access order
2. **No Async Support**: Uses `std::sync::Mutex` which blocks threads
3. **Clone Requirement**: Keys and values must implement `Clone`

### Potential Improvements

1. **RwLock with Separate Metadata**: Use `RwLock` for data and separate structure for access tracking
2. **Lock-Free Design**: Use atomic operations and compare-and-swap for lock-free reads
3. **Async Support**: Replace `Mutex` with `tokio::sync::Mutex` for async runtimes
4. **Sharded Cache**: Multiple LRU caches with a hash-based shard selection for higher concurrency
5. **Generic Lifetimes**: Support references instead of requiring ownership with `Clone`

## License

MIT OR Apache-2.0
