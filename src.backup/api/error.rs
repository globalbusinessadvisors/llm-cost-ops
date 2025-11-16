// API error types and handling

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;

/// API error type
#[derive(Debug)]
pub enum ApiError {
    /// Bad request (400)
    BadRequest(String),

    /// Unauthorized (401)
    Unauthorized(String),

    /// Forbidden (403)
    Forbidden(String),

    /// Not found (404)
    NotFound(String),

    /// Conflict (409)
    Conflict(String),

    /// Unprocessable entity (422)
    UnprocessableEntity(String),

    /// Too many requests (429)
    TooManyRequests(String),

    /// Internal server error (500)
    InternalServerError(String),

    /// Service unavailable (503)
    ServiceUnavailable(String),

    /// Validation error
    ValidationError(Vec<ValidationError>),

    /// Database error
    DatabaseError(String),

    /// Domain error
    DomainError(String),
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(msg) => write!(f, "Bad request: {}", msg),
            Self::Unauthorized(msg) => write!(f, "Unauthorized: {}", msg),
            Self::Forbidden(msg) => write!(f, "Forbidden: {}", msg),
            Self::NotFound(msg) => write!(f, "Not found: {}", msg),
            Self::Conflict(msg) => write!(f, "Conflict: {}", msg),
            Self::UnprocessableEntity(msg) => write!(f, "Unprocessable entity: {}", msg),
            Self::TooManyRequests(msg) => write!(f, "Too many requests: {}", msg),
            Self::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
            Self::ServiceUnavailable(msg) => write!(f, "Service unavailable: {}", msg),
            Self::ValidationError(errors) => write!(f, "Validation error: {:?}", errors),
            Self::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            Self::DomainError(msg) => write!(f, "Domain error: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

/// Validation error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
    pub code: Option<String>,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: None,
        }
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

/// Error response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiError {
    /// Get HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::UnprocessableEntity(_) | Self::ValidationError(_) => {
                StatusCode::UNPROCESSABLE_ENTITY
            }
            Self::TooManyRequests(_) => StatusCode::TOO_MANY_REQUESTS,
            Self::InternalServerError(_) | Self::DatabaseError(_) | Self::DomainError(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    /// Get error code
    pub fn error_code(&self) -> String {
        match self {
            Self::BadRequest(_) => "BAD_REQUEST",
            Self::Unauthorized(_) => "UNAUTHORIZED",
            Self::Forbidden(_) => "FORBIDDEN",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::UnprocessableEntity(_) => "UNPROCESSABLE_ENTITY",
            Self::ValidationError(_) => "VALIDATION_ERROR",
            Self::TooManyRequests(_) => "TOO_MANY_REQUESTS",
            Self::InternalServerError(_) => "INTERNAL_SERVER_ERROR",
            Self::ServiceUnavailable(_) => "SERVICE_UNAVAILABLE",
            Self::DatabaseError(_) => "DATABASE_ERROR",
            Self::DomainError(_) => "DOMAIN_ERROR",
        }
        .to_string()
    }

    /// Convert to error response
    pub fn to_response(&self) -> ErrorResponse {
        let details = match self {
            Self::ValidationError(errors) => Some(serde_json::to_value(errors).unwrap()),
            _ => None,
        };

        ErrorResponse {
            error: ErrorDetail {
                code: self.error_code(),
                message: self.to_string(),
                details,
            },
            request_id: None,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let body = Json(self.to_response());
        (status, body).into_response()
    }
}

// Conversions from other error types
impl From<crate::domain::CostOpsError> for ApiError {
    fn from(err: crate::domain::CostOpsError) -> Self {
        Self::DomainError(err.to_string())
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        match err {
            sqlx::Error::RowNotFound => Self::NotFound("Resource not found".to_string()),
            _ => Self::DatabaseError(err.to_string()),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self::BadRequest(format!("Invalid JSON: {}", err))
    }
}

impl From<validator::ValidationErrors> for ApiError {
    fn from(errors: validator::ValidationErrors) -> Self {
        let validation_errors: Vec<ValidationError> = errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    ValidationError::new(
                        field.to_string(),
                        error.message.clone().unwrap_or_default().to_string(),
                    )
                    .with_code(error.code.to_string())
                })
            })
            .collect();

        Self::ValidationError(validation_errors)
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_status_codes() {
        assert_eq!(
            ApiError::BadRequest("test".to_string()).status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            ApiError::Unauthorized("test".to_string()).status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            ApiError::NotFound("test".to_string()).status_code(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            ApiError::InternalServerError("test".to_string()).status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_validation_error_creation() {
        let error = ValidationError::new("email", "Invalid email format")
            .with_code("INVALID_EMAIL");

        assert_eq!(error.field, "email");
        assert_eq!(error.message, "Invalid email format");
        assert_eq!(error.code, Some("INVALID_EMAIL".to_string()));
    }

    #[test]
    fn test_error_response_serialization() {
        let api_error = ApiError::BadRequest("Invalid request".to_string());
        let response = api_error.to_response();

        assert_eq!(response.error.code, "BAD_REQUEST");
        assert!(response.error.message.contains("Invalid request"));
    }
}
