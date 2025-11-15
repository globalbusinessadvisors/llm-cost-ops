// Dead Letter Queue types and models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Dead Letter Queue item representing a failed operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqItem {
    /// Unique identifier for this DLQ item
    pub id: Uuid,

    /// Organization ID associated with the failed item
    pub organization_id: String,

    /// Original payload that failed (serialized as JSON)
    pub payload: String,

    /// Type of the failed item (e.g., "usage_record", "webhook", "batch")
    pub item_type: String,

    /// Current status of the DLQ item
    pub status: DlqItemStatus,

    /// Reason for the failure
    pub failure_reason: FailureReason,

    /// Original error message
    pub error_message: String,

    /// Stack trace or detailed error information
    pub error_details: Option<String>,

    /// Number of retry attempts made
    pub retry_count: u32,

    /// Maximum number of retries allowed
    pub max_retries: u32,

    /// When the item was first added to the DLQ
    pub created_at: DateTime<Utc>,

    /// When the item was last updated
    pub updated_at: DateTime<Utc>,

    /// When the next retry should be attempted
    pub next_retry_at: Option<DateTime<Utc>>,

    /// When the item was successfully processed (if applicable)
    pub processed_at: Option<DateTime<Utc>>,

    /// When the item expires and should be removed from DLQ
    pub expires_at: Option<DateTime<Utc>>,

    /// Additional metadata about the item
    pub metadata: DlqMetadata,
}

/// Status of a DLQ item
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DlqItemStatus {
    /// Waiting for retry
    Pending,

    /// Currently being retried
    Retrying,

    /// Successfully processed after retry
    Processed,

    /// Permanently failed (max retries exceeded)
    Failed,

    /// Manually marked for review
    ReviewRequired,

    /// Archived (no longer active)
    Archived,
}

/// Reason for failure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailureReason {
    /// Validation error in the payload
    ValidationError,

    /// Rate limit exceeded
    RateLimitExceeded,

    /// Database error
    DatabaseError,

    /// Network/connectivity error
    NetworkError,

    /// Timeout during processing
    Timeout,

    /// Invalid authentication/authorization
    AuthenticationError,

    /// Parsing/deserialization error
    ParseError,

    /// Downstream service unavailable
    ServiceUnavailable,

    /// Internal server error
    InternalError,

    /// Unknown error
    Unknown,
}

/// Additional metadata for DLQ items
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DlqMetadata {
    /// Source of the failed item (e.g., "webhook", "api", "batch_upload")
    pub source: Option<String>,

    /// Original request ID or correlation ID
    pub correlation_id: Option<String>,

    /// HTTP status code if applicable
    pub status_code: Option<u16>,

    /// Retry history with timestamps
    pub retry_history: Vec<RetryAttempt>,

    /// Custom tags for categorization
    pub tags: Vec<String>,

    /// Additional key-value pairs
    pub custom_fields: std::collections::HashMap<String, String>,
}

/// Record of a retry attempt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryAttempt {
    /// When the retry was attempted
    pub attempted_at: DateTime<Utc>,

    /// Whether the retry succeeded
    pub succeeded: bool,

    /// Error message if retry failed
    pub error: Option<String>,

    /// Duration of the retry attempt in milliseconds
    pub duration_ms: Option<f64>,
}

impl DlqItem {
    /// Create a new DLQ item from a failed operation
    pub fn new(
        organization_id: String,
        payload: String,
        item_type: String,
        failure_reason: FailureReason,
        error_message: String,
        max_retries: u32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            organization_id,
            payload,
            item_type,
            status: DlqItemStatus::Pending,
            failure_reason,
            error_message,
            error_details: None,
            retry_count: 0,
            max_retries,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            next_retry_at: None,
            processed_at: None,
            expires_at: None,
            metadata: DlqMetadata::default(),
        }
    }

    /// Create a DLQ item with expiration
    pub fn with_expiration(mut self, expires_in_hours: u64) -> Self {
        self.expires_at = Some(Utc::now() + chrono::Duration::hours(expires_in_hours as i64));
        self
    }

    /// Add metadata to the DLQ item
    pub fn with_metadata(mut self, metadata: DlqMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Add error details
    pub fn with_error_details(mut self, details: String) -> Self {
        self.error_details = Some(details);
        self
    }

    /// Record a retry attempt
    pub fn record_retry(&mut self, succeeded: bool, error: Option<String>, duration_ms: f64) {
        self.retry_count += 1;
        self.updated_at = Utc::now();

        self.metadata.retry_history.push(RetryAttempt {
            attempted_at: Utc::now(),
            succeeded,
            error,
            duration_ms: Some(duration_ms),
        });

        if succeeded {
            self.status = DlqItemStatus::Processed;
            self.processed_at = Some(Utc::now());
            self.next_retry_at = None;
        } else if self.retry_count >= self.max_retries {
            self.status = DlqItemStatus::Failed;
            self.next_retry_at = None;
        } else {
            self.status = DlqItemStatus::Pending;
        }
    }

    /// Mark the item as ready for retry
    pub fn schedule_retry(&mut self, next_retry_at: DateTime<Utc>) {
        self.next_retry_at = Some(next_retry_at);
        self.status = DlqItemStatus::Pending;
        self.updated_at = Utc::now();
    }

    /// Mark the item for manual review
    pub fn mark_for_review(&mut self) {
        self.status = DlqItemStatus::ReviewRequired;
        self.updated_at = Utc::now();
    }

    /// Archive the item
    pub fn archive(&mut self) {
        self.status = DlqItemStatus::Archived;
        self.updated_at = Utc::now();
    }

    /// Check if the item has expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if the item can be retried
    pub fn can_retry(&self) -> bool {
        self.status == DlqItemStatus::Pending
            && self.retry_count < self.max_retries
            && !self.is_expired()
    }

    /// Check if the item is ready for retry
    pub fn is_ready_for_retry(&self) -> bool {
        if !self.can_retry() {
            return false;
        }

        if let Some(next_retry) = self.next_retry_at {
            Utc::now() >= next_retry
        } else {
            true
        }
    }

    /// Get the age of the item in seconds
    pub fn age_seconds(&self) -> i64 {
        (Utc::now() - self.created_at).num_seconds()
    }

    /// Get remaining retries
    pub fn remaining_retries(&self) -> u32 {
        self.max_retries.saturating_sub(self.retry_count)
    }
}

