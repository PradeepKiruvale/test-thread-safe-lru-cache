use lru_cache::LruCache;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use rand::Rng;

fn main() {
    println!("=== Thread-Safe LRU Cache Stress Test ===\n");

    let cache_capacity = 1000;
    let num_threads = 8;
    let operations_per_thread = 10_000;

    println!("Configuration:");
    println!("  Cache capacity: {cache_capacity}");
    println!("  Concurrent threads: {num_threads}");
    println!("  Operations per thread: {operations_per_thread}");
    println!("  Total operations: {}\n", num_threads * operations_per_thread);

    let cache = Arc::new(LruCache::new(cache_capacity));
    
    println!("Starting concurrent stress test...");
    let start = Instant::now();

    let mut writer_handles = vec![];
    let mut reader_handles = vec![];

    // Spawn writer threads
    for thread_id in 0..num_threads / 2 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut local_puts = 0;
            
            for _ in 0..operations_per_thread {
                let key = rng.gen_range(0..cache_capacity * 2);
                let value = format!("thread-{thread_id}-value-{key}");
                cache_clone.put(key, value).unwrap();
                local_puts += 1;
                
                // Occasionally sleep to simulate real work
                if local_puts % 1000 == 0 {
                    thread::sleep(Duration::from_micros(10));
                }
            }
            
            local_puts
        });
        writer_handles.push(handle);
    }

    // Spawn reader threads
    for _ in 0..num_threads / 2 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let mut hits = 0;
            let mut misses = 0;
            
            for _ in 0..operations_per_thread {
                let key = rng.gen_range(0..cache_capacity * 2);
                
                if cache_clone.get(&key).unwrap().is_some() {
                    hits += 1;
                } else {
                    misses += 1;
                }
                
                // Occasionally sleep
                if (hits + misses) % 1000 == 0 {
                    thread::sleep(Duration::from_micros(10));
                }
            }
            
            (hits, misses)
        });
        reader_handles.push(handle);
    }

    // Collect results
    let mut total_puts = 0;
    let mut total_hits = 0;
    let mut total_misses = 0;

    for handle in writer_handles {
        let puts = handle.join().unwrap();
        total_puts += puts;
    }

    for handle in reader_handles {
        let (hits, misses) = handle.join().unwrap();
        total_hits += hits;
        total_misses += misses;
    }

    let duration = start.elapsed();

    println!("\nTest completed in {duration:?}");
    println!("\nResults:");
    println!("  Total PUT operations: {total_puts}");
    println!("  Total GET operations: {}", total_hits + total_misses);
    println!("    Cache hits: {total_hits}");
    println!("    Cache misses: {total_misses}");
    println!("    Hit rate: {:.2}%", 
             (total_hits as f64 / (total_hits + total_misses) as f64) * 100.0);
    println!("\n  Final cache size: {}", cache.len().unwrap());
    println!("  Cache capacity: {}", cache.capacity().unwrap());
    println!("  Operations per second: {:.0}", 
             (num_threads * operations_per_thread) as f64 / duration.as_secs_f64());

    // Verify cache integrity
    println!("\nVerifying cache integrity...");
    let final_size = cache.len().unwrap();
    assert!(final_size <= cache_capacity, "Cache exceeded capacity!");
    println!("  ✓ Cache size within capacity");
    println!("  ✓ No deadlocks detected");
    println!("  ✓ All threads completed successfully");

    println!("\n=== Stress test PASSED ===");
}
