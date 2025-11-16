/// Comprehensive engine tests for cost calculation, aggregation, and normalization
///
/// Tests the core engine components that power cost calculation

use chrono::{Duration, Utc};
use llm_cost_ops::domain::*;
use llm_cost_ops::engine::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

mod helpers;
use helpers::assertions::*;
use helpers::builders::*;

// === Cost Calculator Tests ===

#[test]
fn test_cost_calculator_creation() {
    let calculator = CostCalculator::new();
    assert!(std::mem::size_of_val(&calculator) > 0);
}

#[test]
fn test_calculate_per_token_pricing() {
    let calculator = CostCalculator::new();

    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let usage = UsageRecord::new(
        Provider::OpenAI,
        model.clone(),
        "org-123".to_string(),
        1_000_000, // 1M input tokens
        500_000,   // 500k output tokens
    );

    let pricing = PricingTable {
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
    };

    let result = calculator.calculate(&usage, &pricing);
    assert!(result.is_ok());

    let cost_record = result.unwrap();
    assert_eq!(cost_record.input_cost, dec!(30.00));  // 1M tokens * $30/M
    assert_eq!(cost_record.output_cost, dec!(30.00)); // 500k tokens * $60/M
    assert_eq!(cost_record.total_cost, dec!(60.00));
}

#[test]
fn test_calculate_with_cached_tokens() {
    let calculator = CostCalculator::new();

    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let usage = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        1_000_000,
        500_000,
    )
    .with_cached_tokens(500_000); // 50% cached

    let pricing = PricingTable {
        id: Uuid::new_v4(),
        provider: Provider::OpenAI,
        model: "gpt-4".to_string(),
        pricing: PricingStructure::PerToken {
            input_price_per_million: dec!(30.00),
            output_price_per_million: dec!(60.00),
            cached_input_discount: dec!(0.5), // 50% discount on cached tokens
        },
        currency: Currency::USD,
        effective_from: Utc::now() - Duration::days(1),
        effective_until: None,
    };

    let result = calculator.calculate(&usage, &pricing);
    assert!(result.is_ok());

    let cost_record = result.unwrap();
    // 500k full price + 500k half price = 22.50
    // 500k output = 30.00
    // Total = 52.50
    assert!(cost_record.input_cost < dec!(30.00));
    assert_decimal_positive(cost_record.total_cost);
}

#[test]
fn test_calculate_per_request_pricing() {
    let calculator = CostCalculator::new();

    let model = ModelIdentifier::new("embedding-model".to_string(), 8192);
    let usage = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        100,
        0,
    );

    let pricing = PricingTable {
        id: Uuid::new_v4(),
        provider: Provider::OpenAI,
        model: "embedding-model".to_string(),
        pricing: PricingStructure::PerRequest {
            price_per_request: dec!(0.01),
            included_tokens: 1000,
            overage_price_per_million: dec!(10.00),
        },
        currency: Currency::USD,
        effective_from: Utc::now() - Duration::days(1),
        effective_until: None,
    };

    let result = calculator.calculate(&usage, &pricing);
    assert!(result.is_ok());

    let cost_record = result.unwrap();
    assert_decimal_positive(cost_record.total_cost);
}

#[test]
fn test_calculate_with_overage() {
    let calculator = CostCalculator::new();

    let model = ModelIdentifier::new("embedding-model".to_string(), 8192);
    let usage = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        2000, // Exceeds included 1000 tokens
        0,
    );

    let pricing = PricingTable {
        id: Uuid::new_v4(),
        provider: Provider::OpenAI,
        model: "embedding-model".to_string(),
        pricing: PricingStructure::PerRequest {
            price_per_request: dec!(0.01),
            included_tokens: 1000,
            overage_price_per_million: dec!(10.00),
        },
        currency: Currency::USD,
        effective_from: Utc::now() - Duration::days(1),
        effective_until: None,
    };

    let result = calculator.calculate(&usage, &pricing);
    assert!(result.is_ok());

    let cost_record = result.unwrap();
    // Base price + overage for 1000 tokens
    assert!(cost_record.total_cost > dec!(0.01));
}

#[test]
fn test_calculate_tiered_pricing() {
    let calculator = CostCalculator::new();

    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let usage = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        5_000_000, // 5M tokens
        2_000_000, // 2M tokens
    );

    let pricing = PricingTable {
        id: Uuid::new_v4(),
        provider: Provider::OpenAI,
        model: "gpt-4".to_string(),
        pricing: PricingStructure::Tiered {
            tiers: vec![
                PricingTier {
                    up_to_tokens: 1_000_000,
                    input_price_per_million: dec!(30.00),
                    output_price_per_million: dec!(60.00),
                },
                PricingTier {
                    up_to_tokens: 10_000_000,
                    input_price_per_million: dec!(25.00),
                    output_price_per_million: dec!(50.00),
                },
            ],
        },
        currency: Currency::USD,
        effective_from: Utc::now() - Duration::days(1),
        effective_until: None,
    };

    let result = calculator.calculate(&usage, &pricing);
    assert!(result.is_ok());

    let cost_record = result.unwrap();
    assert_decimal_positive(cost_record.input_cost);
    assert_decimal_positive(cost_record.output_cost);
    assert_decimal_positive(cost_record.total_cost);
}

