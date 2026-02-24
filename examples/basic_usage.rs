use lru_cache::LruCache;

fn main() {
    println!("=== Basic LRU Cache Usage ===\n");

    // Create a cache with capacity of 3
    let cache = LruCache::new(3);

    println!("Cache capacity: {}", cache.capacity());
    println!("Initial size: {}\n", cache.len());

    // Add some items
    println!("Adding items:");
    cache.put(1, "one");
    println!("  put(1, 'one') -> size: {}", cache.len());
    
    cache.put(2, "two");
    println!("  put(2, 'two') -> size: {}", cache.len());
    
    cache.put(3, "three");
    println!("  put(3, 'three') -> size: {}\n", cache.len());

    // Get items
    println!("Getting items:");
    println!("  get(1) = {:?}", cache.get(&1));
    println!("  get(2) = {:?}", cache.get(&2));
    println!("  get(3) = {:?}\n", cache.get(&3));

    // Access key 1 to make it recently used
    println!("Accessing key 1 to update recency:");
    let _ = cache.get(&1);
    println!("  get(1) = {:?}\n", cache.get(&1));

    // Add a 4th item - should evict least recently used (key 2)
    println!("Cache is full. Adding 4th item will evict LRU:");
    cache.put(4, "four");
    println!("  put(4, 'four') -> size: {}", cache.len());
    println!("  Key 2 should be evicted...\n");

    // Check what's in the cache
    println!("Current cache contents:");
    println!("  get(1) = {:?}", cache.get(&1));
    println!("  get(2) = {:?} (evicted)", cache.get(&2));
    println!("  get(3) = {:?}", cache.get(&3));
    println!("  get(4) = {:?}\n", cache.get(&4));

    // Update an existing key
    println!("Updating existing key:");
    cache.put(3, "THREE");
    println!("  put(3, 'THREE')");
    println!("  get(3) = {:?}\n", cache.get(&3));

    // Clear the cache
    println!("Clearing cache:");
    cache.clear();
    println!("  clear() -> size: {}", cache.len());
    println!("  is_empty: {}\n", cache.is_empty());

    // Demonstrate with different types
    println!("=== Using String keys and i32 values ===\n");
    let string_cache = LruCache::new(2);
    
    string_cache.put("user:1".to_string(), 100);
    string_cache.put("user:2".to_string(), 200);
    
    println!("  get('user:1') = {:?}", string_cache.get(&"user:1".to_string()));
    println!("  get('user:2') = {:?}", string_cache.get(&"user:2".to_string()));
}
