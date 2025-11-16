//! Comprehensive integration tests for the SDK

use chrono::Utc;
use llm_cost_ops::sdk::{
    ClientConfig, CostOpsClient, SdkError, RetryConfig, UsageRequest,
};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_client_builder_success() {
    let result = CostOpsClient::builder()
        .base_url("https://api.example.com")
        .unwrap()
        .api_key("test-api-key-12345")
        .timeout(Duration::from_secs(30))
        .build();

    assert!(result.is_ok());
    let client = result.unwrap();
    assert_eq!(client.config().timeout, Duration::from_secs(30));
    assert_eq!(client.config().api_key, "test-api-key-12345");
}

#[test]
fn test_client_builder_missing_api_key() {
    let result = CostOpsClient::builder()
        .base_url("https://api.example.com")
        .unwrap()
        .build();

    assert!(result.is_err());
    match result {
        Err(SdkError::Config(msg)) => {
            assert!(msg.contains("api_key"));
        }
        _ => panic!("Expected Config error"),
    }
}

#[test]
fn test_client_builder_invalid_url() {
    let result = CostOpsClient::builder()
        .base_url("not-a-valid-url")
        .unwrap_err();

    assert!(matches!(result, SdkError::Config(_)));
}

#[test]
fn test_retry_config_validation() {
    let config = RetryConfig {
        max_attempts: 0, // Invalid
        initial_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(30),
        multiplier: 2.0,
        jitter: true,
    };

    assert!(config.validate().is_err());
}

#[test]
fn test_retry_config_backoff_validation() {
    let config = RetryConfig {
        max_attempts: 3,
        initial_backoff: Duration::from_secs(100), // Greater than max
        max_backoff: Duration::from_secs(10),
        multiplier: 2.0,
        jitter: true,
    };

    assert!(config.validate().is_err());
}

