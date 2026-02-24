/// Performance comparison benchmark example
/// 
/// This example compares the performance of different caching strategies:
/// - Single-threaded baseline
/// - Standard LRU with single Mutex
/// - Sharded LRU cache
/// - Different eviction policies (LRU, LFU, FIFO)
use lru_cache::{LruCache, ShardedLruCache};
use lru_cache::eviction::{ThreadSafeEvictableCache, LfuPolicy, FifoPolicy};
use std::time::{Instant, Duration};
use std::thread;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use rand::Rng;

const CACHE_CAPACITY: usize = 1000;
const TOTAL_OPS: usize = 100_000;
const NUM_THREADS: usize = 8;

struct BenchmarkResult {
    name: String,
    duration: Duration,
    ops_per_sec: f64,
    hits: usize,
    misses: usize,
    hit_rate: f64,
}

impl BenchmarkResult {
    fn print(&self) {
        println!("\n{}", "=".repeat(60));
        println!("Benchmark: {}", self.name);
        println!("{}", "-".repeat(60));
        println!("Duration:     {:?}", self.duration);
        println!("Throughput:   {:.2} ops/sec", self.ops_per_sec);
        println!("Hits:         {}", self.hits);
        println!("Misses:       {}", self.misses);
        println!("Hit Rate:     {:.2}%", self.hit_rate * 100.0);
        println!("{}", "=".repeat(60));
    }
}

