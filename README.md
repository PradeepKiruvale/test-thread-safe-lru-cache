# Thread-Safe LRU Cache

## Summary

You have been tasked with implementing a **thread-safe Least Recently Used (LRU)
cache** in Rust.

The cache will be accessed concurrently by multiple threads and must maintain
correct eviction behavior while providing efficient lookups and updates.

The goal is to design a cache that is **correct under concurrency**, has
**predictable memory usage**, and demonstrates sound engineering trade-offs.

## Requirements

### Core Functionality

* Fixed maximum capacity
* Store key–value pairs
* `get(key) -> Option<V>`
* `put(key, value)`
* Automatically evict the **least recently used** item when the cache exceeds
  its capacity

## Concurrency Requirements

* Must be safe for concurrent access by multiple threads
* Must support multiple readers and writers
* Must not deadlock
* Must maintain correctness under contention

## Performance Expectations

* Average **O(1)** time complexity for `get` and `put`
* Avoid unnecessary locking where possible
* Memory usage must remain bounded by the configured capacity

## Constraints

* Use only safe Rust
* Do not use global mutable state
* Blocking for extended periods is discouraged
* The cache must behave correctly under high contention

## Additional Requirements

* Your source should contain both unit and concurrency tests
* Tests should validate eviction behavior and thread safety
* All code must be formatted using the standard formatting tool
* Code must compile without clippy errors

## Design & Reasoning (Required)

Along with the code, include a document (for example `DESIGN.md`) explaining:

* Data structures used to implement the cache
* Synchronization strategy (e.g. mutexes, read-write locks)
* How LRU ordering is maintained under concurrency
* Trade-offs between simplicity, performance, and scalability
* Known limitations of the solution

Submissions without a design explanation will not be reviewed.

## Submission

Please fork this repository to your own GitHub account and submit a pull request
to your own repository.

Your pull request should include:

* A clear description of your approach
* Any assumptions or trade-offs made
* Instructions on how to run tests

A link to the pull request can be submitted once it is ready for review.

## Bonus

* Async-compatible version of the cache
* Configurable eviction policies
* Performance comparison of different locking strategies
* Sharded or lock-minimized implementation

## 📚 Documentation

**All features above have been implemented!**

For comprehensive documentation, see **[DOCUMENTATION_INDEX.md](DOCUMENTATION_INDEX.md)** which provides:

- **Getting Started**: Quick start guide and basic usage
- **Core Design**: Architecture, synchronization strategy, and design decisions
- **Advanced Features**: Async cache, eviction policies, sharded cache
- **Performance Guide**: Benchmarks, comparisons, and optimization tips
- **Examples**: Runnable code examples for all features

### Quick Links

- **Design Document**: [docs/DESIGN.md](docs/DESIGN.md) - Core architecture and thread safety
- **Implementation Analysis**: [docs/reasoning.md](docs/reasoning.md) - Design reasoning and trade-offs
- **Advanced Features**: [docs/ADVANCED_FEATURES.md](docs/ADVANCED_FEATURES.md) - Complete feature guide
- **Performance Guide**: [docs/PERFORMANCE_GUIDE.md](docs/PERFORMANCE_GUIDE.md) - Benchmarks and tuning

### Running Examples

```bash
# Basic usage
cargo run --example basic_usage

# Async cache (requires tokio)
cargo run --example async_cache_example --features async

# Eviction policies comparison
cargo run --example eviction_policies_example

# Sharded cache demonstration
cargo run --example sharded_cache_example

# Performance benchmarks
cargo run --example performance_comparison --release
```
