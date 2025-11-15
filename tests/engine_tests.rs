use chrono::Utc;
use llm_cost_ops::{
    domain::{
        CostRecord, ModelIdentifier, PricingStructure, PricingTable, PricingTier, Provider,
        UsageRecord,
    },
    engine::{CostAggregator, CostCalculator, TokenNormalizer},
};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

fn create_test_usage_record(
    provider: Provider,
    model_name: &str,
    prompt_tokens: u64,
    completion_tokens: u64,
    cached_tokens: Option<u64>,
) -> UsageRecord {
    UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider,
        model: ModelIdentifier {
            name: model_name.to_string(),
            version: None,
            context_window: None,
        },
        organization_id: "org-test".to_string(),
        project_id: Some("proj-test".to_string()),
        user_id: None,
        prompt_tokens,
        completion_tokens,
        total_tokens: prompt_tokens + completion_tokens,
        cached_tokens,
        reasoning_tokens: None,
        latency_ms: None,
        tags: vec![],
        metadata: serde_json::json!({}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "test".to_string(),
            endpoint: None,
        },
    }
}

#[test]
fn test_cost_calculator_per_token_pricing() {
    let calculator = CostCalculator::new();

    let usage = create_test_usage_record(Provider::OpenAI, "gpt-4", 1000, 500, None);

    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),  // $10 per million input tokens
        Decimal::from_str("30.0").unwrap(),  // $30 per million output tokens
    );

    let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);

    let result = calculator.calculate(&usage, &pricing_table).unwrap();

    // Expected: (1000 * 10 / 1,000,000) + (500 * 30 / 1,000,000)
    // = 0.01 + 0.015 = 0.025
    assert_eq!(
        result.total_cost,
        Decimal::from_str("0.0250000000").unwrap()
    );
    assert_eq!(result.input_cost, Decimal::from_str("0.0100000000").unwrap());
    assert_eq!(
        result.output_cost,
        Decimal::from_str("0.0150000000").unwrap()
    );
    assert_eq!(result.currency.as_str(), "USD");
}

#[test]
fn test_cost_calculator_with_cached_tokens() {
    let calculator = CostCalculator::new();

    let usage = create_test_usage_record(Provider::Anthropic, "claude-3-sonnet", 2000, 1000, Some(500));

    let pricing = PricingStructure::with_cache_discount(
        Decimal::from_str("3.0").unwrap(),   // $3 per million input tokens
        Decimal::from_str("15.0").unwrap(),  // $15 per million output tokens
        Decimal::from_str("0.9").unwrap(),   // 90% discount on cached tokens
    );

    let pricing_table =
        PricingTable::new(Provider::Anthropic, "claude-3-sonnet".to_string(), pricing);

    let result = calculator.calculate(&usage, &pricing_table).unwrap();

    // Input cost: 2000 * 3 / 1,000,000 = 0.006
    // Cached discount: 500 * 3 / 1,000,000 * 0.9 = 0.00135
    // Net input cost: 0.006 - 0.00135 = 0.00465
    // Output cost: 1000 * 15 / 1,000,000 = 0.015
    // Total: 0.00465 + 0.015 = 0.01965
    assert_eq!(
        result.total_cost,
        Decimal::from_str("0.0196500000").unwrap()
    );
}

#[test]
fn test_cost_calculator_per_request_pricing() {
    let calculator = CostCalculator::new();

    let usage = create_test_usage_record(Provider::Custom("test".to_string()), "model-1", 800, 200, None);

    let pricing = PricingStructure::PerRequest {
        price_per_request: Decimal::from_str("0.01").unwrap(),
        included_tokens: 1000,
        overage_price_per_million: Decimal::from_str("5.0").unwrap(),
    };

    let pricing_table = PricingTable::new(
        Provider::Custom("test".to_string()),
        "model-1".to_string(),
        pricing,
    );

    let result = calculator.calculate(&usage, &pricing_table).unwrap();

    // 1000 total tokens, all within included limit
    // Cost should be just the base price
    assert_eq!(result.total_cost, Decimal::from_str("0.0100000000").unwrap());
}

