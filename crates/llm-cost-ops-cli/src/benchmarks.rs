//! Benchmark execution and reporting module
//!
//! This module provides functionality to run performance benchmarks
//! and generate comprehensive reports.

use anyhow::{Context, Result};
use chrono::Utc;
use llm_cost_ops::{
    domain::{CostRecord, ModelIdentifier, PricingStructure, PricingTable, Provider, UsageRecord},
    engine::{CostAggregator, CostCalculator},
};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::str::FromStr;
use std::time::{Duration, Instant};
use tracing::{info, warn};
use uuid::Uuid;

/// Benchmark result for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub name: String,
    pub iterations: usize,
    pub total_duration_ms: f64,
    pub avg_duration_us: f64,
    pub min_duration_us: f64,
    pub max_duration_us: f64,
    pub throughput_per_sec: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Collection of all benchmark results
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub results: Vec<BenchmarkResult>,
    pub summary: BenchmarkSummary,
}

/// Summary statistics across all benchmarks
#[derive(Debug, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub total_benchmarks: usize,
    pub total_iterations: usize,
    pub total_duration_ms: f64,
    pub fastest_benchmark: String,
    pub slowest_benchmark: String,
}

/// Run all benchmarks and generate reports
pub async fn run_all_benchmarks(
    output_dir: &Path,
    generate_summary: bool,
    filter: Option<&str>,
) -> Result<()> {
    info!("Starting benchmark suite");

    // Create output directories
    let raw_dir = output_dir.join("raw");
    tokio::fs::create_dir_all(&raw_dir)
        .await
        .context("Failed to create raw output directory")?;

    let mut results = Vec::new();
    let start_time = Instant::now();

    // Run individual benchmarks
    if should_run_benchmark("single_cost_calculation", filter) {
        results.push(bench_single_cost_calculation().await?);
    }

    if should_run_benchmark("batch_cost_calculation", filter) {
        results.extend(bench_batch_cost_calculation().await?);
    }

    if should_run_benchmark("cached_token_calculation", filter) {
        results.push(bench_cached_token_calculation().await?);
    }

    if should_run_benchmark("cost_aggregation", filter) {
        results.extend(bench_cost_aggregation().await?);
    }

    if should_run_benchmark("usage_validation", filter) {
        results.push(bench_usage_validation().await?);
    }

    if should_run_benchmark("multi_provider", filter) {
        results.extend(bench_multi_provider().await?);
    }

    let total_duration = start_time.elapsed();

    // Create summary
    let summary = create_summary(&results, total_duration);

    let suite = BenchmarkSuite {
        version: llm_cost_ops::VERSION.to_string(),
        timestamp: Utc::now(),
        results: results.clone(),
        summary,
    };

    // Write JSON results
    let json_output = raw_dir.join(format!("benchmark-results-{}.json",
        Utc::now().format("%Y%m%d-%H%M%S")));
    let json = serde_json::to_string_pretty(&suite)?;
    tokio::fs::write(&json_output, json)
        .await
        .context("Failed to write JSON results")?;

    info!("Benchmark results written to: {:?}", json_output);

    // Generate summary markdown if requested
    if generate_summary {
        let summary_path = output_dir.join("summary.md");
        let markdown = generate_markdown_summary(&suite)?;
        tokio::fs::write(&summary_path, markdown)
            .await
            .context("Failed to write summary markdown")?;

        info!("Summary report written to: {:?}", summary_path);
    }

    // Print summary to stdout
    print_summary(&suite);

    Ok(())
}

fn should_run_benchmark(name: &str, filter: Option<&str>) -> bool {
    if let Some(f) = filter {
        name.contains(f)
    } else {
        true
    }
}

async fn bench_single_cost_calculation() -> Result<BenchmarkResult> {
    let name = "single_cost_calculation";
    info!("Running benchmark: {}", name);

    let calculator = CostCalculator::new();
    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0")?,
        Decimal::from_str("30.0")?,
    );
    let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);
    let usage = create_test_usage(1000, 500);

    let iterations = 10_000;
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let _result = calculator.calculate(&usage, &pricing_table)?;
        durations.push(start.elapsed());
    }

    Ok(calculate_result(name.to_string(), durations))
}

async fn bench_batch_cost_calculation() -> Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    for size in [100, 1_000, 10_000] {
        let name = format!("batch_cost_calculation_{}", size);
        info!("Running benchmark: {}", name);

        let calculator = CostCalculator::new();
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_str("10.0")?,
            Decimal::from_str("30.0")?,
        );
        let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);
        let usage_records: Vec<_> = (0..size).map(|_| create_test_usage(1000, 500)).collect();

        let iterations = std::cmp::max(100, 10_000 / size);
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            for usage in &usage_records {
                let _result = calculator.calculate(usage, &pricing_table)?;
            }
            durations.push(start.elapsed());
        }

        results.push(calculate_result(name, durations));
    }

    Ok(results)
}

