//! Validation Module
//!
//! Input/output validation for agent contracts.

use serde::{Deserialize, Serialize};

/// Validation error types
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum ValidationError {
    #[error("Required field missing: {0}")]
    RequiredField(String),

    #[error("Invalid field '{field}': {reason}")]
    InvalidField {
        field: String,
        reason: String,
    },

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Schema mismatch: {0}")]
    SchemaMismatch(String),

    #[error("Range error for '{field}': value {value} not in range [{min}, {max}]")]
    RangeError {
        field: String,
        value: String,
        min: String,
        max: String,
    },

    #[error("Invalid format for '{field}': {reason}")]
    InvalidFormat {
        field: String,
        reason: String,
    },
}

/// Result type for validation
pub type ValidationResult<T> = Result<T, ValidationError>;

/// Validator trait for contract types
pub trait Validator {
    /// Validate the value
    fn validate(&self) -> ValidationResult<()>;

    /// Validate with additional context
    fn validate_with_context(&self, _context: &str) -> ValidationResult<()> {
        self.validate()
    }
}

/// Validation helpers
pub mod validators {
    use super::*;

    /// Validate a string is not empty
    pub fn non_empty(field: &str, value: &str) -> ValidationResult<()> {
        if value.is_empty() {
            return Err(ValidationError::RequiredField(field.to_string()));
        }
        Ok(())
    }

    /// Validate a value is within a range (inclusive)
    pub fn in_range<T: PartialOrd + std::fmt::Display>(
        field: &str,
        value: T,
        min: T,
        max: T,
    ) -> ValidationResult<()> {
        if value < min || value > max {
            return Err(ValidationError::RangeError {
                field: field.to_string(),
                value: value.to_string(),
                min: min.to_string(),
                max: max.to_string(),
            });
        }
        Ok(())
    }

    /// Validate a positive value
    pub fn positive<T: PartialOrd + Default + std::fmt::Display>(
        field: &str,
        value: T,
    ) -> ValidationResult<()> {
        if value <= T::default() {
            return Err(ValidationError::InvalidField {
                field: field.to_string(),
                reason: "must be positive".to_string(),
            });
        }
        Ok(())
    }

    /// Validate a non-negative value
    pub fn non_negative<T: PartialOrd + Default + std::fmt::Display>(
        field: &str,
        value: T,
    ) -> ValidationResult<()> {
        if value < T::default() {
            return Err(ValidationError::InvalidField {
                field: field.to_string(),
                reason: "must be non-negative".to_string(),
            });
        }
        Ok(())
    }

    /// Validate a vector has minimum length
    pub fn min_length<T>(field: &str, value: &[T], min: usize) -> ValidationResult<()> {
        if value.len() < min {
            return Err(ValidationError::InvalidField {
                field: field.to_string(),
                reason: format!("must have at least {} elements, got {}", min, value.len()),
            });
        }
        Ok(())
    }

    /// Validate a confidence value (0.0 to 1.0)
    pub fn confidence(field: &str, value: f64) -> ValidationResult<()> {
        if !(0.0..=1.0).contains(&value) {
            return Err(ValidationError::RangeError {
                field: field.to_string(),
                value: value.to_string(),
                min: "0.0".to_string(),
                max: "1.0".to_string(),
            });
        }
        Ok(())
    }

    /// Validate a percentage value (0 to 100)
    pub fn percentage(field: &str, value: f64) -> ValidationResult<()> {
        if !(0.0..=100.0).contains(&value) {
            return Err(ValidationError::RangeError {
                field: field.to_string(),
                value: value.to_string(),
                min: "0".to_string(),
                max: "100".to_string(),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use validators::*;

    #[test]
    fn test_non_empty_validation() {
        assert!(non_empty("test", "value").is_ok());
        assert!(non_empty("test", "").is_err());
    }

    #[test]
    fn test_in_range_validation() {
        assert!(in_range("test", 5, 0, 10).is_ok());
        assert!(in_range("test", 0, 0, 10).is_ok());
        assert!(in_range("test", 10, 0, 10).is_ok());
        assert!(in_range("test", 11, 0, 10).is_err());
        assert!(in_range("test", -1, 0, 10).is_err());
    }

    #[test]
    fn test_positive_validation() {
        assert!(positive("test", 1).is_ok());
        assert!(positive("test", 0).is_err());
        assert!(positive("test", -1).is_err());
    }

    #[test]
    fn test_non_negative_validation() {
        assert!(non_negative("test", 1).is_ok());
        assert!(non_negative("test", 0).is_ok());
        assert!(non_negative("test", -1).is_err());
    }

    #[test]
    fn test_min_length_validation() {
        let vec = vec![1, 2, 3];
        assert!(min_length("test", &vec, 2).is_ok());
        assert!(min_length("test", &vec, 3).is_ok());
        assert!(min_length("test", &vec, 4).is_err());
    }

    #[test]
    fn test_confidence_validation() {
        assert!(confidence("test", 0.0).is_ok());
        assert!(confidence("test", 0.5).is_ok());
        assert!(confidence("test", 1.0).is_ok());
        assert!(confidence("test", -0.1).is_err());
        assert!(confidence("test", 1.1).is_err());
    }

    #[test]
    fn test_percentage_validation() {
        assert!(percentage("test", 0.0).is_ok());
        assert!(percentage("test", 50.0).is_ok());
        assert!(percentage("test", 100.0).is_ok());
        assert!(percentage("test", -1.0).is_err());
        assert!(percentage("test", 101.0).is_err());
    }

    #[test]
    fn test_validation_error_display() {
        let err = ValidationError::RequiredField("name".to_string());
        assert!(err.to_string().contains("name"));

        let err = ValidationError::InvalidField {
            field: "age".to_string(),
            reason: "must be positive".to_string(),
        };
        assert!(err.to_string().contains("age"));
        assert!(err.to_string().contains("positive"));
    }
}
