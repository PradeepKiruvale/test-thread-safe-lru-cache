use lru_cache::LruCache;

fn main() {
    println!("=== Basic LRU Cache Usage ===\n");

    // Create a cache with capacity of 3
    let cache = LruCache::new(3);

    println!("Cache capacity: {}", cache.capacity().unwrap());
    println!("Initial size: {}\n", cache.len().unwrap());

    // Add some items
    println!("Adding items:");
    cache.put(1, "one").unwrap();
    println!("  put(1, 'one') -> size: {}", cache.len().unwrap());
    
    cache.put(2, "two").unwrap();
    println!("  put(2, 'two') -> size: {}", cache.len().unwrap());
    
    cache.put(3, "three").unwrap();
    println!("  put(3, 'three') -> size: {}\n", cache.len().unwrap());

    // Get items
    println!("Getting items:");
    println!("  get(1) = {:?}", cache.get(&1).unwrap());
    println!("  get(2) = {:?}", cache.get(&2).unwrap());
    println!("  get(3) = {:?}\n", cache.get(&3).unwrap());

    // Access key 1 to make it recently used
    println!("Accessing key 1 to update recency:");
    let _ = cache.get(&1).unwrap();
    println!("  get(1) = {:?}\n", cache.get(&1).unwrap());

    // Add a 4th item - should evict least recently used (key 2)
    println!("Cache is full. Adding 4th item will evict LRU:");
    cache.put(4, "four").unwrap();
    println!("  put(4, 'four') -> size: {}", cache.len().unwrap());
    println!("  Key 2 should be evicted...\n");

    // Check what's in the cache
    println!("Current cache contents:");
    println!("  get(1) = {:?}", cache.get(&1).unwrap());
    println!("  get(2) = {:?} (evicted)", cache.get(&2).unwrap());
    println!("  get(3) = {:?}", cache.get(&3).unwrap());
    println!("  get(4) = {:?}\n", cache.get(&4).unwrap());

    // Update an existing key
    println!("Updating existing key:");
    cache.put(3, "THREE").unwrap();
    println!("  put(3, 'THREE')");
    println!("  get(3) = {:?}\n", cache.get(&3).unwrap());

    // Clear the cache
    println!("Clearing cache:");
    cache.clear().unwrap();
    println!("  clear() -> size: {}", cache.len().unwrap());
    println!("  is_empty: {}\n", cache.is_empty().unwrap());

    // Demonstrate with different types
    println!("=== Using String keys and i32 values ===\n");
    let string_cache = LruCache::new(2);
    
    string_cache.put("user:1".to_string(), 100).unwrap();
    string_cache.put("user:2".to_string(), 200).unwrap();
    
    println!("  get('user:1') = {:?}", string_cache.get(&"user:1".to_string()).unwrap());
    println!("  get('user:2') = {:?}", string_cache.get(&"user:2".to_string()).unwrap());
}
