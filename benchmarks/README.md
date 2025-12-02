# LLM-CostOps Benchmark Suite

**Version:** 1.0
**Compatibility:** Canonical Benchmark Interface v1.0

---

## Overview

This directory contains the standardized benchmark suite for the LLM-CostOps platform. The benchmark interface is designed to be compatible with 25 benchmark-target repositories, enabling cross-repository performance comparison and aggregation.

## Quick Start

### Running All Benchmarks

```bash
# From repository root
cargo build --release
./target/release/cost-ops run
```

### Running Specific Benchmarks

```bash
# Run only cost calculation and aggregation benchmarks
./target/release/cost-ops run --targets cost_calculator,aggregator
```

### Viewing Results

```bash
# View summary report
cat benchmarks/output/summary.md

# View individual JSON results
cat benchmarks/output/raw/cost_calculator.json
```

## Benchmark Targets

| Target ID | Module | Description | Typical Performance |
|-----------|--------|-------------|---------------------|
| `cost_calculator` | `engine::CostCalculator` | Single cost calculation | ~125K ops/sec |
| `aggregator` | `engine::CostAggregator` | Aggregate 1K cost records | ~5K ops/sec |
| `forecasting` | `forecasting::ForecastEngine` | Time series forecast (100 points) | ~500 ops/sec |
| `compression` | `compression::Compressor` | Compress 1MB data (gzip) | ~100 ops/sec |
| `export` | `export::ReportGenerator` | Generate cost report | ~200 ops/sec |
| `storage` | `storage::*Repository` | Database write operations | ~1K ops/sec |

*Note: Performance numbers are approximate and hardware-dependent*

## Output Structure

```
benchmarks/
├── output/
│   ├── raw/                    # Individual JSON results
│   │   ├── cost_calculator.json
│   │   ├── aggregator.json
│   │   ├── forecasting.json
│   │   ├── compression.json
│   │   ├── export.json
│   │   └── storage.json
│   └── summary.md              # Human-readable summary
└── README.md                   # This file
```

## Result Format

### JSON Schema

Each benchmark target produces a JSON file with the following structure:

```json
{
  "target_id": "string",           // Unique benchmark identifier
  "metrics": {                     // Performance metrics (flexible schema)
    "operations_per_second": 0.0,  // Required: throughput metric
    "avg_duration_ns": 0,          // Required: average latency
    "p50_duration_ns": 0,          // Optional: median latency
    "p95_duration_ns": 0,          // Optional: 95th percentile
    "p99_duration_ns": 0,          // Optional: 99th percentile
    "throughput_mb_per_sec": 0.0,  // Optional: for I/O benchmarks
    "total_iterations": 0,         // Recommended: sample size
    "...": "..."                   // Target-specific metrics
  },
  "timestamp": "2025-12-02T12:34:56.789Z"  // UTC timestamp
}
```

### Markdown Summary

The summary report includes:
- Header with generation timestamp
- Performance summary table
- Detailed JSON metrics for each target
- Footer with version information

## Interpreting Results

### Operations Per Second (ops/sec)

Higher is better. Represents the number of operations completed per second.

**Guidance:**
- `> 100K ops/sec`: Excellent for computational operations
- `10K - 100K ops/sec`: Good for typical business logic
- `1K - 10K ops/sec`: Acceptable for I/O-bound operations
- `< 1K ops/sec`: May indicate optimization opportunity

### Average Duration (ns)

Lower is better. Average time to complete one operation in nanoseconds.

**Guidance:**
- `< 10 μs (10,000 ns)`: Excellent
- `10-100 μs`: Good
- `100-1000 μs (1 ms)`: Acceptable
- `> 1 ms`: Consider optimization

### Percentile Metrics

- **p50 (median):** Half of operations complete faster
- **p95:** 95% of operations complete faster
- **p99:** 99% of operations complete faster

Large gaps between p50 and p99 indicate inconsistent performance.

## Comparison Across Runs

### Tracking Performance Over Time

