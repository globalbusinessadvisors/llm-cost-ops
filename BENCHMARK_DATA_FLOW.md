# Benchmark System Data Flow

**Visual guide to understanding how benchmarks execute and produce results**

---

## Complete Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                         USER INITIATES BENCHMARK                          │
│                                                                           │
│                    $ cost-ops run --targets xyz                           │
└────────────────────────────────┬──────────────────────────────────────────┘
                                 │
                                 ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                          CLI LAYER (main.rs)                              │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │ 1. Parse command-line arguments                                    │  │
│  │    - Extract target filter                                         │  │
│  │    - Extract output directory                                      │  │
│  │    - Extract summary flag                                          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────┬──────────────────────────────────────────┘
                                 │
                                 ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                   FRAMEWORK LAYER (benchmarks::mod)                       │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │ 2. Get all registered targets                                      │  │
│  │    - Call adapters::all_targets()                                  │  │
│  │    - Returns Vec<Box<dyn BenchTarget>>                             │  │
│  └────────────────────┬───────────────────────────────────────────────┘  │
│                       │                                                   │
│  ┌────────────────────▼───────────────────────────────────────────────┐  │
│  │ 3. Filter targets (if specified)                                   │  │
│  │    - Match against --targets argument                              │  │
│  │    - Keep only requested targets                                   │  │
│  └────────────────────┬───────────────────────────────────────────────┘  │
└────────────────────────┼──────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────────────┐
│              EXECUTION LAYER (runner::execute_all)                       │
│                                                                          │
│  For each target:                                                        │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ 4. Execute target.run()                                            │ │
│  │    - Calls specific BenchTarget implementation                     │ │
│  │    - Returns metrics JSON on success                               │ │
│  │    - Returns error on failure                                      │ │
│  └────────────────────┬───────────────────────────────────────────────┘ │
│                       │                                                  │
│  ┌────────────────────▼───────────────────────────────────────────────┐ │
│  │ 5. Wrap result in BenchmarkResult                                  │ │
│  │    - Create BenchmarkResult {                                      │ │
│  │        target_id: "xyz",                                           │ │
│  │        metrics: {...},                                             │ │
│  │        timestamp: Utc::now()                                       │ │
│  │      }                                                              │ │
│  └────────────────────┬───────────────────────────────────────────────┘ │
└────────────────────────┼──────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    I/O LAYER (io::write_json)                            │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ 6. Write individual JSON files                                     │ │
│  │    - For each result:                                              │ │
│  │      - Serialize to JSON                                           │ │
│  │      - Write to benchmarks/output/raw/{target_id}.json             │ │
│  └────────────────────┬───────────────────────────────────────────────┘ │
└────────────────────────┼──────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────────────┐
│              REPORTING LAYER (markdown::write_summary)                   │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ 7. Generate summary report (if requested)                          │ │
│  │    - Collect all results                                           │ │
│  │    - Format as Markdown table                                      │ │
│  │    - Add detailed metrics for each target                          │ │
│  │    - Write to benchmarks/output/summary.md                         │ │
│  └────────────────────┬───────────────────────────────────────────────┘ │
└────────────────────────┼──────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                           COMPLETION                                     │
│                                                                          │
│  Output files created:                                                   │
│  ✓ benchmarks/output/raw/target1.json                                   │
│  ✓ benchmarks/output/raw/target2.json                                   │
│  ✓ benchmarks/output/summary.md                                         │
│                                                                          │
│  User sees:                                                              │
│  ✓ cost_calculator - 125000 ops/sec                                     │
│  ✓ aggregator - 5000 ops/sec                                            │
│  ✓ Summary written to benchmarks/output/summary.md                      │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Individual Target Execution Flow

