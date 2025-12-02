# Benchmark System Usage Guide

## Quick Start

### Running All Benchmarks

```rust
use llm_cost_ops::benchmarks::run_all_benchmarks;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Run all benchmarks and save results to output directory
    let summary = run_all_benchmarks("benchmarks/output")?;

    println!("Total benchmarks: {}", summary.total_count);
    println!("Passed: {}", summary.passed_count);
    println!("Failed: {}", summary.failed_count);
    println!("Success rate: {:.2}%", summary.success_rate());

    Ok(())
}
```

### Running Category-Specific Benchmarks

```rust
use llm_cost_ops::benchmarks::run_category_benchmarks;

// Run only engine benchmarks
let summary = run_category_benchmarks("engine", "benchmarks/output")?;

// Run only compression benchmarks
let summary = run_category_benchmarks("compression", "benchmarks/output")?;

// Run only forecasting benchmarks
let summary = run_category_benchmarks("forecasting", "benchmarks/output")?;
```

### Quick Validation (No File Output)

```rust
use llm_cost_ops::benchmarks::run_quick_benchmark;

// Fast validation run without saving results
let summary = run_quick_benchmark();
println!("Quick check: {} benchmarks, {:.2}% success rate",
    summary.total_count,
    summary.success_rate()
);
```

## Available Benchmarks

### Engine Category (10 benchmarks)

| Benchmark | Description | Iterations |
|-----------|-------------|------------|
| Single Cost Calculation | Basic cost calculation | 10,000 |
| Batch Cost Calculation (100) | Batch processing 100 records | 100 |
| Batch Cost Calculation (1K) | Batch processing 1,000 records | 100 |
| Batch Cost Calculation (10K) | Batch processing 10,000 records | 100 |
| Cached Token Calculation | Cost calc with cache discount | 10,000 |
| Token Normalization | Token count normalization | 10,000 |
| Cost Aggregation (100) | Aggregate 100 cost records | 1,000 |
| Cost Aggregation (1K) | Aggregate 1,000 cost records | 1,000 |
| Cost Aggregation (10K) | Aggregate 10,000 cost records | 1,000 |
| Validation Overhead | Usage validation performance | 100,000 |

### Compression Category (13 benchmarks)

| Benchmark | Description | Iterations |
|-----------|-------------|------------|
| Gzip Compression (Fastest, 1KB) | Fast gzip on small data | 1,000 |
| Gzip Compression (Default, 1KB) | Default gzip on small data | 1,000 |
| Gzip Compression (Best, 1KB) | Best gzip on small data | 1,000 |
| Gzip Compression (Default, 10KB) | Default gzip on medium data | 1,000 |
| Gzip Compression (Default, 100KB) | Default gzip on large data | 100 |
| Brotli Compression (Fastest, 1KB) | Fast brotli on small data | 1,000 |
| Brotli Compression (Default, 1KB) | Default brotli on small data | 1,000 |
| Brotli Compression (Best, 1KB) | Best brotli on small data | 1,000 |
| Brotli Compression (Default, 10KB) | Default brotli on medium data | 1,000 |
| Brotli Compression (Default, 100KB) | Default brotli on large data | 100 |
| Gzip Decompression (1KB) | Decompress small data | 1,000 |
| Gzip Decompression (10KB) | Decompress medium data | 1,000 |
| Gzip Decompression (100KB) | Decompress large data | 100 |

### Forecasting Category (16 benchmarks)

| Benchmark | Description | Iterations |
|-----------|-------------|------------|
| Linear Trend Forecasting (30) | Linear forecast 30 days | 1,000 |
| Linear Trend Forecasting (90) | Linear forecast 90 days | 1,000 |
| Linear Trend Forecasting (365) | Linear forecast 1 year | 1,000 |
| Moving Average (30, window 7) | MA forecast 30 days | 1,000 |
| Moving Average (90, window 14) | MA forecast 90 days | 1,000 |
| Moving Average (365, window 30) | MA forecast 1 year | 1,000 |
| Exponential Smoothing (30) | ES forecast 30 days | 1,000 |
| Exponential Smoothing (90) | ES forecast 90 days | 1,000 |
| Exponential Smoothing (365) | ES forecast 1 year | 1,000 |
| Anomaly Detection (Z-Score, 30) | Detect anomalies in 30 days | 1,000 |
| Anomaly Detection (Z-Score, 90) | Detect anomalies in 90 days | 1,000 |
| Anomaly Detection (IQR, 30) | IQR anomalies in 30 days | 1,000 |
| Anomaly Detection (IQR, 90) | IQR anomalies in 90 days | 1,000 |
| Budget Forecast (30) | Budget forecast 30 days | 1,000 |
| Budget Forecast (90) | Budget forecast 90 days | 1,000 |

