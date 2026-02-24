// Demonstrating why we need BOTH Vec indices AND prev/next pointers

use std::collections::HashMap;

#[derive(Debug)]
struct Node {
    key: String,
    value: i32,
    prev: Option<usize>,
    next: Option<usize>,
}

struct SimpleLruCache {
    map: HashMap<String, usize>,
    nodes: Vec<Option<Node>>,
    head: Option<usize>,
    tail: Option<usize>,
}

impl SimpleLruCache {
    fn new() -> Self {
        SimpleLruCache {
            map: HashMap::new(),
            nodes: Vec::new(),
            head: None,
            tail: None,
        }
    }
    
    fn display_state(&self) {
        println!("\n=== Cache State ===");
        
        println!("\nPhysical Storage (Vec):");
        for (idx, node_opt) in self.nodes.iter().enumerate() {
            match node_opt {
                Some(node) => {
                    println!("  Index {}: key='{}', value={}, prev={:?}, next={:?}",
                             idx, node.key, node.value, node.prev, node.next);
                }
                None => {
                    println!("  Index {idx}: (empty/freed)");
                }
            }
        }
        
        println!("\nHashMap (Key → Vec Index):");
        for (key, idx) in &self.map {
            println!("  '{key}'  → {idx}");
        }
        
        println!("\nLRU Order (following prev/next chain):");
        print!("  [Most Recent] ");
        let mut current = self.head;
        let mut order = Vec::new();
        while let Some(idx) = current {
            if let Some(ref node) = self.nodes[idx] {
                order.push(format!("{}:{}", node.key, idx));
                current = node.next;
            } else {
                break;
            }
        }
        println!("{}", order.join(" → "));
        println!("  [Least Recent]\n");
    }
    
    fn insert_simple(&mut self, key: String, value: i32) {
        let idx = self.nodes.len();
        
        let new_node = Node {
            key: key.clone(),
            value,
            prev: None,
            next: self.head,
        };
        
        if let Some(old_head_idx) = self.head {
            if let Some(ref mut old_head) = self.nodes[old_head_idx] {
                old_head.prev = Some(idx);
            }
        }
        
        self.nodes.push(Some(new_node));
        self.map.insert(key, idx);
        self.head = Some(idx);
        
        if self.tail.is_none() {
            self.tail = Some(idx);
        }
    }
}

fn main() {
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  Why We Need BOTH Vec Indices AND prev/next Pointers          ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    let mut cache = SimpleLruCache::new();
    
    println!("\n📝 The Vec stores nodes at FIXED locations (indices 0, 1, 2, ...)");
    println!("📝 The prev/next pointers create a LOGICAL ORDER (LRU chain)");
    println!("📝 These are DIFFERENT things!\n");
    
    println!("──────────────────────────────────────────────────────────────");
    println!("Step 1: Insert 'alice' with value 100");
    println!("──────────────────────────────────────────────────────────────");
    cache.insert_simple("alice".to_string(), 100);
    cache.display_state();
    
    println!("📌 Notice: Alice is at Vec Index 0");
    println!("📌 Alice is also at the HEAD of LRU order");
    println!("📌 Vec position = LRU position (for now)");
    
    println!("\n──────────────────────────────────────────────────────────────");
    println!("Step 2: Insert 'bob' with value 200");
    println!("──────────────────────────────────────────────────────────────");
    cache.insert_simple("bob".to_string(), 200);
    cache.display_state();
    
    println!("📌 Bob is at Vec Index 1");
    println!("📌 But Bob is at HEAD of LRU order (most recent)");
    println!("📌 Alice moved to 2nd in LRU order (just by updating pointers!)");
    println!("📌 Alice is STILL at Vec Index 0 (didn't move in memory)");
    
    println!("\n──────────────────────────────────────────────────────────────");
    println!("Step 3: Insert 'carol' with value 300");
    println!("──────────────────────────────────────────────────────────────");
    cache.insert_simple("carol".to_string(), 300);
    cache.display_state();
    
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  KEY OBSERVATION:                                              ║");
    println!("║                                                                ║");
    println!("║  Vec Order:  alice(0), bob(1), carol(2)                       ║");
    println!("║  LRU Order:  carol(2) → bob(1) → alice(0)                     ║");
    println!("║                                                                ║");
    println!("║  THEY'RE DIFFERENT! That's why we need prev/next pointers.    ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    
    println!("\n\n📖 Explanation:");
    println!("───────────────");
    println!("• Vec indices tell us WHERE each node is stored in memory");
    println!("• prev/next pointers tell us the ORDER for LRU eviction");
    println!("• When we access a node, we only update prev/next (O(1))");
    println!("• We DON'T move the node in the Vec (that would be O(n))");
    println!("• HashMap stores Vec index, which never changes");
    println!("\n✨ This separation gives us O(1) operations!");
    
    println!("\n\n❌ What if we ONLY used Vec indices (no prev/next)?");
    println!("──────────────────────────────────────────────────────────");
    println!("• To move bob to front, we'd need to:");
    println!("  1. Remove bob from Vec[1]         → shifts all elements (O(n))");
    println!("  2. Insert bob at Vec[0]           → shifts all elements (O(n))");
    println!("  3. Update ALL HashMap entries     → every index changed (O(n))");
    println!("• Result: O(n) instead of O(1) ❌");
    
    println!("\n\n✅ With prev/next pointers:");
    println!("──────────────────────────────────────────────────────────");
    println!("• To move bob to front, we only:");
    println!("  1. Update bob's prev/next         → O(1)");
    println!("  2. Update neighbors' pointers     → O(1)");
    println!("  3. Update head pointer            → O(1)");
    println!("  4. HashMap entries unchanged      → O(1)");
    println!("• Result: O(1) ✅");
}
