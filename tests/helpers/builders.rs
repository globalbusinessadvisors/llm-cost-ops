/// Builder patterns for test data
///
/// Provides flexible builder patterns for creating test objects
/// with custom fields

use chrono::{DateTime, Utc};
use llm_cost_ops::domain::{
    cost::CostRecord,
    provider::LLMProvider,
    usage::UsageRecord,
};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Builder for UsageRecord
pub struct UsageRecordBuilder {
    id: Uuid,
    tenant_id: Uuid,
    request_id: String,
    provider: LLMProvider,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    total_tokens: u64,
    timestamp: DateTime<Utc>,
    metadata: serde_json::Value,
}

impl Default for UsageRecordBuilder {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            request_id: Uuid::new_v4().to_string(),
            provider: LLMProvider::OpenAI,
            model: "gpt-4".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }
}

impl UsageRecordBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn tenant_id(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    pub fn provider(mut self, provider: LLMProvider) -> Self {
        self.provider = provider;
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn input_tokens(mut self, tokens: u64) -> Self {
        self.input_tokens = tokens;
        self.total_tokens = self.input_tokens + self.output_tokens;
        self
    }

    pub fn output_tokens(mut self, tokens: u64) -> Self {
        self.output_tokens = tokens;
        self.total_tokens = self.input_tokens + self.output_tokens;
        self
    }

    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    pub fn build(self) -> UsageRecord {
        UsageRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            request_id: self.request_id,
            provider: self.provider,
            model: self.model,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            total_tokens: self.total_tokens,
            timestamp: self.timestamp,
            metadata: self.metadata,
        }
    }
}

/// Builder for CostRecord
pub struct CostRecordBuilder {
    id: Uuid,
    tenant_id: Uuid,
    usage_id: Uuid,
    provider: LLMProvider,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    input_cost: Decimal,
    output_cost: Decimal,
    total_cost: Decimal,
    timestamp: DateTime<Utc>,
    metadata: serde_json::Value,
}

impl Default for CostRecordBuilder {
    fn default() -> Self {
        let input_cost = Decimal::new(3, 3);
        let output_cost = Decimal::new(3, 3);
        Self {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            usage_id: Uuid::new_v4(),
            provider: LLMProvider::OpenAI,
            model: "gpt-4".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            input_cost,
            output_cost,
            total_cost: input_cost + output_cost,
            timestamp: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }
}

impl CostRecordBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn id(mut self, id: Uuid) -> Self {
        self.id = id;
        self
    }

    pub fn tenant_id(mut self, tenant_id: Uuid) -> Self {
        self.tenant_id = tenant_id;
        self
    }

    pub fn usage_id(mut self, usage_id: Uuid) -> Self {
        self.usage_id = usage_id;
        self
    }

    pub fn provider(mut self, provider: LLMProvider) -> Self {
        self.provider = provider;
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn input_tokens(mut self, tokens: u64) -> Self {
        self.input_tokens = tokens;
        self
    }

    pub fn output_tokens(mut self, tokens: u64) -> Self {
        self.output_tokens = tokens;
        self
    }

    pub fn input_cost(mut self, cost: Decimal) -> Self {
        self.input_cost = cost;
        self.total_cost = self.input_cost + self.output_cost;
        self
    }

    pub fn output_cost(mut self, cost: Decimal) -> Self {
        self.output_cost = cost;
        self.total_cost = self.input_cost + self.output_cost;
        self
    }

    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }

    pub fn build(self) -> CostRecord {
        CostRecord {
            id: self.id,
            tenant_id: self.tenant_id,
            usage_id: self.usage_id,
            provider: self.provider,
            model: self.model,
            input_tokens: self.input_tokens,
            output_tokens: self.output_tokens,
            input_cost: self.input_cost,
            output_cost: self.output_cost,
            total_cost: self.total_cost,
            timestamp: self.timestamp,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_record_builder() {
        let record = UsageRecordBuilder::new()
            .model("gpt-3.5-turbo")
            .input_tokens(200)
            .output_tokens(100)
            .build();

        assert_eq!(record.model, "gpt-3.5-turbo");
        assert_eq!(record.input_tokens, 200);
        assert_eq!(record.output_tokens, 100);
        assert_eq!(record.total_tokens, 300);
    }

    #[test]
    fn test_cost_record_builder() {
        let input_cost = Decimal::new(10, 3);
        let output_cost = Decimal::new(20, 3);

        let record = CostRecordBuilder::new()
            .model("gpt-4")
            .input_cost(input_cost)
            .output_cost(output_cost)
            .build();

        assert_eq!(record.model, "gpt-4");
        assert_eq!(record.input_cost, input_cost);
        assert_eq!(record.output_cost, output_cost);
        assert_eq!(record.total_cost, input_cost + output_cost);
    }
}
