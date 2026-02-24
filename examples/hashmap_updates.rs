// Demonstrating how HashMap gets updated during PUT and GET operations

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
}

impl LruCache {
    fn new() -> Self {
        LruCache {
            map: HashMap::new(),
            nodes: Vec::new(),
            head: None,
            tail: None,
        }
    }
    
    fn display_state(&self, operation: &str) {
        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("After: {operation}");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        println!("\n📋 HashMap (Key → Vec Index):");
        if self.map.is_empty() {
            println!("   (empty)");
        } else {
            let mut entries: Vec<_> = self.map.iter().collect();
            entries.sort_by_key(|(k, _)| k.to_string());
            for (key, idx) in entries {
                println!("   '{key}' → {idx}");
            }
        }
        
        println!("\n📦 Vec Storage:");
        for (idx, node_opt) in self.nodes.iter().enumerate() {
            match node_opt {
                Some(node) => {
                    println!("   [{}] key='{}', value={}, prev={:?}, next={:?}",
                             idx, node.key, node.value, node.prev, node.next);
                }
                None => {
                    println!("   [{idx}] (freed)");
                }
            }
        }
        
        println!("\n🔗 LRU Chain:");
        print!("   head({:?}) → ", self.head);
        let mut current = self.head;
        let mut chain = Vec::new();
        while let Some(idx) = current {
            if let Some(ref node) = self.nodes[idx] {
                chain.push(format!("{}[{}]", node.key, idx));
                current = node.next;
            } else {
                break;
            }
        }
        println!("{} ← tail({:?})", chain.join(" → "), self.tail);
    }
    
    fn put(&mut self, key: String, value: i32) {
        println!("\n\n╔════════════════════════════════════════════════════╗");
        println!("║  PUT('{key}', {value})                                    ");
        println!("╚════════════════════════════════════════════════════╝");
        
        // Check if key already exists in HashMap
        if let Some(&idx) = self.map.get(&key) {
            println!("\n✓ Key '{key}' found in HashMap at index {idx}");
            println!("  → HashMap entry UNCHANGED: '{key}' → {idx}");
            println!("  → Only updating the value in nodes[{idx}]");
            
            if let Some(ref mut node) = self.nodes[idx] {
                println!("    Before: nodes[{}].value = {}", idx, node.value);
                node.value = value;
                println!("    After:  nodes[{idx}].value = {value}");
            }
            
            println!("\n  → Moving node to front (updating prev/next pointers)");
            self.move_to_front(idx);
            
            self.display_state(&format!("PUT('{key}', {value}) - UPDATED EXISTING"));
            return;
        }
        
        // Key doesn't exist - insert new entry
        println!("\n✗ Key '{key}' NOT in HashMap - inserting new");
        
        let idx = self.nodes.len();
        println!("  → New node will be at Vec index {idx}");
        
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
        
        println!("  → Inserting into HashMap: '{key}' → {idx}");
        self.map.insert(key.clone(), idx);
        println!("    HashMap.insert('{key}', {idx})");
        println!("    HashMap size is now: {}", self.map.len());
        
        self.head = Some(idx);
        
        if self.tail.is_none() {
            self.tail = Some(idx);
        }
        
        self.display_state(&format!("PUT('{key}', {value}) - NEW ENTRY"));
    }
    
    fn get(&mut self, key: &str) -> Option<i32> {
        println!("\n\n╔════════════════════════════════════════════════════╗");
        println!("║  GET('{key}')                                        ");
        println!("╚════════════════════════════════════════════════════╝");
        
        if let Some(&idx) = self.map.get(key) {
            println!("\n✓ Key '{key}' found in HashMap at index {idx}");
            println!("  → HashMap entry UNCHANGED: '{key}' → {idx}");
            println!("  → Retrieved nodes[{idx}].value");
            
            let value = self.nodes[idx].as_ref().map(|n| n.value);
            
            println!("  → Moving node to front (updating prev/next pointers only)");
            self.move_to_front(idx);
            
            self.display_state(&format!("GET('{key}') - FOUND"));
            return value;
        }
        
        println!("\n✗ Key '{key}' NOT in HashMap");
        println!("  → HashMap UNCHANGED");
        println!("  → Returning None");
        
        self.display_state(&format!("GET('{key}') - NOT FOUND"));
        None
    }
    
