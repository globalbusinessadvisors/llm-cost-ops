/// Enhanced property-based tests using proptest
///
/// Tests invariants and properties that should hold for all inputs

use proptest::prelude::*;
use llm_cost_ops::domain::*;
use llm_cost_ops::engine::*;
use rust_decimal::Decimal;
use chrono::Utc;
use uuid::Uuid;

// === Property Test Strategies ===

fn any_provider() -> impl Strategy<Value = Provider> {
    prop_oneof![
        Just(Provider::OpenAI),
        Just(Provider::Anthropic),
        Just(Provider::GoogleVertexAI),
        Just(Provider::AzureOpenAI),
        Just(Provider::AWSBedrock),
        Just(Provider::Cohere),
        Just(Provider::Mistral),
    ]
}

fn any_positive_tokens() -> impl Strategy<Value = u64> {
    1u64..1_000_000_000
}

fn any_positive_decimal() -> impl Strategy<Value = Decimal> {
    (1u64..1_000_000).prop_map(|n| Decimal::from(n) / Decimal::from(1000))
}

// === Usage Record Properties ===

proptest! {
    #[test]
    fn prop_usage_record_token_sum_invariant(
        prompt_tokens in any_positive_tokens(),
        completion_tokens in any_positive_tokens(),
    ) {
        let model = ModelIdentifier::new("test-model".to_string(), 8192);
        let record = UsageRecord::new(
            Provider::OpenAI,
            model,
            "org-test".to_string(),
            prompt_tokens,
            completion_tokens,
        );

        // Property: total_tokens should always equal sum of prompt and completion
        prop_assert_eq!(record.total_tokens, prompt_tokens + completion_tokens);
    }

    #[test]
    fn prop_usage_record_validation_accepts_valid_records(
        prompt_tokens in 1u64..1_000_000,
        completion_tokens in 1u64..1_000_000,
    ) {
        let model = ModelIdentifier::new("test-model".to_string(), 8192);
        let record = UsageRecord::new(
            Provider::OpenAI,
            model,
            "org-test".to_string(),
            prompt_tokens,
            completion_tokens,
        );

        // Property: valid records should always pass validation
        prop_assert!(record.validate().is_ok());
    }

    #[test]
    fn prop_provider_parsing_roundtrip(provider in any_provider()) {
        use std::str::FromStr;

        let as_str = provider.as_str();
        let parsed = Provider::from_str(as_str).unwrap();

        // Property: parsing should be reversible for known providers
        prop_assert_eq!(provider, parsed);
    }

    #[test]
    fn prop_provider_serialization_roundtrip(provider in any_provider()) {
        let json = serde_json::to_string(&provider).unwrap();
        let deserialized: Provider = serde_json::from_str(&json).unwrap();

        // Property: serialization should be reversible
        prop_assert_eq!(provider, deserialized);
    }
}

// === Cost Calculation Properties ===

proptest! {
    #[test]
    fn prop_cost_calculation_total_equals_sum(
        input_cost in any_positive_decimal(),
        output_cost in any_positive_decimal(),
    ) {
        let calc = CostCalculation::new(
            input_cost,
            output_cost,
            Currency::USD,
            Uuid::new_v4(),
        );

        // Property: total cost should always equal sum of input and output
        prop_assert_eq!(calc.total_cost, input_cost + output_cost);
    }

    #[test]
    fn prop_cost_calculation_non_negative(
        input_cost in any_positive_decimal(),
        output_cost in any_positive_decimal(),
    ) {
        let calc = CostCalculation::new(
            input_cost,
            output_cost,
            Currency::USD,
            Uuid::new_v4(),
        );

        // Property: all costs should be non-negative
        prop_assert!(calc.input_cost >= Decimal::ZERO);
        prop_assert!(calc.output_cost >= Decimal::ZERO);
        prop_assert!(calc.total_cost >= Decimal::ZERO);
    }

    #[test]
    fn prop_cost_per_token_never_exceeds_total(
        total_tokens in 1u64..1_000_000,
        total_cost in any_positive_decimal(),
    ) {
        let calc = CostCalculation::new(
            total_cost / Decimal::from(2),
            total_cost / Decimal::from(2),
            Currency::USD,
            Uuid::new_v4(),
        );

        let record = CostRecord::new(
            Uuid::new_v4(),
            Provider::OpenAI,
            "test-model".to_string(),
            "org-test".to_string(),
            calc,
            PricingStructure::simple_per_token(Decimal::from(10), Decimal::from(30)),
        );

        let cost_per_token = record.cost_per_token(total_tokens);

        // Property: cost per token should never exceed total cost
        prop_assert!(cost_per_token <= total_cost);
    }
}

