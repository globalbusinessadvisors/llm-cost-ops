/// Additional benchmarks for engine performance
///
/// Complements cost_calculation.rs with more comprehensive benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use llm_cost_ops::domain::*;
use llm_cost_ops::engine::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use chrono::{Duration, Utc};
use uuid::Uuid;

fn create_test_usage(tokens: u64) -> UsageRecord {
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-test".to_string(),
        tokens,
        tokens / 2,
    )
}

fn create_test_pricing() -> PricingTable {
    PricingTable {
        id: Uuid::new_v4(),
        provider: Provider::OpenAI,
        model: "gpt-4".to_string(),
        pricing: PricingStructure::PerToken {
            input_price_per_million: dec!(30.00),
            output_price_per_million: dec!(60.00),
            cached_input_discount: dec!(0.5),
        },
        currency: Currency::USD,
        effective_from: Utc::now() - Duration::days(1),
        effective_until: None,
    }
}

fn create_test_cost() -> CostRecord {
    CostRecord::new(
        Uuid::new_v4(),
        Provider::OpenAI,
        "gpt-4".to_string(),
        "org-test".to_string(),
        CostCalculation::new(dec!(0.01), dec!(0.02), Currency::USD, Uuid::new_v4()),
        PricingStructure::simple_per_token(dec!(10.0), dec!(30.0)),
    )
}

fn bench_cost_calculation(c: &mut Criterion) {
    let calculator = CostCalculator::new();
    let pricing = create_test_pricing();

    let mut group = c.benchmark_group("cost_calculation");

    for size in [100, 1_000, 10_000, 100_000, 1_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("tokens", size), size, |b, &size| {
            let usage = create_test_usage(size);
            b.iter(|| {
                black_box(calculator.calculate(&usage, &pricing))
            });
        });
    }

    group.finish();
}

fn bench_token_normalization(c: &mut Criterion) {
    let normalizer = TokenNormalizer::new();

    let mut group = c.benchmark_group("token_normalization");

    for size in [1_000, 10_000, 100_000, 1_000_000, 10_000_000].iter() {
        group.bench_with_input(BenchmarkId::new("to_millions", size), size, |b, &size| {
            b.iter(|| {
                black_box(normalizer.normalize(size, TokenUnit::Tokens, TokenUnit::Millions))
            });
        });
    }

    group.finish();
}

fn bench_cost_aggregation(c: &mut Criterion) {
    let aggregator = CostAggregator::new();

    let mut group = c.benchmark_group("cost_aggregation");

    for count in [10, 100, 1_000, 10_000].iter() {
        let costs: Vec<_> = (0..*count).map(|_| create_test_cost()).collect();

        group.bench_with_input(BenchmarkId::new("sum_total", count), &costs, |b, costs| {
            b.iter(|| {
                black_box(aggregator.sum_total_costs(costs))
            });
        });

        group.bench_with_input(BenchmarkId::new("sum_by_provider", count), &costs, |b, costs| {
            b.iter(|| {
                black_box(aggregator.sum_by_provider(costs))
            });
        });

        group.bench_with_input(BenchmarkId::new("sum_by_model", count), &costs, |b, costs| {
            b.iter(|| {
                black_box(aggregator.sum_by_model(costs))
            });
        });
    }

    group.finish();
}

fn bench_usage_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("usage_validation");

    for size in [100, 1_000, 10_000].iter() {
        let usage = create_test_usage(*size);
        group.bench_with_input(BenchmarkId::new("validate", size), &usage, |b, usage| {
            b.iter(|| {
                black_box(usage.validate())
            });
        });
    }

    group.finish();
}

fn bench_bulk_processing(c: &mut Criterion) {
    let calculator = CostCalculator::new();
    let pricing = create_test_pricing();

    let mut group = c.benchmark_group("bulk_processing");

    for count in [100, 500, 1_000].iter() {
        let usage_records: Vec<_> = (0..*count).map(|_| create_test_usage(1000)).collect();

        group.bench_with_input(BenchmarkId::new("calculate_all", count), &usage_records, |b, records| {
            b.iter(|| {
                for record in records {
                    black_box(calculator.calculate(record, &pricing));
                }
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_cost_calculation,
    bench_token_normalization,
    bench_cost_aggregation,
    bench_usage_validation,
    bench_bulk_processing,
);
criterion_main!(benches);
