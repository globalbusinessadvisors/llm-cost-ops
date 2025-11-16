/// Test fixtures for common test data
///
/// Provides factory methods for creating test data objects

use chrono::{DateTime, Utc};
use llm_cost_ops::domain::{
    cost::CostRecord,
    pricing::PricingTier,
    provider::{LLMProvider, ModelConfig},
    usage::UsageRecord,
};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Create a test provider
pub fn test_provider() -> LLMProvider {
    LLMProvider::OpenAI
}

/// Create a test model config
pub fn test_model_config() -> ModelConfig {
    ModelConfig {
        model_name: "gpt-4".to_string(),
        provider: LLMProvider::OpenAI,
        input_price_per_1k: Decimal::new(30, 3), // $0.03
        output_price_per_1k: Decimal::new(60, 3), // $0.06
        context_window: 8192,
        max_output_tokens: 4096,
    }
}

/// Create a test usage record
pub fn test_usage_record() -> UsageRecord {
    UsageRecord {
        id: Uuid::new_v4(),
        tenant_id: test_tenant_id(),
        request_id: Uuid::new_v4().to_string(),
        provider: LLMProvider::OpenAI,
        model: "gpt-4".to_string(),
        input_tokens: 100,
        output_tokens: 50,
        total_tokens: 150,
        timestamp: Utc::now(),
        metadata: serde_json::json!({
            "user_id": "test_user",
            "session_id": "test_session"
        }),
    }
}

/// Create a test cost record
pub fn test_cost_record() -> CostRecord {
    CostRecord {
        id: Uuid::new_v4(),
        tenant_id: test_tenant_id(),
        usage_id: Uuid::new_v4(),
        provider: LLMProvider::OpenAI,
        model: "gpt-4".to_string(),
        input_tokens: 100,
        output_tokens: 50,
        input_cost: Decimal::new(3, 3), // $0.003
        output_cost: Decimal::new(3, 3), // $0.003
        total_cost: Decimal::new(6, 3), // $0.006
        timestamp: Utc::now(),
        metadata: serde_json::json!({}),
    }
}

/// Create a test pricing tier
pub fn test_pricing_tier() -> PricingTier {
    PricingTier {
        id: Uuid::new_v4(),
        name: "Standard".to_string(),
        provider: LLMProvider::OpenAI,
        model: "gpt-4".to_string(),
        input_price_per_1k: Decimal::new(30, 3),
        output_price_per_1k: Decimal::new(60, 3),
        effective_from: Utc::now(),
        effective_until: None,
    }
}

/// Create a test tenant ID
pub fn test_tenant_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
}

/// Create a test user ID
pub fn test_user_id() -> Uuid {
    Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
}

/// Create a test API key
pub fn test_api_key() -> String {
    "test_api_key_1234567890abcdef".to_string()
}

/// Create a test JWT token
pub fn test_jwt_token() -> String {
    "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixtures_create_valid_data() {
        let provider = test_provider();
        assert_eq!(provider, LLMProvider::OpenAI);

        let config = test_model_config();
        assert_eq!(config.model_name, "gpt-4");
        assert_eq!(config.context_window, 8192);

        let usage = test_usage_record();
        assert_eq!(usage.input_tokens, 100);
        assert_eq!(usage.output_tokens, 50);
        assert_eq!(usage.total_tokens, 150);

        let cost = test_cost_record();
        assert_eq!(cost.input_tokens, 100);
        assert!(cost.total_cost > Decimal::ZERO);
    }
}
