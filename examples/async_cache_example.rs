/// Example demonstrating async LRU cache usage
/// 
/// This example shows how to use the async-compatible cache with Tokio runtime

use lru_cache::AsyncLruCache;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    println!("=== Async LRU Cache Example ===\n");
    
    // Create an async cache with capacity 5
    let cache = AsyncLruCache::new(5);
    
    // Basic operations
    println!("1. Basic async operations:");
    cache.put(1, "one").await;
    cache.put(2, "two").await;
    cache.put(3, "three").await;
    
    println!("   Get key 1: {:?}", cache.get(&1).await);
    println!("   Get key 2: {:?}", cache.get(&2).await);
    println!("   Cache size: {}\n", cache.len().await);
    
    // Demonstrate async concurrent access
    println!("2. Concurrent async operations:");
    let cache_clone1 = cache.clone();
    let cache_clone2 = cache.clone();
    let cache_clone3 = cache.clone();
    
    // Spawn multiple async tasks
    let task1 = tokio::spawn(async move {
        for i in 10..15 {
            cache_clone1.put(i, format!("value-{}", i)).await;
            sleep(Duration::from_millis(10)).await;
        }
        println!("   Task 1 completed");
    });
    
    let task2 = tokio::spawn(async move {
        for i in 20..25 {
            cache_clone2.put(i, format!("value-{}", i)).await;
            sleep(Duration::from_millis(10)).await;
        }
        println!("   Task 2 completed");
    });
    
    let task3 = tokio::spawn(async move {
        sleep(Duration::from_millis(50)).await;
        for i in 10..25 {
            let _ = cache_clone3.get(&i).await;
        }
        println!("   Task 3 (reader) completed");
    });
    
    // Wait for all tasks
    let _ = tokio::join!(task1, task2, task3);
    
    println!("\n3. Final cache state:");
    println!("   Cache size: {}", cache.len().await);
    println!("   Cache capacity: {}", cache.capacity().await);
    
    // Demonstrate eviction
    println!("\n4. Demonstrating LRU eviction:");
    cache.clear().await;
    println!("   Cleared cache");
    
    for i in 1..=5 {
        cache.put(i, format!("value-{}", i)).await;
        println!("   Added key {}", i);
    }
    
    println!("\n   Cache is full (size: {})", cache.len().await);
    println!("   Adding key 6 will evict the LRU item (key 1)...");
    cache.put(6, "value-6".to_string()).await;
    
    println!("\n   After adding key 6:");
    println!("   Get key 1: {:?} (should be None)", cache.get(&1).await);
    println!("   Get key 6: {:?} (should be Some)", cache.get(&6).await);
    println!("   Cache size: {}", cache.len().await);
    
    // Demonstrate async web server simulation
    println!("\n5. Simulating async web server cache:");
    let server_cache = AsyncLruCache::new(100);
    
    // Simulate multiple concurrent requests
    let mut request_tasks = vec![];
    
    for request_id in 0..10 {
        let cache_clone = server_cache.clone();
        let task = tokio::spawn(async move {
            // Simulate fetching data (cache miss)
            if cache_clone.get(&request_id).await.is_none() {
                // Simulate slow database query
                sleep(Duration::from_millis(50)).await;
                let data = format!("data-for-request-{}", request_id);
                cache_clone.put(request_id, data.clone()).await;
                println!("   Request {}: Cache miss, fetched from DB", request_id);
                data
            } else {
                println!("   Request {}: Cache hit!", request_id);
                cache_clone.get(&request_id).await.unwrap()
            }
        });
        request_tasks.push(task);
    }
    
    // Wait for all requests
    for task in request_tasks {
        let _ = task.await;
    }
    
    println!("\nAsync cache example completed!");
}