#[test]
fn test_calculate_provider_mismatch_error() {
    let calculator = CostCalculator::new();

    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let usage = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        1000,
        500,
    );

    let pricing = PricingTable {
        id: Uuid::new_v4(),
        provider: Provider::Anthropic, // Different provider!
        model: "claude-3-opus".to_string(),
        pricing: PricingStructure::PerToken {
            input_price_per_million: dec!(15.00),
            output_price_per_million: dec!(75.00),
            cached_input_discount: dec!(0.9),
        },
        currency: Currency::USD,
        effective_from: Utc::now() - Duration::days(1),
        effective_until: None,
    };

    let result = calculator.calculate(&usage, &pricing);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CostOpsError::InvalidPricingStructure(_)));
}

#[test]
fn test_calculate_inactive_pricing_warning() {
    let calculator = CostCalculator::new();

    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let usage = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        1000,
        500,
    );

    let pricing = PricingTable {
        id: Uuid::new_v4(),
        provider: Provider::OpenAI,
        model: "gpt-4".to_string(),
        pricing: PricingStructure::PerToken {
            input_price_per_million: dec!(30.00),
            output_price_per_million: dec!(60.00),
            cached_input_discount: dec!(0.5),
        },
        currency: Currency::USD,
        effective_from: Utc::now() + Duration::days(1), // Future pricing
        effective_until: None,
    };

    // Should still calculate but log a warning
    let result = calculator.calculate(&usage, &pricing);
    assert!(result.is_ok());
}

#[test]
fn test_calculate_preserves_metadata() {
    let calculator = CostCalculator::new();

    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let usage = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        1000,
        500,
    )
    .with_project("proj-456".to_string())
    .with_tags(vec!["production".to_string(), "api".to_string()]);

    let pricing = PricingTable {
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
    };

    let result = calculator.calculate(&usage, &pricing);
    assert!(result.is_ok());

    let cost_record = result.unwrap();
    assert_eq!(cost_record.project_id, Some("proj-456".to_string()));
    assert_eq!(cost_record.tags.len(), 2);
    assert!(cost_record.tags.contains(&"production".to_string()));
    assert!(cost_record.tags.contains(&"api".to_string()));
}

// === Token Normalizer Tests ===

#[test]
fn test_token_normalizer_normalize_to_common_unit() {
    let normalizer = TokenNormalizer::new();

    let tokens = normalizer.normalize(
        100_000,
        TokenUnit::Tokens,
        TokenUnit::Thousands,
    );

    assert_eq!(tokens, dec!(100.0));
}

#[test]
fn test_token_normalizer_normalize_to_millions() {
    let normalizer = TokenNormalizer::new();

    let tokens = normalizer.normalize(
        1_500_000,
        TokenUnit::Tokens,
        TokenUnit::Millions,
    );

    assert_eq!(tokens, dec!(1.5));
}

#[test]
fn test_token_normalizer_identity_transformation() {
    let normalizer = TokenNormalizer::new();

    let tokens = normalizer.normalize(
        1000,
        TokenUnit::Tokens,
        TokenUnit::Tokens,
    );

    assert_eq!(tokens, dec!(1000));
}

#[test]
fn test_token_normalizer_precision() {
    let normalizer = TokenNormalizer::new();

    let tokens = normalizer.normalize(
        123_456,
        TokenUnit::Tokens,
        TokenUnit::Millions,
    );

    assert_decimal_in_range(tokens, dec!(0.123), dec!(0.124));
}

// === Cost Aggregator Tests ===

#[test]
fn test_aggregator_sum_costs() {
    let aggregator = CostAggregator::new();

    let costs = vec![
        CostRecordBuilder::new()
            .input_cost(dec!(0.10))
            .output_cost(dec!(0.20))
            .build(),
        CostRecordBuilder::new()
            .input_cost(dec!(0.15))
            .output_cost(dec!(0.25))
            .build(),
        CostRecordBuilder::new()
            .input_cost(dec!(0.05))
            .output_cost(dec!(0.10))
            .build(),
    ];

    let total = aggregator.sum_total_costs(&costs);
    assert_eq!(total, dec!(0.85)); // Sum of all costs
}

