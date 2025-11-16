// Performance Benchmarks for Cost Calculation Engine
// Validates throughput claims and identifies bottlenecks

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llm_cost_ops::{
    domain::{ModelIdentifier, PricingStructure, PricingTable, Provider, UsageRecord},
    engine::{CostAggregator, CostCalculator},
};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

// Benchmark: Single cost calculation
fn bench_single_cost_calculation(c: &mut Criterion) {
    let calculator = CostCalculator::new();
    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );
    let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);
    let usage = create_test_usage(1000, 500);

    c.bench_function("single_cost_calculation", |b| {
        b.iter(|| {
            let result = calculator.calculate(black_box(&usage), black_box(&pricing_table));
            black_box(result)
        })
    });
}

// Benchmark: Batch cost calculations (different sizes)
fn bench_batch_cost_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_cost_calculation");

    for size in [100, 1_000, 10_000, 100_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        let calculator = CostCalculator::new();
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_str("10.0").unwrap(),
            Decimal::from_str("30.0").unwrap(),
        );
        let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);
        let usage_records: Vec<_> = (0..*size).map(|_| create_test_usage(1000, 500)).collect();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                for usage in &usage_records {
                    let result = calculator.calculate(black_box(usage), black_box(&pricing_table));
                    black_box(result);
                }
            })
        });
    }

    group.finish();
}

// Benchmark: Cost calculation with caching
fn bench_cached_token_calculation(c: &mut Criterion) {
    let calculator = CostCalculator::new();
    let pricing = PricingStructure::with_cache_discount(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
        Decimal::from_str("0.9").unwrap(),
    );
    let pricing_table = PricingTable::new(Provider::Anthropic, "claude-3".to_string(), pricing);

    let mut usage = create_test_usage(5000, 2000);
    usage.cached_tokens = Some(2000);

    c.bench_function("cached_token_calculation", |b| {
        b.iter(|| {
            let result = calculator.calculate(black_box(&usage), black_box(&pricing_table));
            black_box(result)
        })
    });
}

// Benchmark: Cost aggregation
fn bench_cost_aggregation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cost_aggregation");

    for size in [100, 1_000, 10_000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        let calculator = CostCalculator::new();
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_str("10.0").unwrap(),
            Decimal::from_str("30.0").unwrap(),
        );
        let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);

        let cost_records: Vec<_> = (0..*size)
            .map(|_| {
                let usage = create_test_usage(1000, 500);
                calculator
                    .calculate(&usage, &pricing_table)
                    .expect("Cost calculation failed")
            })
            .collect();

        let aggregator = CostAggregator::new();
        let start = chrono::Utc::now() - chrono::Duration::days(7);
        let end = chrono::Utc::now();

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let result = aggregator.aggregate(black_box(&cost_records), start, end);
                black_box(result)
            })
        });
    }

    group.finish();
}

// Benchmark: Different pricing structures
fn bench_pricing_structures(c: &mut Criterion) {
    let mut group = c.benchmark_group("pricing_structures");

    let calculator = CostCalculator::new();
    let usage = create_test_usage(10000, 5000);

    // Simple per-token
    let simple_pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );
    let simple_table = PricingTable::new(Provider::OpenAI, "test".to_string(), simple_pricing);

    group.bench_function("simple_per_token", |b| {
        b.iter(|| {
            let result = calculator.calculate(black_box(&usage), black_box(&simple_table));
            black_box(result)
        })
    });

    // With cache discount
    let cache_pricing = PricingStructure::with_cache_discount(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
        Decimal::from_str("0.9").unwrap(),
    );
    let cache_table = PricingTable::new(Provider::Anthropic, "test".to_string(), cache_pricing);

    let mut cached_usage = usage.clone();
    cached_usage.cached_tokens = Some(3000);

    group.bench_function("with_cache_discount", |b| {
        b.iter(|| {
            let result = calculator.calculate(black_box(&cached_usage), black_box(&cache_table));
            black_box(result)
        })
    });

    group.finish();
}

