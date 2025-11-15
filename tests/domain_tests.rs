use chrono::Utc;
use llm_cost_ops::domain::{
    CostOpsError, ModelIdentifier, PricingStructure, PricingTable, Provider, UsageRecord,
};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

#[test]
fn test_provider_from_str() {
    assert_eq!(Provider::from_str("openai"), Provider::OpenAI);
    assert_eq!(Provider::from_str("anthropic"), Provider::Anthropic);
    assert_eq!(Provider::from_str("google"), Provider::GoogleVertexAI);
    assert_eq!(
        Provider::from_str("custom-provider"),
        Provider::Custom("custom-provider".to_string())
    );
}

#[test]
fn test_provider_display() {
    assert_eq!(Provider::OpenAI.to_string(), "openai");
    assert_eq!(Provider::Anthropic.to_string(), "anthropic");
    assert_eq!(Provider::GoogleVertexAI.to_string(), "google");
    assert_eq!(
        Provider::Custom("test".to_string()).to_string(),
        "custom:test"
    );
}

#[test]
fn test_provider_supports_token_validation() {
    assert!(Provider::OpenAI.supports_token_validation());
    assert!(Provider::Anthropic.supports_token_validation());
    assert!(Provider::GoogleVertexAI.supports_token_validation());
    assert!(!Provider::AzureOpenAI.supports_token_validation());
}

#[test]
fn test_usage_record_validation_success() {
    let record = UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "gpt-4".to_string(),
            version: Some("gpt-4-0613".to_string()),
            context_window: Some(8192),
        },
        organization_id: "org-123".to_string(),
        project_id: Some("proj-456".to_string()),
        user_id: Some("user-789".to_string()),
        prompt_tokens: 1000,
        completion_tokens: 500,
        total_tokens: 1500,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: Some(2500),
        tags: vec!["production".to_string()],
        metadata: serde_json::json!({}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "api".to_string(),
            endpoint: Some("https://api.example.com".to_string()),
        },
    };

    assert!(record.validate().is_ok());
}

#[test]
fn test_usage_record_validation_zero_tokens() {
    let mut record = UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "gpt-4".to_string(),
            version: None,
            context_window: None,
        },
        organization_id: "org-123".to_string(),
        project_id: None,
        user_id: None,
        prompt_tokens: 0,
        completion_tokens: 0,
        total_tokens: 0,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: None,
        tags: vec![],
        metadata: serde_json::json!({}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "api".to_string(),
            endpoint: None,
        },
    };

    assert!(matches!(
        record.validate(),
        Err(CostOpsError::InvalidTokenCount(_))
    ));

    record.total_tokens = 100;
    assert!(record.validate().is_ok());
}

#[test]
fn test_usage_record_validation_token_mismatch() {
    let record = UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "gpt-4".to_string(),
            version: None,
            context_window: None,
        },
        organization_id: "org-123".to_string(),
        project_id: None,
        user_id: None,
        prompt_tokens: 1000,
        completion_tokens: 500,
        total_tokens: 2000, // Should be 1500
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: None,
        tags: vec![],
        metadata: serde_json::json!({}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "api".to_string(),
            endpoint: None,
        },
    };

    assert!(matches!(
        record.validate(),
        Err(CostOpsError::TokenCountMismatch { .. })
    ));
}

#[test]
fn test_usage_record_with_cached_tokens() {
    let record = UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::Anthropic,
        model: ModelIdentifier {
            name: "claude-3-sonnet".to_string(),
            version: Some("20240229".to_string()),
            context_window: Some(200000),
        },
        organization_id: "org-123".to_string(),
        project_id: None,
        user_id: None,
        prompt_tokens: 2000,
        completion_tokens: 1000,
        total_tokens: 3000,
        cached_tokens: Some(500),
        reasoning_tokens: None,
        latency_ms: Some(3200),
        tags: vec![],
        metadata: serde_json::json!({}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "api".to_string(),
            endpoint: None,
        },
    };

    assert!(record.validate().is_ok());
}

