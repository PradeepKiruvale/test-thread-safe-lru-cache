/// Example demonstrating different eviction policies
/// 
/// This example compares LRU, LFU, and FIFO eviction strategies
use lru_cache::eviction::{
    ThreadSafeEvictableCache, LruPolicy, LfuPolicy, FifoPolicy
};

fn main() {
    println!("=== Eviction Policy Comparison ===\n");
    
    // 1. LRU (Least Recently Used) Policy
    println!("1. LRU (Least Recently Used) Policy:");
    println!("   Evicts the item that hasn't been accessed for the longest time\n");
    
    let lru_cache = ThreadSafeEvictableCache::new(3, LruPolicy::new());
    
    lru_cache.put(1, "one");
    lru_cache.put(2, "two");
    lru_cache.put(3, "three");
    println!("   Added keys 1, 2, 3");
    
    lru_cache.get(&1); // Access key 1
    println!("   Accessed key 1 (making it most recently used)");
    
    lru_cache.put(4, "four"); // Will evict key 2 (least recently used)
    println!("   Added key 4 (capacity reached)");
    
    println!("\n   Results:");
    println!("   Key 1: {:?} (accessed recently, kept)", lru_cache.get(&1));
    println!("   Key 2: {:?} (least recently used, evicted)", lru_cache.get(&2));
    println!("   Key 3: {:?} (kept)", lru_cache.get(&3));
    println!("   Key 4: {:?} (just added, kept)", lru_cache.get(&4));
    
    // 2. LFU (Least Frequently Used) Policy
    println!("\n2. LFU (Least Frequently Used) Policy:");
    println!("   Evicts the item that has been accessed the fewest times\n");
    
    let lfu_cache = ThreadSafeEvictableCache::new(3, LfuPolicy::new());
    
    lfu_cache.put(1, "one");
    lfu_cache.put(2, "two");
    lfu_cache.put(3, "three");
    println!("   Added keys 1, 2, 3");
    
    // Access key 1 multiple times
    lfu_cache.get(&1);
    lfu_cache.get(&1);
    lfu_cache.get(&1);
    println!("   Accessed key 1 three times (frequency = 4)");
    
    lfu_cache.get(&2); // Access key 2 once
    println!("   Accessed key 2 once (frequency = 2)");
    
    // Key 3 has frequency = 1 (only put, no gets)
    println!("   Key 3 has frequency = 1 (only inserted)");
    
    lfu_cache.put(4, "four"); // Will evict key 3 (least frequently used)
    println!("\n   Added key 4 (capacity reached)");
    
    println!("\n   Results:");
    println!("   Key 1: {:?} (frequency 4, kept)", lfu_cache.get(&1));
    println!("   Key 2: {:?} (frequency 2, kept)", lfu_cache.get(&2));
    println!("   Key 3: {:?} (frequency 1, evicted)", lfu_cache.get(&3));
    println!("   Key 4: {:?} (just added, kept)", lfu_cache.get(&4));
    
    // 3. FIFO (First In First Out) Policy
    println!("\n3. FIFO (First In First Out) Policy:");
    println!("   Evicts the oldest item regardless of access pattern\n");
    
    let fifo_cache = ThreadSafeEvictableCache::new(3, FifoPolicy::new());
    
    fifo_cache.put(1, "one");
    fifo_cache.put(2, "two");
    fifo_cache.put(3, "three");
    println!("   Added keys 1, 2, 3 (in order)");
    
    // Access key 1 many times
    for _ in 0..10 {
        fifo_cache.get(&1);
    }
    println!("   Accessed key 1 ten times (FIFO ignores this)");
    
    fifo_cache.put(4, "four"); // Will evict key 1 (first in)
    println!("\n   Added key 4 (capacity reached)");
    
    println!("\n   Results:");
    println!("   Key 1: {:?} (first in, evicted despite frequent access)", fifo_cache.get(&1));
    println!("   Key 2: {:?} (kept)", fifo_cache.get(&2));
    println!("   Key 3: {:?} (kept)", fifo_cache.get(&3));
    println!("   Key 4: {:?} (just added, kept)", fifo_cache.get(&4));
    
    // 4. Practical comparison
    println!("\n4. Practical Scenario - Web Page Caching:");
    println!("   Simulating caching of web pages with different access patterns\n");
    
    let lru = ThreadSafeEvictableCache::new(3, LruPolicy::new());
    let lfu = ThreadSafeEvictableCache::new(3, LfuPolicy::new());
    let fifo = ThreadSafeEvictableCache::new(3, FifoPolicy::new());
    
    // Scenario: Homepage, Product, About pages
    // Homepage accessed frequently, Product occasionally, About rarely
    
    // Initial population
    lru.put("home", "Home Page");
    lru.put("product", "Product Page");
    lru.put("about", "About Page");
    
    lfu.put("home", "Home Page");
    lfu.put("product", "Product Page");
    lfu.put("about", "About Page");
    
    fifo.put("home", "Home Page");
    fifo.put("product", "Product Page");
    fifo.put("about", "About Page");
    
    // Simulate access pattern
    // Homepage: 10 accesses
    for _ in 0..10 {
        lru.get(&"home");
        lfu.get(&"home");
        fifo.get(&"home");
    }
    
    // Product: 3 accesses
    for _ in 0..3 {
        lru.get(&"product");
        lfu.get(&"product");
        fifo.get(&"product");
    }
    
    // About: 0 accesses
    
    // Add new page (forces eviction)
    println!("   Adding 'contact' page (capacity reached)...\n");
    lru.put("contact", "Contact Page");
    lfu.put("contact", "Contact Page");
    fifo.put("contact", "Contact Page");
    
    println!("   Results after adding 'contact' page:");
    println!("\n   LRU Cache:");
    println!("   - home: {:?} (recently accessed, kept)", lru.get(&"home"));
    println!("   - product: {:?} (recently accessed, kept)", lru.get(&"product"));
    println!("   - about: {:?} (not recently accessed, evicted)", lru.get(&"about"));
    println!("   - contact: {:?} (just added, kept)", lru.get(&"contact"));
    
    println!("\n   LFU Cache:");
    println!("   - home: {:?} (high frequency, kept)", lfu.get(&"home"));
    println!("   - product: {:?} (medium frequency, kept)", lfu.get(&"product"));
    println!("   - about: {:?} (low frequency, evicted)", lfu.get(&"about"));
    println!("   - contact: {:?} (just added, kept)", lfu.get(&"contact"));
    
    println!("\n   FIFO Cache:");
    println!("   - home: {:?} (first in, evicted despite high usage)", fifo.get(&"home"));
    println!("   - product: {:?} (kept)", fifo.get(&"product"));
    println!("   - about: {:?} (kept)", fifo.get(&"about"));
    println!("   - contact: {:?} (just added, kept)", fifo.get(&"contact"));
    
    println!("\n{}", "=".repeat(60));
    println!("Summary:");
    println!("- LRU: Best for temporal locality (recent = important)");
    println!("- LFU: Best for frequency-based patterns (popular = important)");
    println!("- FIFO: Simplest, but ignores access patterns");
    println!("{}", "=".repeat(60));
}