async fn bench_cached_token_calculation() -> Result<BenchmarkResult> {
    let name = "cached_token_calculation";
    info!("Running benchmark: {}", name);

    let calculator = CostCalculator::new();
    let pricing = PricingStructure::with_cache_discount(
        Decimal::from_str("10.0")?,
        Decimal::from_str("30.0")?,
        Decimal::from_str("0.9")?,
    );
    let pricing_table = PricingTable::new(Provider::Anthropic, "claude-3".to_string(), pricing);

    let mut usage = create_test_usage(5000, 2000);
    usage.cached_tokens = Some(2000);

    let iterations = 10_000;
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let _result = calculator.calculate(&usage, &pricing_table)?;
        durations.push(start.elapsed());
    }

    Ok(calculate_result(name.to_string(), durations))
}

async fn bench_cost_aggregation() -> Result<Vec<BenchmarkResult>> {
    let mut results = Vec::new();

    for size in [100, 1_000, 10_000] {
        let name = format!("cost_aggregation_{}", size);
        info!("Running benchmark: {}", name);

        let calculator = CostCalculator::new();
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_str("10.0")?,
            Decimal::from_str("30.0")?,
        );
        let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);

        let cost_records: Vec<_> = (0..size)
            .map(|_| {
                let usage = create_test_usage(1000, 500);
                calculator.calculate(&usage, &pricing_table).unwrap()
            })
            .collect();

        let aggregator = CostAggregator::new();
        let start = Utc::now() - chrono::Duration::days(7);
        let end = Utc::now();

        let iterations = std::cmp::max(100, 10_000 / size);
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start_time = Instant::now();
            let _result = aggregator.aggregate(&cost_records, start, end)?;
            durations.push(start_time.elapsed());
        }

        results.push(calculate_result(name, durations));
    }

    Ok(results)
}

async fn bench_usage_validation() -> Result<BenchmarkResult> {
    let name = "usage_validation";
    info!("Running benchmark: {}", name);

    let usage = create_test_usage(1000, 500);
    let iterations = 10_000;
    let mut durations = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let _result = usage.validate()?;
        durations.push(start.elapsed());
    }

    Ok(calculate_result(name.to_string(), durations))
}

async fn bench_multi_provider() -> Result<Vec<BenchmarkResult>> {
    let calculator = CostCalculator::new();
    let mut results = Vec::new();

    let providers = vec![
        (Provider::OpenAI, "gpt-4"),
        (Provider::Anthropic, "claude-3"),
        (Provider::GoogleVertexAI, "gemini-pro"),
        (Provider::AzureOpenAI, "gpt-4"),
    ];

    for (provider, model) in providers {
        let name = format!("multi_provider_{}", provider.to_string().to_lowercase());
        info!("Running benchmark: {}", name);

        let pricing = PricingStructure::simple_per_token(
            Decimal::from_str("10.0")?,
            Decimal::from_str("30.0")?,
        );
        let pricing_table = PricingTable::new(provider, model.to_string(), pricing);
        let usage = create_test_usage(1000, 500);

        let iterations = 10_000;
        let mut durations = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            let _result = calculator.calculate(&usage, &pricing_table)?;
            durations.push(start.elapsed());
        }

        results.push(calculate_result(name, durations));
    }

    Ok(results)
}

fn calculate_result(name: String, durations: Vec<Duration>) -> BenchmarkResult {
    let iterations = durations.len();
    let total_duration: Duration = durations.iter().sum();
    let min_duration = durations.iter().min().unwrap();
    let max_duration = durations.iter().max().unwrap();

    let total_duration_ms = total_duration.as_secs_f64() * 1000.0;
    let avg_duration_us = (total_duration.as_secs_f64() * 1_000_000.0) / iterations as f64;
    let throughput_per_sec = if total_duration.as_secs_f64() > 0.0 {
        iterations as f64 / total_duration.as_secs_f64()
    } else {
        0.0
    };

    BenchmarkResult {
        name,
        iterations,
        total_duration_ms,
        avg_duration_us,
        min_duration_us: min_duration.as_secs_f64() * 1_000_000.0,
        max_duration_us: max_duration.as_secs_f64() * 1_000_000.0,
        throughput_per_sec,
        timestamp: Utc::now(),
    }
}

