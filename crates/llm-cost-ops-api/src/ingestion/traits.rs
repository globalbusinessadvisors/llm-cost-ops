// Ingestion handler traits for pluggable data sources

use async_trait::async_trait;
use llm_cost_ops::{Result, UsageRecord};

use super::models::{IngestionResponse, UsageWebhookPayload};

/// Trait for ingestion handlers that process incoming usage data
#[async_trait]
pub trait IngestionHandler: Send + Sync + Clone {
    /// Handle a single usage record
    async fn handle_single(&self, payload: UsageWebhookPayload) -> Result<IngestionResponse>;

    /// Handle a batch of usage records
    async fn handle_batch(&self, payloads: Vec<UsageWebhookPayload>)
        -> Result<IngestionResponse>;

    /// Get handler name for logging/metrics
    fn name(&self) -> &str;

    /// Check if handler is healthy
    async fn health_check(&self) -> Result<bool>;
}

/// Trait for storage backends that persist usage records
#[async_trait]
pub trait IngestionStorage: Send + Sync {
    /// Store a single usage record
    async fn store_usage(&self, record: UsageRecord) -> Result<()>;

    /// Store multiple usage records (batch)
    async fn store_batch(&self, records: Vec<UsageRecord>) -> Result<Vec<Result<()>>>;

    /// Check storage health
    async fn health(&self) -> Result<bool>;
}

/// Trait for validation of incoming data
pub trait PayloadValidator: Send + Sync {
    /// Validate a single payload
    fn validate(&self, payload: &UsageWebhookPayload) -> Result<()>;

    /// Validate a batch of payloads
    fn validate_batch(&self, payloads: &[UsageWebhookPayload]) -> Vec<Result<()>>;
}

/// Trait for rate limiting ingestion requests
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Check if a request should be allowed
    async fn check_rate_limit(&self, organization_id: &str) -> Result<bool>;

    /// Record a request for rate limiting
    async fn record_request(&self, organization_id: &str) -> Result<()>;
}

/// Trait for buffering and batching records
#[async_trait]
pub trait RecordBuffer: Send + Sync {
    /// Add a record to the buffer
    async fn add(&self, record: UsageRecord) -> Result<()>;

    /// Flush the buffer and return records
    async fn flush(&self) -> Result<Vec<UsageRecord>>;

    /// Get current buffer size
    async fn size(&self) -> usize;

    /// Check if buffer should be flushed
    async fn should_flush(&self) -> bool;
}
