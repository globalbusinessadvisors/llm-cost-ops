use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::provider::Provider;
use super::error::{CostOpsError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageRecord {
    /// Unique identifier for this usage record
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    /// Timestamp when the request was made
    pub timestamp: DateTime<Utc>,

    /// LLM provider
    pub provider: Provider,

    /// Model identifier
    pub model: ModelIdentifier,

    /// Organization/tenant identifier
    pub organization_id: String,

    /// Optional project identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,

    /// Optional user identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,

    /// Token counts
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,

    /// Optional cached tokens (for prompt caching)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u64>,

    /// Optional reasoning tokens (for o1-style models)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,

    /// Performance metrics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_to_first_token_ms: Option<u64>,

    /// Cost attribution tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Free-form metadata
    #[serde(default)]
    pub metadata: serde_json::Value,

    /// Ingestion tracking
    pub ingested_at: DateTime<Utc>,
    pub source: IngestionSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelIdentifier {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_window: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum IngestionSource {
    Api { endpoint: String },
    File { path: String },
    Webhook { endpoint: String },
    Stream { topic: String },
}

impl UsageRecord {
    pub fn new(
        provider: Provider,
        model: ModelIdentifier,
        organization_id: String,
        prompt_tokens: u64,
        completion_tokens: u64,
    ) -> Self {
        let total_tokens = prompt_tokens + completion_tokens;

        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            provider,
            model,
            organization_id,
            project_id: None,
            user_id: None,
            prompt_tokens,
            completion_tokens,
            total_tokens,
            cached_tokens: None,
            reasoning_tokens: None,
            latency_ms: None,
            time_to_first_token_ms: None,
            tags: Vec::new(),
            metadata: serde_json::Value::Null,
            ingested_at: Utc::now(),
            source: IngestionSource::Api {
                endpoint: "default".to_string(),
            },
        }
    }

    pub fn validate(&self) -> Result<()> {
        // Token counts must be positive
        if self.total_tokens == 0 {
            return Err(CostOpsError::InvalidTokenCount(
                "Total tokens cannot be zero".to_string(),
            ));
        }

        // Token sum must equal total (with tolerance)
        let calculated_total = self.prompt_tokens + self.completion_tokens;
        if calculated_total != self.total_tokens {
            return Err(CostOpsError::TokenCountMismatch {
                calculated: calculated_total,
                reported: self.total_tokens,
            });
        }

        // Organization ID must not be empty
        if self.organization_id.is_empty() {
            return Err(CostOpsError::MissingOrganizationId);
        }

        // Timestamp must not be in the future
        if self.timestamp > Utc::now() {
            return Err(CostOpsError::FutureTimestamp);
        }

        Ok(())
    }

    pub fn with_project(mut self, project_id: String) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_latency(mut self, latency_ms: u64) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }

    pub fn with_cached_tokens(mut self, cached_tokens: u64) -> Self {
        self.cached_tokens = Some(cached_tokens);
        self
    }
}

impl ModelIdentifier {
    pub fn new(name: String, context_window: u64) -> Self {
        Self {
            name,
            version: None,
            context_window: Some(context_window),
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usage_record_validation() {
        let record = UsageRecord::new(
            Provider::OpenAI,
            ModelIdentifier::new("gpt-4".to_string(), 8192),
            "org-123".to_string(),
            100,
            50,
        );

        assert!(record.validate().is_ok());
    }

    #[test]
    fn test_token_count_mismatch() {
        let mut record = UsageRecord::new(
            Provider::OpenAI,
            ModelIdentifier::new("gpt-4".to_string(), 8192),
            "org-123".to_string(),
            100,
            50,
        );
        record.total_tokens = 200; // Intentional mismatch

        assert!(matches!(
            record.validate(),
            Err(CostOpsError::TokenCountMismatch { .. })
        ));
    }

    #[test]
    fn test_builder_pattern() {
        let record = UsageRecord::new(
            Provider::OpenAI,
            ModelIdentifier::new("gpt-4".to_string(), 8192),
            "org-123".to_string(),
            100,
            50,
        )
        .with_project("project-456".to_string())
        .with_tags(vec!["production".to_string(), "api".to_string()])
        .with_latency(250);

        assert_eq!(record.project_id, Some("project-456".to_string()));
        assert_eq!(record.tags.len(), 2);
        assert_eq!(record.latency_ms, Some(250));
    }
}
