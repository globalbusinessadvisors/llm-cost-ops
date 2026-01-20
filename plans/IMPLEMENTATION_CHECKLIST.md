# Benchmark Interface Implementation Checklist

**Project:** LLM-CostOps Canonical Benchmark Interface
**Reference:** BENCHMARK_INTERFACE_ARCHITECTURE.md
**Target Date:** Week 4 from start

---

## Phase 1: Core Infrastructure (Week 1)

### Day 1-2: Result, I/O, and Markdown Modules

- [ ] **Create benchmarks module structure**
  - [ ] Create directory: `crates/llm-cost-ops/src/benchmarks/`
  - [ ] Create `mod.rs` with module declarations

- [ ] **Implement result.rs**
  - [ ] Define `BenchmarkResult` struct with exact specification
    - [ ] `target_id: String`
    - [ ] `metrics: serde_json::Value`
    - [ ] `timestamp: DateTime<Utc>`
  - [ ] Implement `new()` constructor
  - [ ] Implement `get_metric_f64()` helper
  - [ ] Implement `get_metric_u64()` helper
  - [ ] Add Serialize/Deserialize derives
  - [ ] Write unit tests for serialization

- [ ] **Implement io.rs**
  - [ ] Implement `write_json()` function
  - [ ] Implement `read_json()` function
  - [ ] Implement `read_all()` function
  - [ ] Ensure directory creation logic
  - [ ] Write unit tests with tempdir

- [ ] **Implement markdown.rs**
  - [ ] Implement `generate_summary()` function
  - [ ] Create summary table formatter
  - [ ] Create detailed results section
  - [ ] Add header and footer
  - [ ] Implement `write_summary()` function
  - [ ] Write unit tests

### Day 3-4: Adapters and Runner

- [ ] **Implement adapters.rs (trait definition)**
  - [ ] Define `BenchTarget` trait
    - [ ] `fn id(&self) -> String`
    - [ ] `async fn run(&self) -> Result<Value, Box<dyn Error>>`
  - [ ] Add async_trait attribute
  - [ ] Document trait requirements

- [ ] **Implement adapters.rs (helper functions)**
  - [ ] `create_test_usage()` helper
  - [ ] `create_test_pricing()` helper
  - [ ] `create_test_cost_records()` helper
  - [ ] `create_test_time_series()` helper
  - [ ] `create_test_json_data()` helper
  - [ ] `create_test_report_request()` helper

- [ ] **Implement runner.rs**
  - [ ] Implement `execute()` function
  - [ ] Implement `execute_all()` function
  - [ ] Add logging with tracing
  - [ ] Add error handling
  - [ ] Write unit tests

### Day 5: Module Integration and Tests

- [ ] **Implement mod.rs orchestration**
  - [ ] Define module structure
  - [ ] Implement `run_all_benchmarks()` function
  - [ ] Implement `run_and_save()` convenience function
  - [ ] Add module-level documentation

- [ ] **Update lib.rs**
  - [ ] Add `pub mod benchmarks;`
  - [ ] Re-export key types:
    - [ ] `BenchmarkResult`
    - [ ] `BenchTarget`
    - [ ] `run_all_benchmarks`
    - [ ] `run_and_save`
    - [ ] `all_targets`

- [ ] **Write integration tests**
  - [ ] Test module imports
  - [ ] Test result serialization round-trip
  - [ ] Test I/O operations
  - [ ] Test markdown generation

- [ ] **Validation**
  - [ ] Run `cargo test` - all tests pass
  - [ ] Run `cargo build` - no errors
  - [ ] Run `cargo clippy` - no warnings
  - [ ] Run `cargo doc` - documentation builds

---

## Phase 2: Target Implementations (Week 2)

### Day 1: Calculator and Aggregator Targets

