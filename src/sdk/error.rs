//! Error types for the SDK with comprehensive error handling

use thiserror::Error;

/// Result type alias for SDK operations
pub type SdkResult<T> = std::result::Result<T, SdkError>;

/// Comprehensive error type for SDK operations
#[derive(Error, Debug)]
pub enum SdkError {
    /// HTTP client errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Authentication errors
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Authorization errors
    #[error("Authorization failed: {0}")]
    Authorization(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: retry after {retry_after:?}")]
    RateLimitExceeded {
        retry_after: Option<std::time::Duration>,
    },

    /// Request timeout
    #[error("Request timeout after {0:?}")]
    Timeout(std::time::Duration),

    /// API errors from the server
    #[error("API error: {status} - {message}")]
    Api {
        status: u16,
        message: String,
        details: Option<serde_json::Value>,
    },

    /// Network errors
    #[error("Network error: {0}")]
    Network(String),

    /// Retry exhausted
    #[error("Retry exhausted after {attempts} attempts: {last_error}")]
    RetryExhausted {
        attempts: usize,
        last_error: Box<SdkError>,
    },

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Internal SDK error
    #[error("Internal SDK error: {0}")]
    Internal(String),

    /// Database error
    #[error("Database error: {0}")]
    Database(String),

    /// Invalid state
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

impl SdkError {
    /// Create a configuration error
    pub fn config<S: Into<String>>(msg: S) -> Self {
        SdkError::Config(msg.into())
    }

    /// Create a validation error
    pub fn validation<S: Into<String>>(msg: S) -> Self {
        SdkError::Validation(msg.into())
    }

    /// Create an internal error
    pub fn internal<S: Into<String>>(msg: S) -> Self {
        SdkError::Internal(msg.into())
    }

    /// Create an API error
    pub fn api(status: u16, message: String, details: Option<serde_json::Value>) -> Self {
        SdkError::Api {
            status,
            message,
            details,
        }
    }

    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            SdkError::Http(e) => {
                // Retry on network errors, timeouts, connection errors
                e.is_timeout() || e.is_connect() || e.is_request()
            }
            SdkError::RateLimitExceeded { .. } => true,
            SdkError::Timeout(_) => true,
            SdkError::Network(_) => true,
            SdkError::Api { status, .. } => {
                // Retry on 5xx server errors and 429 rate limit
                *status >= 500 || *status == 429
            }
            _ => false,
        }
    }

    /// Get HTTP status code if available
    pub fn status_code(&self) -> Option<u16> {
        match self {
            SdkError::Api { status, .. } => Some(*status),
            SdkError::Http(e) => e.status().map(|s| s.as_u16()),
            _ => None,
        }
    }

    /// Check if error is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        self.status_code()
            .map(|s| (400..500).contains(&s))
            .unwrap_or(false)
    }

    /// Check if error is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        self.status_code()
            .map(|s| s >= 500)
            .unwrap_or(false)
    }
}

/// Extension trait for converting errors to SDK errors
pub trait IntoSdkError {
    fn into_sdk_error(self) -> SdkError;
}

impl<E: std::error::Error + Send + Sync + 'static> IntoSdkError for E {
    fn into_sdk_error(self) -> SdkError {
        SdkError::internal(self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = SdkError::config("Invalid API key");
        assert!(matches!(err, SdkError::Config(_)));

        let err = SdkError::validation("Invalid input");
        assert!(matches!(err, SdkError::Validation(_)));

        let err = SdkError::api(404, "Not found".to_string(), None);
        assert!(matches!(err, SdkError::Api { .. }));
    }

    #[test]
    fn test_error_retryable() {
        let err = SdkError::api(500, "Server error".to_string(), None);
        assert!(err.is_retryable());

        let err = SdkError::api(429, "Rate limited".to_string(), None);
        assert!(err.is_retryable());

        let err = SdkError::api(404, "Not found".to_string(), None);
        assert!(!err.is_retryable());

        let err = SdkError::RateLimitExceeded { retry_after: None };
        assert!(err.is_retryable());
    }

    #[test]
    fn test_error_status_code() {
        let err = SdkError::api(404, "Not found".to_string(), None);
        assert_eq!(err.status_code(), Some(404));

        let err = SdkError::config("Invalid config");
        assert_eq!(err.status_code(), None);
    }

    #[test]
    fn test_error_classification() {
        let err = SdkError::api(404, "Not found".to_string(), None);
        assert!(err.is_client_error());
        assert!(!err.is_server_error());

        let err = SdkError::api(500, "Server error".to_string(), None);
        assert!(!err.is_client_error());
        assert!(err.is_server_error());
    }
}
