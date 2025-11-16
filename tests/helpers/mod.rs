/// Test utilities and helpers for LLM Cost Ops
///
/// This module provides common test utilities, fixtures, and helpers
/// used across the test suite.

pub mod fixtures;
pub mod builders;
pub mod assertions;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::{PgPool, SqlitePool};
use std::sync::Arc;
use tempfile::TempDir;
use uuid::Uuid;

/// Create a temporary SQLite database for testing
pub async fn create_test_sqlite_db() -> (SqlitePool, TempDir) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    let pool = SqlitePool::connect(&db_url)
        .await
        .expect("Failed to create test database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    (pool, temp_dir)
}

/// Create a test timestamp
pub fn test_timestamp() -> DateTime<Utc> {
    Utc::now()
}

/// Create a test UUID
pub fn test_uuid() -> Uuid {
    Uuid::new_v4()
}

/// Create a test decimal value
pub fn test_decimal(value: &str) -> Decimal {
    value.parse().expect("Invalid decimal")
}

/// Assert that two decimals are approximately equal
pub fn assert_decimal_approx_eq(left: Decimal, right: Decimal, epsilon: Decimal) {
    let diff = (left - right).abs();
    assert!(
        diff <= epsilon,
        "Decimals not approximately equal: {} vs {} (diff: {})",
        left, right, diff
    );
}

/// Assert that a future completes within a timeout
#[macro_export]
macro_rules! assert_timeout {
    ($future:expr, $duration:expr) => {
        tokio::time::timeout($duration, $future)
            .await
            .expect("Future timed out")
    };
}

/// Assert that an error matches a specific pattern
#[macro_export]
macro_rules! assert_error_contains {
    ($result:expr, $pattern:expr) => {
        match $result {
            Ok(_) => panic!("Expected error, got Ok"),
            Err(e) => {
                let error_msg = e.to_string();
                assert!(
                    error_msg.contains($pattern),
                    "Error message '{}' does not contain '{}'",
                    error_msg,
                    $pattern
                );
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uuid_generation() {
        let uuid1 = test_uuid();
        let uuid2 = test_uuid();
        assert_ne!(uuid1, uuid2);
    }

    #[test]
    fn test_decimal_creation() {
        let decimal = test_decimal("123.45");
        assert_eq!(decimal.to_string(), "123.45");
    }

    #[test]
    fn test_decimal_approx_eq() {
        let a = test_decimal("1.0001");
        let b = test_decimal("1.0002");
        assert_decimal_approx_eq(a, b, test_decimal("0.001"));
    }

    #[test]
    #[should_panic]
    fn test_decimal_approx_eq_fails() {
        let a = test_decimal("1.0");
        let b = test_decimal("2.0");
        assert_decimal_approx_eq(a, b, test_decimal("0.001"));
    }
}