fn create_summary(results: &[BenchmarkResult], total_duration: Duration) -> BenchmarkSummary {
    let total_iterations: usize = results.iter().map(|r| r.iterations).sum();
    let fastest = results.iter().min_by(|a, b| {
        a.avg_duration_us.partial_cmp(&b.avg_duration_us).unwrap()
    });
    let slowest = results.iter().max_by(|a, b| {
        a.avg_duration_us.partial_cmp(&b.avg_duration_us).unwrap()
    });

    BenchmarkSummary {
        total_benchmarks: results.len(),
        total_iterations,
        total_duration_ms: total_duration.as_secs_f64() * 1000.0,
        fastest_benchmark: fastest.map(|r| r.name.clone()).unwrap_or_default(),
        slowest_benchmark: slowest.map(|r| r.name.clone()).unwrap_or_default(),
    }
}

fn generate_markdown_summary(suite: &BenchmarkSuite) -> Result<String> {
    let mut md = String::new();

    md.push_str("# Benchmark Results Summary\n\n");
    md.push_str(&format!("**Generated:** {}\n", suite.timestamp.format("%Y-%m-%d %H:%M:%S UTC")));
    md.push_str(&format!("**Version:** {}\n\n", suite.version));

    md.push_str("## Overall Statistics\n\n");
    md.push_str(&format!("- Total Benchmarks: {}\n", suite.summary.total_benchmarks));
    md.push_str(&format!("- Total Iterations: {}\n", suite.summary.total_iterations));
    md.push_str(&format!("- Total Duration: {:.2} ms\n", suite.summary.total_duration_ms));
    md.push_str(&format!("- Fastest Benchmark: {}\n", suite.summary.fastest_benchmark));
    md.push_str(&format!("- Slowest Benchmark: {}\n\n", suite.summary.slowest_benchmark));

    md.push_str("## Detailed Results\n\n");
    md.push_str("| Benchmark | Iterations | Avg (μs) | Min (μs) | Max (μs) | Throughput (ops/sec) |\n");
    md.push_str("|-----------|------------|----------|----------|----------|---------------------|\n");

    for result in &suite.results {
        md.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {:.2} | {:.0} |\n",
            result.name,
            result.iterations,
            result.avg_duration_us,
            result.min_duration_us,
            result.max_duration_us,
            result.throughput_per_sec
        ));
    }

    md.push_str("\n## Performance Analysis\n\n");

    // Group by benchmark type
    let mut by_type: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
    for result in &suite.results {
        let prefix = result.name.split('_').take(2).collect::<Vec<_>>().join("_");
        by_type.entry(prefix).or_default().push(result);
    }

    for (bench_type, results) in by_type {
        md.push_str(&format!("### {}\n\n", bench_type));
        for result in results {
            md.push_str(&format!(
                "- **{}**: {:.2} μs/op ({:.0} ops/sec)\n",
                result.name, result.avg_duration_us, result.throughput_per_sec
            ));
        }
        md.push_str("\n");
    }

    Ok(md)
}

fn print_summary(suite: &BenchmarkSuite) {
    println!("\n{}", "=".repeat(80));
    println!("BENCHMARK RESULTS SUMMARY");
    println!("{}", "=".repeat(80));
    println!("\nVersion: {}", suite.version);
    println!("Timestamp: {}", suite.timestamp.format("%Y-%m-%d %H:%M:%S UTC"));

    println!("\nOverall Statistics:");
    println!("  Total Benchmarks: {}", suite.summary.total_benchmarks);
    println!("  Total Iterations: {}", suite.summary.total_iterations);
    println!("  Total Duration: {:.2} ms", suite.summary.total_duration_ms);
    println!("  Fastest: {}", suite.summary.fastest_benchmark);
    println!("  Slowest: {}", suite.summary.slowest_benchmark);

    println!("\nTop 5 Fastest Operations:");
    let mut sorted = suite.results.clone();
    sorted.sort_by(|a, b| a.avg_duration_us.partial_cmp(&b.avg_duration_us).unwrap());
    for (i, result) in sorted.iter().take(5).enumerate() {
        println!(
            "  {}. {} - {:.2} μs/op ({:.0} ops/sec)",
            i + 1, result.name, result.avg_duration_us, result.throughput_per_sec
        );
    }

    println!("\n{}", "=".repeat(80));
}

fn create_test_usage(input_tokens: u64, output_tokens: u64) -> UsageRecord {
    UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "gpt-4".to_string(),
            version: Some("gpt-4-0613".to_string()),
            context_window: Some(8192),
        },
        organization_id: "org-benchmark".to_string(),
        project_id: None,
        user_id: None,
        prompt_tokens: input_tokens,
        completion_tokens: output_tokens,
        total_tokens: input_tokens + output_tokens,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: None,
        time_to_first_token_ms: None,
        tags: vec![],
        metadata: serde_json::json!({}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource::Api {
            endpoint: "benchmark".to_string(),
        },
    }
}
