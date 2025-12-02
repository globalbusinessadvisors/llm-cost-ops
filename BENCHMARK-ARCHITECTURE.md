# Benchmark System Architecture

## Component Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         CLI Entry Point                              │
│                    (cost-ops run [OPTIONS])                          │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Command Parser                               │
│                     (clap - cli/mod.rs)                              │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ Commands::Run {                                              │   │
│  │   output: PathBuf,                                           │   │
│  │   no_summary: bool,                                          │   │
│  │   filter: Option<String>                                     │   │
│  │ }                                                            │   │
│  └─────────────────────────────────────────────────────────────┘   │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Main Handler                                    │
│                  (bin/main.rs)                                       │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ async fn run_benchmarks(                                     │   │
│  │   output: &Path,                                             │   │
│  │   generate_summary: bool,                                    │   │
│  │   filter: Option<&str>                                       │   │
│  │ ) -> Result<()>                                              │   │
│  └─────────────────────────────────────────────────────────────┘   │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                   Benchmark Orchestrator                             │
│              (benchmarks::run_all_benchmarks)                        │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ 1. Create output directories                                 │   │
│  │ 2. Execute benchmarks (filtered if requested)                │   │
│  │ 3. Collect results                                           │   │
│  │ 4. Generate summary statistics                               │   │
│  │ 5. Write JSON output                                         │   │
│  │ 6. Generate markdown report (if enabled)                     │   │
│  │ 7. Print console summary                                     │   │
│  └─────────────────────────────────────────────────────────────┘   │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                     Benchmark Executors                              │
│                  (Individual Bench Functions)                        │
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │  Single Cost     │  │  Batch Cost      │  │  Aggregation     │  │
│  │  Calculation     │  │  Calculation     │  │  Operations      │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │  Cached Token    │  │  Usage           │  │  Multi-Provider  │  │
│  │  Calculation     │  │  Validation      │  │  Operations      │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Core Engine                                   │
│                  (llm-cost-ops crate)                                │
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐  │
│  │  CostCalculator  │  │  CostAggregator  │  │  UsageRecord     │  │
│  └──────────────────┘  └──────────────────┘  └──────────────────┘  │
│                                                                      │
│  ┌──────────────────┐  ┌──────────────────┐                         │
│  │  PricingTable    │  │  Domain Models   │                         │
│  └──────────────────┘  └──────────────────┘                         │
└─────────────────────────────────────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                       Result Processing                              │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ calculate_result(name, durations) -> BenchmarkResult         │   │
│  │   - Compute statistics (avg, min, max)                       │   │
│  │   - Calculate throughput                                     │   │
│  │   - Add timestamp                                            │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │ create_summary(results, duration) -> BenchmarkSummary        │   │
│  │   - Count totals                                             │   │
│  │   - Identify fastest/slowest                                 │   │
│  │   - Aggregate statistics                                     │   │
│  └─────────────────────────────────────────────────────────────┘   │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Output Generation                            │
│                                                                      │
│  ┌────────────────────┐  ┌────────────────────┐  ┌──────────────┐  │
│  │  JSON File         │  │  Markdown Report   │  │  Console     │  │
│  │  (Structured)      │  │  (Human-Readable)  │  │  Summary     │  │
│  │                    │  │                    │  │              │  │
│  │  • Raw metrics     │  │  • Statistics      │  │  • Key stats │  │
│  │  • Timestamps      │  │  • Tables          │  │  • Top 5     │  │
│  │  • All benchmarks  │  │  • Analysis        │  │              │  │
│  └────────────────────┘  └────────────────────┘  └──────────────┘  │
│         │                        │                       │           │
│         ▼                        ▼                       ▼           │
│  benchmarks/output/        benchmarks/output/      stdout           │
│  raw/*.json               summary.md                                │
└─────────────────────────────────────────────────────────────────────┘
```

## Data Flow

### 1. Input Processing
```
User Command
    ↓
CLI Parser (clap)
    ↓
Validate Options
    ↓
Create Configuration
```

### 2. Benchmark Execution
```
For each benchmark:
    ↓
Create test data (UsageRecord, PricingTable)
    ↓
Run N iterations (typically 10,000)
    ↓
Measure each iteration with Instant::now()
    ↓
Collect all durations
    ↓
Calculate statistics
    ↓
Create BenchmarkResult
```

### 3. Result Aggregation
```
Collect all BenchmarkResults
    ↓
Create BenchmarkSuite
    ↓
Calculate summary statistics
    ↓
Identify fastest/slowest
```

### 4. Output Generation
```
BenchmarkSuite
    ├─→ Serialize to JSON → Write to file
    ├─→ Generate markdown → Write to summary.md
    └─→ Format console output → Print to stdout
```

## Module Dependencies

```
llm-cost-ops-cli/
├── src/
│   ├── cli/
│   │   └── mod.rs ──────────────┐
│   │                             │
│   ├── benchmarks.rs ←───────────┤
│   │   │                         │
│   │   └──→ llm-cost-ops ────────┤
│   │        (domain, engine)     │
│   │                             │
│   ├── lib.rs ←──────────────────┤
│   │                             │
│   └── bin/                      │
│       └── main.rs ←─────────────┘
│
└── Cargo.toml
    └── dependencies:
        ├── llm-cost-ops (internal)
        ├── clap (CLI parsing)
        ├── tokio (async runtime)
        ├── serde/serde_json (serialization)
        └── chrono (timestamps)
```

## Benchmark Execution Model

```
┌─────────────────────────────────────────────────────────────┐
│                    Benchmark Function                        │
│                                                              │
│  async fn bench_xxx() -> Result<BenchmarkResult>            │
│  {                                                           │
│    ┌──────────────────────────────────────────────────┐     │
│    │ 1. Setup                                         │     │
│    │    - Create test data                            │     │
│    │    - Initialize components                       │     │
│    │    - Prepare iterations vector                   │     │
│    └──────────────────────────────────────────────────┘     │
│                           ↓                                  │
│    ┌──────────────────────────────────────────────────┐     │
│    │ 2. Measurement Loop                              │     │
│    │    for _ in 0..iterations {                      │     │
│    │      let start = Instant::now();                 │     │
│    │      perform_operation();                        │     │
│    │      durations.push(start.elapsed());            │     │
│    │    }                                             │     │
│    └──────────────────────────────────────────────────┘     │
│                           ↓                                  │
│    ┌──────────────────────────────────────────────────┐     │
│    │ 3. Statistics Calculation                        │     │
│    │    - Average duration                            │     │
│    │    - Min/max durations                           │     │
│    │    - Throughput (ops/sec)                        │     │
│    │    - Total duration                              │     │
│    └──────────────────────────────────────────────────┘     │
│                           ↓                                  │
│    ┌──────────────────────────────────────────────────┐     │
│    │ 4. Result Construction                           │     │
│    │    BenchmarkResult {                             │     │
│    │      name, iterations, durations,                │     │
│    │      throughput, timestamp                       │     │
│    │    }                                             │     │
│    └──────────────────────────────────────────────────┘     │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
```

## Filtering Mechanism

```
Filter: Option<String>
        │
        ├── None ──────────────────────→ Run all benchmarks
        │
        └── Some(pattern) ──────┐
                                │
                                ▼
                    should_run_benchmark(name, filter)
                                │
                                ▼
                    name.contains(pattern)?
                                │
                    ┌───────────┴───────────┐
                    │                       │
                    ▼                       ▼
                  true                    false
                    │                       │
                    ▼                       ▼
            Run benchmark              Skip benchmark
```

## Error Handling Flow

```
Operation
    │
    ├─→ Success ─────→ Continue
    │
    └─→ Error ────┐
                  │
                  ▼
            anyhow::Result<T>
                  │
                  ├─→ Context added
                  │
                  ▼
            Propagate up call stack
                  │
                  ▼
            Main error handler
                  │
                  ▼
            Log error & exit
```

## Performance Measurement Strategy

```
Single Operation Benchmark:
├── Iterations: 10,000
├── Warmup: Implicit (first few iterations)
└── Measurement: Instant::now() per iteration

Batch Operation Benchmark:
├── Iterations: Adaptive (100 to 1,000)
│   └── Based on batch size (larger batches = fewer iterations)
├── Operations per iteration: N (batch size)
└── Total operations: iterations × batch_size

Statistical Validity:
├── Multiple iterations reduce variance
├── Min/Max capture outliers
└── Average provides central tendency
```

## Output Format Examples

### JSON Structure
```json
{
  "version": "string",
  "timestamp": "ISO8601",
  "results": [
    {
      "name": "string",
      "iterations": number,
      "total_duration_ms": number,
      "avg_duration_us": number,
      "min_duration_us": number,
      "max_duration_us": number,
      "throughput_per_sec": number,
      "timestamp": "ISO8601"
    }
  ],
  "summary": {
    "total_benchmarks": number,
    "total_iterations": number,
    "total_duration_ms": number,
    "fastest_benchmark": "string",
    "slowest_benchmark": "string"
  }
}
```

### Markdown Structure
```
# Benchmark Results Summary
- Metadata
- Overall Statistics
- Detailed Results Table
- Performance Analysis (grouped)
```

### Console Structure
```
=== BENCHMARK RESULTS SUMMARY ===
- Version & Timestamp
- Overall Statistics
- Top 5 Fastest Operations
====================================
```

## Extension Points

The architecture supports future enhancements:

1. **New Benchmark Types**
   - Add new async functions following existing pattern
   - Register in `run_all_benchmarks()`

2. **Additional Output Formats**
   - Extend output generation section
   - Add new format option to CLI

3. **Advanced Filtering**
   - Extend `should_run_benchmark()` logic
   - Support regex or multiple filters

4. **Comparison Mode**
   - Load historical results
   - Compare with current run
   - Generate delta reports

5. **CI Integration**
   - Exit codes based on regression detection
   - JSON schema for automated processing
   - Trend analysis over time