- [ ] **Implement CostCalculatorTarget**
  - [ ] Create struct
  - [ ] Implement `BenchTarget::id()`
  - [ ] Implement `BenchTarget::run()`
    - [ ] Setup test data
    - [ ] Warmup phase (100 iterations)
    - [ ] Measurement phase (10K iterations)
    - [ ] Calculate metrics
  - [ ] Return JSON with:
    - [ ] `operations_per_second`
    - [ ] `avg_duration_ns`
    - [ ] `total_iterations`
  - [ ] Test manually

- [ ] **Implement AggregatorTarget**
  - [ ] Create struct
  - [ ] Implement `BenchTarget::id()`
  - [ ] Implement `BenchTarget::run()`
    - [ ] Setup test data (1K records)
    - [ ] Warmup phase
    - [ ] Measurement phase
    - [ ] Calculate metrics
  - [ ] Return JSON with metrics
  - [ ] Test manually

### Day 2: Forecasting Target

- [ ] **Implement ForecastingTarget**
  - [ ] Create struct
  - [ ] Implement `BenchTarget::id()`
  - [ ] Implement `BenchTarget::run()`
    - [ ] Create test time series (100 points)
    - [ ] Warmup phase
    - [ ] Measurement phase
    - [ ] Calculate metrics
  - [ ] Return JSON with:
    - [ ] `operations_per_second`
    - [ ] `avg_duration_ns`
    - [ ] `data_points`
  - [ ] Test manually

### Day 3: Compression and Export Targets

- [ ] **Implement CompressionTarget**
  - [ ] Create struct
  - [ ] Implement `BenchTarget::id()`
  - [ ] Implement `BenchTarget::run()`
    - [ ] Create test data (1MB JSON)
    - [ ] Warmup phase
    - [ ] Measurement phase
    - [ ] Calculate metrics
  - [ ] Return JSON with:
    - [ ] `operations_per_second`
    - [ ] `throughput_mb_per_sec`
    - [ ] `uncompressed_size_bytes`
  - [ ] Test manually

- [ ] **Implement ExportTarget**
  - [ ] Create struct
  - [ ] Implement `BenchTarget::id()`
  - [ ] Implement `BenchTarget::run()`
    - [ ] Create test report request
    - [ ] Warmup phase
    - [ ] Measurement phase
    - [ ] Calculate metrics
  - [ ] Return JSON with metrics
  - [ ] Test manually

### Day 4: Storage Target

- [ ] **Implement StorageTarget**
  - [ ] Create struct
  - [ ] Implement `BenchTarget::id()`
  - [ ] Implement `BenchTarget::run()`
    - [ ] Setup in-memory SQLite
    - [ ] Create test usage record
    - [ ] Measurement phase
    - [ ] Calculate metrics
  - [ ] Return JSON with:
    - [ ] `operations_per_second`
    - [ ] `note` about in-memory performance
  - [ ] Test manually

### Day 5: Integration and Validation

- [ ] **Update adapters.rs**
  - [ ] Implement `all_targets()` function
  - [ ] Register all 6 targets:
    - [ ] CostCalculatorTarget
    - [ ] AggregatorTarget
    - [ ] ForecastingTarget
    - [ ] CompressionTarget
    - [ ] ExportTarget
    - [ ] StorageTarget

- [ ] **Integration testing**
  - [ ] Test `all_targets()` returns correct count
  - [ ] Test each target individually
  - [ ] Test `run_all_benchmarks()` end-to-end
  - [ ] Validate JSON schema for all outputs
  - [ ] Verify metrics are reasonable

- [ ] **Performance validation**
  - [ ] Run benchmarks on reference hardware
  - [ ] Document baseline performance numbers
  - [ ] Verify no performance regressions

---

## Phase 3: CLI Integration (Week 3)

### Day 1-2: Command Definition and Parsing

- [ ] **Update CLI module**
  - [ ] Edit `crates/llm-cost-ops-cli/src/cli/mod.rs`
  - [ ] Add `Run` variant to `Commands` enum:
    - [ ] `output: PathBuf` with default
    - [ ] `summary: bool` with default true
    - [ ] `targets: Option<String>` for filtering
  - [ ] Add documentation comments

