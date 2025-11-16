// Dead Letter Queue module for handling persistent failures

pub mod types;
pub mod storage;
pub mod retry;
pub mod processor;
pub mod config;

pub use types::{DlqItem, DlqItemStatus, FailureReason, DlqMetadata};
pub use storage::{DlqStore, InMemoryDlqStore};
pub use retry::{RetryPolicy, BackoffStrategy, ExponentialBackoff, FixedBackoff};
pub use processor::{DlqProcessor, ProcessingResult, DlqItemHandler};
pub use config::DlqConfig;

/// Dead Letter Queue error types
#[derive(Debug, thiserror::Error)]
pub enum DlqError {
    #[error("DLQ item not found: {0}")]
    ItemNotFound(String),

    #[error("DLQ storage error: {0}")]
    StorageError(String),

    #[error("DLQ processing error: {0}")]
    ProcessingError(String),

    #[error("Retry limit exceeded: {0}")]
    RetryLimitExceeded(String),

    #[error("Invalid DLQ configuration: {0}")]
    ConfigError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<DlqError> for crate::domain::CostOpsError {
    fn from(err: DlqError) -> Self {
        crate::domain::CostOpsError::Internal(err.to_string())
    }
}

pub type DlqResult<T> = Result<T, DlqError>;