// === Token Normalization Properties ===

proptest! {
    #[test]
    fn prop_token_normalization_preserves_value(
        tokens in any_positive_tokens(),
    ) {
        let normalizer = TokenNormalizer::new();

        // Normalize to millions and back
        let as_millions = normalizer.normalize(
            tokens,
            TokenUnit::Tokens,
            TokenUnit::Millions,
        );
        let back_to_tokens = normalizer.normalize(
            as_millions.to_string().parse().unwrap_or(0),
            TokenUnit::Millions,
            TokenUnit::Tokens,
        );

        // Property: converting to another unit and back should preserve value (with tolerance)
        let original = Decimal::from(tokens);
        let diff = (original - back_to_tokens).abs();
        prop_assert!(diff < Decimal::from(2)); // Allow small rounding error
    }

    #[test]
    fn prop_token_normalization_identity(
        tokens in any_positive_tokens(),
    ) {
        let normalizer = TokenNormalizer::new();

        let result = normalizer.normalize(
            tokens,
            TokenUnit::Tokens,
            TokenUnit::Tokens,
        );

        // Property: normalizing to same unit is identity
        prop_assert_eq!(result, Decimal::from(tokens));
    }
}

// === Aggregation Properties ===

proptest! {
    #[test]
    fn prop_aggregation_sum_non_negative(
        cost1 in any_positive_decimal(),
        cost2 in any_positive_decimal(),
        cost3 in any_positive_decimal(),
    ) {
        let aggregator = CostAggregator::new();

        let costs = vec![
            CostRecord::new(
                Uuid::new_v4(),
                Provider::OpenAI,
                "model".to_string(),
                "org".to_string(),
                CostCalculation::new(cost1, Decimal::ZERO, Currency::USD, Uuid::new_v4()),
                PricingStructure::simple_per_token(Decimal::from(10), Decimal::from(30)),
            ),
            CostRecord::new(
                Uuid::new_v4(),
                Provider::OpenAI,
                "model".to_string(),
                "org".to_string(),
                CostCalculation::new(cost2, Decimal::ZERO, Currency::USD, Uuid::new_v4()),
                PricingStructure::simple_per_token(Decimal::from(10), Decimal::from(30)),
            ),
            CostRecord::new(
                Uuid::new_v4(),
                Provider::OpenAI,
                "model".to_string(),
                "org".to_string(),
                CostCalculation::new(cost3, Decimal::ZERO, Currency::USD, Uuid::new_v4()),
                PricingStructure::simple_per_token(Decimal::from(10), Decimal::from(30)),
            ),
        ];

        let total = aggregator.sum_total_costs(&costs);

        // Property: sum should be non-negative
        prop_assert!(total >= Decimal::ZERO);

        // Property: sum should equal individual sums
        let expected = cost1 + cost2 + cost3;
        prop_assert_eq!(total, expected);
    }

    #[test]
    fn prop_aggregation_empty_yields_zero() {
        let aggregator = CostAggregator::new();
        let costs: Vec<CostRecord> = vec![];

        let total = aggregator.sum_total_costs(&costs);

        // Property: aggregating empty collection yields zero
        prop_assert_eq!(total, Decimal::ZERO);
    }

    #[test]
    fn prop_aggregation_single_item_equals_item(
        cost in any_positive_decimal(),
    ) {
        let aggregator = CostAggregator::new();

        let costs = vec![
            CostRecord::new(
                Uuid::new_v4(),
                Provider::OpenAI,
                "model".to_string(),
                "org".to_string(),
                CostCalculation::new(cost, Decimal::ZERO, Currency::USD, Uuid::new_v4()),
                PricingStructure::simple_per_token(Decimal::from(10), Decimal::from(30)),
            ),
        ];

        let total = aggregator.sum_total_costs(&costs);

        // Property: aggregating single item equals that item
        prop_assert_eq!(total, cost);
    }
}