fn benchmark_standard_lru() -> BenchmarkResult {
    let cache = LruCache::new(CACHE_CAPACITY);
    let hits = Arc::new(AtomicUsize::new(0));
    let misses = Arc::new(AtomicUsize::new(0));
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for _ in 0..NUM_THREADS {
        let cache_clone = cache.clone();
        let hits_clone = Arc::clone(&hits);
        let misses_clone = Arc::clone(&misses);
        
        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let ops_per_thread = TOTAL_OPS / NUM_THREADS;
            
            for _ in 0..ops_per_thread {
                let key = rng.gen_range(0..(CACHE_CAPACITY * 2));
                
                if rng.gen_bool(0.7) {
                    // 70% reads
                    if cache_clone.get(&key).unwrap().is_some() {
                        hits_clone.fetch_add(1, Ordering::Relaxed);
                    } else {
                        misses_clone.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    // 30% writes
                    cache_clone.put(key, format!("value-{key}")).unwrap();
                }
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    let total_hits = hits.load(Ordering::Relaxed);
    let total_misses = misses.load(Ordering::Relaxed);
    let hit_rate = total_hits as f64 / (total_hits + total_misses) as f64;
    
    BenchmarkResult {
        name: "Standard LRU (Single Mutex)".to_string(),
        duration,
        ops_per_sec: TOTAL_OPS as f64 / duration.as_secs_f64(),
        hits: total_hits,
        misses: total_misses,
        hit_rate,
    }
}

fn benchmark_sharded_lru(shard_count: usize) -> BenchmarkResult {
    let cache = ShardedLruCache::new(CACHE_CAPACITY, shard_count);
    let hits = Arc::new(AtomicUsize::new(0));
    let misses = Arc::new(AtomicUsize::new(0));
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for _ in 0..NUM_THREADS {
        let cache_clone = cache.clone();
        let hits_clone = Arc::clone(&hits);
        let misses_clone = Arc::clone(&misses);
        
        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let ops_per_thread = TOTAL_OPS / NUM_THREADS;
            
            for _ in 0..ops_per_thread {
                let key = rng.gen_range(0..(CACHE_CAPACITY * 2));
                
                if rng.gen_bool(0.7) {
                    // 70% reads
                    if cache_clone.get(&key).unwrap().is_some() {
                        hits_clone.fetch_add(1, Ordering::Relaxed);
                    } else {
                        misses_clone.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    // 30% writes
                    cache_clone.put(key, format!("value-{key}")).unwrap();
                }
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    let total_hits = hits.load(Ordering::Relaxed);
    let total_misses = misses.load(Ordering::Relaxed);
    let hit_rate = total_hits as f64 / (total_hits + total_misses) as f64;
    
    BenchmarkResult {
        name: format!("Sharded LRU ({shard_count} shards)"),
        duration,
        ops_per_sec: TOTAL_OPS as f64 / duration.as_secs_f64(),
        hits: total_hits,
        misses: total_misses,
        hit_rate,
    }
}

fn benchmark_lfu() -> BenchmarkResult {
    let cache = ThreadSafeEvictableCache::new(CACHE_CAPACITY, LfuPolicy::new());
    let hits = Arc::new(AtomicUsize::new(0));
    let misses = Arc::new(AtomicUsize::new(0));
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for _ in 0..NUM_THREADS {
        let cache_clone = cache.clone();
        let hits_clone = Arc::clone(&hits);
        let misses_clone = Arc::clone(&misses);
        
        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let ops_per_thread = TOTAL_OPS / NUM_THREADS;
            
            for _ in 0..ops_per_thread {
                let key = rng.gen_range(0..(CACHE_CAPACITY * 2));
                
                if rng.gen_bool(0.7) {
                    if cache_clone.get(&key).unwrap().is_some() {
                        hits_clone.fetch_add(1, Ordering::Relaxed);
                    } else {
                        misses_clone.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    cache_clone.put(key, format!("value-{key}")).unwrap();
                }
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    let total_hits = hits.load(Ordering::Relaxed);
    let total_misses = misses.load(Ordering::Relaxed);
    let hit_rate = total_hits as f64 / (total_hits + total_misses) as f64;
    
    BenchmarkResult {
        name: "LFU Policy".to_string(),
        duration,
        ops_per_sec: TOTAL_OPS as f64 / duration.as_secs_f64(),
        hits: total_hits,
        misses: total_misses,
        hit_rate,
    }
}

fn benchmark_fifo() -> BenchmarkResult {
    let cache = ThreadSafeEvictableCache::new(CACHE_CAPACITY, FifoPolicy::new());
    let hits = Arc::new(AtomicUsize::new(0));
    let misses = Arc::new(AtomicUsize::new(0));
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for _ in 0..NUM_THREADS {
        let cache_clone = cache.clone();
        let hits_clone = Arc::clone(&hits);
        let misses_clone = Arc::clone(&misses);
        
        let handle = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let ops_per_thread = TOTAL_OPS / NUM_THREADS;
            
            for _ in 0..ops_per_thread {
                let key = rng.gen_range(0..(CACHE_CAPACITY * 2));
                
                if rng.gen_bool(0.7) {
                    if cache_clone.get(&key).unwrap().is_some() {
                        hits_clone.fetch_add(1, Ordering::Relaxed);
                    } else {
                        misses_clone.fetch_add(1, Ordering::Relaxed);
                    }
                } else {
                    cache_clone.put(key, format!("value-{key}")).unwrap();
                }
            }
        });
        
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    let duration = start.elapsed();
    let total_hits = hits.load(Ordering::Relaxed);
    let total_misses = misses.load(Ordering::Relaxed);
    let hit_rate = total_hits as f64 / (total_hits + total_misses) as f64;
    
    BenchmarkResult {
        name: "FIFO Policy".to_string(),
        duration,
        ops_per_sec: TOTAL_OPS as f64 / duration.as_secs_f64(),
        hits: total_hits,
        misses: total_misses,
        hit_rate,
    }
}

fn main() {
    println!("\n{}", "=".repeat(60));
    println!("LRU Cache Performance Comparison");
    println!("{}", "=".repeat(60));
    println!("Configuration:");
    println!("  Cache Capacity: {CACHE_CAPACITY}");
    println!("  Total Operations: {TOTAL_OPS}");
    println!("  Number of Threads: {NUM_THREADS}");
    println!("  Read/Write Ratio: 70/30");
    println!("{}", "=".repeat(60));
    
    // Run benchmarks
    let standard_result = benchmark_standard_lru();
    standard_result.print();
    
    let sharded_4_result = benchmark_sharded_lru(4);
    sharded_4_result.print();
    
    let sharded_8_result = benchmark_sharded_lru(8);
    sharded_8_result.print();
    
    let sharded_16_result = benchmark_sharded_lru(16);
    sharded_16_result.print();
    
    let lfu_result = benchmark_lfu();
    lfu_result.print();
    
    let fifo_result = benchmark_fifo();
    fifo_result.print();
    
    // Summary comparison
    println!("\n{}", "=".repeat(60));
    println!("SUMMARY COMPARISON");
    println!("{}", "=".repeat(60));
    println!("{:<35} {:>12} {:>10}", "Implementation", "Throughput", "Hit Rate");
    println!("{}", "-".repeat(60));
    
    let results = vec![
        &standard_result,
        &sharded_4_result,
        &sharded_8_result,
        &sharded_16_result,
        &lfu_result,
        &fifo_result,
    ];
    
    for result in &results {
        println!("{:<35} {:>9.0} ops/s {:>7.1}%",
            result.name,
            result.ops_per_sec,
            result.hit_rate * 100.0);
    }
    
    // Find best performer
    let best = results.iter().max_by(|a, b| {
        a.ops_per_sec.partial_cmp(&b.ops_per_sec).unwrap()
    }).unwrap();
    
    println!("{}", "=".repeat(60));
    println!("Best Performance: {} ({:.0} ops/sec)",
        best.name, best.ops_per_sec);
    
    // Calculate speedup
    let speedup = best.ops_per_sec / standard_result.ops_per_sec;
    println!("Speedup vs Standard: {speedup:.2}x");
    println!("{}", "=".repeat(60));
}
