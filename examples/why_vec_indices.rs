// This file demonstrates WHY we need Vec with indices
// instead of a traditional pointer-based doubly-linked list

use std::collections::HashMap;

// ============================================================================
// ATTEMPT 1: Traditional Box-based linked list (FAILS)
// ============================================================================

// #[allow(dead_code)]
// struct Node<K, V> {
//     key: K,
//     value: V,
//     prev: Option<Box<Node<K, V>>>,
//     next: Option<Box<Node<K, V>>>,
// }
//
// struct LruCache<K, V> {
//     map: HashMap<K, ???>,  // ❌ PROBLEM: What type goes here?
//     head: Option<Box<Node<K, V>>>,
// }
//
// Why this fails:
// - Box has unique ownership
// - If `next` owns the node with Box, the HashMap can't also point to it
// - We'd need the HashMap to store a reference, but references have lifetimes
// - The reference would be invalid as soon as we modify the list

// ============================================================================
// ATTEMPT 2: Using Rc + RefCell (NOT THREAD-SAFE)
// ============================================================================

use std::rc::Rc;
use std::cell::RefCell;

#[allow(dead_code)]
struct NodeRc<K, V> {
    key: K,
    value: V,
    prev: Option<Rc<RefCell<NodeRc<K, V>>>>,
    next: Option<Rc<RefCell<NodeRc<K, V>>>>,
}

// This actually compiles!
#[allow(dead_code)]
struct LruCacheRc<K, V> {
    map: HashMap<K, Rc<RefCell<NodeRc<K, V>>>>,
    head: Option<Rc<RefCell<NodeRc<K, V>>>>,
}

// ❌ BUT: Rc is not Send, so this can't be shared across threads!
// Uncommenting this shows the error:
// fn check_thread_safety() {
//     let cache: LruCacheRc<i32, String> = LruCacheRc {
//         map: HashMap::new(),
//         head: None,
//     };
//     
//     std::thread::spawn(move || {
//         // ERROR: `Rc<RefCell<NodeRc<i32, String>>>` cannot be sent between threads safely
//         drop(cache);
//     });
// }

// ============================================================================
// ATTEMPT 3: Our Vec-based solution (WORKS!)
// ============================================================================

#[allow(dead_code)]
struct NodeVec<K, V> {
    key: K,
    value: V,
    prev: Option<usize>,  // ✅ Just an index - copyable, no ownership issues
    next: Option<usize>,  // ✅ Just an index
}

#[allow(dead_code)]
struct LruCacheVec<K, V> {
    map: HashMap<K, usize>,           // ✅ usize is Copy - no ownership issues
    nodes: Vec<Option<NodeVec<K, V>>>, // ✅ Vec owns all nodes
    head: Option<usize>,
    tail: Option<usize>,
}

// ✅ This IS thread-safe! (with Mutex wrapper)
// The key insight: usize is Copy, so HashMap can store indices without
// worrying about ownership or borrowing

fn demonstrate_the_problem() {
    println!("Why Vec with Indices?");
    println!("====================\n");
    
    println!("The Core Problem:");
    println!("----------------");
    println!("We need TWO things to reference the same nodes:");
    println!("  1. HashMap: For O(1) key lookup");
    println!("  2. Linked List: For O(1) reordering\n");
    
    println!("Traditional Approach (Box):");
    println!("  ❌ Box has unique ownership");
    println!("  ❌ Can't have list AND map both 'own' the node\n");
    
    println!("Shared Ownership (Rc + RefCell):");
    println!("  ❌ Rc is not Send (can't use across threads)");
    println!("  ❌ RefCell has runtime overhead");
    println!("  ❌ Easy to panic with borrow errors\n");
    
    println!("Vec with Indices:");
    println!("  ✅ Vec owns all nodes");
    println!("  ✅ Indices are just numbers (Copy)");
    println!("  ✅ HashMap stores indices (no ownership issues)");
    println!("  ✅ Indices are stable (won't change)");
    println!("  ✅ Thread-safe with Mutex wrapper");
    println!("  ✅ Cache-friendly (contiguous memory)\n");
    
    println!("Example:");
    println!("--------");
    println!("HashMap: {{ 'key1': 0, 'key2': 1, 'key3': 2 }}");
    println!("Vec:     [Node@0 ← → Node@1 ← → Node@2]");
    println!("         Index 0   Index 1   Index 2");
    println!("\nNode@1.prev = Some(0)  // Points to index 0");
    println!("Node@1.next = Some(2)  // Points to index 2");
}

fn main() {
    demonstrate_the_problem();
}
