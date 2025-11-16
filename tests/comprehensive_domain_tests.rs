/// Comprehensive domain tests for LLM Cost Ops
///
/// Tests all domain models including cost records, usage records, providers, and pricing

use chrono::{Duration, Utc};
use llm_cost_ops::domain::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

mod helpers;
use helpers::assertions::*;
use helpers::fixtures::*;

// === Provider Tests ===

#[test]
fn test_provider_parsing_all_variants() {
    use std::str::FromStr;

    assert_eq!(Provider::from_str("openai").unwrap(), Provider::OpenAI);
    assert_eq!(Provider::from_str("OpenAI").unwrap(), Provider::OpenAI);
    assert_eq!(Provider::from_str("OPENAI").unwrap(), Provider::OpenAI);

    assert_eq!(Provider::from_str("anthropic").unwrap(), Provider::Anthropic);
    assert_eq!(Provider::from_str("google").unwrap(), Provider::GoogleVertexAI);
    assert_eq!(Provider::from_str("vertex").unwrap(), Provider::GoogleVertexAI);
    assert_eq!(Provider::from_str("azure").unwrap(), Provider::AzureOpenAI);
    assert_eq!(Provider::from_str("aws").unwrap(), Provider::AWSBedrock);
    assert_eq!(Provider::from_str("bedrock").unwrap(), Provider::AWSBedrock);
    assert_eq!(Provider::from_str("cohere").unwrap(), Provider::Cohere);
    assert_eq!(Provider::from_str("mistral").unwrap(), Provider::Mistral);

    match Provider::from_str("custom-llm").unwrap() {
        Provider::Custom(name) => assert_eq!(name, "custom-llm"),
        _ => panic!("Expected Custom provider"),
    }
}

#[test]
fn test_provider_display() {
    assert_eq!(Provider::OpenAI.to_string(), "openai");
    assert_eq!(Provider::Anthropic.to_string(), "anthropic");
    assert_eq!(Provider::GoogleVertexAI.to_string(), "google");
    assert_eq!(Provider::Custom("test".to_string()).to_string(), "test");
}

#[test]
fn test_provider_serialization_all_formats() {
    let providers = vec![
        Provider::OpenAI,
        Provider::Anthropic,
        Provider::GoogleVertexAI,
        Provider::AzureOpenAI,
        Provider::AWSBedrock,
        Provider::Cohere,
        Provider::Mistral,
        Provider::Custom("test".to_string()),
    ];

    for provider in providers {
        let json = serde_json::to_string(&provider).unwrap();
        let deserialized: Provider = serde_json::from_str(&json).unwrap();
        assert_eq!(provider, deserialized);
    }
}

#[test]
fn test_provider_token_validation_support() {
    assert!(Provider::OpenAI.supports_token_validation());
    assert!(Provider::Anthropic.supports_token_validation());
    assert!(Provider::GoogleVertexAI.supports_token_validation());
    assert!(!Provider::Cohere.supports_token_validation());
    assert!(!Provider::Custom("test".to_string()).supports_token_validation());
}

#[test]
fn test_provider_default_context_windows() {
    assert_eq!(Provider::OpenAI.default_context_window("gpt-4"), 8192);
    assert_eq!(Provider::OpenAI.default_context_window("gpt-4-turbo"), 8192);
    assert_eq!(Provider::OpenAI.default_context_window("gpt-3.5-turbo"), 4096);
    assert_eq!(Provider::Anthropic.default_context_window("claude-3-opus"), 200000);
    assert_eq!(Provider::GoogleVertexAI.default_context_window("gemini-pro"), 32768);
    assert_eq!(Provider::Cohere.default_context_window("command"), 4096);
}

// === Usage Record Tests ===

#[test]
fn test_usage_record_creation() {
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let record = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        1000,
        500,
    );

    assert_eq!(record.provider, Provider::OpenAI);
    assert_eq!(record.model.name, "gpt-4");
    assert_eq!(record.organization_id, "org-123");
    assert_eq!(record.prompt_tokens, 1000);
    assert_eq!(record.completion_tokens, 500);
    assert_eq!(record.total_tokens, 1500);
}

#[test]
fn test_usage_record_validation_success() {
    let record = test_usage_record();
    assert!(record.validate().is_ok());
}

