// GDPR Compliance Error Types

use thiserror::Error;

pub type GdprResult<T> = std::result::Result<T, GdprError>;

#[derive(Error, Debug)]
pub enum GdprError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("User not found: {0}")]
    UserNotFound(String),

    #[error("Data export failed: {0}")]
    ExportFailed(String),

    #[error("Data deletion failed: {0}")]
    DeletionFailed(String),

    #[error("Consent not found for user: {0}")]
    ConsentNotFound(String),

    #[error("Invalid consent status: {0}")]
    InvalidConsentStatus(String),

    #[error("Consent required for purpose: {0}")]
    ConsentRequired(String),

    #[error("Breach notification failed: {0}")]
    BreachNotificationFailed(String),

    #[error("Data anonymization failed: {0}")]
    AnonymizationFailed(String),

    #[error("Retention policy violation: {0}")]
    RetentionPolicyViolation(String),

    #[error("Processing restriction active: {0}")]
    ProcessingRestricted(String),

    #[error("Invalid export format: {0}")]
    InvalidExportFormat(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl GdprError {
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        GdprError::Validation(msg.into())
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        GdprError::Internal(msg.into())
    }
}