```
┌──────────────────────────────────────────────────────────────────────────┐
│            runner::execute(Box<dyn BenchTarget>)                          │
└────────────────────────────────┬──────────────────────────────────────────┘
                                 │
                                 ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                     TARGET IMPLEMENTATION                                 │
│                   (e.g., CostCalculatorTarget)                            │
│                                                                           │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │ STEP 1: Setup Test Data                                            │  │
│  │  let usage = create_test_usage(10_000, 5_000);                     │  │
│  │  let pricing = create_test_pricing();                              │  │
│  └────────────────────┬───────────────────────────────────────────────┘  │
│                       │                                                   │
│  ┌────────────────────▼───────────────────────────────────────────────┐  │
│  │ STEP 2: Warm-up Phase                                              │  │
│  │  for _ in 0..100 {                                                 │  │
│  │      let _ = calculator.calculate(&usage, &pricing);               │  │
│  │  }                                                                  │  │
│  │                                                                     │  │
│  │  Purpose: JIT compilation, cache warming, stabilize performance    │  │
│  └────────────────────┬───────────────────────────────────────────────┘  │
│                       │                                                   │
│  ┌────────────────────▼───────────────────────────────────────────────┐  │
│  │ STEP 3: Measurement Phase                                          │  │
│  │  const ITERATIONS: u64 = 10_000;                                   │  │
│  │  let start = Instant::now();                                       │  │
│  │                                                                     │  │
│  │  for _ in 0..ITERATIONS {                                          │  │
│  │      let _ = calculator.calculate(&usage, &pricing)?;              │  │
│  │  }                                                                  │  │
│  │                                                                     │  │
│  │  let elapsed = start.elapsed();                                    │  │
│  └────────────────────┬───────────────────────────────────────────────┘  │
│                       │                                                   │
│  ┌────────────────────▼───────────────────────────────────────────────┐  │
│  │ STEP 4: Calculate Metrics                                          │  │
│  │  let avg_duration_ns = elapsed.as_nanos() as u64 / ITERATIONS;     │  │
│  │  let ops_per_second = ITERATIONS as f64 / elapsed.as_secs_f64();   │  │
│  └────────────────────┬───────────────────────────────────────────────┘  │
│                       │                                                   │
│  ┌────────────────────▼───────────────────────────────────────────────┐  │
│  │ STEP 5: Return JSON                                                │  │
│  │  Ok(json!({                                                         │  │
│  │      "operations_per_second": ops_per_second,                      │  │
│  │      "avg_duration_ns": avg_duration_ns,                           │  │
│  │      "total_iterations": ITERATIONS,                               │  │
│  │  }))                                                                │  │
│  └────────────────────┬───────────────────────────────────────────────┘  │
└────────────────────────┼──────────────────────────────────────────────────┘
                         │
                         ▼
                  BenchmarkResult created
                  with metrics JSON
```

---

## JSON Data Structure Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Target returns metrics JSON                          │
│                                                                          │
│  {                                                                       │
│    "operations_per_second": 125000.0,                                   │
│    "avg_duration_ns": 8000,                                             │
│    "total_iterations": 10000                                            │
│  }                                                                       │
└────────────────────────────────┬─────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                  Wrapped in BenchmarkResult                              │
│                                                                          │
│  BenchmarkResult {                                                       │
│    target_id: "cost_calculator",                                        │
│    metrics: {                                                            │
│      "operations_per_second": 125000.0,                                 │
│      "avg_duration_ns": 8000,                                           │
│      "total_iterations": 10000                                          │
│    },                                                                    │
│    timestamp: "2025-12-02T12:34:56.789Z"                                │
│  }                                                                       │
└────────────────────────────────┬─────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│              Serialized to benchmarks/output/raw/                        │
│                                                                          │
│  File: cost_calculator.json                                             │
│  {                                                                       │
│    "target_id": "cost_calculator",                                      │
│    "metrics": {                                                          │
│      "operations_per_second": 125000.0,                                 │
│      "avg_duration_ns": 8000,                                           │
│      "total_iterations": 10000                                          │
│    },                                                                    │
│    "timestamp": "2025-12-02T12:34:56.789Z"                              │
│  }                                                                       │
└────────────────────────────────┬─────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                   Included in summary.md                                 │
│                                                                          │
│  | Target | Operations/sec | Avg Duration (ns) | Notes |                │
│  |--------|----------------|-------------------|-------|                │
│  | cost_calculator | 125000 | 8000 | - |                                │
│                                                                          │
│  ### cost_calculator                                                     │
│  ```json                                                                 │
│  {                                                                       │
│    "operations_per_second": 125000.0,                                   │
│    "avg_duration_ns": 8000,                                             │
│    "total_iterations": 10000                                            │
│  }                                                                       │
│  ```                                                                     │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Module Interaction Sequence