- [ ] **Add dependencies if needed**
  - [ ] Check Cargo.toml for HashSet support
  - [ ] No new dependencies expected

### Day 3: Command Handler Implementation

- [ ] **Implement handle_run_command()**
  - [ ] Edit `crates/llm-cost-ops-cli/src/bin/main.rs`
  - [ ] Add function signature
  - [ ] Implement target filtering logic
  - [ ] Call `benchmarks::runner::execute_all()`
  - [ ] Write JSON results in loop
  - [ ] Generate summary if requested
  - [ ] Add logging with info! macro

- [ ] **Update main() match statement**
  - [ ] Add `Commands::Run` arm
  - [ ] Call `handle_run_command()` with parameters
  - [ ] Handle async properly

### Day 4: Polish and Error Handling

- [ ] **Add error handling**
  - [ ] Handle benchmark failures gracefully
  - [ ] Provide helpful error messages
  - [ ] Return appropriate exit codes

- [ ] **Add progress indicators**
  - [ ] Use `indicatif` for progress bar
  - [ ] Show current target being benchmarked
  - [ ] Show completion percentage

- [ ] **Add logging**
  - [ ] Log benchmark start
  - [ ] Log each target completion with ops/sec
  - [ ] Log summary file creation
  - [ ] Log total completion

### Day 5: Testing and Validation

- [ ] **Manual testing**
  - [ ] Test `cost-ops run` with all benchmarks
  - [ ] Test `cost-ops run --targets xyz`
  - [ ] Test `cost-ops run --output /tmp/bench`
  - [ ] Test `cost-ops run --summary false`
  - [ ] Test error cases

- [ ] **Integration tests**
  - [ ] Write test for CLI parsing
  - [ ] Write test for command execution
  - [ ] Write test for output file creation

- [ ] **Documentation**
  - [ ] Update CLI help text
  - [ ] Update README.md with examples

---

## Phase 4: Documentation & Release (Week 4)

### Day 1-2: Documentation

- [ ] **Create benchmarks/README.md**
  - [x] Overview section
  - [x] Quick start guide
  - [x] Benchmark targets table
  - [x] Output structure
  - [x] Result format specification
  - [x] Interpretation guide
  - [x] Comparison guide
  - [x] CI/CD integration examples
  - [x] Adding new targets guide
  - [x] Best practices
  - [x] Troubleshooting

- [ ] **Update main README.md**
  - [ ] Add "Benchmarking" section
  - [ ] Link to benchmarks/README.md
  - [ ] Add usage examples
  - [ ] Add performance numbers table

- [ ] **Create BENCHMARK_INTERFACE_ARCHITECTURE.md**
  - [x] Executive summary
  - [x] Module hierarchy
  - [x] Core module structure
  - [x] Interface definitions
  - [x] Target implementations
  - [x] Integration points
  - [x] CLI integration
  - [x] Output structure
  - [x] Migration path
  - [x] Implementation roadmap

### Day 3: CI/CD Pipeline

- [ ] **Create GitHub Actions workflow**
  - [ ] Create `.github/workflows/benchmarks.yml`
  - [ ] Add checkout step
  - [ ] Add Rust toolchain setup
  - [ ] Add build step
  - [ ] Add benchmark run step
  - [ ] Add artifact upload
  - [ ] Configure triggers (push to main, PRs)

- [ ] **Add benchmark comparison**
  - [ ] Create `scripts/compare_benchmarks.py`
  - [ ] Implement JSON diff logic
  - [ ] Implement regression detection
  - [ ] Add PR comment integration

- [ ] **Test CI pipeline**
  - [ ] Push to test branch
  - [ ] Verify workflow runs
  - [ ] Check artifacts
  - [ ] Fix any issues

### Day 4: Examples and Communication

- [ ] **Create example scripts**
  - [ ] `scripts/run_benchmarks.sh`
  - [ ] `scripts/historical_comparison.sh`
  - [ ] `scripts/generate_charts.py` (optional)

