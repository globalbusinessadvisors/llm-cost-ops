// Ingestion domain models for Observatory integration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

use llm_cost_ops::domain::{ModelIdentifier, Provider};

/// Webhook payload for usage data ingestion
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UsageWebhookPayload {
    /// Request identifier (optional, will be generated if not provided)
    #[serde(default = "Uuid::new_v4")]
    pub request_id: Uuid,

    /// Timestamp of the LLM request
    pub timestamp: DateTime<Utc>,

    /// Provider information
    #[validate(length(min = 1))]
    pub provider: String,

    /// Model information
    #[validate]
    pub model: ModelWebhook,

    /// Organization identifier
    #[validate(length(min = 1, max = 255))]
    pub organization_id: String,

    /// Project identifier (optional)
    #[validate(length(max = 255))]
    pub project_id: Option<String>,

    /// User identifier (optional)
    #[validate(length(max = 255))]
    pub user_id: Option<String>,

    /// Token usage information
    #[validate]
    pub usage: TokenUsageWebhook,

    /// Performance metrics (optional)
    pub performance: Option<PerformanceMetrics>,

    /// Custom tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Model information in webhook payload
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ModelWebhook {
    #[validate(length(min = 1, max = 255))]
    pub name: String,

    #[validate(length(max = 100))]
    pub version: Option<String>,

    #[validate(range(min = 1))]
    pub context_window: Option<u64>,
}

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TokenUsageWebhook {
    #[validate(range(min = 0))]
    pub prompt_tokens: u64,

    #[validate(range(min = 0))]
    pub completion_tokens: u64,

    #[validate(range(min = 0))]
    pub total_tokens: u64,

    #[validate(range(min = 0))]
    pub cached_tokens: Option<u64>,

    #[validate(range(min = 0))]
    pub reasoning_tokens: Option<u64>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PerformanceMetrics {
    /// Total request latency in milliseconds
    #[validate(range(min = 0))]
    pub latency_ms: Option<u64>,

    /// Time to first token in milliseconds
    #[validate(range(min = 0))]
    pub time_to_first_token_ms: Option<u64>,
}

/// Batch ingestion request for multiple usage records
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct BatchIngestionRequest {
    /// Batch identifier
    #[serde(default = "Uuid::new_v4")]
    pub batch_id: Uuid,

    /// Source of the batch
    #[validate(length(min = 1, max = 255))]
    pub source: String,

    /// Records to ingest
    #[validate(length(min = 1, max = 1000))]
    #[validate]
    pub records: Vec<UsageWebhookPayload>,
}

/// Ingestion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionResponse {
    /// Request identifier
    pub request_id: Uuid,

    /// Status of the ingestion
    pub status: IngestionStatus,

    /// Number of records accepted
    pub accepted: usize,

    /// Number of records rejected
    pub rejected: usize,

    /// Validation errors (if any)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<IngestionError>,

    /// Processing timestamp
    pub processed_at: DateTime<Utc>,
}

/// Ingestion status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IngestionStatus {
    /// All records accepted
    Success,

    /// Some records accepted, some rejected
    Partial,

    /// All records rejected
    Failed,

    /// Request queued for processing
    Queued,
}

/// Ingestion error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionError {
    /// Record index (for batch requests)
    pub index: Option<usize>,

    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Field that caused the error (if applicable)
    pub field: Option<String>,
}

/// Event stream message envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamMessage {
    /// Message identifier
    pub message_id: String,

    /// Event type
    pub event_type: StreamEventType,

    /// Timestamp when message was created
    pub created_at: DateTime<Utc>,

    /// Payload data
    pub payload: UsageWebhookPayload,

    /// Retry count
    #[serde(default)]
    pub retry_count: u32,
}

/// Stream event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StreamEventType {
    /// New usage record
    UsageCreated,

    /// Usage record updated
    UsageUpdated,

    /// Batch upload
    BatchUploaded,
}

/// Configuration for ingestion sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    /// Enable webhook endpoint
    pub webhook_enabled: bool,

    /// Webhook server bind address
    pub webhook_bind: String,

    /// Enable NATS stream consumer
    pub nats_enabled: bool,

    /// NATS server URLs
    pub nats_urls: Vec<String>,

    /// NATS subject to subscribe to
    pub nats_subject: String,

    /// Enable Redis stream consumer
    pub redis_enabled: bool,

    /// Redis connection URL
    pub redis_url: Option<String>,

    /// Redis stream key
    pub redis_stream_key: String,

    /// Buffer size for incoming requests
    pub buffer_size: usize,

    /// Maximum batch size
    pub max_batch_size: usize,

    /// Request timeout in seconds
    pub request_timeout_secs: u64,

    /// Retry configuration
    pub retry: RetryConfig,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,

    /// Initial retry delay in milliseconds
    pub initial_delay_ms: u64,

    /// Maximum retry delay in milliseconds
    pub max_delay_ms: u64,

    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            webhook_enabled: true,
            webhook_bind: "0.0.0.0:8080".to_string(),
            nats_enabled: false,
            nats_urls: vec!["nats://localhost:4222".to_string()],
            nats_subject: "llm.usage".to_string(),
            redis_enabled: false,
            redis_url: None,
            redis_stream_key: "llm:usage".to_string(),
            buffer_size: 10000,
            max_batch_size: 1000,
            request_timeout_secs: 30,
            retry: RetryConfig::default(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

impl UsageWebhookPayload {
    /// Convert webhook payload to domain UsageRecord
    pub fn to_usage_record(&self) -> llm_cost_ops::UsageRecord {
        use llm_cost_ops::domain::IngestionSource;

        llm_cost_ops::UsageRecord {
            id: self.request_id,
            timestamp: self.timestamp,
            provider: Provider::parse(&self.provider),
            model: ModelIdentifier {
                name: self.model.name.clone(),
                version: self.model.version.clone(),
                context_window: self.model.context_window,
            },
            organization_id: self.organization_id.clone(),
            project_id: self.project_id.clone(),
            user_id: self.user_id.clone(),
            prompt_tokens: self.usage.prompt_tokens,
            completion_tokens: self.usage.completion_tokens,
            total_tokens: self.usage.total_tokens,
            cached_tokens: self.usage.cached_tokens,
            reasoning_tokens: self.usage.reasoning_tokens,
            latency_ms: self.performance.as_ref().and_then(|p| p.latency_ms),
            time_to_first_token_ms: self
                .performance
                .as_ref()
                .and_then(|p| p.time_to_first_token_ms),
            tags: self.tags.clone(),
            metadata: serde_json::to_value(&self.metadata).unwrap_or(serde_json::json!({})),
            ingested_at: Utc::now(),
            source: IngestionSource::Webhook {
                endpoint: "/v1/usage".to_string(),
            },
        }
    }
}