    fn move_to_front(&mut self, idx: usize) {
        if self.head == Some(idx) {
            println!("    Already at head, no pointer updates needed");
            return;
        }
        
        // Extract indices
        let (prev_idx, next_idx) = if let Some(ref node) = self.nodes[idx] {
            (node.prev, node.next)
        } else {
            return;
        };
        
        // Update pointers (Vec indices stay the same!)
        if let Some(prev_idx) = prev_idx {
            if let Some(ref mut prev_node) = self.nodes[prev_idx] {
                prev_node.next = next_idx;
            }
        }
        
        if let Some(next_idx) = next_idx {
            if let Some(ref mut next_node) = self.nodes[next_idx] {
                next_node.prev = prev_idx;
            }
        } else {
            self.tail = prev_idx;
        }
        
        if let Some(ref mut node) = self.nodes[idx] {
            node.prev = None;
            node.next = self.head;
        }
        
        if let Some(old_head_idx) = self.head {
            if let Some(ref mut old_head) = self.nodes[old_head_idx] {
                old_head.prev = Some(idx);
            }
        }
        
        self.head = Some(idx);
    }
    
    fn evict_lru(&mut self) {
        println!("\n\n╔════════════════════════════════════════════════════╗");
        println!("║  EVICT LRU                                         ║");
        println!("╚════════════════════════════════════════════════════╝");
        
        if let Some(tail_idx) = self.tail {
            if let Some(tail_node) = self.nodes[tail_idx].as_ref() {
                let key = tail_node.key.clone();
                
                println!("\n→ Evicting least recently used: '{key}' at index {tail_idx}");
                println!("  → Removing from HashMap: '{key}' → {tail_idx}");
                
                self.map.remove(&key);
                println!("    HashMap.remove('{key}')");
                println!("    HashMap size is now: {}", self.map.len());
                
                println!("  → Setting nodes[{tail_idx}] = None (freeing slot)");
            }
            
            self.tail = self.nodes[tail_idx].as_ref().and_then(|n| n.prev);
            
            if let Some(new_tail_idx) = self.tail {
                if let Some(ref mut new_tail) = self.nodes[new_tail_idx] {
                    new_tail.next = None;
                }
            } else {
                self.head = None;
            }
            
            self.nodes[tail_idx] = None;
            
            self.display_state("EVICT LRU");
        }
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     How HashMap Gets Updated During Operations              ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    let mut cache = LruCache::new();
    
    println!("\n📌 KEY POINTS:");
    println!("   1. HashMap stores: Key → Vec Index");
    println!("   2. HashMap ONLY changes when:");
    println!("      - INSERT new key (HashMap.insert)");
    println!("      - EVICT old key (HashMap.remove)");
    println!("   3. HashMap NEVER changes when:");
    println!("      - UPDATE existing value");
    println!("      - GET/access existing item");
    println!("      - Moving nodes in LRU order");
    println!("   4. Vec indices NEVER change once assigned!");
    
    cache.display_state("Initial state");
    
    // Scenario 1: Add new item
    cache.put("alice".to_string(), 100);
    
    // Scenario 2: Add another new item
    cache.put("bob".to_string(), 200);
    
    // Scenario 3: Add third new item
    cache.put("carol".to_string(), 300);
    
    // Scenario 4: GET existing item (HashMap unchanged!)
    cache.get("alice");
    
    // Scenario 5: UPDATE existing item (HashMap unchanged!)
    cache.put("bob".to_string(), 999);
    
    // Scenario 6: Evict LRU (HashMap removes entry)
    cache.evict_lru();
    
    println!("\n\n╔══════════════════════════════════════════════════════════════╗");
    println!("║  Summary: HashMap Operations                                ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    
    println!("\n✅ HashMap.insert() called when:");
    println!("   • PUT with NEW key");
    println!("   • Creates mapping: key → Vec index");
    println!("\n✅ HashMap.remove() called when:");
    println!("   • EVICT (capacity exceeded)");
    println!("   • Deletes mapping for evicted key");
    println!("\n❌ HashMap UNCHANGED when:");
    println!("   • PUT with EXISTING key (just update nodes[idx].value)");
    println!("   • GET (just lookup index, update prev/next pointers)");
    println!("   • Moving nodes in LRU order (only prev/next change)");
    println!("\n💡 The Magic:");
    println!("   Vec indices are PERMANENT - once alice→2, always alice→2");
    println!("   Only the prev/next pointers change to track LRU order");
    println!("   This keeps HashMap stable and operations O(1)!");
}