// Benchmark: Validation overhead
fn bench_validation(c: &mut Criterion) {
    let usage = create_test_usage(1000, 500);

    c.bench_function("usage_validation", |b| {
        b.iter(|| {
            let result = black_box(&usage).validate();
            black_box(result)
        })
    });
}

// Benchmark: Token counting for different sizes
fn bench_token_counting_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_counting");

    for tokens in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.throughput(Throughput::Elements(*tokens as u64));

        let usage = create_test_usage(*tokens / 2, *tokens / 2);

        group.bench_with_input(BenchmarkId::from_parameter(tokens), tokens, |b, _| {
            b.iter(|| {
                // Simulate token counting overhead
                let total = black_box(usage.prompt_tokens + usage.completion_tokens);
                black_box(total)
            })
        });
    }

    group.finish();
}

// Benchmark: Multi-provider calculations
fn bench_multi_provider(c: &mut Criterion) {
    let calculator = CostCalculator::new();

    let providers = vec![
        (Provider::OpenAI, "gpt-4"),
        (Provider::Anthropic, "claude-3"),
        (Provider::GoogleVertexAI, "gemini-pro"),
        (Provider::AzureOpenAI, "gpt-4"),
    ];

    let mut group = c.benchmark_group("multi_provider");

    for (provider, model) in providers {
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_str("10.0").unwrap(),
            Decimal::from_str("30.0").unwrap(),
        );
        let pricing_table = PricingTable::new(provider.clone(), model.to_string(), pricing);
        let usage = create_test_usage(1000, 500);

        group.bench_function(provider.to_string(), |b| {
            b.iter(|| {
                let result = calculator.calculate(black_box(&usage), black_box(&pricing_table));
                black_box(result)
            })
        });
    }

    group.finish();
}

// Benchmark: Decimal arithmetic precision
fn bench_decimal_arithmetic(c: &mut Criterion) {
    c.bench_function("decimal_multiplication", |b| {
        let tokens = Decimal::from(1_000_000);
        let rate = Decimal::from_str("0.000001").unwrap();

        b.iter(|| {
            let result = black_box(tokens) * black_box(rate);
            black_box(result)
        })
    });

    c.bench_function("decimal_division", |b| {
        let tokens = Decimal::from(1_000_000);
        let divisor = Decimal::from(1_000_000);

        b.iter(|| {
            let result = black_box(tokens) / black_box(divisor);
            black_box(result)
        })
    });
}

// Benchmark: Concurrent cost calculations
fn bench_concurrent_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_calculations");

    for num_threads in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_threads),
            num_threads,
            |b, &threads| {
                b.iter(|| {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(threads)
                        .build()
                        .unwrap();

                    rt.block_on(async {
                        let calculator = CostCalculator::new();
                        let pricing = PricingStructure::simple_per_token(
                            Decimal::from_str("10.0").unwrap(),
                            Decimal::from_str("30.0").unwrap(),
                        );
                        let pricing_table = PricingTable::new(
                            Provider::OpenAI,
                            "gpt-4".to_string(),
                            pricing,
                        );

                        let handles: Vec<_> = (0..1000)
                            .map(|_| {
                                let calc = calculator.clone();
                                let table = pricing_table.clone();
                                tokio::spawn(async move {
                                    let usage = create_test_usage(1000, 500);
                                    calc.calculate(&usage, &table).unwrap()
                                })
                            })
                            .collect();

                        for handle in handles {
                            handle.await.unwrap();
                        }
                    });
                })
            },
        );
    }

    group.finish();
}

// Helper function
fn create_test_usage(input_tokens: u64, output_tokens: u64) -> UsageRecord {
    UsageRecord {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
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
        tags: vec![],
        metadata: serde_json::json!({}),
        ingested_at: chrono::Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "benchmark".to_string(),
            endpoint: None,
        },
    }
}

criterion_group!(
    benches,
    bench_single_cost_calculation,
    bench_batch_cost_calculation,
    bench_cached_token_calculation,
    bench_cost_aggregation,
    bench_pricing_structures,
    bench_validation,
    bench_token_counting_overhead,
    bench_multi_provider,
    bench_decimal_arithmetic,
    bench_concurrent_calculations,
);

criterion_main!(benches);
