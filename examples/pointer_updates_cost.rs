// Demonstrating the pointer updates for each operation

use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Node {
    key: String,
    value: i32,
    prev: Option<usize>,
    next: Option<usize>,
}

struct LruCache {
    map: HashMap<String, usize>,
    nodes: Vec<Option<Node>>,
    head: Option<usize>,
    tail: Option<usize>,
    operation_count: usize,
}

impl LruCache {
    fn new() -> Self {
        LruCache {
            map: HashMap::new(),
            nodes: Vec::new(),
            head: None,
            tail: None,
            operation_count: 0,
        }
    }
    
    fn count_operation(&mut self, op: &str) {
        self.operation_count += 1;
        println!("  └─ Pointer update #{}: {}", self.operation_count, op);
    }
    
    fn move_to_front(&mut self, idx: usize) {
        if self.head == Some(idx) {
            println!("  ✓ Already at front, no updates needed");
            return;
        }
        
        println!("  Moving index {idx} to front:");
        self.operation_count = 0;
        
        // Extract indices
        let (prev_idx, next_idx) = if let Some(ref node) = self.nodes[idx] {
            (node.prev, node.next)
        } else {
            return;
        };
        
        // Update previous node
        if let Some(prev_idx) = prev_idx {
            if let Some(ref mut prev_node) = self.nodes[prev_idx] {
                prev_node.next = next_idx;
                self.count_operation(&format!("Node[{prev_idx}].next = {next_idx:?}"));
            }
        }
        
        // Update next node
        if let Some(next_idx) = next_idx {
            if let Some(ref mut next_node) = self.nodes[next_idx] {
                next_node.prev = prev_idx;
                self.count_operation(&format!("Node[{next_idx}].prev = {prev_idx:?}"));
            }
        } else {
            self.tail = prev_idx;
            self.count_operation(&format!("tail = {prev_idx:?}"));
        }
        
        // Update current node
        if let Some(ref mut node) = self.nodes[idx] {
            node.prev = None;
            node.next = self.head;
            self.count_operation(&format!("Node[{idx}].prev = None"));
            self.count_operation(&format!("Node[{}].next = {:?}", idx, self.head));
        }
        
        // Update old head
        if let Some(old_head_idx) = self.head {
            if let Some(ref mut old_head) = self.nodes[old_head_idx] {
                old_head.prev = Some(idx);
                self.count_operation(&format!("Node[{old_head_idx}].prev = Some({idx})"));
            }
        }
        
        self.head = Some(idx);
        self.count_operation(&format!("head = Some({idx})"));
        
        println!("  ✓ Total pointer updates: {}", self.operation_count);
    }
    
    fn put_simple(&mut self, key: String, value: i32) {
        println!("\n=== PUT('{key}', {value}) ===");
        self.operation_count = 0;
        
        if let Some(&idx) = self.map.get(&key) {
            println!("  Key exists, updating value and moving to front");
            if let Some(ref mut node) = self.nodes[idx] {
                node.value = value;
                self.count_operation("Update value");
            }
            self.move_to_front(idx);
            return;
        }
        
        println!("  New key, inserting at front:");
        let idx = self.nodes.len();
        
        let new_node = Node {
            key: key.clone(),
            value,
            prev: None,
            next: self.head,
        };
        
        self.count_operation(&format!("Create new node at index {idx}"));
        
        if let Some(old_head_idx) = self.head {
            if let Some(ref mut old_head) = self.nodes[old_head_idx] {
                old_head.prev = Some(idx);
                self.count_operation(&format!("Node[{old_head_idx}].prev = Some({idx})"));
            }
        }
        
        self.nodes.push(Some(new_node));
        self.map.insert(key, idx);
        self.head = Some(idx);
        self.count_operation(&format!("head = Some({idx})"));
        
        if self.tail.is_none() {
            self.tail = Some(idx);
            self.count_operation(&format!("tail = Some({idx})"));
        }
        
        println!("  ✓ Total operations: {}", self.operation_count);
    }
    
    fn get(&mut self, key: &str) {
        println!("\n=== GET('{key}') ===");
        
        if let Some(&idx) = self.map.get(key) {
            println!("  Found at index {idx}");
            self.move_to_front(idx);
        } else {
            println!("  Not found");
        }
    }
    
    fn display_chain(&self) {
        print!("  LRU: [MRU] ");
        let mut current = self.head;
        let mut chain = Vec::new();
        while let Some(idx) = current {
            if let Some(ref node) = self.nodes[idx] {
                chain.push(format!("{}:{}", node.key, idx));
                current = node.next;
            } else {
                break;
            }
        }
        println!("{} [LRU]", chain.join(" → "));
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║  Pointer Updates: The Trade-off for O(1) Performance        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    let mut cache = LruCache::new();
    
    println!("📌 KEY INSIGHT:");
    println!("   We update prev/next pointers on EVERY operation");
    println!("   BUT it's always a CONSTANT number of updates (O(1))");
    println!("   Compare to shifting Vec elements which is O(n)\n");
    
    // Insert first item
    cache.put_simple("alice".to_string(), 100);
    cache.display_chain();
    
    // Insert second item
    cache.put_simple("bob".to_string(), 200);
    cache.display_chain();
    
    // Insert third item
    cache.put_simple("carol".to_string(), 300);
    cache.display_chain();
    
    // Access middle item - requires pointer updates
    cache.get("bob");
    cache.display_chain();
    
    // Access it again - already at front
    cache.get("bob");
    cache.display_chain();
    
    // Update existing key
    cache.put_simple("alice".to_string(), 999);
    cache.display_chain();
    
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Analysis                                                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
    
    println!("✅ YES - We update pointers on every operation");
    println!("✅ BUT - It's always ≤ 7 pointer updates (constant!)");
    println!("\n📊 Breakdown of operations:");
    println!("   • GET (item at front):    0 updates");
    println!("   • GET (item elsewhere):   ~6 updates");
    println!("   • PUT (new):              ~3 updates");
    println!("   • PUT (update + move):    ~7 updates");
    println!("\n🎯 Why this is O(1):");
    println!("   • Number of updates doesn't depend on cache size");
    println!("   • 6 pointer updates for 10 items = same as 1000 items");
    println!("   • Compare to Vec.remove(): needs to shift n elements");
    println!("\n💡 The trade-off:");
    println!("   • We pay: Small constant overhead (pointer updates)");
    println!("   • We get: O(1) time regardless of cache size");
    println!("   • Alternative: O(n) time that grows with cache size");
}
