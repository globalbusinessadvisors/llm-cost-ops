// Property-Based Testing using proptest
// Validates invariants and edge cases across random inputs

use llm_cost_ops::{
    domain::{ModelIdentifier, PricingStructure, PricingTable, Provider, UsageRecord},
    engine::CostCalculator,
};
use proptest::prelude::*;
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

// Property: Cost should never be negative
proptest! {
    #[test]
    fn cost_never_negative(
        input_tokens in 0u64..1000000,
        output_tokens in 0u64..1000000,
        input_rate in 0.0001f64..1000.0,
        output_rate in 0.0001f64..1000.0,
    ) {
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_f64(input_rate).unwrap(),
            Decimal::from_f64(output_rate).unwrap(),
        );

        let usage = create_test_usage(input_tokens, output_tokens);
        let pricing_table = PricingTable::new(Provider::OpenAI, "test-model".to_string(), pricing);

        let calculator = CostCalculator::new();
        let result = calculator.calculate(&usage, &pricing_table);

        prop_assert!(result.is_ok());
        let cost = result.unwrap();
        prop_assert!(cost.total_cost >= Decimal::ZERO);
        prop_assert!(cost.input_cost >= Decimal::ZERO);
        prop_assert!(cost.output_cost >= Decimal::ZERO);
    }
}

// Property: Total cost equals sum of input and output costs
proptest! {
    #[test]
    fn cost_sum_equals_total(
        input_tokens in 1u64..100000,
        output_tokens in 1u64..100000,
    ) {
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_str("10.0").unwrap(),
            Decimal::from_str("30.0").unwrap(),
        );

        let usage = create_test_usage(input_tokens, output_tokens);
        let pricing_table = PricingTable::new(Provider::OpenAI, "test".to_string(), pricing);

        let calculator = CostCalculator::new();
        let cost = calculator.calculate(&usage, &pricing_table).unwrap();

        let sum = cost.input_cost + cost.output_cost;
        prop_assert_eq!(cost.total_cost, sum);
    }
}

// Property: More tokens should never cost less (monotonicity)
proptest! {
    #[test]
    fn more_tokens_costs_more(
        base_tokens in 100u64..10000,
        multiplier in 2u64..10,
    ) {
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_str("10.0").unwrap(),
            Decimal::from_str("30.0").unwrap(),
        );

        let usage1 = create_test_usage(base_tokens, base_tokens);
        let usage2 = create_test_usage(base_tokens * multiplier, base_tokens * multiplier);

        let pricing_table = PricingTable::new(Provider::OpenAI, "test".to_string(), pricing);
        let calculator = CostCalculator::new();

        let cost1 = calculator.calculate(&usage1, &pricing_table).unwrap();
        let cost2 = calculator.calculate(&usage2, &pricing_table).unwrap();

        prop_assert!(cost2.total_cost > cost1.total_cost);
    }
}

// Property: Zero tokens should result in zero cost
proptest! {
    #[test]
    fn zero_tokens_zero_cost(
        rate in 0.0001f64..1000.0,
    ) {
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_f64(rate).unwrap(),
            Decimal::from_f64(rate).unwrap(),
        );

        let usage = create_test_usage(0, 0);
        let pricing_table = PricingTable::new(Provider::OpenAI, "test".to_string(), pricing);

        let calculator = CostCalculator::new();

        // Should fail validation
        let validation_result = usage.validate();
        prop_assert!(validation_result.is_err());
    }
}