#[test]
fn test_usage_record_validation_zero_tokens() {
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let mut record = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        0,
        0,
    );
    record.total_tokens = 0;

    let result = record.validate();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CostOpsError::InvalidTokenCount(_)));
}

#[test]
fn test_usage_record_validation_token_mismatch() {
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let mut record = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        100,
        50,
    );
    record.total_tokens = 200; // Wrong total

    let result = record.validate();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CostOpsError::TokenCountMismatch { .. }));
}

#[test]
fn test_usage_record_validation_empty_org_id() {
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let mut record = UsageRecord::new(
        Provider::OpenAI,
        model,
        "".to_string(), // Empty org ID
        100,
        50,
    );

    let result = record.validate();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CostOpsError::MissingOrganizationId));
}

#[test]
fn test_usage_record_validation_future_timestamp() {
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let mut record = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        100,
        50,
    );
    record.timestamp = Utc::now() + Duration::hours(1);

    let result = record.validate();
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CostOpsError::FutureTimestamp));
}

#[test]
fn test_usage_record_builder_pattern() {
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let record = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        100,
        50,
    )
    .with_project("proj-456".to_string())
    .with_tags(vec!["test".to_string(), "production".to_string()])
    .with_latency(250)
    .with_cached_tokens(20);

    assert_eq!(record.project_id, Some("proj-456".to_string()));
    assert_eq!(record.tags.len(), 2);
    assert_eq!(record.latency_ms, Some(250));
    assert_eq!(record.cached_tokens, Some(20));
}

#[test]
fn test_usage_record_with_reasoning_tokens() {
    let model = ModelIdentifier::new("o1-preview".to_string(), 8192);
    let mut record = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        100,
        50,
    );
    record.reasoning_tokens = Some(1000);

    assert_eq!(record.reasoning_tokens, Some(1000));
    assert!(record.validate().is_ok());
}

#[test]
fn test_usage_record_serialization() {
    let record = test_usage_record();
    let json = serde_json::to_string(&record).unwrap();
    let deserialized: UsageRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(record.id, deserialized.id);
    assert_eq!(record.organization_id, deserialized.organization_id);
    assert_eq!(record.total_tokens, deserialized.total_tokens);
}

// === Cost Record Tests ===

#[test]
fn test_cost_record_creation() {
    let calculation = CostCalculation::new(
        dec!(0.01),
        dec!(0.02),
        Currency::USD,
        Uuid::new_v4(),
    );

    let record = CostRecord::new(
        Uuid::new_v4(),
        Provider::OpenAI,
        "gpt-4".to_string(),
        "org-123".to_string(),
        calculation,
        PricingStructure::simple_per_token(dec!(10.0), dec!(30.0)),
    );

    assert_eq!(record.provider, Provider::OpenAI);
    assert_eq!(record.model, "gpt-4");
    assert_eq!(record.input_cost, dec!(0.01));
    assert_eq!(record.output_cost, dec!(0.02));
    assert_eq!(record.total_cost, dec!(0.03));
}

#[test]
fn test_cost_calculation_total() {
    let calc = CostCalculation::new(
        dec!(0.05),
        dec!(0.15),
        Currency::USD,
        Uuid::new_v4(),
    );

    assert_eq!(calc.input_cost, dec!(0.05));
    assert_eq!(calc.output_cost, dec!(0.15));
    assert_eq!(calc.total_cost, dec!(0.20));
}

#[test]
fn test_cost_per_token_calculation() {
    let calculation = CostCalculation::new(
        dec!(0.10),
        dec!(0.30),
        Currency::USD,
        Uuid::new_v4(),
    );

    let record = CostRecord::new(
        Uuid::new_v4(),
        Provider::OpenAI,
        "gpt-4".to_string(),
        "org-123".to_string(),
        calculation,
        PricingStructure::simple_per_token(dec!(10.0), dec!(30.0)),
    );

    let cost_per_token = record.cost_per_token(1000);
    assert_eq!(cost_per_token, dec!(0.0004));
}

#[test]
fn test_cost_per_token_zero_tokens() {
    let record = test_cost_record();
    let cost_per_token = record.cost_per_token(0);
    assert_eq!(cost_per_token, Decimal::ZERO);
}