#[test]
fn test_cost_calculator_per_request_with_overage() {
    let calculator = CostCalculator::new();

    let usage = create_test_usage_record(Provider::Custom("test".to_string()), "model-1", 1200, 300, None);

    let pricing = PricingStructure::PerRequest {
        price_per_request: Decimal::from_str("0.01").unwrap(),
        included_tokens: 1000,
        overage_price_per_million: Decimal::from_str("5.0").unwrap(),
    };

    let pricing_table = PricingTable::new(
        Provider::Custom("test".to_string()),
        "model-1".to_string(),
        pricing,
    );

    let result = calculator.calculate(&usage, &pricing_table).unwrap();

    // 1500 total tokens - 1000 included = 500 overage tokens
    // Base: 0.01
    // Overage: 500 * 5 / 1,000,000 = 0.0025
    // Total: 0.0125
    assert_eq!(result.total_cost, Decimal::from_str("0.0125000000").unwrap());
}

#[test]
fn test_cost_calculator_tiered_pricing() {
    let calculator = CostCalculator::new();

    let usage = create_test_usage_record(Provider::Custom("test".to_string()), "model-1", 600, 400, None);

    let pricing = PricingStructure::Tiered {
        tiers: vec![
            PricingTier {
                threshold: 0,
                input_price_per_million: Decimal::from_str("10.0").unwrap(),
                output_price_per_million: Decimal::from_str("30.0").unwrap(),
            },
            PricingTier {
                threshold: 1_000_000,
                input_price_per_million: Decimal::from_str("8.0").unwrap(),
                output_price_per_million: Decimal::from_str("24.0").unwrap(),
            },
        ],
    };

    let pricing_table = PricingTable::new(
        Provider::Custom("test".to_string()),
        "model-1".to_string(),
        pricing,
    );

    let result = calculator.calculate(&usage, &pricing_table).unwrap();

    // Using tier 0 (below 1M tokens)
    // Input: 600 * 10 / 1,000,000 = 0.006
    // Output: 400 * 30 / 1,000,000 = 0.012
    // Total: 0.018
    assert_eq!(result.total_cost, Decimal::from_str("0.0180000000").unwrap());
}

#[test]
fn test_token_normalizer() {
    let normalizer = TokenNormalizer::new();

    // Test OpenAI normalization (no change)
    let openai_usage = create_test_usage_record(Provider::OpenAI, "gpt-4", 1000, 500, None);
    let normalized = normalizer.normalize(&openai_usage).unwrap();
    assert_eq!(normalized.prompt_tokens, 1000);
    assert_eq!(normalized.completion_tokens, 500);

    // Test Anthropic normalization (no change for now)
    let anthropic_usage = create_test_usage_record(Provider::Anthropic, "claude-3-sonnet", 2000, 1000, None);
    let normalized = normalizer.normalize(&anthropic_usage).unwrap();
    assert_eq!(normalized.prompt_tokens, 2000);
    assert_eq!(normalized.completion_tokens, 1000);
}