## Discovering Benchmarks

### List All Categories

```rust
use llm_cost_ops::benchmarks::available_categories;

let categories = available_categories();
for category in categories {
    println!("- {}", category);
}
```

Output:
```
- compression
- engine
- forecasting
```

### List Benchmarks in a Category

```rust
use llm_cost_ops::benchmarks::targets_in_category;

let targets = targets_in_category("engine");
for target in targets {
    println!("- {}", target);
}
```

## Output Files

After running benchmarks, the following files are generated:

```
benchmarks/output/
├── summary.json          # JSON summary with all statistics
├── summary.md            # Human-readable markdown report
└── raw/                  # Individual benchmark JSON files
    ├── engine_single_cost_calculation.json
    ├── engine_batch_cost_calculation_100.json
    ├── compression_gzip_Default_1024.json
    └── ... (one file per benchmark)
```

## Reading Results Programmatically

### Load Summary

```rust
use llm_cost_ops::benchmarks::BenchmarkIo;

let io = BenchmarkIo::new("benchmarks/output")?;
let summary = io.read_summary()?;

println!("Total duration: {:.2}s", summary.total_duration.as_secs_f64());

// Access results by category
for (category, results) in &summary.by_category {
    println!("\n{}: {} benchmarks", category, results.len());
    for result in results {
        if result.passed {
            println!("  ✓ {}: {}", result.name, result.throughput_string());
        } else {
            println!("  ✗ {}: FAILED", result.name);
        }
    }
}
```

### Load Individual Result

```rust
use llm_cost_ops::benchmarks::BenchmarkIo;

let io = BenchmarkIo::new("benchmarks/output")?;
let result = io.read_result("raw/engine_single_cost_calculation.json")?;

println!("Benchmark: {}", result.name);
println!("Throughput: {}", result.throughput_string());
println!("Avg time: {}", result.avg_time_string());
println!("Min time: {:?}", result.min_time);
println!("Max time: {:?}", result.max_time);
```

### Load All Results

```rust
use llm_cost_ops::benchmarks::BenchmarkIo;

let io = BenchmarkIo::new("benchmarks/output")?;
let results = io.read_all_results()?;

println!("Loaded {} benchmark results", results.len());

// Find fastest benchmark
if let Some(fastest) = results.iter().max_by_key(|r| r.ops_per_sec as u64) {
    println!("Fastest: {} at {}", fastest.name, fastest.throughput_string());
}

// Find slowest benchmark
if let Some(slowest) = results.iter().min_by_key(|r| r.ops_per_sec as u64) {
    println!("Slowest: {} at {}", slowest.name, slowest.throughput_string());
}
```

## Generating Reports

### Generate Markdown Report

```rust
use llm_cost_ops::benchmarks::{BenchmarkIo, MarkdownGenerator};

let io = BenchmarkIo::new("benchmarks/output")?;
let summary = io.read_summary()?;

let report = MarkdownGenerator::generate_report(&summary);
std::fs::write("custom_report.md", report)?;
```

### Generate Compact Summary

```rust
use llm_cost_ops::benchmarks::{BenchmarkIo, MarkdownGenerator};

let io = BenchmarkIo::new("benchmarks/output")?;
let results = io.read_all_results()?;

let compact = MarkdownGenerator::generate_compact_summary(&results);
println!("{}", compact);
```

### Compare Two Benchmark Runs

```rust
use llm_cost_ops::benchmarks::{BenchmarkIo, MarkdownGenerator};

let baseline_io = BenchmarkIo::new("benchmarks/baseline")?;
let current_io = BenchmarkIo::new("benchmarks/current")?;

let baseline = baseline_io.read_all_results()?;
let current = current_io.read_all_results()?;

let comparison = MarkdownGenerator::generate_comparison_table(&baseline, &current);
std::fs::write("comparison.md", comparison)?;
```