#[test]
fn test_cost_record_with_project_and_tags() {
    let calculation = CostCalculation::new(
        dec!(0.01),
        dec!(0.02),
        Currency::USD,
        Uuid::new_v4(),
    );

    let record = CostRecord::new(
        Uuid::new_v4(),
        Provider::OpenAI,
        "gpt-4".to_string(),
        "org-123".to_string(),
        calculation,
        PricingStructure::simple_per_token(dec!(10.0), dec!(30.0)),
    )
    .with_project("proj-456".to_string())
    .with_tags(vec!["prod".to_string(), "api".to_string()]);

    assert_eq!(record.project_id, Some("proj-456".to_string()));
    assert_eq!(record.tags.len(), 2);
}

#[test]
fn test_cost_record_serialization() {
    let record = test_cost_record();
    let json = serde_json::to_string(&record).unwrap();
    let deserialized: CostRecord = serde_json::from_str(&json).unwrap();

    assert_eq!(record.id, deserialized.id);
    assert_eq!(record.total_cost, deserialized.total_cost);
}

// === Model Identifier Tests ===

#[test]
fn test_model_identifier_creation() {
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    assert_eq!(model.name, "gpt-4");
    assert_eq!(model.context_window, Some(8192));
    assert_eq!(model.version, None);
}

#[test]
fn test_model_identifier_with_version() {
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192)
        .with_version("0613".to_string());

    assert_eq!(model.version, Some("0613".to_string()));
}

// === Ingestion Source Tests ===

#[test]
fn test_ingestion_source_variants() {
    let api_source = IngestionSource::Api {
        endpoint: "/v1/usage".to_string(),
    };
    let file_source = IngestionSource::File {
        path: "/data/usage.json".to_string(),
    };
    let webhook_source = IngestionSource::Webhook {
        endpoint: "https://example.com/webhook".to_string(),
    };
    let stream_source = IngestionSource::Stream {
        topic: "usage-events".to_string(),
    };

    // Verify serialization
    let sources = vec![api_source, file_source, webhook_source, stream_source];
    for source in sources {
        let json = serde_json::to_string(&source).unwrap();
        let deserialized: IngestionSource = serde_json::from_str(&json).unwrap();
        // Just verify it doesn't panic
        let _ = serde_json::to_string(&deserialized);
    }
}

// === Edge Cases and Boundary Tests ===

#[test]
fn test_large_token_counts() {
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let record = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        1_000_000,
        500_000,
    );

    assert_eq!(record.total_tokens, 1_500_000);
    assert!(record.validate().is_ok());
}

#[test]
fn test_very_small_costs() {
    let calculation = CostCalculation::new(
        Decimal::new(1, 10), // 0.0000000001
        Decimal::new(2, 10), // 0.0000000002
        Currency::USD,
        Uuid::new_v4(),
    );

    assert_decimal_positive(calculation.total_cost);
}

#[test]
fn test_timestamp_precision() {
    let now = Utc::now();
    let model = ModelIdentifier::new("gpt-4".to_string(), 8192);
    let mut record = UsageRecord::new(
        Provider::OpenAI,
        model,
        "org-123".to_string(),
        100,
        50,
    );
    record.timestamp = now;

    assert_timestamp_near(record.timestamp, now, Duration::milliseconds(1));
}

// === Currency Tests ===

#[test]
fn test_currency_variants() {
    let currencies = vec![Currency::USD];

    for currency in currencies {
        let json = serde_json::to_string(&currency).unwrap();
        let deserialized: Currency = serde_json::from_str(&json).unwrap();
        assert_eq!(currency, deserialized);
    }
}

// === Performance Tests ===

#[test]
fn test_bulk_record_creation() {
    let start = std::time::Instant::now();

    let records: Vec<_> = (0..1000)
        .map(|i| {
            let model = ModelIdentifier::new(format!("model-{}", i), 8192);
            UsageRecord::new(
                Provider::OpenAI,
                model,
                format!("org-{}", i),
                100,
                50,
            )
        })
        .collect();

    let elapsed = start.elapsed();
    assert_eq!(records.len(), 1000);
    assert!(elapsed.as_secs() < 1, "Bulk creation too slow: {:?}", elapsed);
}