```bash
# Create dated snapshots
mkdir -p benchmarks/history/$(date +%Y-%m-%d)
cp -r benchmarks/output/raw benchmarks/history/$(date +%Y-%m-%d)/
```

### Regression Detection

Significant performance changes (>10% regression) should be investigated:

```bash
# Compare with previous run
python scripts/compare_benchmarks.py \
  benchmarks/history/2025-12-01 \
  benchmarks/output
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Benchmarks

on:
  push:
    branches: [ main ]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build
        run: cargo build --release
      - name: Run benchmarks
        run: ./target/release/cost-ops run
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: benchmarks/output/
```

## Adding New Benchmark Targets

### 1. Implement BenchTarget Trait

```rust
// In crates/llm-cost-ops/src/benchmarks/adapters.rs

pub struct MyNewTarget;

#[async_trait]
impl BenchTarget for MyNewTarget {
    fn id(&self) -> String {
        "my_new_target".to_string()
    }

    async fn run(&self) -> Result<serde_json::Value, Box<dyn Error>> {
        // Your benchmark implementation
        Ok(json!({
            "operations_per_second": 10000.0,
            "avg_duration_ns": 100000,
        }))
    }
}
```

### 2. Register in all_targets()

```rust
pub fn all_targets() -> Vec<Box<dyn BenchTarget>> {
    vec![
        // ... existing targets ...
        Box::new(MyNewTarget),
    ]
}
```

### 3. Test and Validate

```bash
cargo test
./target/release/cost-ops run --targets my_new_target
```

## Benchmark Best Practices

### 1. Warm-up Phase

Always include warm-up iterations to ensure JIT compilation, cache population, etc.:

```rust
// Warm-up
for _ in 0..100 {
    let _ = operation_to_benchmark();
}

// Measurement
let start = Instant::now();
for _ in 0..10_000 {
    let _ = operation_to_benchmark();
}
let elapsed = start.elapsed();
```

### 2. Sufficient Sample Size

Use at least 100 iterations for statistical significance:

```rust
const ITERATIONS: u64 = 10_000;  // Good
const ITERATIONS: u64 = 10;      // Too few
```

### 3. Consistent Test Data

Use deterministic, representative inputs:

```rust
// Good: deterministic
let usage = create_test_usage(10_000, 5_000);

// Bad: random data may cause variance
let usage = create_random_usage();
```

### 4. Isolation

Minimize external dependencies:

```rust
// Good: in-memory database
let pool = SqlitePool::connect(":memory:").await?;

// Avoid: network calls, file I/O during measurement
```

### 5. Report Complete Metrics

Include context for result interpretation:

```rust
Ok(json!({
    "operations_per_second": ops_per_sec,
    "avg_duration_ns": avg_duration,
    "total_iterations": ITERATIONS,      // Sample size
    "test_data_size": 1000,              // Input characteristics
    "warmup_iterations": 100,            // Warmup info
}))
```

## Troubleshooting

### Benchmark Fails to Run

**Problem:** `Error: Benchmark 'xyz' failed`

**Solutions:**
1. Check that test data generation functions are available
2. Verify module imports are correct
3. Ensure async runtime is properly initialized
4. Check for missing dependencies

### Inconsistent Results

**Problem:** Performance varies significantly between runs

**Solutions:**
1. Increase sample size (ITERATIONS)
2. Add longer warm-up phase
3. Close other applications to reduce system load
4. Run on dedicated hardware for CI
5. Use CPU pinning for critical benchmarks

### Out of Memory

**Problem:** Benchmark consumes too much memory

**Solutions:**
1. Reduce test data size
2. Use streaming/iterative approaches
3. Clear allocations between iterations
4. Check for memory leaks in tested code

## Support

For questions or issues:

1. Check [BENCHMARK_INTERFACE_ARCHITECTURE.md](../BENCHMARK_INTERFACE_ARCHITECTURE.md) for design details
2. Review implementation in `crates/llm-cost-ops/src/benchmarks/`
3. File issues on GitHub with `benchmark` label

## License

Same as parent repository (Apache-2.0)

---

**Document Version:** 1.0
**Last Updated:** 2025-12-02
