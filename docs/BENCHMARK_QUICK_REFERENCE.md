# Benchmark Interface Quick Reference

**For:** Developers implementing or using the canonical benchmark interface

---

## Core Concepts

### The Three-Layer Architecture

```
┌─────────────────────────────────────────────────────────┐
│ LAYER 1: CLI Interface                                   │
│  • User-facing command: cost-ops run                     │
│  • Handles argument parsing and output formatting        │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ LAYER 2: Benchmark Framework                            │
│  • BenchmarkResult: Standard result format              │
│  • BenchTarget trait: Common interface                  │
│  • Runner: Execution engine                             │
│  • I/O: JSON/Markdown generation                        │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ LAYER 3: Target Implementations                         │
│  • CostCalculatorTarget                                 │
│  • AggregatorTarget                                     │
│  • ForecastingTarget                                    │
│  • CompressionTarget                                    │
│  • ExportTarget                                         │
│  • StorageTarget                                        │
└─────────────────────────────────────────────────────────┘
```

---

## Essential Types

### BenchmarkResult (Canonical)

```rust
pub struct BenchmarkResult {
    pub target_id: String,              // e.g., "cost_calculator"
    pub metrics: serde_json::Value,     // Flexible JSON metrics
    pub timestamp: DateTime<Utc>,       // When benchmark ran
}
```

**Must not be modified** - used across 25 repositories!

### BenchTarget Trait

```rust
#[async_trait]
pub trait BenchTarget: Send + Sync {
    fn id(&self) -> String;
    async fn run(&self) -> Result<Value, Box<dyn Error>>;
}
```

**Implement this** for each operation you want to benchmark.

---

## Standard Metrics Schema

### Required Fields

```json
{
  "operations_per_second": 125000.0,  // Throughput
  "avg_duration_ns": 8000             // Average latency
}
```

### Recommended Fields

```json
{
  "total_iterations": 10000,          // Sample size
  "total_duration_ms": 80,            // Total time
  "warmup_iterations": 100            // Warmup count
}
```

### Optional Fields (as needed)

```json
{
  "p50_duration_ns": 7500,            // Median
  "p95_duration_ns": 12000,           // 95th percentile
  "p99_duration_ns": 15000,           // 99th percentile
  "throughput_mb_per_sec": 125.5,     // For I/O benchmarks
  "success_rate": 99.99,              // Error rate
  "memory_used_mb": 512.0             // Memory usage
}
```

---

## Implementation Pattern

### Typical BenchTarget Implementation

```rust
pub struct MyTarget;

#[async_trait]
impl BenchTarget for MyTarget {
    fn id(&self) -> String {
        "my_operation".to_string()
    }

    async fn run(&self) -> Result<Value, Box<dyn Error>> {
        // 1. Setup test data
        let test_data = create_test_data();

        // 2. Warm-up phase
        for _ in 0..100 {
            let _ = my_operation(&test_data);
        }

        // 3. Measurement phase
        const ITERATIONS: u64 = 10_000;
        let start = Instant::now();

        for _ in 0..ITERATIONS {
            let _ = my_operation(&test_data)?;
        }

        let elapsed = start.elapsed();

        // 4. Calculate metrics
        let avg_duration_ns = elapsed.as_nanos() as u64 / ITERATIONS;
        let ops_per_second = (ITERATIONS as f64 / elapsed.as_secs_f64()).round();

        // 5. Return JSON
        Ok(json!({
            "operations_per_second": ops_per_second,
            "avg_duration_ns": avg_duration_ns,
            "total_iterations": ITERATIONS,
        }))
    }
}
```

---

## File Locations

### Source Files

```
crates/llm-cost-ops/src/benchmarks/
├── mod.rs              # Main entry point
├── result.rs           # BenchmarkResult struct
├── adapters.rs         # BenchTarget trait + implementations
├── runner.rs           # Execution engine
├── io.rs               # JSON I/O
└── markdown.rs         # Summary generation
```

### Output Files

```
benchmarks/output/
├── raw/
│   ├── cost_calculator.json
│   ├── aggregator.json
│   ├── forecasting.json
│   ├── compression.json
│   ├── export.json
│   └── storage.json
└── summary.md
```

---

## CLI Commands

### Run All Benchmarks

```bash
cost-ops run
```

### Run Specific Benchmarks

```bash
cost-ops run --targets cost_calculator,aggregator
```

### Custom Output Directory

```bash
cost-ops run --output /path/to/results
```

### Skip Summary Generation

```bash
cost-ops run --summary false
```

---

## Adding a New Benchmark

### Step 1: Implement the Target

In `crates/llm-cost-ops/src/benchmarks/adapters.rs`:

