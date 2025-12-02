# Benchmark Interface Architecture Specification
## LLM-CostOps Repository

**Document Version:** 1.0
**Date:** 2025-12-02
**Status:** Canonical Design Specification

---

## Executive Summary

This document specifies the canonical benchmark interface architecture for the cost-ops repository, designed to integrate seamlessly with 25 benchmark-target repositories while maintaining zero breaking changes to existing code.

The architecture introduces a standardized benchmarking framework that:
- Provides a unified interface for measuring performance across all CostOps operations
- Enables cross-repository benchmark aggregation and comparison
- Maintains backward compatibility with existing Criterion benchmarks
- Supports both synchronous and async operations
- Generates standardized JSON and Markdown reports

---

## Table of Contents

1. [Module Hierarchy](#1-module-hierarchy)
2. [Core Module Structure](#2-core-module-structure)
3. [Interface Definitions](#3-interface-definitions)
4. [Benchmark Target Implementations](#4-benchmark-target-implementations)
5. [Integration Points](#5-integration-points)
6. [CLI Integration](#6-cli-integration)
7. [Output Structure](#7-output-structure)
8. [Migration Path](#8-migration-path)
9. [Implementation Roadmap](#9-implementation-roadmap)

---

## 1. Module Hierarchy

### 1.1 Complete Directory Structure

```
cost-ops/
├── crates/
│   ├── llm-cost-ops/                    # Core library
│   │   └── src/
│   │       ├── benchmarks/              # NEW: Benchmark interface
│   │       │   ├── mod.rs              # Main module with run_all_benchmarks()
│   │       │   ├── result.rs           # BenchmarkResult struct
│   │       │   ├── markdown.rs         # Markdown report generation
│   │       │   ├── io.rs               # File I/O for results
│   │       │   ├── adapters.rs         # BenchTarget trait and implementations
│   │       │   └── runner.rs           # Benchmark execution engine
│   │       ├── domain/
│   │       ├── engine/
│   │       ├── storage/
│   │       ├── forecasting/
│   │       ├── compression/
│   │       ├── export/
│   │       └── lib.rs                  # Re-export benchmarks module
│   │
│   └── llm-cost-ops-cli/               # CLI binary
│       └── src/
│           ├── cli/
│           │   └── mod.rs              # NEW: Add `run` command
│           └── bin/
│               └── main.rs             # NEW: Handle benchmark command
│
├── benchmarks/                          # NEW: Benchmark suite root
│   ├── output/                         # NEW: Benchmark outputs
│   │   ├── raw/                        # JSON results per target
│   │   │   ├── cost_calculator.json
│   │   │   ├── aggregator.json
│   │   │   ├── forecasting.json
│   │   │   ├── compression.json
│   │   │   └── export.json
│   │   └── summary.md                  # Human-readable summary
│   └── README.md                       # Benchmark documentation
│
└── benches/                            # Existing Criterion benchmarks (unchanged)
    ├── cost_calculation.rs
    └── engine_benchmarks.rs
```

### 1.2 Module Dependency Graph

```
┌─────────────────────────────────────────────────────────────┐
│                    llm-cost-ops-cli                         │
│                   (Binary Crate)                            │
│  ┌────────────────────────────────────────────────────┐    │
│  │  CLI::Commands::Run                                │    │
│  │    - Invokes run_all_benchmarks()                  │    │
│  │    - Generates reports                             │    │
│  └────────────────┬───────────────────────────────────┘    │
└───────────────────┼────────────────────────────────────────┘
                    │
                    │ calls
                    ▼
┌─────────────────────────────────────────────────────────────┐
│                    llm-cost-ops                             │
│                   (Library Crate)                           │
│  ┌────────────────────────────────────────────────────┐    │
│  │  benchmarks::run_all_benchmarks()                  │    │
│  │    ├─> adapters::all_targets()                     │    │
│  │    ├─> runner::execute()                           │    │
│  │    ├─> result::BenchmarkResult                     │    │
│  │    ├─> io::write_json()                            │    │
│  │    └─> markdown::generate_summary()                │    │
│  └────────────────┬───────────────────────────────────┘    │
│                   │                                         │
│  ┌────────────────▼───────────────────────────────────┐    │
│  │  adapters::BenchTarget implementations             │    │
│  │    ├─> CostCalculatorTarget                        │    │
│  │    ├─> AggregatorTarget                            │    │
│  │    ├─> ForecastingTarget                           │    │
│  │    ├─> CompressionTarget                           │    │
│  │    ├─> ExportTarget                                │    │
│  │    └─> StorageTarget                               │    │
│  └────────────────┬───────────────────────────────────┘    │
│                   │                                         │
│                   │ uses                                    │
│                   ▼                                         │
│  ┌──────────────────────────────────────────────────┐      │
│  │  Existing Modules (Zero Changes)                 │      │
│  │    ├─> engine::CostCalculator                    │      │
│  │    ├─> engine::CostAggregator                    │      │
│  │    ├─> forecasting::ForecastEngine               │      │
│  │    ├─> compression::Compressor                   │      │
│  │    ├─> export::ReportGenerator                   │      │
│  │    └─> storage repositories                      │      │
│  └──────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
```

---

## 2. Core Module Structure

### 2.1 File: `crates/llm-cost-ops/src/benchmarks/mod.rs`

**Purpose:** Main entry point for the benchmark framework

```rust
//! Canonical benchmark interface for cost-ops
//!
//! This module provides a standardized interface for benchmarking CostOps
//! operations, compatible with the benchmark aggregation system used across
//! 25 benchmark-target repositories.

pub mod result;
pub mod markdown;
pub mod io;
pub mod adapters;
pub mod runner;

pub use result::BenchmarkResult;
pub use adapters::{BenchTarget, all_targets};

use crate::domain::Result;

/// Execute all registered benchmarks and return results
///
/// This is the main entry point called by the CLI `run` command.
/// It discovers all benchmark targets, executes them, and returns
/// structured results for JSON serialization and report generation.
///
/// # Example
///
/// ```rust
/// use llm_cost_ops::benchmarks::run_all_benchmarks;
///
/// #[tokio::main]
/// async fn main() {
///     let results = run_all_benchmarks().await.unwrap();
///     println!("Executed {} benchmarks", results.len());
/// }
/// ```
pub async fn run_all_benchmarks() -> Result<Vec<BenchmarkResult>> {
    let targets = adapters::all_targets();
    let mut results = Vec::new();

    for target in targets {
        let result = runner::execute(target).await?;
        results.push(result);
    }

    Ok(results)
}

/// Execute benchmarks and write output files
///
/// Convenience function that runs benchmarks and generates both
/// JSON and Markdown outputs in the standard location.
pub async fn run_and_save() -> Result<()> {
    let results = run_all_benchmarks().await?;

    // Write individual JSON files
    for result in &results {
        io::write_json(result)?;
    }

    // Generate summary markdown
    markdown::write_summary(&results)?;

    Ok(())
}
```

---

### 2.2 File: `crates/llm-cost-ops/src/benchmarks/result.rs`

**Purpose:** Canonical BenchmarkResult structure (exact specification)

```rust
//! BenchmarkResult type definition
//!
//! This module defines the exact structure required for cross-repository
//! benchmark aggregation. DO NOT modify this structure without coordinating
//! with all 25 benchmark-target repositories.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Canonical benchmark result structure
///
/// This structure is used across all benchmark-target repositories for
/// consistent result aggregation and analysis.
///
/// **IMPORTANT:** This exact structure is required for compatibility.
/// Fields must match name, type, and serialization format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Unique identifier for the benchmark target
    ///
    /// Format: module_name or operation_name (e.g., "cost_calculator", "forecasting")
    pub target_id: String,

    /// Metrics data in flexible JSON format
    ///
    /// Each target defines its own metrics schema, but common fields include:
    /// - operations_per_second: f64
    /// - avg_duration_ns: u64
    /// - p50_duration_ns: u64
    /// - p95_duration_ns: u64
    /// - p99_duration_ns: u64
    /// - throughput_mb_per_sec: f64 (for I/O operations)
    /// - success_rate: f64 (percentage)
    pub metrics: serde_json::Value,

    /// Timestamp when the benchmark was executed
    ///
    /// Always in UTC for consistent cross-repository comparison
    pub timestamp: DateTime<Utc>,
}

impl BenchmarkResult {
    /// Create a new benchmark result
    pub fn new(target_id: String, metrics: serde_json::Value) -> Self {
        Self {
            target_id,
            metrics,
            timestamp: Utc::now(),
        }
    }

    /// Get a specific metric value as f64
    pub fn get_metric_f64(&self, key: &str) -> Option<f64> {
        self.metrics.get(key)?.as_f64()
    }

    /// Get a specific metric value as u64
    pub fn get_metric_u64(&self, key: &str) -> Option<u64> {
        self.metrics.get(key)?.as_u64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_benchmark_result_serialization() {
        let result = BenchmarkResult::new(
            "test_target".to_string(),
            json!({
                "operations_per_second": 10000.0,
                "avg_duration_ns": 100000,
            }),
        );

        let serialized = serde_json::to_string(&result).unwrap();
        let deserialized: BenchmarkResult = serde_json::from_str(&serialized).unwrap();

        assert_eq!(result.target_id, deserialized.target_id);
        assert_eq!(result.metrics, deserialized.metrics);
    }
}
```

---

### 2.3 File: `crates/llm-cost-ops/src/benchmarks/adapters.rs`

**Purpose:** BenchTarget trait and target implementations

```rust
//! Benchmark target adapters
//!
//! This module defines the BenchTarget trait and provides implementations
//! for each CostOps operation that should be benchmarked.

use async_trait::async_trait;
use std::error::Error;
use serde_json::{json, Value};

/// Trait for benchmark targets
///
/// All benchmarkable operations must implement this trait to be included
/// in the automated benchmark suite.
#[async_trait]
pub trait BenchTarget: Send + Sync {
    /// Unique identifier for this benchmark target
    ///
    /// Should be lowercase with underscores (e.g., "cost_calculator")
    fn id(&self) -> String;

    /// Execute the benchmark and return metrics
    ///
    /// Returns a JSON object containing performance metrics.
    /// Common metrics include:
    /// - operations_per_second
    /// - avg_duration_ns
    /// - p50/p95/p99 latencies
    async fn run(&self) -> Result<Value, Box<dyn Error>>;
}

/// Get all registered benchmark targets
///
/// This function returns all available benchmark targets for the cost-ops
/// repository. Each target corresponds to a major operation or module.
pub fn all_targets() -> Vec<Box<dyn BenchTarget>> {
    vec![
        Box::new(CostCalculatorTarget),
        Box::new(AggregatorTarget),
        Box::new(ForecastingTarget),
        Box::new(CompressionTarget),
        Box::new(ExportTarget),
        Box::new(StorageTarget),
    ]
}

// ============================================================================
// Target Implementations
// ============================================================================

/// Cost calculation benchmark
pub struct CostCalculatorTarget;

#[async_trait]
impl BenchTarget for CostCalculatorTarget {
    fn id(&self) -> String {
        "cost_calculator".to_string()
    }

    async fn run(&self) -> Result<Value, Box<dyn Error>> {
        use crate::domain::*;
        use crate::engine::CostCalculator;
        use rust_decimal_macros::dec;
        use std::time::Instant;

        let calculator = CostCalculator::new();

        // Create test data
        let usage = create_test_usage(10_000, 5_000);
        let pricing = create_test_pricing();

        // Warm-up
        for _ in 0..100 {
            let _ = calculator.calculate(&usage, &pricing);
        }

        // Benchmark
        const ITERATIONS: u64 = 10_000;
        let start = Instant::now();

        for _ in 0..ITERATIONS {
            let _ = calculator.calculate(&usage, &pricing)?;
        }

        let elapsed = start.elapsed();
        let avg_duration_ns = elapsed.as_nanos() as u64 / ITERATIONS;
        let ops_per_second = (ITERATIONS as f64 / elapsed.as_secs_f64()).round();

        Ok(json!({
            "operations_per_second": ops_per_second,
            "avg_duration_ns": avg_duration_ns,
            "total_iterations": ITERATIONS,
            "total_duration_ms": elapsed.as_millis(),
        }))
    }
}

/// Cost aggregation benchmark
pub struct AggregatorTarget;

#[async_trait]
impl BenchTarget for AggregatorTarget {
    fn id(&self) -> String {
        "aggregator".to_string()
    }

    async fn run(&self) -> Result<Value, Box<dyn Error>> {
        use crate::engine::CostAggregator;
        use std::time::Instant;
        use chrono::Utc;

        let aggregator = CostAggregator::new();

        // Create test cost records
        let records = create_test_cost_records(1_000);
        let start_time = Utc::now() - chrono::Duration::days(7);
        let end_time = Utc::now();

        // Warm-up
        for _ in 0..10 {
            let _ = aggregator.aggregate(&records, start_time, end_time);
        }

        // Benchmark
        const ITERATIONS: u64 = 1_000;
        let start = Instant::now();

        for _ in 0..ITERATIONS {
            let _ = aggregator.aggregate(&records, start_time, end_time)?;
        }

        let elapsed = start.elapsed();
        let avg_duration_ns = elapsed.as_nanos() as u64 / ITERATIONS;
        let ops_per_second = (ITERATIONS as f64 / elapsed.as_secs_f64()).round();

        Ok(json!({
            "operations_per_second": ops_per_second,
            "avg_duration_ns": avg_duration_ns,
            "records_per_aggregation": 1_000,
            "total_iterations": ITERATIONS,
        }))
    }
}

/// Forecasting engine benchmark
pub struct ForecastingTarget;

#[async_trait]
impl BenchTarget for ForecastingTarget {
    fn id(&self) -> String {
        "forecasting".to_string()
    }

    async fn run(&self) -> Result<Value, Box<dyn Error>> {
        use crate::forecasting::*;
        use std::time::Instant;

        let engine = ForecastEngine::new();

        // Create test time series data
        let data = create_test_time_series(100);
        let config = ForecastConfig::default();
        let request = ForecastRequest::new(data, config);

        // Warm-up
        for _ in 0..10 {
            let _ = engine.forecast(&request);
        }

        // Benchmark
        const ITERATIONS: u64 = 100;
        let start = Instant::now();

        for _ in 0..ITERATIONS {
            let _ = engine.forecast(&request)?;
        }

        let elapsed = start.elapsed();
        let avg_duration_ns = elapsed.as_nanos() as u64 / ITERATIONS;
        let ops_per_second = (ITERATIONS as f64 / elapsed.as_secs_f64()).round();

        Ok(json!({
            "operations_per_second": ops_per_second,
            "avg_duration_ns": avg_duration_ns,
            "data_points": 100,
            "total_iterations": ITERATIONS,
        }))
    }
}

/// Compression benchmark
pub struct CompressionTarget;

#[async_trait]
impl BenchTarget for CompressionTarget {
    fn id(&self) -> String {
        "compression".to_string()
    }

    async fn run(&self) -> Result<Value, Box<dyn Error>> {
        use crate::compression::*;
        use std::time::Instant;

        let compressor = Compressor::new(CompressionConfig::default());

        // Create test data (1MB of JSON)
        let test_data = create_test_json_data(1_000_000);

        // Benchmark compression
        const ITERATIONS: u64 = 100;
        let start = Instant::now();

        for _ in 0..ITERATIONS {
            let _ = compress(&test_data, CompressionAlgorithm::Gzip)?;
        }

        let elapsed = start.elapsed();
        let avg_duration_ns = elapsed.as_nanos() as u64 / ITERATIONS;
        let ops_per_second = (ITERATIONS as f64 / elapsed.as_secs_f64()).round();
        let throughput_mb_per_sec = (test_data.len() as f64 * ITERATIONS as f64)
            / (elapsed.as_secs_f64() * 1_000_000.0);

        Ok(json!({
            "operations_per_second": ops_per_second,
            "avg_duration_ns": avg_duration_ns,
            "throughput_mb_per_sec": throughput_mb_per_sec,
            "uncompressed_size_bytes": test_data.len(),
            "total_iterations": ITERATIONS,
        }))
    }
}

/// Export/Report generation benchmark
pub struct ExportTarget;

#[async_trait]
impl BenchTarget for ExportTarget {
    fn id(&self) -> String {
        "export".to_string()
    }

    async fn run(&self) -> Result<Value, Box<dyn Error>> {
        use crate::export::*;
        use std::time::Instant;

        let generator = ReportGenerator::new();

        // Create test report request
        let request = create_test_report_request();

        // Warm-up
        for _ in 0..10 {
            let _ = generator.generate(&request);
        }

        // Benchmark
        const ITERATIONS: u64 = 100;
        let start = Instant::now();

        for _ in 0..ITERATIONS {
            let _ = generator.generate(&request)?;
        }

        let elapsed = start.elapsed();
        let avg_duration_ns = elapsed.as_nanos() as u64 / ITERATIONS;
        let ops_per_second = (ITERATIONS as f64 / elapsed.as_secs_f64()).round();

        Ok(json!({
            "operations_per_second": ops_per_second,
            "avg_duration_ns": avg_duration_ns,
            "total_iterations": ITERATIONS,
        }))
    }
}

/// Storage operations benchmark
pub struct StorageTarget;

#[async_trait]
impl BenchTarget for StorageTarget {
    fn id(&self) -> String {
        "storage".to_string()
    }

    async fn run(&self) -> Result<Value, Box<dyn Error>> {
        use std::time::Instant;
        use sqlx::sqlite::SqlitePool;
        use crate::storage::*;

        // Use in-memory SQLite for benchmarking
        let pool = SqlitePool::connect(":memory:").await?;

        // Initialize schema (skipped for now - would need migrations)
        // sqlx::migrate!().run(&pool).await?;

        let usage_repo = SqliteUsageRepository::new(pool.clone());

        // Create test usage record
        let usage = create_test_usage(1000, 500);

        // Benchmark
        const ITERATIONS: u64 = 100;
        let start = Instant::now();

        for _ in 0..ITERATIONS {
            let _ = usage_repo.create(&usage).await?;
        }

        let elapsed = start.elapsed();
        let avg_duration_ns = elapsed.as_nanos() as u64 / ITERATIONS;
        let ops_per_second = (ITERATIONS as f64 / elapsed.as_secs_f64()).round();

        Ok(json!({
            "operations_per_second": ops_per_second,
            "avg_duration_ns": avg_duration_ns,
            "total_iterations": ITERATIONS,
            "note": "In-memory SQLite - production performance may vary",
        }))
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_usage(prompt_tokens: u64, completion_tokens: u64) -> crate::domain::UsageRecord {
    use crate::domain::*;
    use uuid::Uuid;

    UsageRecord {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier::new("gpt-4".to_string(), 8192),
        organization_id: "bench-org".to_string(),
        project_id: None,
        user_id: None,
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: Some(100),
        tags: vec![],
        metadata: serde_json::json!({}),
        ingested_at: chrono::Utc::now(),
        source: IngestionSource {
            source_type: "benchmark".to_string(),
            endpoint: None,
        },
    }
}

fn create_test_pricing() -> crate::domain::PricingTable {
    use crate::domain::*;
    use rust_decimal_macros::dec;

    PricingTable::new(
        Provider::OpenAI,
        "gpt-4".to_string(),
        PricingStructure::simple_per_token(dec!(10.0), dec!(30.0)),
    )
}

fn create_test_cost_records(count: usize) -> Vec<crate::domain::CostRecord> {
    use crate::domain::*;
    use uuid::Uuid;
    use rust_decimal_macros::dec;

    (0..count)
        .map(|_| {
            CostRecord::new(
                Uuid::new_v4(),
                Provider::OpenAI,
                "gpt-4".to_string(),
                "bench-org".to_string(),
                CostCalculation::new(dec!(0.01), dec!(0.02), Currency::USD, Uuid::new_v4()),
                PricingStructure::simple_per_token(dec!(10.0), dec!(30.0)),
            )
        })
        .collect()
}

fn create_test_time_series(points: usize) -> crate::forecasting::TimeSeriesData {
    use crate::forecasting::*;
    use chrono::Utc;
    use rust_decimal::Decimal;

    let data_points = (0..points)
        .map(|i| {
            DataPoint {
                timestamp: Utc::now() - chrono::Duration::hours(points as i64 - i as i64),
                value: Decimal::from(100 + i * 2),
            }
        })
        .collect();

    TimeSeriesData { data_points }
}

fn create_test_json_data(size: usize) -> Vec<u8> {
    use serde_json::json;

    // Create JSON object that will be approximately `size` bytes
    let records: Vec<_> = (0..size / 100)
        .map(|i| {
            json!({
                "id": i,
                "provider": "OpenAI",
                "model": "gpt-4",
                "cost": 0.123456,
            })
        })
        .collect();

    serde_json::to_vec(&records).unwrap()
}

fn create_test_report_request() -> crate::export::ReportRequest {
    use crate::export::*;
    use chrono::Utc;

    ReportRequest {
        report_type: ReportType::Cost,
        filters: ReportFilters {
            date_range: DateRange {
                start: Utc::now() - chrono::Duration::days(7),
                end: Utc::now(),
            },
            organization_id: Some("bench-org".to_string()),
            provider: None,
            model: None,
            project_id: None,
        },
        format: ExportFormat::Json,
    }
}
```

---

### 2.4 File: `crates/llm-cost-ops/src/benchmarks/markdown.rs`

**Purpose:** Generate human-readable Markdown summary reports

```rust
//! Markdown report generation
//!
//! Generates human-readable summary reports in Markdown format from
//! benchmark results.

use crate::benchmarks::BenchmarkResult;
use crate::domain::Result;
use std::fs;
use std::path::Path;

/// Generate a Markdown summary from benchmark results
pub fn generate_summary(results: &[BenchmarkResult]) -> String {
    let mut md = String::new();

    // Header
    md.push_str("# LLM-CostOps Benchmark Results\n\n");
    md.push_str(&format!("**Generated:** {}\n\n", chrono::Utc::now().to_rfc3339()));
    md.push_str(&format!("**Total Benchmarks:** {}\n\n", results.len()));

    // Summary table
    md.push_str("## Performance Summary\n\n");
    md.push_str("| Target | Operations/sec | Avg Duration (ns) | Notes |\n");
    md.push_str("|--------|----------------|-------------------|-------|\n");

    for result in results {
        let ops_per_sec = result.get_metric_f64("operations_per_second")
            .map(|v| format!("{:.0}", v))
            .unwrap_or_else(|| "N/A".to_string());

        let avg_duration = result.get_metric_u64("avg_duration_ns")
            .map(|v| format!("{}", v))
            .unwrap_or_else(|| "N/A".to_string());

        md.push_str(&format!(
            "| {} | {} | {} | - |\n",
            result.target_id,
            ops_per_sec,
            avg_duration
        ));
    }

    md.push_str("\n");

    // Detailed results
    md.push_str("## Detailed Results\n\n");

    for result in results {
        md.push_str(&format!("### {}\n\n", result.target_id));
        md.push_str("```json\n");
        md.push_str(&serde_json::to_string_pretty(&result.metrics).unwrap());
        md.push_str("\n```\n\n");
    }

    // Footer
    md.push_str("---\n");
    md.push_str(&format!(
        "*Generated by llm-cost-ops benchmark suite v{}*\n",
        crate::VERSION
    ));

    md
}

/// Write Markdown summary to file
pub fn write_summary(results: &[BenchmarkResult]) -> Result<()> {
    let output_path = Path::new("benchmarks/output/summary.md");

    // Ensure directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let summary = generate_summary(results);
    fs::write(output_path, summary)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_generate_summary() {
        let results = vec![
            BenchmarkResult::new(
                "test_target".to_string(),
                json!({
                    "operations_per_second": 10000.0,
                    "avg_duration_ns": 100000,
                }),
            ),
        ];

        let summary = generate_summary(&results);

        assert!(summary.contains("# LLM-CostOps Benchmark Results"));
        assert!(summary.contains("test_target"));
        assert!(summary.contains("10000"));
    }
}
```

---

### 2.5 File: `crates/llm-cost-ops/src/benchmarks/io.rs`

**Purpose:** File I/O operations for benchmark results

```rust
//! File I/O for benchmark results
//!
//! Handles reading and writing benchmark results to/from disk in JSON format.

use crate::benchmarks::BenchmarkResult;
use crate::domain::Result;
use std::fs;
use std::path::Path;

/// Write a benchmark result to JSON file
///
/// Writes to `benchmarks/output/raw/{target_id}.json`
pub fn write_json(result: &BenchmarkResult) -> Result<()> {
    let filename = format!("{}.json", result.target_id);
    let output_path = Path::new("benchmarks/output/raw").join(filename);

    // Ensure directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let json = serde_json::to_string_pretty(result)?;
    fs::write(&output_path, json)?;

    Ok(())
}

/// Read a benchmark result from JSON file
pub fn read_json(target_id: &str) -> Result<BenchmarkResult> {
    let filename = format!("{}.json", target_id);
    let input_path = Path::new("benchmarks/output/raw").join(filename);

    let contents = fs::read_to_string(input_path)?;
    let result = serde_json::from_str(&contents)?;

    Ok(result)
}

/// Read all benchmark results from the output directory
pub fn read_all() -> Result<Vec<BenchmarkResult>> {
    let dir_path = Path::new("benchmarks/output/raw");

    if !dir_path.exists() {
        return Ok(Vec::new());
    }

    let mut results = Vec::new();

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            let contents = fs::read_to_string(&path)?;
            let result: BenchmarkResult = serde_json::from_str(&contents)?;
            results.push(result);
        }
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_write_and_read_json() {
        let temp_dir = TempDir::new().unwrap();
        let old_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let result = BenchmarkResult::new(
            "test".to_string(),
            json!({"operations_per_second": 10000.0}),
        );

        write_json(&result).unwrap();
        let read_result = read_json("test").unwrap();

        assert_eq!(result.target_id, read_result.target_id);
        assert_eq!(result.metrics, read_result.metrics);

        std::env::set_current_dir(old_dir).unwrap();
    }
}
```

---

### 2.6 File: `crates/llm-cost-ops/src/benchmarks/runner.rs`

**Purpose:** Benchmark execution engine

```rust
//! Benchmark execution engine
//!
//! Provides the runtime for executing benchmark targets and collecting results.

use crate::benchmarks::{BenchTarget, BenchmarkResult};
use crate::domain::Result;
use tracing::{info, warn};

/// Execute a single benchmark target
///
/// Runs the benchmark, captures metrics, and returns a structured result.
pub async fn execute(target: Box<dyn BenchTarget>) -> Result<BenchmarkResult> {
    let target_id = target.id();

    info!("Starting benchmark: {}", target_id);

    match target.run().await {
        Ok(metrics) => {
            info!("Benchmark completed: {}", target_id);
            Ok(BenchmarkResult::new(target_id, metrics))
        }
        Err(e) => {
            warn!("Benchmark failed: {} - {}", target_id, e);
            Err(crate::domain::CostOpsError::internal(format!(
                "Benchmark '{}' failed: {}",
                target_id, e
            )))
        }
    }
}

/// Execute multiple benchmark targets in sequence
pub async fn execute_all(targets: Vec<Box<dyn BenchTarget>>) -> Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    for target in targets {
        let result = execute(target).await?;
        results.push(result);
    }

    Ok(results)
}
```

---

## 3. Interface Definitions

### 3.1 Public API Surface

```rust
// Re-exported from lib.rs
pub use benchmarks::{
    // Core types
    BenchmarkResult,
    BenchTarget,

    // Main entry points
    run_all_benchmarks,
    run_and_save,

    // Target registry
    all_targets,
};
```

### 3.2 Trait Contract

```rust
#[async_trait]
pub trait BenchTarget: Send + Sync {
    fn id(&self) -> String;
    async fn run(&self) -> Result<serde_json::Value, Box<dyn std::error::Error>>;
}
```

**Contract Requirements:**
- `id()` must return a unique, stable identifier (lowercase, underscores)
- `run()` must be idempotent and thread-safe
- Metrics JSON should include at least `operations_per_second` or `avg_duration_ns`
- Benchmarks should include warmup iterations before measurement

---

## 4. Benchmark Target Implementations

### 4.1 Target Catalog

| Target ID | Module | Description | Key Metrics |
|-----------|--------|-------------|-------------|
| `cost_calculator` | `engine::CostCalculator` | Single cost calculation | ops/sec, avg_duration_ns |
| `aggregator` | `engine::CostAggregator` | Cost aggregation over 1K records | ops/sec, records_per_aggregation |
| `forecasting` | `forecasting::ForecastEngine` | Time series forecasting | ops/sec, data_points |
| `compression` | `compression::Compressor` | Data compression (gzip) | ops/sec, throughput_mb_per_sec |
| `export` | `export::ReportGenerator` | Report generation | ops/sec, avg_duration_ns |
| `storage` | `storage::*Repository` | Database operations | ops/sec, avg_duration_ns |

### 4.2 Implementation Guidelines

Each target should:
1. **Warm-up phase:** Run 10-100 iterations before measurement
2. **Measurement phase:** Run sufficient iterations for statistical significance (≥100)
3. **Consistent test data:** Use deterministic, representative inputs
4. **Isolation:** Avoid external dependencies (use in-memory databases, etc.)
5. **Metrics:** Return standardized JSON with documented fields

---

## 5. Integration Points

### 5.1 Library Integration

**File:** `crates/llm-cost-ops/src/lib.rs`

```rust
pub mod benchmarks;  // NEW

pub use benchmarks::{
    BenchmarkResult,
    BenchTarget,
    run_all_benchmarks,
    run_and_save,
};
```

**Impact:** Zero breaking changes. Purely additive API.

### 5.2 Existing Module Integration

**No changes required** to existing modules:
- `domain/*` - Used as-is by benchmark adapters
- `engine/*` - Benchmarked through public API
- `storage/*` - Benchmarked through repository interfaces
- `forecasting/*` - Benchmarked through ForecastEngine
- `compression/*` - Benchmarked through Compressor
- `export/*` - Benchmarked through ReportGenerator

---

## 6. CLI Integration

### 6.1 Command Definition

**File:** `crates/llm-cost-ops-cli/src/cli/mod.rs`

```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands ...

    /// Run performance benchmarks
    Run {
        /// Output directory for results
        #[arg(short, long, default_value = "benchmarks/output")]
        output: PathBuf,

        /// Generate markdown summary
        #[arg(long, default_value = "true")]
        summary: bool,

        /// Specific benchmark targets to run (comma-separated)
        #[arg(long)]
        targets: Option<String>,
    },
}
```

### 6.2 Command Handler

**File:** `crates/llm-cost-ops-cli/src/bin/main.rs`

```rust
async fn main() -> Result<()> {
    // ... existing setup ...

    match &cli.command {
        // ... existing commands ...

        Commands::Run { output, summary, targets } => {
            handle_run_command(output, *summary, targets.as_deref()).await?;
        }
    }

    Ok(())
}

async fn handle_run_command(
    output_dir: &Path,
    generate_summary: bool,
    target_filter: Option<&str>,
) -> Result<()> {
    use llm_cost_ops::benchmarks;

    info!("Running benchmarks...");

    // Get all targets or filter by name
    let all_targets = benchmarks::all_targets();
    let targets = if let Some(filter) = target_filter {
        let filter_set: HashSet<&str> = filter.split(',').collect();
        all_targets
            .into_iter()
            .filter(|t| filter_set.contains(t.id().as_str()))
            .collect()
    } else {
        all_targets
    };

    info!("Executing {} benchmarks", targets.len());

    // Run benchmarks
    let results = benchmarks::runner::execute_all(targets).await?;

    // Write JSON results
    for result in &results {
        benchmarks::io::write_json(result)?;
        info!("✓ {} - {:.0} ops/sec",
            result.target_id,
            result.get_metric_f64("operations_per_second").unwrap_or(0.0)
        );
    }

    // Generate summary
    if generate_summary {
        benchmarks::markdown::write_summary(&results)?;
        info!("✓ Summary written to benchmarks/output/summary.md");
    }

    info!("Benchmark suite completed successfully");
    Ok(())
}
```

### 6.3 Usage Examples

```bash
# Run all benchmarks
$ cost-ops run

# Run specific benchmarks
$ cost-ops run --targets cost_calculator,aggregator

# Custom output directory
$ cost-ops run --output /tmp/bench-results

# Skip summary generation
$ cost-ops run --summary false
```

---

## 7. Output Structure

### 7.1 Directory Layout

```
benchmarks/
├── output/
│   ├── raw/
│   │   ├── cost_calculator.json
│   │   ├── aggregator.json
│   │   ├── forecasting.json
│   │   ├── compression.json
│   │   ├── export.json
│   │   └── storage.json
│   └── summary.md
└── README.md
```

### 7.2 JSON Output Format

**File:** `benchmarks/output/raw/cost_calculator.json`

```json
{
  "target_id": "cost_calculator",
  "metrics": {
    "operations_per_second": 125000.0,
    "avg_duration_ns": 8000,
    "total_iterations": 10000,
    "total_duration_ms": 80
  },
  "timestamp": "2025-12-02T12:34:56.789Z"
}
```

### 7.3 Markdown Summary Format

**File:** `benchmarks/output/summary.md`

```markdown
# LLM-CostOps Benchmark Results

**Generated:** 2025-12-02T12:34:56.789Z

**Total Benchmarks:** 6

## Performance Summary

| Target | Operations/sec | Avg Duration (ns) | Notes |
|--------|----------------|-------------------|-------|
| cost_calculator | 125000 | 8000 | - |
| aggregator | 5000 | 200000 | - |
| forecasting | 500 | 2000000 | - |
| compression | 100 | 10000000 | - |
| export | 200 | 5000000 | - |
| storage | 1000 | 1000000 | - |

## Detailed Results

### cost_calculator

```json
{
  "operations_per_second": 125000.0,
  "avg_duration_ns": 8000,
  "total_iterations": 10000
}
```

...

---
*Generated by llm-cost-ops benchmark suite v0.1.1*
```

---

## 8. Migration Path

### 8.1 Phase 1: Core Infrastructure (Week 1)

**Goal:** Establish benchmark framework without disrupting existing code

**Tasks:**
1. Create `crates/llm-cost-ops/src/benchmarks/` directory
2. Implement core modules:
   - `result.rs` - BenchmarkResult struct
   - `adapters.rs` - BenchTarget trait (empty implementations)
   - `io.rs` - JSON I/O functions
   - `markdown.rs` - Summary generation
   - `runner.rs` - Execution engine
   - `mod.rs` - Module orchestration
3. Add re-exports to `lib.rs`
4. Write unit tests for each module

**Validation:**
- All existing tests pass
- No breaking changes to public API
- Documentation builds successfully

### 8.2 Phase 2: Target Implementations (Week 2)

**Goal:** Implement benchmark adapters for each CostOps operation

**Tasks:**
1. Implement `CostCalculatorTarget`
2. Implement `AggregatorTarget`
3. Implement `ForecastingTarget`
4. Implement `CompressionTarget`
5. Implement `ExportTarget`
6. Implement `StorageTarget`
7. Add helper functions for test data generation
8. Validate metrics schema consistency

**Validation:**
- Each target runs successfully
- Metrics JSON validates against schema
- Performance numbers are reasonable

### 8.3 Phase 3: CLI Integration (Week 3)

**Goal:** Add `run` command to CLI

**Tasks:**
1. Add `Run` variant to `Commands` enum
2. Implement `handle_run_command()` in main.rs
3. Add target filtering logic
4. Implement progress indicators
5. Add error handling and logging
6. Write integration tests

**Validation:**
- `cost-ops run` executes successfully
- Output files are created correctly
- Summary markdown is readable
- Error cases are handled gracefully

### 8.4 Phase 4: Documentation & Release (Week 4)

**Goal:** Document and release the benchmark framework

**Tasks:**
1. Create `benchmarks/README.md` with usage guide
2. Update main README.md with benchmark section
3. Add benchmark CI/CD workflow
4. Create example benchmark comparison scripts
5. Write blog post / announcement
6. Version bump and release

**Validation:**
- Documentation is complete and accurate
- CI pipeline runs benchmarks on each PR
- Benchmark results are published
- Community can contribute new targets

---

## 9. Implementation Roadmap

### 9.1 Development Timeline

```
Week 1: Core Infrastructure
├─ Day 1-2: result.rs, io.rs, markdown.rs
├─ Day 3-4: adapters.rs (trait + helpers), runner.rs
└─ Day 5: mod.rs, lib.rs integration, tests

Week 2: Target Implementations
├─ Day 1: CostCalculatorTarget, AggregatorTarget
├─ Day 2: ForecastingTarget
├─ Day 3: CompressionTarget, ExportTarget
├─ Day 4: StorageTarget
└─ Day 5: Integration tests, metric validation

Week 3: CLI Integration
├─ Day 1-2: CLI command definition and parsing
├─ Day 3: Command handler implementation
├─ Day 4: Error handling, progress indicators
└─ Day 5: Integration tests, manual testing

Week 4: Documentation & Release
├─ Day 1-2: Documentation writing
├─ Day 3: CI/CD pipeline setup
├─ Day 4: Example scripts, blog post
└─ Day 5: Release prep, announcement
```

### 9.2 Success Criteria

- ✅ Zero breaking changes to existing code
- ✅ All existing tests pass
- ✅ New benchmark suite runs successfully
- ✅ JSON output validates against schema
- ✅ Markdown summary is human-readable
- ✅ CLI integration works seamlessly
- ✅ Documentation is complete
- ✅ CI/CD pipeline is operational
- ✅ Compatible with 25 benchmark-target repositories

### 9.3 Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking existing code | Purely additive changes; comprehensive test coverage |
| Performance regression | Benchmarks are opt-in; don't run in normal tests |
| Incompatible metrics | Strict schema validation; documentation |
| CLI conflicts | New command namespace; no overlap with existing |
| Dependency issues | Use existing workspace dependencies only |

---

## Appendix A: File Checklist

**New Files to Create:**

```
✅ crates/llm-cost-ops/src/benchmarks/mod.rs
✅ crates/llm-cost-ops/src/benchmarks/result.rs
✅ crates/llm-cost-ops/src/benchmarks/markdown.rs
✅ crates/llm-cost-ops/src/benchmarks/io.rs
✅ crates/llm-cost-ops/src/benchmarks/adapters.rs
✅ crates/llm-cost-ops/src/benchmarks/runner.rs
✅ benchmarks/output/.gitkeep
✅ benchmarks/output/raw/.gitkeep
✅ benchmarks/README.md
```

**Files to Modify:**

```
✅ crates/llm-cost-ops/src/lib.rs (add module re-export)
✅ crates/llm-cost-ops-cli/src/cli/mod.rs (add Run command)
✅ crates/llm-cost-ops-cli/src/bin/main.rs (add command handler)
✅ README.md (add benchmark section)
✅ .gitignore (add benchmarks/output/*)
```

**Files Unchanged:**

```
✅ All domain modules (domain/*)
✅ All engine modules (engine/*)
✅ All storage modules (storage/*)
✅ All forecasting modules (forecasting/*)
✅ All compression modules (compression/*)
✅ All export modules (export/*)
✅ Existing benchmark files (benches/*)
✅ All tests (tests/*)
```

---

## Appendix B: Compatibility Matrix

| Repository | Interface Version | Status |
|------------|-------------------|--------|
| cost-ops (this) | 1.0 | ✅ Primary |
| benchmark-target-01 | 1.0 | Compatible |
| benchmark-target-02 | 1.0 | Compatible |
| ... | 1.0 | Compatible |
| benchmark-target-25 | 1.0 | Compatible |

**Interface Versioning:**
- Version 1.0 = Current BenchmarkResult structure
- Breaking changes require major version bump
- All 25 repositories must coordinate on upgrades

---

## Appendix C: Example Workflow

```bash
# Developer workflow
$ cd cost-ops

# Run all benchmarks
$ cargo build --release
$ ./target/release/cost-ops run

# Output:
# Running benchmarks...
# Executing 6 benchmarks
# Starting benchmark: cost_calculator
# Benchmark completed: cost_calculator
# ✓ cost_calculator - 125000 ops/sec
# ...
# ✓ Summary written to benchmarks/output/summary.md
# Benchmark suite completed successfully

# View results
$ cat benchmarks/output/summary.md
$ cat benchmarks/output/raw/cost_calculator.json

# Run specific benchmarks
$ ./target/release/cost-ops run --targets cost_calculator,aggregator

# CI/CD integration
$ cargo test
$ cargo run --bin cost-ops -- run --output=./ci-bench-results
```

---

## Conclusion

This architecture provides a **canonical, standardized benchmark interface** for the cost-ops repository that:

1. **Maintains backward compatibility** - Zero breaking changes to existing code
2. **Follows best practices** - Clean separation of concerns, testable components
3. **Scales to 25 repositories** - Standardized JSON format for aggregation
4. **Enables comprehensive performance testing** - Covers all major CostOps operations
5. **Integrates seamlessly with CLI** - Simple, intuitive command interface
6. **Generates actionable reports** - Both machine-readable JSON and human-readable Markdown

The implementation roadmap provides a clear, phased approach to delivery with defined success criteria and risk mitigation strategies.

**Next Steps:**
1. Review and approve this specification
2. Begin Phase 1 implementation (Core Infrastructure)
3. Conduct code reviews at each phase boundary
4. Deploy to production once all phases complete

---

**Document End**