impl FailureReason {
    /// Check if this failure reason is potentially retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            FailureReason::NetworkError
                | FailureReason::Timeout
                | FailureReason::ServiceUnavailable
                | FailureReason::RateLimitExceeded
                | FailureReason::DatabaseError
        )
    }

    /// Get suggested retry delay in seconds based on failure reason
    pub fn suggested_retry_delay_secs(&self) -> u64 {
        match self {
            FailureReason::RateLimitExceeded => 60,
            FailureReason::NetworkError | FailureReason::ServiceUnavailable => 30,
            FailureReason::Timeout => 20,
            FailureReason::DatabaseError => 10,
            _ => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_dlq_item() {
        let item = DlqItem::new(
            "org-123".to_string(),
            r#"{"test": "data"}"#.to_string(),
            "usage_record".to_string(),
            FailureReason::ValidationError,
            "Invalid payload".to_string(),
            3,
        );

        assert_eq!(item.organization_id, "org-123");
        assert_eq!(item.retry_count, 0);
        assert_eq!(item.max_retries, 3);
        assert_eq!(item.status, DlqItemStatus::Pending);
        assert!(item.can_retry());
    }

    #[test]
    fn test_record_successful_retry() {
        let mut item = DlqItem::new(
            "org-123".to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::NetworkError,
            "Network error".to_string(),
            3,
        );

        item.record_retry(true, None, 50.0);

        assert_eq!(item.retry_count, 1);
        assert_eq!(item.status, DlqItemStatus::Processed);
        assert!(item.processed_at.is_some());
        assert_eq!(item.metadata.retry_history.len(), 1);
    }

    #[test]
    fn test_record_failed_retry() {
        let mut item = DlqItem::new(
            "org-123".to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::NetworkError,
            "Network error".to_string(),
            3,
        );

        item.record_retry(false, Some("Still failing".to_string()), 25.0);

        assert_eq!(item.retry_count, 1);
        assert_eq!(item.status, DlqItemStatus::Pending);
        assert!(item.can_retry());
        assert_eq!(item.remaining_retries(), 2);
    }

    #[test]
    fn test_max_retries_exceeded() {
        let mut item = DlqItem::new(
            "org-123".to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::ValidationError,
            "Invalid".to_string(),
            2,
        );

        item.record_retry(false, Some("Failed".to_string()), 10.0);
        assert!(item.can_retry());

        item.record_retry(false, Some("Failed again".to_string()), 10.0);
        assert!(!item.can_retry());
        assert_eq!(item.status, DlqItemStatus::Failed);
    }

    #[test]
    fn test_expiration() {
        let item = DlqItem::new(
            "org-123".to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::Timeout,
            "Timeout".to_string(),
            3,
        )
        .with_expiration(24);

        assert!(!item.is_expired());
        assert!(item.expires_at.is_some());
    }

    #[test]
    fn test_schedule_retry() {
        let mut item = DlqItem::new(
            "org-123".to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::NetworkError,
            "Network error".to_string(),
            3,
        );

        let next_retry = Utc::now() + chrono::Duration::seconds(30);
        item.schedule_retry(next_retry);

        assert_eq!(item.status, DlqItemStatus::Pending);
        assert!(item.next_retry_at.is_some());
        assert!(!item.is_ready_for_retry()); // Not ready yet
    }

    #[test]
    fn test_failure_reason_retryable() {
        assert!(FailureReason::NetworkError.is_retryable());
        assert!(FailureReason::ServiceUnavailable.is_retryable());
        assert!(!FailureReason::ValidationError.is_retryable());
        assert!(!FailureReason::ParseError.is_retryable());
    }

    #[test]
    fn test_mark_for_review() {
        let mut item = DlqItem::new(
            "org-123".to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::Unknown,
            "Unknown error".to_string(),
            3,
        );

        item.mark_for_review();
        assert_eq!(item.status, DlqItemStatus::ReviewRequired);
    }
}