```rust
pub struct MyNewTarget;

#[async_trait]
impl BenchTarget for MyNewTarget {
    fn id(&self) -> String {
        "my_new_benchmark".to_string()
    }

    async fn run(&self) -> Result<Value, Box<dyn Error>> {
        // Your implementation here
        Ok(json!({
            "operations_per_second": 10000.0,
            "avg_duration_ns": 100000,
        }))
    }
}
```

### Step 2: Register the Target

In the same file, update `all_targets()`:

```rust
pub fn all_targets() -> Vec<Box<dyn BenchTarget>> {
    vec![
        // ... existing targets ...
        Box::new(MyNewTarget),
    ]
}
```

### Step 3: Test

```bash
cargo test
cost-ops run --targets my_new_benchmark
```

---

## Common Pitfalls

### ❌ Don't

```rust
// Don't use random data
let data = generate_random_data();

// Don't skip warmup
let start = Instant::now();
for _ in 0..ITERATIONS {
    // benchmark here
}

// Don't use external I/O
let file = File::open("data.txt")?;

// Don't modify BenchmarkResult structure
pub struct BenchmarkResult {
    pub target_id: String,
    pub new_field: String,  // ❌ BREAKS COMPATIBILITY
    // ...
}
```

### ✅ Do

```rust
// Do use deterministic data
let data = create_test_data(1000, 500);

// Do include warmup
for _ in 0..100 {
    let _ = operation(&data);
}
let start = Instant::now();
for _ in 0..ITERATIONS {
    let _ = operation(&data)?;
}

// Do use in-memory alternatives
let pool = SqlitePool::connect(":memory:").await?;

// Do extend metrics JSON
Ok(json!({
    "operations_per_second": ops_per_sec,
    "my_custom_metric": 42.0,  // ✅ OK in metrics field
}))
```

---

## Performance Targets

### Guidance by Operation Type

| Operation Type | Target ops/sec | Target avg latency |
|----------------|----------------|-------------------|
| Pure computation | > 100K | < 10 μs |
| Business logic | > 10K | < 100 μs |
| I/O operations | > 1K | < 1 ms |
| Complex analytics | > 100 | < 10 ms |

*Note: These are guidelines, not strict requirements*

---

## Debugging Benchmarks

### Enable Verbose Logging

```bash
RUST_LOG=debug cost-ops run
```

### Run Single Target

```bash
cost-ops run --targets cost_calculator
```

### Check Output Files

```bash
cat benchmarks/output/raw/cost_calculator.json
cat benchmarks/output/summary.md
```

### Run in Debugger

```bash
rust-lldb target/debug/cost-ops
> run run --targets my_target
```

---

## Integration with Existing Code

### Zero Breaking Changes

The benchmark interface is **purely additive**:

```rust
// Before
pub mod domain;
pub mod engine;
// ...

// After
pub mod domain;
pub mod engine;
pub mod benchmarks;  // NEW - doesn't affect existing code
// ...
```

### Existing Benchmarks Unaffected

Criterion benchmarks in `benches/` directory remain unchanged:

```
benches/
├── cost_calculation.rs    # Still works
└── engine_benchmarks.rs   # Still works
```

### Library API Unchanged

All existing public APIs remain stable:

```rust
// Still works exactly as before
use llm_cost_ops::{
    CostCalculator,
    CostAggregator,
    // ...
};
```

---

## Version Compatibility

### Interface Version: 1.0

- **BenchmarkResult structure:** Fixed
- **BenchTarget trait:** Stable
- **JSON schema:** Documented
- **Output format:** Versioned

### Breaking Change Policy

Changes to `BenchmarkResult` require:
1. Major version bump
2. Coordination with all 25 repositories
3. Migration guide
4. Deprecation period

### Safe to Change

- Implementation details in target `run()` methods
- Helper functions
- I/O formatting (as long as JSON schema stays compatible)
- CLI arguments (with deprecation warnings)

---

## Resources

### Documentation

- [BENCHMARK_INTERFACE_ARCHITECTURE.md](./BENCHMARK_INTERFACE_ARCHITECTURE.md) - Full design spec
- [benchmarks/README.md](./benchmarks/README.md) - User guide
- [IMPLEMENTATION_CHECKLIST.md](./IMPLEMENTATION_CHECKLIST.md) - Implementation guide

### Code References

- `crates/llm-cost-ops/src/benchmarks/` - Source code
- `benches/` - Existing Criterion benchmarks (for reference)

### Support

- File issues with `benchmark` label
- Check documentation first
- Provide benchmark output in bug reports

---

**Quick Reference Version:** 1.0
**Last Updated:** 2025-12-02