#[test]
fn test_cost_aggregator() {
    let aggregator = CostAggregator::new();

    let now = Utc::now();
    let start = now - chrono::Duration::days(7);
    let end = now;

    let records = vec![
        CostRecord {
            id: Uuid::new_v4(),
            usage_id: Uuid::new_v4(),
            provider: Provider::OpenAI,
            model: "gpt-4".to_string(),
            input_cost: Decimal::from_str("0.01").unwrap(),
            output_cost: Decimal::from_str("0.02").unwrap(),
            total_cost: Decimal::from_str("0.03").unwrap(),
            currency: llm_cost_ops::domain::Currency::USD,
            timestamp: now - chrono::Duration::days(1),
            organization_id: Some("org-123".to_string()),
            project_id: Some("proj-456".to_string()),
        },
        CostRecord {
            id: Uuid::new_v4(),
            usage_id: Uuid::new_v4(),
            provider: Provider::Anthropic,
            model: "claude-3-sonnet".to_string(),
            input_cost: Decimal::from_str("0.005").unwrap(),
            output_cost: Decimal::from_str("0.015").unwrap(),
            total_cost: Decimal::from_str("0.02").unwrap(),
            currency: llm_cost_ops::domain::Currency::USD,
            timestamp: now - chrono::Duration::days(2),
            organization_id: Some("org-123".to_string()),
            project_id: Some("proj-456".to_string()),
        },
        CostRecord {
            id: Uuid::new_v4(),
            usage_id: Uuid::new_v4(),
            provider: Provider::OpenAI,
            model: "gpt-3.5-turbo".to_string(),
            input_cost: Decimal::from_str("0.001").unwrap(),
            output_cost: Decimal::from_str("0.002").unwrap(),
            total_cost: Decimal::from_str("0.003").unwrap(),
            currency: llm_cost_ops::domain::Currency::USD,
            timestamp: now - chrono::Duration::days(3),
            organization_id: Some("org-123".to_string()),
            project_id: Some("proj-789".to_string()),
        },
    ];

    let summary = aggregator.aggregate(&records, start, end).unwrap();

    // Total cost: 0.03 + 0.02 + 0.003 = 0.053
    assert_eq!(summary.total_cost, Decimal::from_str("0.053").unwrap());
    assert_eq!(summary.total_requests, 3);

    // By provider
    assert_eq!(
        summary.by_provider.get("openai"),
        Some(&Decimal::from_str("0.033").unwrap())
    );
    assert_eq!(
        summary.by_provider.get("anthropic"),
        Some(&Decimal::from_str("0.02").unwrap())
    );

    // By model
    assert_eq!(
        summary.by_model.get("gpt-4"),
        Some(&Decimal::from_str("0.03").unwrap())
    );
    assert_eq!(
        summary.by_model.get("claude-3-sonnet"),
        Some(&Decimal::from_str("0.02").unwrap())
    );
    assert_eq!(
        summary.by_model.get("gpt-3.5-turbo"),
        Some(&Decimal::from_str("0.003").unwrap())
    );

    // By project
    assert_eq!(
        summary.by_project.get("proj-456"),
        Some(&Decimal::from_str("0.05").unwrap())
    );
    assert_eq!(
        summary.by_project.get("proj-789"),
        Some(&Decimal::from_str("0.003").unwrap())
    );
}

#[test]
fn test_cost_aggregator_empty_records() {
    let aggregator = CostAggregator::new();

    let now = Utc::now();
    let start = now - chrono::Duration::days(7);
    let end = now;

    let records: Vec<CostRecord> = vec![];

    let summary = aggregator.aggregate(&records, start, end).unwrap();

    assert_eq!(summary.total_cost, Decimal::ZERO);
    assert_eq!(summary.total_requests, 0);
    assert_eq!(summary.avg_cost_per_request, Decimal::ZERO);
    assert!(summary.by_provider.is_empty());
    assert!(summary.by_model.is_empty());
    assert!(summary.by_project.is_empty());
}

#[test]
fn test_cost_record_creation() {
    let cost_record = CostRecord {
        id: Uuid::new_v4(),
        usage_id: Uuid::new_v4(),
        provider: Provider::OpenAI,
        model: "gpt-4".to_string(),
        input_cost: Decimal::from_str("0.01").unwrap(),
        output_cost: Decimal::from_str("0.03").unwrap(),
        total_cost: Decimal::from_str("0.04").unwrap(),
        currency: llm_cost_ops::domain::Currency::USD,
        timestamp: Utc::now(),
        organization_id: Some("org-123".to_string()),
        project_id: Some("proj-456".to_string()),
    };

    assert_eq!(cost_record.provider, Provider::OpenAI);
    assert_eq!(cost_record.model, "gpt-4");
    assert_eq!(cost_record.currency.as_str(), "USD");
}

#[test]
fn test_decimal_precision() {
    let calculator = CostCalculator::new();

    // Test with very small numbers to verify precision
    let usage = create_test_usage_record(Provider::OpenAI, "gpt-4", 1, 1, None);

    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );

    let pricing_table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);

    let result = calculator.calculate(&usage, &pricing_table).unwrap();

    // 1 token at $10/M = 0.00001
    // 1 token at $30/M = 0.00003
    // Total = 0.00004
    assert_eq!(
        result.total_cost,
        Decimal::from_str("0.0000400000").unwrap()
    );
}