#[test]
fn test_usage_request_serialization() {
    let request = UsageRequest {
        organization_id: Uuid::new_v4(),
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        prompt_tokens: 100,
        completion_tokens: 50,
        total_tokens: 150,
        timestamp: Utc::now(),
        request_id: Some("test-request-123".to_string()),
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: UsageRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(request.provider, deserialized.provider);
    assert_eq!(request.model, deserialized.model);
    assert_eq!(request.prompt_tokens, deserialized.prompt_tokens);
    assert_eq!(request.completion_tokens, deserialized.completion_tokens);
}

#[test]
fn test_error_retryable() {
    // Server errors should be retryable
    let err = SdkError::api(500, "Internal Server Error".to_string(), None);
    assert!(err.is_retryable());

    // Rate limit errors should be retryable
    let err = SdkError::RateLimitExceeded { retry_after: None };
    assert!(err.is_retryable());

    // Client errors should not be retryable
    let err = SdkError::api(404, "Not Found".to_string(), None);
    assert!(!err.is_retryable());

    // Validation errors should not be retryable
    let err = SdkError::Validation("Invalid input".to_string());
    assert!(!err.is_retryable());
}

#[test]
fn test_error_status_code() {
    let err = SdkError::api(404, "Not Found".to_string(), None);
    assert_eq!(err.status_code(), Some(404));

    let err = SdkError::Config("Invalid config".to_string());
    assert_eq!(err.status_code(), None);
}

#[test]
fn test_error_classification() {
    // 4xx errors are client errors
    let err = SdkError::api(400, "Bad Request".to_string(), None);
    assert!(err.is_client_error());
    assert!(!err.is_server_error());

    // 5xx errors are server errors
    let err = SdkError::api(503, "Service Unavailable".to_string(), None);
    assert!(!err.is_client_error());
    assert!(err.is_server_error());

    // Non-HTTP errors are neither
    let err = SdkError::Config("Invalid".to_string());
    assert!(!err.is_client_error());
    assert!(!err.is_server_error());
}

#[test]
fn test_config_builder_with_headers() {
    let config = ClientConfig::builder()
        .base_url("https://api.example.com")
        .unwrap()
        .api_key("test-key")
        .add_header("X-Custom-Header", "custom-value")
        .add_header("X-Another-Header", "another-value")
        .build()
        .unwrap();

    assert_eq!(config.default_headers.len(), 2);
    assert!(config
        .default_headers
        .iter()
        .any(|(k, v)| k == "X-Custom-Header" && v == "custom-value"));
}

#[test]
fn test_telemetry_config() {
    use llm_cost_ops::sdk::TelemetryConfig;

    let config = TelemetryConfig {
        metrics_enabled: true,
        tracing_enabled: true,
        logging_enabled: true,
        metrics_endpoint: Some("http://localhost:9090".to_string()),
        trace_endpoint: Some("http://localhost:4317".to_string()),
    };

    assert!(config.metrics_enabled);
    assert!(config.tracing_enabled);
    assert!(config.metrics_endpoint.is_some());
}

#[test]
fn test_pool_config() {
    use llm_cost_ops::sdk::PoolConfig;

    let config = PoolConfig {
        max_idle: 10,
        max_per_host: 20,
        idle_timeout: Duration::from_secs(90),
    };

    assert!(config.validate().is_ok());

    // Invalid config
    let invalid = PoolConfig {
        max_idle: 0,
        max_per_host: 20,
        idle_timeout: Duration::from_secs(90),
    };

    assert!(invalid.validate().is_err());
}

#[test]
fn test_rate_limit_config() {
    use llm_cost_ops::sdk::RateLimitConfig;

    let config = RateLimitConfig {
        requests_per_second: Some(100),
        burst_size: Some(10),
        enabled: true,
    };

    assert!(config.validate().is_ok());

    // Invalid config
    let invalid = RateLimitConfig {
        requests_per_second: Some(0),
        burst_size: Some(10),
        enabled: true,
    };

    assert!(invalid.validate().is_err());
}

#[tokio::test]
async fn test_retry_policy_success_after_failures() {
    use llm_cost_ops::sdk::RetryPolicy;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let config = RetryConfig {
        max_attempts: 3,
        initial_backoff: Duration::from_millis(10),
        max_backoff: Duration::from_millis(100),
        multiplier: 2.0,
        jitter: false,
    };

    let policy = RetryPolicy::new(config);

    let result = policy
        .execute(|| async {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            if count < 2 {
                Err(SdkError::api(500, "Server error".to_string(), None))
            } else {
                Ok("Success")
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Success");
    assert_eq!(counter.load(Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_retry_policy_exhausted() {
    use llm_cost_ops::sdk::RetryPolicy;

    let config = RetryConfig {
        max_attempts: 3,
        initial_backoff: Duration::from_millis(10),
        max_backoff: Duration::from_millis(100),
        multiplier: 2.0,
        jitter: false,
    };

    let policy = RetryPolicy::new(config);

    let result = policy
        .execute(|| async {
            Err::<(), _>(SdkError::api(500, "Server error".to_string(), None))
        })
        .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), SdkError::RetryExhausted { .. }));
}

#[tokio::test]
async fn test_retry_policy_non_retryable() {
    use llm_cost_ops::sdk::RetryPolicy;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();

    let config = RetryConfig {
        max_attempts: 3,
        initial_backoff: Duration::from_millis(10),
        max_backoff: Duration::from_millis(100),
        multiplier: 2.0,
        jitter: false,
    };

    let policy = RetryPolicy::new(config);

    let result = policy
        .execute(|| async {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            Err::<(), _>(SdkError::api(404, "Not found".to_string(), None))
        })
        .await;

    assert!(result.is_err());
    // Should only attempt once for non-retryable errors
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[test]
fn test_backoff_strategies() {
    use llm_cost_ops::sdk::BackoffStrategy;

    // Fixed backoff
    let fixed = BackoffStrategy::Fixed(Duration::from_secs(1));
    assert_eq!(fixed.calculate(1), Duration::from_secs(1));
    assert_eq!(fixed.calculate(10), Duration::from_secs(1));

    // Exponential backoff
    let exponential = BackoffStrategy::Exponential {
        initial: Duration::from_secs(1),
        max: Duration::from_secs(60),
        multiplier: 2.0,
        jitter: false,
    };
    assert_eq!(exponential.calculate(1), Duration::from_secs(1));
    assert_eq!(exponential.calculate(2), Duration::from_secs(2));
    assert_eq!(exponential.calculate(3), Duration::from_secs(4));

    // Linear backoff
    let linear = BackoffStrategy::Linear {
        initial: Duration::from_secs(1),
        increment: Duration::from_secs(1),
        max: Duration::from_secs(10),
    };
    assert_eq!(linear.calculate(1), Duration::from_secs(1));
    assert_eq!(linear.calculate(2), Duration::from_secs(2));
    assert_eq!(linear.calculate(3), Duration::from_secs(3));
    assert_eq!(linear.calculate(20), Duration::from_secs(10)); // Capped
}

#[test]
fn test_telemetry_collector() {
    use llm_cost_ops::sdk::TelemetryCollector;

    let collector = TelemetryCollector::new("test_sdk", true);

    // Test timer
    let timer = collector.start_timer();
    std::thread::sleep(Duration::from_millis(10));
    assert!(timer.elapsed().as_millis() >= 10);
    timer.success();

    // Test metrics recording
    collector.record_rate_limit_hit();
    collector.set_active_connections(5);
}