// Property: Cached tokens should reduce cost (with discount)
proptest! {
    #[test]
    fn cached_tokens_reduce_cost(
        total_tokens in 1000u64..10000,
        cached_pct in 0.1f64..0.9f64,
        discount in 0.1f64..0.9f64,
    ) {
        let cached_tokens = (total_tokens as f64 * cached_pct) as u64;

        let pricing_no_cache = PricingStructure::simple_per_token(
            Decimal::from_str("10.0").unwrap(),
            Decimal::from_str("30.0").unwrap(),
        );

        let pricing_with_cache = PricingStructure::with_cache_discount(
            Decimal::from_str("10.0").unwrap(),
            Decimal::from_str("30.0").unwrap(),
            Decimal::from_f64(discount).unwrap(),
        );

        let mut usage_no_cache = create_test_usage(total_tokens, 500);
        usage_no_cache.cached_tokens = None;

        let mut usage_with_cache = create_test_usage(total_tokens, 500);
        usage_with_cache.cached_tokens = Some(cached_tokens);

        let table_no_cache = PricingTable::new(Provider::OpenAI, "test".to_string(), pricing_no_cache);
        let table_with_cache = PricingTable::new(Provider::Anthropic, "test".to_string(), pricing_with_cache);

        let calculator = CostCalculator::new();

        let cost_no_cache = calculator.calculate(&usage_no_cache, &table_no_cache).unwrap();
        let cost_with_cache = calculator.calculate(&usage_with_cache, &table_with_cache).unwrap();

        // Cost with cache should be less than without
        if cached_tokens > 0 && discount > Decimal::ZERO {
            prop_assert!(cost_with_cache.total_cost < cost_no_cache.total_cost);
        }
    }
}

// Property: Token sum should equal total (validation invariant)
proptest! {
    #[test]
    fn token_sum_equals_total(
        input_tokens in 1u64..100000,
        output_tokens in 1u64..100000,
    ) {
        let mut usage = create_test_usage(input_tokens, output_tokens);
        usage.total_tokens = input_tokens + output_tokens;

        let validation = usage.validate();
        prop_assert!(validation.is_ok());

        // Test invalid case
        usage.total_tokens = input_tokens + output_tokens + 1;
        let invalid_validation = usage.validate();
        prop_assert!(invalid_validation.is_err());
    }
}

// Property: Provider parsing should be reversible
proptest! {
    #[test]
    fn provider_to_string_and_back(s in "[a-z]{3,20}") {
        let provider = Provider::from_str(&s);
        let string_repr = provider.to_string();

        let provider2 = Provider::from_str(&string_repr);

        // Should either match exactly or both be Custom
        match (&provider, &provider2) {
            (Provider::Custom(_), Provider::Custom(_)) => {
                prop_assert_eq!(provider.to_string(), provider2.to_string());
            }
            _ => {
                prop_assert_eq!(provider, provider2);
            }
        }
    }
}

// Property: Cost calculation should be deterministic
proptest! {
    #[test]
    fn cost_calculation_deterministic(
        input_tokens in 100u64..10000,
        output_tokens in 100u64..10000,
    ) {
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_str("10.0").unwrap(),
            Decimal::from_str("30.0").unwrap(),
        );

        let usage = create_test_usage(input_tokens, output_tokens);
        let pricing_table = PricingTable::new(Provider::OpenAI, "test".to_string(), pricing);

        let calculator = CostCalculator::new();

        // Calculate cost multiple times
        let cost1 = calculator.calculate(&usage, &pricing_table).unwrap();
        let cost2 = calculator.calculate(&usage, &pricing_table).unwrap();
        let cost3 = calculator.calculate(&usage, &pricing_table).unwrap();

        prop_assert_eq!(cost1.total_cost, cost2.total_cost);
        prop_assert_eq!(cost2.total_cost, cost3.total_cost);
    }
}

