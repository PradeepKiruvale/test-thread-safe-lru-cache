# Quick Start Guide

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
# For now, use as a path dependency (publish to crates.io later)
lru_cache = { path = "../lru_cache" }
```

## Basic Usage

```rust
use lru_cache::LruCache;

fn main() {
    // Create a cache with capacity of 100
    let cache = LruCache::new(100);
    
    // Insert some key-value pairs
    cache.put("user:123", "Alice");
    cache.put("user:456", "Bob");
    
    // Retrieve values
    if let Some(name) = cache.get(&"user:123") {
        println!("Found user: {}", name);
    }
    
    // Cache automatically evicts LRU items when full
    for i in 0..100 {
        cache.put(format!("key:{}", i), i * 2);
    }
    
    println!("Cache size: {}", cache.len());
    println!("Capacity: {}", cache.capacity());
}
```

## Thread-Safe Usage

```rust
use lru_cache::LruCache;
use std::sync::Arc;
use std::thread;

fn main() {
    // Wrap in Arc for sharing across threads
    let cache = Arc::new(LruCache::new(1000));
    
    let mut handles = vec![];
    
    // Spawn multiple threads
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        
        let handle = thread::spawn(move || {
            // Each thread can safely access the cache
            for j in 0..100 {
                let key = i * 100 + j;
                cache_clone.put(key, format!("value-{}", key));
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("Final cache size: {}", cache.len());
}
```

## Common Patterns

### Session Storage

```rust
use lru_cache::LruCache;
use std::sync::Arc;

struct SessionStore {
    cache: Arc<LruCache<String, UserSession>>,
}

#[derive(Clone)]
struct UserSession {
    user_id: u64,
    token: String,
    created_at: u64,
}

impl SessionStore {
    fn new(max_sessions: usize) -> Self {
        SessionStore {
            cache: Arc::new(LruCache::new(max_sessions)),
        }
    }
    
    fn store_session(&self, token: String, session: UserSession) {
        self.cache.put(token, session);
    }
    
    fn get_session(&self, token: &str) -> Option<UserSession> {
        self.cache.get(&token.to_string())
    }
}
```

### HTTP Cache

```rust
use lru_cache::LruCache;

#[derive(Clone)]
struct CachedResponse {
    status: u16,
    body: String,
    headers: Vec<(String, String)>,
}

struct HttpCache {
    cache: LruCache<String, CachedResponse>,
}

impl HttpCache {
    fn new(capacity: usize) -> Self {
        HttpCache {
            cache: LruCache::new(capacity),
        }
    }
    
    fn cache_response(&self, url: String, response: CachedResponse) {
        self.cache.put(url, response);
    }
    
    fn get_cached(&self, url: &str) -> Option<CachedResponse> {
        self.cache.get(&url.to_string())
    }
}
```

### Database Query Cache

```rust
use lru_cache::LruCache;

#[derive(Clone, Debug)]
struct QueryResult {
    rows: Vec<Vec<String>>,
    columns: Vec<String>,
}

struct QueryCache {
    cache: LruCache<String, QueryResult>,
}

impl QueryCache {
    fn new(capacity: usize) -> Self {
        QueryCache {
            cache: LruCache::new(capacity),
        }
    }
    
    fn get_or_execute<F>(&self, query: &str, execute: F) -> QueryResult
    where
        F: FnOnce() -> QueryResult,
    {
        if let Some(cached) = self.cache.get(&query.to_string()) {
            println!("Cache hit for query: {}", query);
            return cached;
        }
        
        println!("Cache miss, executing query: {}", query);
        let result = execute();
        self.cache.put(query.to_string(), result.clone());
        result
    }
}
```

## Performance Tips

1. **Choose the right capacity:**
   ```rust
   // Too small: frequent evictions
   let cache = LruCache::new(10);  // ⚠️
   
   // Right size: balance between memory and hit rate
   let cache = LruCache::new(1000);  // ✅
   
   // Too large: wasted memory
   let cache = LruCache::new(1_000_000);  // ⚠️
   ```

2. **Pre-warm the cache:**
   ```rust
   let cache = LruCache::new(100);
   
   // Load frequently accessed items first
   for key in frequently_used_keys {
       cache.put(key, fetch_value(key));
   }
   ```

3. **Monitor hit rate:**
   ```rust
   let mut hits = 0;
   let mut misses = 0;
   
   if cache.get(&key).is_some() {
       hits += 1;
   } else {
       misses += 1;
   }
   
   let hit_rate = hits as f64 / (hits + misses) as f64;
   println!("Hit rate: {:.2}%", hit_rate * 100.0);
   ```

## API Reference

### Creating a Cache

```rust
let cache = LruCache::new(capacity);
```

### Inserting Items

```rust
cache.put(key, value);  // Inserts or updates
```

### Retrieving Items

```rust
if let Some(value) = cache.get(&key) {
    // Value found and marked as recently used
    println!("Found: {}", value);
} else {
    // Value not in cache
    println!("Cache miss");
}
```

### Cache Management

```rust
let size = cache.len();           // Current number of items
let capacity = cache.capacity();  // Maximum capacity
let is_empty = cache.is_empty();  // Check if empty
cache.clear();                    // Remove all items
```

## Testing

Run the examples:

```bash
# Basic usage
cargo run --example basic_usage

# Concurrent stress test
cargo run --example concurrent_stress_test --release
```

Run the test suite:

```bash
# All tests
cargo test

# Specific test
cargo test test_concurrent_access

# With output
cargo test -- --nocapture
```

## Troubleshooting

### Compilation Error: "cannot borrow cache as mutable"

```rust
// ❌ Wrong
let mut cache = LruCache::new(10);
cache.put(1, "one");  // Error: cannot borrow as mutable

// ✅ Correct - cache methods take &self, not &mut self
let cache = LruCache::new(10);
cache.put(1, "one");  // Works!
```

### Clone Requirement

Keys and values must implement `Clone`:

```rust
// ✅ These work
let cache: LruCache<i32, String> = LruCache::new(10);
let cache: LruCache<String, Vec<u8>> = LruCache::new(10);

// ❌ This doesn't (if MyStruct doesn't impl Clone)
let cache: LruCache<i32, MyStruct> = LruCache::new(10);
```

Solution: Derive or implement Clone:

```rust
#[derive(Clone)]
struct MyStruct {
    field: String,
}
```

### Thread Safety

The cache is `Send` and `Sync`, so it can be shared across threads with `Arc`:

```rust
use std::sync::Arc;

let cache = Arc::new(LruCache::new(100));
let cache_clone = Arc::clone(&cache);

std::thread::spawn(move || {
    cache_clone.put(1, "value");
});
```

## Next Steps

- Read [DESIGN.md](DESIGN.md) for detailed design decisions
- Check [README.md](README.md) for full documentation
- Explore the examples in the `examples/` directory
- Run the benchmark to see performance characteristics
