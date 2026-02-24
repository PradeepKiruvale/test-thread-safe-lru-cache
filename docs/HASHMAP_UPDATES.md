# How HashMap Gets Updated

## The Key Principle

**HashMap stores**: `Key → Vec Index`

**Vec indices NEVER change** once assigned! This is what makes everything work.

## HashMap Operations Summary

### ✅ HashMap.insert() - ONLY Called When:

**1. PUT with a NEW key**
```rust
cache.put("alice", 100);  // alice not in cache

// What happens:
let idx = nodes.len();           // idx = 0 (new index)
nodes.push(new_node);            // Add to Vec at index 0
map.insert("alice", idx);        // HashMap: "alice" → 0
                                 // ✅ HashMap CHANGED (insert)
```

**Result**: HashMap size increases by 1

### ✅ HashMap.remove() - ONLY Called When:

**2. EVICT (capacity exceeded)**
```rust
// Cache is full, need to evict LRU
evict_lru();

// What happens:
let tail_key = nodes[tail_idx].key;  // Get key of LRU node
map.remove(&tail_key);               // HashMap: remove "carol"
nodes[tail_idx] = None;              // Free the Vec slot
                                     // ✅ HashMap CHANGED (remove)
```

**Result**: HashMap size decreases by 1

### ❌ HashMap UNCHANGED When:

**3. PUT with EXISTING key (update)**
```rust
cache.put("bob", 200);   // bob already at index 1
cache.put("bob", 999);   // Update bob's value

// What happens:
let idx = map.get("bob");            // idx = 1 (already exists)
                                     // ❌ HashMap UNCHANGED
nodes[idx].value = 999;              // Only update value in Vec
move_to_front(idx);                  // Only update prev/next pointers
```

**Result**: HashMap stays exactly the same!
- Still has: `"bob" → 1`
- Only `nodes[1].value` changed: `200 → 999`
- Only `prev/next` pointers changed (not Vec index)

**4. GET (access existing item)**
```rust
cache.get("alice");      // alice at index 0

// What happens:
let idx = map.get("alice");          // idx = 0 (lookup)
                                     // ❌ HashMap UNCHANGED
let value = nodes[idx].value;        // Get value from Vec
move_to_front(idx);                  // Update prev/next pointers only
```

**Result**: HashMap stays exactly the same!
- Still has: `"alice" → 0`
- Only `prev/next` pointers changed (not Vec index)

## Visual Example

```
Operation sequence:

1. PUT("alice", 100)  → NEW KEY
   HashMap: { "alice" → 0 }
   Vec: [alice@0]
   ✅ HashMap.insert("alice", 0)

2. PUT("bob", 200)  → NEW KEY
   HashMap: { "alice" → 0, "bob" → 1 }
   Vec: [alice@0, bob@1]
   ✅ HashMap.insert("bob", 1)

3. PUT("carol", 300)  → NEW KEY
   HashMap: { "alice" → 0, "bob" → 1, "carol" → 2 }
   Vec: [alice@0, bob@1, carol@2]
   ✅ HashMap.insert("carol", 2)

4. GET("alice")  → ACCESS EXISTING
   HashMap: { "alice" → 0, "bob" → 1, "carol" → 2 }  ← UNCHANGED!
   Vec: [alice@0, bob@1, carol@2]  ← indices same, only pointers changed
   ❌ No HashMap operation

5. PUT("bob", 999)  → UPDATE EXISTING
   HashMap: { "alice" → 0, "bob" → 1, "carol" → 2 }  ← UNCHANGED!
   Vec: [alice@0, bob@1(value→999), carol@2]  ← bob still at index 1
   ❌ No HashMap operation

6. EVICT  → REMOVE LRU
   HashMap: { "alice" → 0, "bob" → 1 }  ← "carol" removed
   Vec: [alice@0, bob@1, None@2]
   ✅ HashMap.remove("carol")
```

## The Code Flow

### Scenario 1: PUT new key