#[test]
fn test_pricing_structure_per_token() {
    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );

    match pricing {
        PricingStructure::PerToken {
            input_price_per_million,
            output_price_per_million,
            cached_input_discount,
        } => {
            assert_eq!(input_price_per_million, Decimal::from_str("10.0").unwrap());
            assert_eq!(
                output_price_per_million,
                Decimal::from_str("30.0").unwrap()
            );
            assert_eq!(cached_input_discount, None);
        }
        _ => panic!("Expected PerToken pricing structure"),
    }
}

#[test]
fn test_pricing_structure_with_cache_discount() {
    let pricing = PricingStructure::with_cache_discount(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
        Decimal::from_str("0.5").unwrap(),
    );

    match pricing {
        PricingStructure::PerToken {
            input_price_per_million,
            output_price_per_million,
            cached_input_discount,
        } => {
            assert_eq!(input_price_per_million, Decimal::from_str("10.0").unwrap());
            assert_eq!(
                output_price_per_million,
                Decimal::from_str("30.0").unwrap()
            );
            assert_eq!(
                cached_input_discount,
                Some(Decimal::from_str("0.5").unwrap())
            );
        }
        _ => panic!("Expected PerToken pricing structure"),
    }
}

#[test]
fn test_pricing_table_creation() {
    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );

    let table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);

    assert_eq!(table.provider, Provider::OpenAI);
    assert_eq!(table.model, "gpt-4");
    assert_eq!(table.currency.as_str(), "USD");
    assert!(table.end_date.is_none());
}

#[test]
fn test_pricing_table_is_active() {
    let pricing = PricingStructure::simple_per_token(
        Decimal::from_str("10.0").unwrap(),
        Decimal::from_str("30.0").unwrap(),
    );

    let mut table = PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing);

    // Should be active now
    assert!(table.is_active(&Utc::now()));

    // Set end date in the past
    table.end_date = Some(Utc::now() - chrono::Duration::days(1));
    assert!(!table.is_active(&Utc::now()));

    // Set end date in the future
    table.end_date = Some(Utc::now() + chrono::Duration::days(1));
    assert!(table.is_active(&Utc::now()));
}

#[test]
fn test_model_identifier_display() {
    let model = ModelIdentifier {
        name: "gpt-4".to_string(),
        version: Some("gpt-4-0613".to_string()),
        context_window: Some(8192),
    };

    assert_eq!(model.to_string(), "gpt-4 (gpt-4-0613)");

    let model_no_version = ModelIdentifier {
        name: "gpt-4".to_string(),
        version: None,
        context_window: Some(8192),
    };

    assert_eq!(model_no_version.to_string(), "gpt-4");
}

#[test]
fn test_usage_record_serialization() {
    let record = UsageRecord {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier {
            name: "gpt-4".to_string(),
            version: Some("gpt-4-0613".to_string()),
            context_window: Some(8192),
        },
        organization_id: "org-123".to_string(),
        project_id: Some("proj-456".to_string()),
        user_id: Some("user-789".to_string()),
        prompt_tokens: 1000,
        completion_tokens: 500,
        total_tokens: 1500,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: Some(2500),
        tags: vec!["production".to_string(), "api".to_string()],
        metadata: serde_json::json!({"request_id": "req-123"}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::domain::IngestionSource {
            source_type: "api".to_string(),
            endpoint: Some("https://api.example.com".to_string()),
        },
    };

    // Test serialization
    let json = serde_json::to_string(&record).unwrap();
    assert!(json.contains("gpt-4"));
    assert!(json.contains("org-123"));

    // Test deserialization
    let deserialized: UsageRecord = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, record.id);
    assert_eq!(deserialized.provider, record.provider);
    assert_eq!(deserialized.model.name, record.model.name);
}
