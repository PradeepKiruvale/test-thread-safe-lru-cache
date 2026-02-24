/// Example demonstrating sharded cache for improved concurrency
/// 
/// This example shows the benefits of cache sharding for concurrent workloads
use lru_cache::{LruCache, ShardedLruCache};
use std::thread;
use std::time::Instant;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    println!("=== Sharded Cache Example ===\n");
    
    // 1. Basic sharded cache operations
    println!("1. Basic Sharded Cache Operations:");
    let sharded_cache = ShardedLruCache::new(100, 4);
    
    sharded_cache.put(1, "one").unwrap();
    sharded_cache.put(2, "two").unwrap();
    sharded_cache.put(3, "three").unwrap();
    
    println!("   Added 3 items to sharded cache");
    println!("   Get key 1: {:?}", sharded_cache.get(&1).unwrap());
    println!("   Get key 2: {:?}", sharded_cache.get(&2).unwrap());
    println!("   Shard count: {}", sharded_cache.shard_count());
    
    // 2. Shard statistics
    println!("\n2. Shard Statistics:");
    let cache = ShardedLruCache::new(100, 4);
    
    // Add items
    for i in 0..50 {
        cache.put(i, format!("value-{i}")).unwrap();
    }
    
    let stats = cache.shard_stats().unwrap();
    println!("   Distribution across {} shards:", stats.len());
    for stat in stats {
        println!("   Shard {}: {}/{} items ({:.1}% full)",
            stat.shard_index,
            stat.size,
            stat.capacity,
            stat.utilization * 100.0
        );
    }
    
    // 3. Concurrency comparison
    println!("\n3. Concurrency Performance Comparison:");
    println!("   Running concurrent workload with 8 threads...\n");
    
    const NUM_THREADS: usize = 8;
    const OPS_PER_THREAD: usize = 10_000;
    
    // Test standard cache
    let standard_cache = LruCache::new(1000);
    let standard_time = {
        let start = Instant::now();
        let mut handles = vec![];
        
        for thread_id in 0..NUM_THREADS {
            let cache_clone = standard_cache.clone();
            let handle = thread::spawn(move || {
                for i in 0..OPS_PER_THREAD {
                    let key = thread_id * OPS_PER_THREAD + i;
                    cache_clone.put(key, format!("value-{key}")).unwrap();
                    if i % 3 == 0 {
                        cache_clone.get(&key).unwrap();
                    }
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        start.elapsed()
    };
    
    println!("   Standard cache (single mutex): {standard_time:?}");
    
    // Test sharded cache with 4 shards
    let sharded_cache_4 = ShardedLruCache::new(1000, 4);
    let sharded_time_4 = {
        let start = Instant::now();
        let mut handles = vec![];
        
        for thread_id in 0..NUM_THREADS {
            let cache_clone = sharded_cache_4.clone();
            let handle = thread::spawn(move || {
                for i in 0..OPS_PER_THREAD {
                    let key = thread_id * OPS_PER_THREAD + i;
                    cache_clone.put(key, format!("value-{key}")).unwrap();
                    if i % 3 == 0 {
                        cache_clone.get(&key).unwrap();
                    }
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        start.elapsed()
    };
    
    println!("   Sharded cache (4 shards):       {sharded_time_4:?}");
    
    // Test sharded cache with 16 shards
    let sharded_cache_16 = ShardedLruCache::new(1000, 16);
    let sharded_time_16 = {
        let start = Instant::now();
        let mut handles = vec![];
        
        for thread_id in 0..NUM_THREADS {
            let cache_clone = sharded_cache_16.clone();
            let handle = thread::spawn(move || {
                for i in 0..OPS_PER_THREAD {
                    let key = thread_id * OPS_PER_THREAD + i;
                    cache_clone.put(key, format!("value-{key}")).unwrap();
                    if i % 3 == 0 {
                        cache_clone.get(&key).unwrap();
                    }
                }
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        start.elapsed()
    };
    
    println!("   Sharded cache (16 shards):      {sharded_time_16:?}");
    
    // Calculate speedup
    let speedup_4 = standard_time.as_secs_f64() / sharded_time_4.as_secs_f64();
    let speedup_16 = standard_time.as_secs_f64() / sharded_time_16.as_secs_f64();
    
    println!("\n   Speedup (4 shards):  {speedup_4:.2}x");
    println!("   Speedup (16 shards): {speedup_16:.2}x");
    
    // 4. Demonstrate contention reduction
    println!("\n4. Lock Contention Demonstration:");
    let contention_cache = ShardedLruCache::new(100, 8);
    let contention_counter = Arc::new(AtomicUsize::new(0));
    
    // Simulate high-contention scenario
    let mut handles = vec![];
    for thread_id in 0..16 {
        let cache_clone = contention_cache.clone();
        let counter_clone = Arc::clone(&contention_counter);
        
        let handle = thread::spawn(move || {
            for i in 0..1000 {
                let key = (thread_id * 1000 + i) % 200; // Some key overlap
                cache_clone.put(key, format!("value-{key}")).unwrap();
                counter_clone.fetch_add(1, Ordering::Relaxed);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    println!("   Completed {} concurrent operations", 
        contention_counter.load(Ordering::Relaxed));
    println!("   Final cache size: {}", contention_cache.len().unwrap());
    
    let final_stats = contention_cache.shard_stats().unwrap();
    println!("\n   Final shard distribution:");
    for stat in final_stats {
        let bar_length = (stat.utilization * 40.0) as usize;
        let bar = "█".repeat(bar_length);
        println!("   Shard {:2}: [{:<40}] {:>3}/{:>3}",
            stat.shard_index,
            bar,
            stat.size,
            stat.capacity
        );
    }
    
    // 5. Best practices
    println!("\n5. Sharded Cache Best Practices:");
    println!("   - Use power of 2 for shard count (4, 8, 16, 32)");
    println!("   - More shards = better concurrency, but diminishing returns");
    println!("   - Trade-off: Per-shard LRU vs global LRU semantics");
    println!("   - Ideal shard count ≈ number of CPU cores");
    println!("   - Monitor shard statistics for load balancing");
    
    println!("\n{}", "=".repeat(60));
    println!("Sharded cache example completed!");
}