```
User Command
     │
     └─> CLI Parser (clap)
             │
             └─> handle_run_command()
                     │
                     ├─> benchmarks::all_targets()
                     │       │
                     │       └─> Returns Vec<Box<dyn BenchTarget>>
                     │               ├─> CostCalculatorTarget
                     │               ├─> AggregatorTarget
                     │               ├─> ForecastingTarget
                     │               ├─> CompressionTarget
                     │               ├─> ExportTarget
                     │               └─> StorageTarget
                     │
                     ├─> Filter targets (if --targets specified)
                     │
                     └─> For each target:
                             │
                             ├─> runner::execute(target)
                             │       │
                             │       └─> target.run()
                             │               │
                             │               ├─> Uses existing modules:
                             │               │   ├─> engine::CostCalculator
                             │               │   ├─> engine::CostAggregator
                             │               │   ├─> forecasting::*
                             │               │   ├─> compression::*
                             │               │   ├─> export::*
                             │               │   └─> storage::*
                             │               │
                             │               └─> Returns metrics JSON
                             │
                             ├─> Create BenchmarkResult
                             │
                             ├─> io::write_json(result)
                             │       │
                             │       └─> Write to benchmarks/output/raw/{id}.json
                             │
                             └─> Log success with ops/sec
                                     │
                                     ▼
                     markdown::write_summary(all_results)
                             │
                             └─> Write to benchmarks/output/summary.md
                                     │
                                     ▼
                              Complete!
```

---

## Error Handling Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                          Target Execution                                │
└────────────────────────────────┬─────────────────────────────────────────┘
                                 │
                                 ▼
                        Success or Error?
                                 │
                    ┌────────────┴────────────┐
                    │                         │
                    ▼                         ▼
              ┌──────────┐            ┌──────────────┐
              │ Success  │            │    Error     │
              └─────┬────┘            └──────┬───────┘
                    │                        │
                    ▼                        ▼
         Return metrics JSON      Return Box<dyn Error>
                    │                        │
                    ▼                        ▼
         Create BenchmarkResult     Log error with warn!
                    │                        │
                    ▼                        ▼
         Write JSON file           Convert to CostOpsError
                    │                        │
                    ▼                        ▼
         Add to results           Propagate to caller
                    │                        │
                    │                        │
                    └────────────┬───────────┘
                                 │
                                 ▼
                    Continue to next target
                    or abort if critical
```

---

## Timing and Performance

```
Timeline for single benchmark target (example: cost_calculator)

0 ms    ├─ Start execution
        │
        ├─ Setup test data (< 1 ms)
        │
5 ms    ├─ Warmup phase (100 iterations × ~0.05 ms)
        │
        ├─ Start measurement timer
        │
85 ms   ├─ Measurement phase (10,000 iterations × ~0.008 ms)
        │
        ├─ Stop measurement timer
        │
        ├─ Calculate metrics (< 1 ms)
        │
        ├─ Create JSON (< 1 ms)
        │
90 ms   └─ Return result

Total: ~90ms per target (varies by operation complexity)

Full suite (6 targets): ~1-2 seconds
```

---

## File System State Changes

```
BEFORE running benchmarks:
benchmarks/
└── output/
    └── raw/
        └── .gitkeep

DURING execution:
benchmarks/
└── output/
    └── raw/
        ├── .gitkeep
        └── (empty - files being written)

AFTER successful execution:
benchmarks/
└── output/
    ├── raw/
    │   ├── .gitkeep
    │   ├── cost_calculator.json     ← NEW
    │   ├── aggregator.json           ← NEW
    │   ├── forecasting.json          ← NEW
    │   ├── compression.json          ← NEW
    │   ├── export.json               ← NEW
    │   └── storage.json              ← NEW
    └── summary.md                    ← NEW

Each file contains:
- Structured JSON (for programmatic analysis)
- Timestamped results (for historical tracking)
- Complete metrics (for detailed analysis)
```

---

## Concurrency Model

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Sequential Execution                                 │
│                                                                          │
│  Target 1 → Result 1 → Write 1                                          │
│                ↓                                                         │
│  Target 2 → Result 2 → Write 2                                          │
│                ↓                                                         │
│  Target 3 → Result 3 → Write 3                                          │
│                ↓                                                         │
│  ...                                                                     │
│                ↓                                                         │
│  All Results → Generate Summary                                         │
│                                                                          │
│  Why sequential?                                                         │
│  - Avoid resource contention during benchmarking                        │
│  - Ensure consistent performance measurements                           │
│  - Simplify error handling                                              │
│  - Reduce complexity                                                     │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## Key Takeaways

1. **Simple Flow:** User command → Framework → Targets → Results → Files
2. **Sequential:** Benchmarks run one at a time for consistency
3. **Isolated:** Each target is independent and self-contained
4. **Structured:** Standard JSON format for all outputs
5. **Logged:** Progress visible through console output
6. **Reliable:** Errors handled gracefully with clear messages

---

**Document Version:** 1.0
**Last Updated:** 2025-12-02
