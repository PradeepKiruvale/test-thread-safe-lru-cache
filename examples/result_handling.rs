// Example demonstrating Result-based error handling

use lru_cache::{LruCache, CacheError};

fn main() -> Result<(), CacheError> {
    println!("=== LRU Cache with Result-based Error Handling ===\n");
    
    let cache = LruCache::new(3);
    
    // All operations now return Results
    println!("Adding items to cache...");
    cache.put(1, "one")?;
    cache.put(2, "two")?;
    cache.put(3, "three")?;
    
    println!("Cache size: {}", cache.len()?);
    println!("Cache capacity: {}\n", cache.capacity()?);
    
    // Get operations return Result<Option<V>, CacheError>
    println!("Retrieving values:");
    match cache.get(&1)? {
        Some(value) => println!("  Key 1: {}", value),
        None => println!("  Key 1: not found"),
    }
    
    match cache.get(&2)? {
        Some(value) => println!("  Key 2: {}", value),
        None => println!("  Key 2: not found"),
    }
    
    // Adding a fourth item will evict the least recently used (key 1)
    println!("\nAdding 4th item (triggers eviction)...");
    cache.put(4, "four")?;
    
    println!("After eviction:");
    println!("  Key 1 exists: {}", cache.get(&1)?.is_some());
    println!("  Key 2 exists: {}", cache.get(&2)?.is_some());
    println!("  Key 3 exists: {}", cache.get(&3)?.is_some());
    println!("  Key 4 exists: {}\n", cache.get(&4)?.is_some());
    
    // Using the ? operator for error propagation
    let size = cache.len()?;
    let is_empty = cache.is_empty()?;
    println!("Final state:");
    println!("  Size: {}", size);
    println!("  Is empty: {}", is_empty);
    
    // Clear the cache
    cache.clear()?;
    println!("\nAfter clear:");
    println!("  Size: {}", cache.len()?);
    println!("  Is empty: {}", cache.is_empty()?);
    
    Ok(())
}