- [ ] **Write blog post / announcement**
  - [ ] Overview of benchmark system
  - [ ] Benefits and use cases
  - [ ] How to run benchmarks
  - [ ] How to interpret results
  - [ ] How to contribute new targets

- [ ] **Update CONTRIBUTING.md**
  - [ ] Add section on benchmarking
  - [ ] Explain when to add benchmarks
  - [ ] Explain performance regression policy

### Day 5: Release Preparation

- [ ] **Version bump**
  - [ ] Update version in Cargo.toml
  - [ ] Update CHANGELOG.md
  - [ ] Tag release

- [ ] **Final validation**
  - [ ] All tests pass
  - [ ] All benchmarks run
  - [ ] Documentation builds
  - [ ] CI pipeline succeeds
  - [ ] No clippy warnings

- [ ] **Release checklist**
  - [ ] Create GitHub release
  - [ ] Publish to crates.io (if applicable)
  - [ ] Announce on communication channels
  - [ ] Update project board

---

## Cross-Cutting Concerns

### Code Quality

- [ ] **Throughout all phases**
  - [ ] Follow Rust naming conventions
  - [ ] Add comprehensive documentation
  - [ ] Write descriptive error messages
  - [ ] Use `tracing` for logging
  - [ ] Handle all `Result` types properly
  - [ ] Avoid `unwrap()` in production code
  - [ ] Use `?` operator for error propagation

### Testing

- [ ] **Unit tests**
  - [ ] Test each module independently
  - [ ] Test error cases
  - [ ] Test edge cases
  - [ ] Aim for >80% code coverage

- [ ] **Integration tests**
  - [ ] Test module interactions
  - [ ] Test CLI commands
  - [ ] Test file I/O
  - [ ] Test end-to-end workflows

### Performance

- [ ] **Benchmark efficiency**
  - [ ] Minimize overhead in benchmark harness
  - [ ] Use appropriate iteration counts
  - [ ] Include proper warmup phases
  - [ ] Measure only relevant operations

### Compatibility

- [ ] **Backward compatibility**
  - [ ] No breaking changes to existing APIs
  - [ ] All existing tests pass
  - [ ] Existing benchmarks unaffected

- [ ] **Cross-repository compatibility**
  - [ ] BenchmarkResult matches spec exactly
  - [ ] JSON schema documented
  - [ ] Coordinate with other 24 repos

---

## Success Criteria

- [x] ✅ BENCHMARK_INTERFACE_ARCHITECTURE.md created
- [x] ✅ benchmarks/README.md created
- [x] ✅ Directory structure created
- [ ] All code modules implemented and tested
- [ ] CLI integration complete
- [ ] Documentation complete
- [ ] CI/CD pipeline operational
- [ ] Zero breaking changes
- [ ] All existing tests pass
- [ ] Benchmark suite runs successfully
- [ ] Compatible with 25 benchmark-target repositories

---

## Dependencies

### Internal
- `llm_cost_ops::domain::*` - Domain types
- `llm_cost_ops::engine::*` - Calculation engines
- `llm_cost_ops::forecasting::*` - Forecasting
- `llm_cost_ops::compression::*` - Compression
- `llm_cost_ops::export::*` - Report generation
- `llm_cost_ops::storage::*` - Database operations

### External (from workspace)
- `serde` / `serde_json` - Serialization
- `chrono` - Timestamps
- `async-trait` - Async trait support
- `tokio` - Async runtime
- `tracing` - Logging
- `clap` - CLI parsing
- `uuid` - ID generation
- `rust_decimal` - Decimal math
- `sqlx` - Database (for storage benchmark)

---

## Notes

- Keep existing Criterion benchmarks in `benches/` directory
- New canonical interface is complementary, not a replacement
- Focus on maintainability and clarity
- Document all design decisions
- Get code reviews at phase boundaries
- Communicate early and often with stakeholders

---

**Checklist Version:** 1.0
**Last Updated:** 2025-12-02