## Creating Custom Benchmarks

### Implement BenchTarget Trait

```rust
use llm_cost_ops::adapters::{BenchTarget, run_iterations, calculate_stats};
use llm_cost_ops::benchmarks::BenchmarkResult;

struct MyCustomBenchmark {
    data_size: usize,
}

impl BenchTarget for MyCustomBenchmark {
    fn id(&self) -> String {
        format!("custom/my_benchmark_{}", self.data_size)
    }

    fn name(&self) -> String {
        format!("My Custom Benchmark ({})", self.data_size)
    }

    fn category(&self) -> String {
        "custom".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let iterations = 1000;

        let (total_duration, timings) = run_iterations(iterations, || {
            // Your benchmark code here
            let _result = expensive_operation(self.data_size);
        });

        let (min, max, std_dev) = calculate_stats(&timings);

        BenchmarkResult::success(
            self.id(),
            self.name(),
            self.category(),
            total_duration,
            iterations,
        )
        .with_stats(min, max, std_dev)
    }

    fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Optional: prepare resources before benchmark
        Ok(())
    }

    fn teardown(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Optional: cleanup after benchmark
        Ok(())
    }
}

fn expensive_operation(size: usize) -> Vec<u8> {
    vec![0u8; size]
}
```

### Register Custom Benchmark

```rust
use llm_cost_ops::adapters::BenchmarkRegistry;

let mut registry = BenchmarkRegistry::new();
registry.register(Box::new(MyCustomBenchmark { data_size: 1024 }));
registry.register(Box::new(MyCustomBenchmark { data_size: 10240 }));

let results = registry.run_all();
```

## Command Line Integration (Future)

The benchmark system can be easily integrated into a CLI tool:

```bash
# Run all benchmarks
llm-cost-ops bench --all

# Run specific category
llm-cost-ops bench --category engine

# Quick validation
llm-cost-ops bench --quick

# List available benchmarks
llm-cost-ops bench --list
```

## CI/CD Integration

Example GitHub Actions workflow:

```yaml
- name: Run Benchmarks
  run: |
    cargo test --package llm-cost-ops --lib benchmarks

- name: Generate Benchmark Report
  run: |
    cargo run --example run_benchmarks

- name: Upload Results
  uses: actions/upload-artifact@v3
  with:
    name: benchmark-results
    path: benchmarks/output/
```

## Performance Baselines

Expected performance ranges (will vary by hardware):

- **Single Cost Calculation**: > 100K ops/sec
- **Token Normalization**: > 100K ops/sec
- **Batch Processing (1K)**: > 1K ops/sec
- **Gzip Compression (1KB)**: > 10K ops/sec
- **Forecasting (30 days)**: > 1K ops/sec

## Troubleshooting

### Benchmark Failures

If a benchmark fails, check the error in the results:

```rust
let io = BenchmarkIo::new("benchmarks/output")?;
let results = io.read_all_results()?;

for result in results.iter().filter(|r| !r.passed) {
    println!("Failed: {} - {}",
        result.name,
        result.error.as_ref().unwrap_or(&"Unknown error".to_string())
    );
}
```

### Slow Benchmarks

If benchmarks take too long, consider:
1. Reducing iteration counts in the adapter
2. Running specific categories instead of all benchmarks
3. Using `run_quick_benchmark()` for validation

## Best Practices

1. **Run on consistent hardware** - Use the same machine for comparisons
2. **Close other applications** - Minimize background processes
3. **Run multiple times** - Average results for accuracy
4. **Compare against baseline** - Detect performance regressions
5. **Document changes** - Track what affected performance
6. **CI/CD integration** - Automate benchmark runs

## Additional Resources

- Implementation summary: `/workspaces/cost-ops/BENCHMARK_IMPLEMENTATION_SUMMARY.md`
- Source code: `/workspaces/cost-ops/crates/llm-cost-ops/src/benchmarks/`
- Adapters: `/workspaces/cost-ops/crates/llm-cost-ops/src/adapters/`