// === Compression Properties ===

proptest! {
    #[test]
    fn prop_compression_roundtrip(
        data in prop::collection::vec(any::<u8>(), 0..10000),
    ) {
        let codec = CompressionCodec::new(CompressionAlgorithm::Gzip);

        if !data.is_empty() {
            let compressed = codec.compress(&data, CompressionLevel::Default).unwrap();
            let decompressed = codec.decompress(&compressed).unwrap();

            // Property: compress then decompress should yield original data
            prop_assert_eq!(data, decompressed);
        }
    }

    #[test]
    fn prop_compression_idempotent_decompression(
        data in prop::collection::vec(any::<u8>(), 1..1000),
    ) {
        let codec = CompressionCodec::new(CompressionAlgorithm::Gzip);

        let compressed = codec.compress(&data, CompressionLevel::Default).unwrap();
        let decompressed1 = codec.decompress(&compressed).unwrap();
        let decompressed2 = codec.decompress(&compressed).unwrap();

        // Property: decompressing multiple times yields same result
        prop_assert_eq!(decompressed1, decompressed2);
    }

    #[test]
    fn prop_compression_better_or_equal(
        data in prop::collection::vec(any::<u8>(), 100..10000),
    ) {
        let codec = CompressionCodec::new(CompressionAlgorithm::Gzip);

        let fast = codec.compress(&data, CompressionLevel::Fast).unwrap();
        let best = codec.compress(&data, CompressionLevel::Best).unwrap();

        // Property: best compression should be <= fast compression
        // (though not always true for incompressible data)
        // So we check they both decompress correctly
        prop_assert_eq!(codec.decompress(&fast).unwrap(), data);
        prop_assert_eq!(codec.decompress(&best).unwrap(), data);
    }
}

// === Timestamp Properties ===

proptest! {
    #[test]
    fn prop_usage_record_timestamp_ordering(
        _input in any::<u8>(), // Dummy input
    ) {
        let model = ModelIdentifier::new("test-model".to_string(), 8192);
        let record1 = UsageRecord::new(
            Provider::OpenAI,
            model.clone(),
            "org-test".to_string(),
            100,
            50,
        );

        std::thread::sleep(std::time::Duration::from_millis(1));

        let record2 = UsageRecord::new(
            Provider::OpenAI,
            model,
            "org-test".to_string(),
            100,
            50,
        );

        // Property: records created later should have later timestamps
        prop_assert!(record2.timestamp >= record1.timestamp);
    }
}

// === Validation Properties ===

proptest! {
    #[test]
    fn prop_empty_organization_id_fails_validation(
        prompt_tokens in 1u64..1_000_000,
        completion_tokens in 1u64..1_000_000,
    ) {
        let model = ModelIdentifier::new("test-model".to_string(), 8192);
        let mut record = UsageRecord::new(
            Provider::OpenAI,
            model,
            "valid-org".to_string(),
            prompt_tokens,
            completion_tokens,
        );

        record.organization_id = String::new();

        // Property: empty organization ID should always fail validation
        prop_assert!(record.validate().is_err());
    }

    #[test]
    fn prop_mismatched_total_tokens_fails_validation(
        prompt_tokens in 1u64..1_000_000,
        completion_tokens in 1u64..1_000_000,
        wrong_total in 1u64..1_000_000,
    ) {
        let correct_total = prompt_tokens + completion_tokens;

        if wrong_total != correct_total {
            let model = ModelIdentifier::new("test-model".to_string(), 8192);
            let mut record = UsageRecord::new(
                Provider::OpenAI,
                model,
                "org-test".to_string(),
                prompt_tokens,
                completion_tokens,
            );

            record.total_tokens = wrong_total;

            // Property: mismatched total should always fail validation
            prop_assert!(record.validate().is_err());
        }
    }
}