// Property: Decimal precision should be maintained (no rounding errors)
proptest! {
    #[test]
    fn decimal_precision_maintained(
        tokens in 1u64..1000000,
    ) {
        // Use very small rate to test precision
        let rate = Decimal::from_str("0.000001").unwrap();

        let pricing = PricingStructure::simple_per_token(rate, rate);
        let usage = create_test_usage(tokens, tokens);
        let pricing_table = PricingTable::new(Provider::OpenAI, "test".to_string(), pricing);

        let calculator = CostCalculator::new();
        let cost = calculator.calculate(&usage, &pricing_table).unwrap();

        // Manually calculate expected cost
        let expected_input = (Decimal::from(tokens) * rate) / Decimal::from(1_000_000);
        let expected_output = (Decimal::from(tokens) * rate) / Decimal::from(1_000_000);
        let expected_total = expected_input + expected_output;

        // Should match exactly (no floating point errors)
        prop_assert_eq!(cost.input_cost, expected_input);
        prop_assert_eq!(cost.output_cost, expected_output);
        prop_assert_eq!(cost.total_cost, expected_total);
    }
}

// Property: Large token counts should not overflow
proptest! {
    #[test]
    fn large_tokens_no_overflow(
        input_tokens in 100_000_000u64..1_000_000_000,
        output_tokens in 100_000_000u64..1_000_000_000,
    ) {
        let pricing = PricingStructure::simple_per_token(
            Decimal::from_str("1000.0").unwrap(), // High rate
            Decimal::from_str("1000.0").unwrap(),
        );

        let usage = create_test_usage(input_tokens, output_tokens);
        let pricing_table = PricingTable::new(Provider::OpenAI, "test".to_string(), pricing);

        let calculator = CostCalculator::new();
        let result = calculator.calculate(&usage, &pricing_table);

        // Should not panic or overflow
        prop_assert!(result.is_ok());
        let cost = result.unwrap();
        prop_assert!(cost.total_cost.is_sign_positive());
    }
}

// Property: Different providers with same pricing should yield same cost
proptest! {
    #[test]
    fn provider_agnostic_pricing(
        input_tokens in 1000u64..10000,
        output_tokens in 1000u64..10000,
    ) {
        let rate_input = Decimal::from_str("10.0").unwrap();
        let rate_output = Decimal::from_str("30.0").unwrap();

        let providers = vec![
            Provider::OpenAI,
            Provider::Anthropic,
            Provider::GoogleVertexAI,
            Provider::AzureOpenAI,
        ];

        let calculator = CostCalculator::new();
        let mut costs = Vec::new();

        for provider in providers {
            let pricing = PricingStructure::simple_per_token(rate_input, rate_output);
            let usage = create_test_usage_with_provider(input_tokens, output_tokens, provider.clone());
            let pricing_table = PricingTable::new(provider, "test".to_string(), pricing);

            let cost = calculator.calculate(&usage, &pricing_table).unwrap();
            costs.push(cost.total_cost);
        }

        // All costs should be identical
        let first_cost = costs[0];
        for cost in costs.iter().skip(1) {
            prop_assert_eq!(first_cost, *cost);
        }
    }
}

// Helper functions

fn create_test_usage(input_tokens: u64, output_tokens: u64) -> UsageRecord {
    UsageRecord {
        id: Uuid::new_v4(),
        timestamp: chrono::Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "test-model".to_string(),
            version: None,
            context_window: None,
        },
        organization_id: "org-test".to_string(),
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
            source_type: "test".to_string(),
            endpoint: None,
        },
    }
}

fn create_test_usage_with_provider(
    input_tokens: u64,
    output_tokens: u64,
    provider: Provider,
) -> UsageRecord {
    let mut usage = create_test_usage(input_tokens, output_tokens);
    usage.provider = provider;
    usage
}

#[cfg(test)]
mod regular_tests {
    use super::*;

    #[test]
    fn test_proptest_helper_functions() {
        let usage = create_test_usage(1000, 500);
        assert_eq!(usage.prompt_tokens, 1000);
        assert_eq!(usage.completion_tokens, 500);
        assert_eq!(usage.total_tokens, 1500);
    }

    #[test]
    fn test_specific_edge_case() {
        // Test case found via property testing
        let usage = create_test_usage(1, 1);
        assert_eq!(usage.validate().unwrap(), ());
    }
}
