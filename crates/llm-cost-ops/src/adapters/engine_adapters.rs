/// Benchmark adapters for the cost calculation engine
///
/// Provides benchmark targets for CostCalculator, TokenNormalizer, and CostAggregator.

use super::{BenchTarget, calculate_stats, run_iterations};
use crate::benchmarks::result::BenchmarkResult;
use crate::domain::{ModelIdentifier, PricingStructure, PricingTable, Provider, UsageRecord, CostRecord, CostCalculation, Currency, IngestionSource};
use crate::engine::{CostCalculator, TokenNormalizer, CostAggregator};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::time::Duration;
use chrono::Utc;
use uuid::Uuid;

/// Helper macro replacement for creating Decimal values
fn dec(value: &str) -> Decimal {
    Decimal::from_str(value).unwrap_or(Decimal::ZERO)
}

// Helper function to create test usage
fn create_test_usage(prompt_tokens: u64, completion_tokens: u64) -> UsageRecord {
    UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier::new("gpt-4".to_string(), 8192),
        organization_id: "org-bench".to_string(),
        project_id: None,
        user_id: None,
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: None,
        time_to_first_token_ms: None,
        tags: vec![],
        metadata: serde_json::Value::Null,
        ingested_at: Utc::now(),
        source: IngestionSource::Api {
            endpoint: "benchmark".to_string(),
        },
    }
}

fn create_test_pricing() -> PricingTable {
    PricingTable::new(
        Provider::OpenAI,
        "gpt-4".to_string(),
        PricingStructure::simple_per_token(dec("10.0"), dec("30.0")),
    )
}

fn create_test_cost() -> CostRecord {
    CostRecord::new(
        Uuid::new_v4(),
        Provider::OpenAI,
        "gpt-4".to_string(),
        "org-test".to_string(),
        CostCalculation::new(dec("0.01"), dec("0.02"), Currency::USD, Uuid::new_v4()),
        PricingStructure::simple_per_token(dec("10.0"), dec("30.0")),
    )
}

/// Benchmark: Single cost calculation
pub struct SingleCostCalculation {
    calculator: CostCalculator,
    usage: UsageRecord,
    pricing: PricingTable,
}

impl SingleCostCalculation {
    pub fn new() -> Self {
        Self {
            calculator: CostCalculator::new(),
            usage: create_test_usage(1000, 500),
            pricing: create_test_pricing(),
        }
    }
}

impl BenchTarget for SingleCostCalculation {
    fn id(&self) -> String {
        "engine/single_cost_calculation".to_string()
    }

    fn name(&self) -> String {
        "Single Cost Calculation".to_string()
    }

    fn category(&self) -> String {
        "engine".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let iterations = 10_000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = self.calculator.calculate(&self.usage, &self.pricing);
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
}

/// Benchmark: Batch cost calculations
pub struct BatchCostCalculation {
    size: u64,
}

impl BatchCostCalculation {
    pub fn new(size: u64) -> Self {
        Self { size }
    }
}

impl BenchTarget for BatchCostCalculation {
    fn id(&self) -> String {
        format!("engine/batch_cost_calculation_{}", self.size)
    }

    fn name(&self) -> String {
        format!("Batch Cost Calculation ({})", self.size)
    }

    fn category(&self) -> String {
        "engine".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let calculator = CostCalculator::new();
        let pricing = create_test_pricing();
        let usage_records: Vec<_> = (0..self.size)
            .map(|_| create_test_usage(1000, 500))
            .collect();

        let iterations = 100;
        let (total_duration, timings) = run_iterations(iterations, || {
            for usage in &usage_records {
                let _ = calculator.calculate(usage, &pricing);
            }
        });

        let (min, max, std_dev) = calculate_stats(&timings);

        BenchmarkResult::success(
            self.id(),
            self.name(),
            self.category(),
            total_duration,
            iterations * self.size,
        )
        .with_stats(min, max, std_dev)
    }
}

/// Benchmark: Cost calculation with cache discount
pub struct CachedTokenCalculation;

impl BenchTarget for CachedTokenCalculation {
    fn id(&self) -> String {
        "engine/cached_token_calculation".to_string()
    }

    fn name(&self) -> String {
        "Cached Token Calculation".to_string()
    }

    fn category(&self) -> String {
        "engine".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let calculator = CostCalculator::new();
        let pricing = PricingTable::new(
            Provider::Anthropic,
            "claude-3".to_string(),
            PricingStructure::per_token_with_cache(dec("10.0"), dec("30.0"), dec("0.9")),
        );

        let mut usage = create_test_usage(5000, 2000);
        usage.cached_tokens = Some(2000);

        let iterations = 10_000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = calculator.calculate(&usage, &pricing);
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
}

/// Benchmark: Token normalization
pub struct TokenNormalization;

impl BenchTarget for TokenNormalization {
    fn id(&self) -> String {
        "engine/token_normalization".to_string()
    }

    fn name(&self) -> String {
        "Token Normalization".to_string()
    }

    fn category(&self) -> String {
        "engine".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let normalizer = TokenNormalizer::new();
        let usage = create_test_usage(10000, 5000);

        let iterations = 10_000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = normalizer.normalize(&usage);
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
}

/// Benchmark: Cost aggregation
pub struct CostAggregation {
    size: usize,
}

impl CostAggregation {
    pub fn new(size: usize) -> Self {
        Self { size }
    }
}

impl BenchTarget for CostAggregation {
    fn id(&self) -> String {
        format!("engine/cost_aggregation_{}", self.size)
    }

    fn name(&self) -> String {
        format!("Cost Aggregation ({})", self.size)
    }

    fn category(&self) -> String {
        "engine".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let aggregator = CostAggregator::new();
        let cost_records: Vec<_> = (0..self.size)
            .map(|_| create_test_cost())
            .collect();

        let period_start = Utc::now() - chrono::Duration::days(7);
        let period_end = Utc::now();

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = aggregator.aggregate(&cost_records, period_start, period_end);
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
}

/// Benchmark: Validation overhead
pub struct ValidationOverhead;

impl BenchTarget for ValidationOverhead {
    fn id(&self) -> String {
        "engine/validation_overhead".to_string()
    }

    fn name(&self) -> String {
        "Validation Overhead".to_string()
    }

    fn category(&self) -> String {
        "engine".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let usage = create_test_usage(1000, 500);

        let iterations = 100_000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = usage.validate();
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
}

/// Create all engine benchmark targets
pub fn create_targets() -> Vec<Box<dyn BenchTarget>> {
    vec![
        Box::new(SingleCostCalculation::new()),
        Box::new(BatchCostCalculation::new(100)),
        Box::new(BatchCostCalculation::new(1_000)),
        Box::new(BatchCostCalculation::new(10_000)),
        Box::new(CachedTokenCalculation),
        Box::new(TokenNormalization),
        Box::new(CostAggregation::new(100)),
        Box::new(CostAggregation::new(1_000)),
        Box::new(CostAggregation::new(10_000)),
        Box::new(ValidationOverhead),
    ]
}
