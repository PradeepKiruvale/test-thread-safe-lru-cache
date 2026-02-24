# Documentation Index

This document provides a comprehensive index of all documentation for the Thread-Safe LRU Cache project.

## 📖 Overview

This project implements a high-performance, thread-safe LRU (Least Recently Used) cache in Rust with multiple caching strategies, async support, and advanced features for concurrent applications.

---

## 🚀 Getting Started

### Quick Start
- **[README.md](README.md)** - Project overview, requirements, and constraints
- **[docs/QUICKSTART.md](docs/QUICKSTART.md)** - Quick start guide for basic usage
- **[docs/README.md](docs/README.md)** - Documentation directory overview

### Delivery & Submission
- **[docs/DELIVERY.md](docs/DELIVERY.md)** - Delivery checklist and submission guidelines

---

## 🏗️ Core Design & Implementation

### Architecture & Design Decisions
- **[docs/DESIGN.md](docs/DESIGN.md)** - Comprehensive design document covering:
  - Architecture overview
  - Synchronization strategy (single Mutex approach)
  - Data structures (HashMap + Vec-based doubly-linked list)
  - Thread safety proofs
  - Alternative approaches and why they were rejected
  - Performance characteristics

- **[docs/reasoning.md](docs/reasoning.md)** - Implementation reasoning and analysis:
  - Data structures used to implement the cache
  - Synchronization strategy rationale
  - How LRU ordering is maintained under concurrency
  - Trade-offs between simplicity, performance, and scalability
  - Known limitations of the solution

### Implementation Details
- **[docs/ALTERNATIVES.md](docs/ALTERNATIVES.md)** - Alternative implementation approaches considered
- **[docs/WHY_PREV_NEXT.md](docs/WHY_PREV_NEXT.md)** - Why use index-based pointers instead of raw pointers
- **[docs/HASHMAP_UPDATES.md](docs/HASHMAP_UPDATES.md)** - HashMap update strategies and performance
- **[docs/POINTER_UPDATES.md](docs/POINTER_UPDATES.md)** - Cost analysis of pointer updates

---

## ⚡ Advanced Features

### Feature Overview
- **[docs/FEATURES_SUMMARY.md](docs/FEATURES_SUMMARY.md)** - Complete summary of all features:
  - Async-compatible cache (`AsyncLruCache`)
  - Configurable eviction policies (LRU, LFU, FIFO, Random)
  - Sharded cache for improved concurrency (`ShardedLruCache`)
  - Performance benchmarks and comparisons
  - Usage examples and decision trees

### Comprehensive Guide
- **[docs/ADVANCED_FEATURES.md](docs/ADVANCED_FEATURES.md)** - In-depth guide covering:
  - **Async-Compatible Cache**: Non-blocking cache for Tokio applications
  - **Configurable Eviction Policies**: LRU, LFU, FIFO, Random, and custom policies
  - **Sharded Cache**: Multi-shard cache for reduced lock contention
  - **Performance Comparison**: Detailed benchmarks and results
  - **Choosing the Right Implementation**: Decision tree and feature matrix
  - **Best Practices**: Capacity planning, monitoring, profiling

### Performance & Optimization
- **[docs/PERFORMANCE_GUIDE.md](docs/PERFORMANCE_GUIDE.md)** - Comprehensive performance guide:
  - **Quick Comparison Table**: All implementations side-by-side
  - **Locking Strategy Comparison**: Single Mutex vs Sharded approaches
  - **Performance Benchmarks**: Thread count scaling analysis
  - **Hit Rate Analysis**: Cache effectiveness metrics
  - **Latency Percentiles**: Tail latency comparisons
  - **Scalability Analysis**: Efficiency across thread counts
  - **Memory Usage Comparison**: Overhead analysis
  - **Choosing Optimal Shard Count**: Guidelines and diminishing returns
  - **Workload-Specific Recommendations**: Read-heavy, write-heavy, mixed
  - **Migration Guide**: How to switch between implementations
  - **Profiling Your Application**: A/B testing and measurement