```rust
fn put(&mut self, key: K, value: V) {
    if let Some(&idx) = self.map.get(&key) {
        // Key exists - skip to else branch
    } else {
        // NEW KEY path ✅
        let idx = self.nodes.len();
        self.nodes.push(new_node);
        self.map.insert(key, idx);  // ← HashMap.insert() HERE
    }
}
```

### Scenario 2: PUT existing key (update)

```rust
fn put(&mut self, key: K, value: V) {
    if let Some(&idx) = self.map.get(&key) {
        // EXISTING KEY path ❌
        self.nodes[idx].value = value;  // Just update value
        self.move_to_front(idx);        // Just update pointers
        return;  // ← NO HashMap operation!
    } else {
        // New key branch
    }
}
```

### Scenario 3: GET

```rust
fn get(&self, key: &K) -> Option<V> {
    if let Some(&idx) = self.map.get(key) {  // ← HashMap.get() (lookup only)
        self.move_to_front(idx);              // Update pointers
        return Some(self.nodes[idx].value);   // Return value
        // ← NO HashMap insert/remove!
    }
    None
}
```

### Scenario 4: EVICT

```rust
fn evict_lru(&mut self) {
    let tail_idx = self.tail.unwrap();
    let key = self.nodes[tail_idx].key.clone();
    
    self.map.remove(&key);      // ← HashMap.remove() HERE
    self.nodes[tail_idx] = None;
}
```

## Why This Design Works

### The Magic: Stable Indices

```
HashMap:  "alice" → 0
          "bob"   → 1
          "carol" → 2

These indices (0, 1, 2) NEVER change!
```

When we reorder the LRU chain:
- We only change `prev` and `next` fields in nodes
- The nodes stay at their Vec indices
- HashMap entries remain valid

```
Before GET("alice"):
  HashMap: "alice" → 0    ← Index still 0
  nodes[0]: { prev: Some(1), next: Some(2) }

After GET("alice"):
  HashMap: "alice" → 0    ← Index STILL 0 (unchanged!)
  nodes[0]: { prev: None, next: Some(1) }  ← Only pointers changed
```

## Performance Impact

| Operation | HashMap Operation | Time |
|-----------|-------------------|------|
| PUT (new) | insert() | O(1) |
| PUT (update) | get() only | O(1) |
| GET | get() only | O(1) |
| EVICT | remove() | O(1) |

**All HashMap operations are O(1)** because:
1. HashMap lookups are O(1)
2. HashMap inserts are O(1) amortized
3. HashMap removes are O(1)
4. We rarely insert/remove (only on cache misses and evictions)
5. We frequently get/update (just lookups, very fast)

## Common Misconception

❌ **Wrong**: "Every time we move a node to front, we update the HashMap"

✅ **Right**: "Moving to front only updates prev/next pointers. HashMap is unchanged!"

The HashMap stores **where** the node is (Vec index).
The prev/next pointers store **what order** nodes are in (LRU chain).

**WHERE never changes. Only ORDER changes.**

## Memory View

```
HashMap (on heap):
  ┌─────────────┬───────┐
  │ "alice"     │   0   │  ← Points to Vec index
  │ "bob"       │   1   │  ← Points to Vec index
  │ "carol"     │   2   │  ← Points to Vec index
  └─────────────┴───────┘
         ↓ lookup
Vec (on heap):
  ┌───┬─────────────────────────────────┐
  │ 0 │ Node { key:"alice", prev, next }│  ← Fixed location
  │ 1 │ Node { key:"bob", prev, next }  │  ← Fixed location
  │ 2 │ Node { key:"carol", prev, next }│  ← Fixed location
  └───┴─────────────────────────────────┘
        Only prev/next change ↑
```

## Summary

**HashMap changes ONLY when:**
- Adding a new key (insert)
- Evicting a key (remove)

**HashMap NEVER changes when:**
- Updating existing value
- Getting existing value
- Reordering LRU chain

**This stability is crucial for O(1) performance!**