#[test]
fn test_aggregator_sum_by_provider() {
    let aggregator = CostAggregator::new();

    let costs = vec![
        CostRecordBuilder::new()
            .provider(Provider::OpenAI)
            .input_cost(dec!(0.10))
            .output_cost(dec!(0.20))
            .build(),
        CostRecordBuilder::new()
            .provider(Provider::OpenAI)
            .input_cost(dec!(0.15))
            .output_cost(dec!(0.25))
            .build(),
        CostRecordBuilder::new()
            .provider(Provider::Anthropic)
            .input_cost(dec!(0.05))
            .output_cost(dec!(0.10))
            .build(),
    ];

    let by_provider = aggregator.sum_by_provider(&costs);
    assert_eq!(by_provider.get(&Provider::OpenAI), Some(&dec!(0.70)));
    assert_eq!(by_provider.get(&Provider::Anthropic), Some(&dec!(0.15)));
}

#[test]
fn test_aggregator_sum_by_model() {
    let aggregator = CostAggregator::new();

    let costs = vec![
        CostRecordBuilder::new()
            .model("gpt-4")
            .input_cost(dec!(0.10))
            .output_cost(dec!(0.20))
            .build(),
        CostRecordBuilder::new()
            .model("gpt-4")
            .input_cost(dec!(0.15))
            .output_cost(dec!(0.25))
            .build(),
        CostRecordBuilder::new()
            .model("gpt-3.5-turbo")
            .input_cost(dec!(0.05))
            .output_cost(dec!(0.10))
            .build(),
    ];

    let by_model = aggregator.sum_by_model(&costs);
    assert_eq!(by_model.get("gpt-4"), Some(&dec!(0.70)));
    assert_eq!(by_model.get("gpt-3.5-turbo"), Some(&dec!(0.15)));
}

#[test]
fn test_aggregator_sum_by_organization() {
    let aggregator = CostAggregator::new();

    let org1 = Uuid::new_v4();
    let org2 = Uuid::new_v4();

    let costs = vec![
        CostRecordBuilder::new()
            .tenant_id(org1)
            .input_cost(dec!(0.10))
            .output_cost(dec!(0.20))
            .build(),
        CostRecordBuilder::new()
            .tenant_id(org1)
            .input_cost(dec!(0.15))
            .output_cost(dec!(0.25))
            .build(),
        CostRecordBuilder::new()
            .tenant_id(org2)
            .input_cost(dec!(0.05))
            .output_cost(dec!(0.10))
            .build(),
    ];

    let by_org = aggregator.sum_by_organization(&costs);
    assert_eq!(by_org.get(&org1.to_string()), Some(&dec!(0.70)));
    assert_eq!(by_org.get(&org2.to_string()), Some(&dec!(0.15)));
}

#[test]
fn test_aggregator_empty_collection() {
    let aggregator = CostAggregator::new();
    let costs: Vec<CostRecord> = vec![];

    let total = aggregator.sum_total_costs(&costs);
    assert_eq!(total, Decimal::ZERO);

    let by_provider = aggregator.sum_by_provider(&costs);
    assert!(by_provider.is_empty());
}

#[test]
fn test_aggregator_single_record() {
    let aggregator = CostAggregator::new();

    let costs = vec![
        CostRecordBuilder::new()
            .input_cost(dec!(0.10))
            .output_cost(dec!(0.20))
            .build(),
    ];

    let total = aggregator.sum_total_costs(&costs);
    assert_eq!(total, dec!(0.30));
}

// === Performance Tests ===

#[test]
fn test_bulk_calculation_performance() {
    let calculator = CostCalculator::new();
    let pricing = PricingTable {
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
    };

    let start = std::time::Instant::now();

    let usage_records: Vec<_> = (0..1000)
        .map(|_| {
            let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
            UsageRecord::new(
                Provider::OpenAI,
                model,
                "org-123".to_string(),
                1000,
                500,
            )
        })
        .collect();

    for usage in &usage_records {
        let _ = calculator.calculate(usage, &pricing);
    }

    let elapsed = start.elapsed();
    assert!(elapsed.as_secs() < 1, "Bulk calculation too slow: {:?}", elapsed);
}

#[test]
fn test_aggregation_performance() {
    let aggregator = CostAggregator::new();

    let costs: Vec<_> = (0..10000)
        .map(|_| {
            CostRecordBuilder::new()
                .input_cost(dec!(0.01))
                .output_cost(dec!(0.02))
                .build()
        })
        .collect();

    let start = std::time::Instant::now();
    let total = aggregator.sum_total_costs(&costs);
    let elapsed = start.elapsed();

    assert_decimal_positive(total);
    assert!(elapsed.as_millis() < 100, "Aggregation too slow: {:?}", elapsed);
}