---

## 📚 Examples & Usage

### Code Examples

All examples are located in the `examples/` directory:

#### Basic Usage
- **[examples/basic_usage.rs](examples/basic_usage.rs)** - Simple LRU cache usage example
- **[examples/concurrent_stress_test.rs](examples/concurrent_stress_test.rs)** - Multi-threaded stress testing

#### Advanced Features
- **[examples/async_cache_example.rs](examples/async_cache_example.rs)** - Async cache with Tokio runtime
  ```bash
  cargo run --example async_cache_example --features async
  ```

- **[examples/eviction_policies_example.rs](examples/eviction_policies_example.rs)** - LRU, LFU, FIFO comparison
  ```bash
  cargo run --example eviction_policies_example
  ```

- **[examples/sharded_cache_example.rs](examples/sharded_cache_example.rs)** - Sharded cache demonstration
  ```bash
  cargo run --example sharded_cache_example
  ```

- **[examples/performance_comparison.rs](examples/performance_comparison.rs)** - Comprehensive benchmarks
  ```bash
  cargo run --example performance_comparison --release
  ```

#### Implementation Details
- **[examples/hashmap_updates.rs](examples/hashmap_updates.rs)** - HashMap update patterns
- **[examples/pointer_updates_cost.rs](examples/pointer_updates_cost.rs)** - Pointer manipulation costs
- **[examples/vec_vs_lru_order.rs](examples/vec_vs_lru_order.rs)** - Vec vs LRU ordering comparison
- **[examples/why_vec_indices.rs](examples/why_vec_indices.rs)** - Why use indices instead of pointers

---

## 🗂️ Source Code

### Core Implementation
- **[src/lib.rs](src/lib.rs)** - Main library with standard `LruCache` implementation
- **[src/async_cache.rs](src/async_cache.rs)** - Async-compatible `AsyncLruCache` (requires `async` feature)
- **[src/eviction.rs](src/eviction.rs)** - Eviction policies (LRU, LFU, FIFO, Random)
- **[src/sharded.rs](src/sharded.rs)** - Sharded cache implementation

---

## 🎯 Quick Reference

### Choose Your Implementation

```
Are you using async/await?
├─ Yes → AsyncLruCache (docs/ADVANCED_FEATURES.md#async-compatible-cache)
└─ No → Continue

Do you need >1M ops/sec with >4 threads?
├─ Yes → ShardedLruCache (docs/ADVANCED_FEATURES.md#sharded-cache)
└─ No → Continue

Do you need custom eviction logic?
├─ Yes → ThreadSafeEvictableCache (docs/ADVANCED_FEATURES.md#configurable-eviction-policies)
└─ No → Standard LruCache (docs/QUICKSTART.md)
```

### Performance at a Glance

| Implementation | Threads | Throughput | Use Case |
|----------------|---------|------------|----------|
| Standard LRU | 1-4 | 1.2M ops/s | General purpose |
| Async LRU | Any | 1.0M ops/s | Async runtimes |
| Sharded (8) | 8+ | 3.6M ops/s | High concurrency |
| LFU Policy | Any | 1.1M ops/s | Frequency patterns |
| FIFO Policy | Any | 1.3M ops/s | Simple eviction |

---

## 📊 Documentation Map

```
test-thread-safe-lru-cache/
├── README.md                          # Project overview
├── DOCUMENTATION_INDEX.md             # This file
│
├── docs/
│   ├── README.md                      # Documentation overview
│   ├── QUICKSTART.md                  # Getting started guide
│   ├── DELIVERY.md                    # Submission checklist
│   │
│   ├── DESIGN.md                      # Core design document ⭐
│   ├── reasoning.md                   # Implementation analysis ⭐
│   │
│   ├── ADVANCED_FEATURES.md           # Feature guide ⭐
│   ├── PERFORMANCE_GUIDE.md           # Performance guide ⭐
│   ├── FEATURES_SUMMARY.md            # Quick reference ⭐
│   │
│   ├── ALTERNATIVES.md                # Alternative approaches
│   ├── WHY_PREV_NEXT.md              # Index-based design
│   ├── HASHMAP_UPDATES.md            # HashMap patterns
│   └── POINTER_UPDATES.md            # Pointer costs
│
├── src/
│   ├── lib.rs                         # Standard LRU cache
│   ├── async_cache.rs                 # Async cache
│   ├── eviction.rs                    # Eviction policies
│   └── sharded.rs                     # Sharded cache
│
└── examples/
    ├── basic_usage.rs                 # Basic example
    ├── concurrent_stress_test.rs      # Stress testing
    ├── async_cache_example.rs         # Async demo
    ├── eviction_policies_example.rs   # Policies demo
    ├── sharded_cache_example.rs       # Sharded demo
    └── performance_comparison.rs      # Benchmarks
```

⭐ = Most important documents

---

## 🔍 Search by Topic

### Looking for...

**Thread Safety**
- [docs/DESIGN.md](docs/DESIGN.md) - Thread safety proofs
- [docs/reasoning.md](docs/reasoning.md) - Synchronization strategy

**Performance**
- [docs/PERFORMANCE_GUIDE.md](docs/PERFORMANCE_GUIDE.md) - Complete performance analysis
- [examples/performance_comparison.rs](examples/performance_comparison.rs) - Benchmark code

**Async Support**
- [docs/ADVANCED_FEATURES.md](docs/ADVANCED_FEATURES.md#async-compatible-cache) - Async feature guide
- [src/async_cache.rs](src/async_cache.rs) - Implementation
- [examples/async_cache_example.rs](examples/async_cache_example.rs) - Example

**High Concurrency**
- [docs/ADVANCED_FEATURES.md](docs/ADVANCED_FEATURES.md#sharded-cache) - Sharded cache guide
- [docs/PERFORMANCE_GUIDE.md](docs/PERFORMANCE_GUIDE.md#2-shardedpartitioned-sharded-lru) - Sharding strategy
- [src/sharded.rs](src/sharded.rs) - Implementation

**Eviction Policies**
- [docs/ADVANCED_FEATURES.md](docs/ADVANCED_FEATURES.md#configurable-eviction-policies) - Policy guide
- [src/eviction.rs](src/eviction.rs) - Implementation
- [examples/eviction_policies_example.rs](examples/eviction_policies_example.rs) - Example

**Design Decisions**
- [docs/DESIGN.md](docs/DESIGN.md) - Why single Mutex?
- [docs/ALTERNATIVES.md](docs/ALTERNATIVES.md) - Other approaches
- [docs/WHY_PREV_NEXT.md](docs/WHY_PREV_NEXT.md) - Why indices?

**Implementation Details**
- [docs/HASHMAP_UPDATES.md](docs/HASHMAP_UPDATES.md) - HashMap patterns
- [docs/POINTER_UPDATES.md](docs/POINTER_UPDATES.md) - Pointer manipulation

---

## 🛠️ Building & Testing

```bash
# Build the project
cargo build

# Build with async support
cargo build --features async

# Run tests
cargo test

# Run tests with async feature
cargo test --features async

# Run specific example
cargo run --example sharded_cache_example --release

# Run benchmarks
cargo run --example performance_comparison --release

# Format code
cargo fmt

# Check for issues
cargo clippy
```

---

## 📝 Contributing

When adding new features or documentation:

1. Update this index file
2. Add examples in `examples/` directory
3. Document in `docs/` with appropriate detail level
4. Update [docs/FEATURES_SUMMARY.md](docs/FEATURES_SUMMARY.md) if adding features
5. Add performance metrics to [docs/PERFORMANCE_GUIDE.md](docs/PERFORMANCE_GUIDE.md) if relevant

---

## 📄 License

See [LICENSE](LICENSE) file for details.

---

**Last Updated:** February 24, 2026

For questions or issues, refer to the specific documentation sections above.
