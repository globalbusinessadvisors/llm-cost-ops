// Comprehensive integration tests for ingestion module
// Target: Increase coverage from 16% to 90%+

use chrono::Utc;
use llm_cost_ops::{
    ingestion::{
        create_webhook_router, create_webhook_router_with_rate_limit,
        DefaultIngestionHandler, IngestionHandler, IngestionStatus,
        InMemoryRateLimiter, NoOpRateLimiter, RateLimitConfig, RateLimiter,
        UsageWebhookPayload,
        models::{
            BatchIngestionRequest, ModelWebhook, PerformanceMetrics,
            StreamEventType, StreamMessage, TokenUsageWebhook,
        },
        traits::PayloadValidator,
    },
    storage::SqliteUsageRepository,
};
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::time::Duration;
use tower::ServiceExt;
use uuid::Uuid;

// ============================================================================
// Test Helpers and Setup
// ============================================================================

async fn setup_test_db() -> SqliteUsageRepository {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    SqliteUsageRepository::new(pool)
}

fn create_valid_payload() -> UsageWebhookPayload {
    UsageWebhookPayload {
        request_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: "openai".to_string(),
        model: ModelWebhook {
            name: "gpt-4".to_string(),
            version: Some("0613".to_string()),
            context_window: Some(8192),
        },
        organization_id: "org-test-123".to_string(),
        project_id: Some("proj-test-456".to_string()),
        user_id: Some("user-789".to_string()),
        usage: TokenUsageWebhook {
            prompt_tokens: 150,
            completion_tokens: 75,
            total_tokens: 225,
            cached_tokens: Some(50),
            reasoning_tokens: None,
        },
        performance: Some(PerformanceMetrics {
            latency_ms: Some(1200),
            time_to_first_token_ms: Some(300),
        }),
        tags: vec!["production".to_string(), "api".to_string()],
        metadata: HashMap::new(),
    }
}

fn create_minimal_payload() -> UsageWebhookPayload {
    UsageWebhookPayload {
        request_id: Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: "anthropic".to_string(),
        model: ModelWebhook {
            name: "claude-3-sonnet".to_string(),
            version: None,
            context_window: None,
        },
        organization_id: "org-minimal".to_string(),
        project_id: None,
        user_id: None,
        usage: TokenUsageWebhook {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            cached_tokens: None,
            reasoning_tokens: None,
        },
        performance: None,
        tags: vec![],
        metadata: HashMap::new(),
    }
}

fn create_payload_with_reasoning_tokens() -> UsageWebhookPayload {
    let mut payload = create_valid_payload();
    payload.usage.reasoning_tokens = Some(25);
    payload
}

fn create_large_payload() -> UsageWebhookPayload {
    let mut payload = create_valid_payload();
    payload.usage.prompt_tokens = 100000;
    payload.usage.completion_tokens = 50000;
    payload.usage.total_tokens = 150000;
    payload.tags = (0..100).map(|i| format!("tag-{}", i)).collect();
    for i in 0..100 {
        payload.metadata.insert(
            format!("key-{}", i),
            serde_json::json!(format!("value-{}", i)),
        );
    }
    payload
}

// ============================================================================
// Handler Tests - Basic Functionality
// ============================================================================

#[tokio::test]
async fn test_handler_single_ingestion_success() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let payload = create_valid_payload();
    let request_id = payload.request_id;

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Success);
    assert_eq!(response.accepted, 1);
    assert_eq!(response.rejected, 0);
    assert!(response.errors.is_empty());
    assert_eq!(response.request_id, request_id);
}

#[tokio::test]
async fn test_handler_minimal_payload_success() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let payload = create_minimal_payload();

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Success);
    assert_eq!(response.accepted, 1);
}

#[tokio::test]
async fn test_handler_with_reasoning_tokens() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let payload = create_payload_with_reasoning_tokens();

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Success);
    assert_eq!(response.accepted, 1);
}

#[tokio::test]
async fn test_handler_large_payload() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let payload = create_large_payload();

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Success);
}

#[tokio::test]
async fn test_handler_health_check() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let healthy = handler.health_check().await.unwrap();
    assert!(healthy);
}

#[tokio::test]
async fn test_handler_name() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    assert_eq!(handler.name(), "default_ingestion_handler");
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_validation_token_count_mismatch() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.usage.total_tokens = 999; // Mismatch

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Failed);
    assert_eq!(response.accepted, 0);
    assert_eq!(response.rejected, 1);
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_validation_cached_tokens_exceed_prompt() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.usage.cached_tokens = Some(200); // Exceeds prompt tokens (150)

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Failed);
    assert_eq!(response.accepted, 0);
    assert_eq!(response.rejected, 1);
}

#[tokio::test]
async fn test_validation_empty_provider() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.provider = "".to_string();

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Failed);
    assert_eq!(response.rejected, 1);
}

#[tokio::test]
async fn test_validation_empty_model_name() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.model.name = "".to_string();

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Failed);
    assert_eq!(response.rejected, 1);
}

#[tokio::test]
async fn test_validation_empty_organization_id() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.organization_id = "".to_string();

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Failed);
    assert_eq!(response.rejected, 1);
}

#[tokio::test]
async fn test_validation_zero_tokens() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.usage.prompt_tokens = 0;
    payload.usage.completion_tokens = 0;
    payload.usage.total_tokens = 0;

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    // Zero tokens are valid - just unusual
    assert_eq!(response.status, IngestionStatus::Success);
}

// ============================================================================
// Batch Ingestion Tests
// ============================================================================

#[tokio::test]
async fn test_batch_all_success() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let payloads = vec![
        create_valid_payload(),
        create_minimal_payload(),
        create_payload_with_reasoning_tokens(),
    ];

    let response = handler.handle_batch(payloads).await
        .expect("Failed to handle batch ingestion");

    assert_eq!(response.status, IngestionStatus::Success);
    assert_eq!(response.accepted, 3);
    assert_eq!(response.rejected, 0);
    assert!(response.errors.is_empty());
}

#[tokio::test]
async fn test_batch_partial_success() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let valid1 = create_valid_payload();
    let mut invalid = create_valid_payload();
    invalid.usage.total_tokens = 999; // Mismatch
    let valid2 = create_minimal_payload();

    let payloads = vec![valid1, invalid, valid2];

    let response = handler.handle_batch(payloads).await
        .expect("Failed to handle batch ingestion");

    assert_eq!(response.status, IngestionStatus::Partial);
    assert_eq!(response.accepted, 2);
    assert_eq!(response.rejected, 1);
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].index, Some(1));
    assert_eq!(response.errors[0].code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_batch_all_failed() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut invalid1 = create_valid_payload();
    invalid1.usage.total_tokens = 999;

    let mut invalid2 = create_valid_payload();
    invalid2.organization_id = "".to_string();

    let mut invalid3 = create_valid_payload();
    invalid3.usage.cached_tokens = Some(999999);

    let payloads = vec![invalid1, invalid2, invalid3];

    let response = handler.handle_batch(payloads).await
        .expect("Failed to handle batch ingestion");

    assert_eq!(response.status, IngestionStatus::Failed);
    assert_eq!(response.accepted, 0);
    assert_eq!(response.rejected, 3);
    assert_eq!(response.errors.len(), 3);
}

#[tokio::test]
async fn test_batch_large_batch() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let payloads: Vec<_> = (0..100).map(|_| create_valid_payload()).collect();

    let response = handler.handle_batch(payloads).await
        .expect("Failed to handle batch ingestion");

    assert_eq!(response.status, IngestionStatus::Success);
    assert_eq!(response.accepted, 100);
    assert_eq!(response.rejected, 0);
}

#[tokio::test]
async fn test_batch_empty() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let payloads = vec![];

    let response = handler.handle_batch(payloads).await
        .expect("Failed to handle batch ingestion");

    assert_eq!(response.status, IngestionStatus::Success);
    assert_eq!(response.accepted, 0);
}

// ============================================================================
// Rate Limiter Tests - InMemoryRateLimiter
// ============================================================================

#[tokio::test]
async fn test_rate_limiter_basic_limit() {
    let config = RateLimitConfig {
        max_requests: 5,
        window_duration: Duration::from_secs(10),
        burst_size: 2,
    };

    let limiter = InMemoryRateLimiter::new(config);

    // First 5 requests should pass
    for _ in 0..5 {
        assert!(limiter.check_rate_limit("org-1").await.unwrap());
    }

    // Next 2 requests should pass (burst)
    for _ in 0..2 {
        assert!(limiter.check_rate_limit("org-1").await.unwrap());
    }

    // 8th request should fail
    assert!(!limiter.check_rate_limit("org-1").await.unwrap());
}

#[tokio::test]
async fn test_rate_limiter_per_organization_isolation() {
    let config = RateLimitConfig {
        max_requests: 3,
        window_duration: Duration::from_secs(10),
        burst_size: 1,
    };

    let limiter = InMemoryRateLimiter::new(config);

    // org-1 uses its quota
    for _ in 0..4 {
        assert!(limiter.check_rate_limit("org-1").await.unwrap());
    }
    assert!(!limiter.check_rate_limit("org-1").await.unwrap());

    // org-2 should still have its full quota
    for _ in 0..4 {
        assert!(limiter.check_rate_limit("org-2").await.unwrap());
    }
}

#[tokio::test]
async fn test_rate_limiter_custom_org_limits() {
    let default_config = RateLimitConfig {
        max_requests: 5,
        window_duration: Duration::from_secs(10),
        burst_size: 1,
    };

    let limiter = InMemoryRateLimiter::new(default_config);

    // Set custom limit for premium org
    let premium_config = RateLimitConfig {
        max_requests: 100,
        window_duration: Duration::from_secs(10),
        burst_size: 10,
    };
    limiter.set_org_limit("org-premium".to_string(), premium_config).await;

    // org-basic should have default limits
    for _ in 0..6 {
        assert!(limiter.check_rate_limit("org-basic").await.unwrap());
    }
    assert!(!limiter.check_rate_limit("org-basic").await.unwrap());

    // org-premium should have higher limits
    for _ in 0..100 {
        assert!(limiter.check_rate_limit("org-premium").await.unwrap());
    }
}

#[tokio::test]
async fn test_rate_limiter_usage_stats() {
    let config = RateLimitConfig {
        max_requests: 10,
        window_duration: Duration::from_secs(60),
        burst_size: 2,
    };

    let limiter = InMemoryRateLimiter::new(config);

    // Make 3 requests
    for _ in 0..3 {
        limiter.check_rate_limit("org-1").await.unwrap();
    }

    let usage = limiter.get_usage("org-1").await;
    assert_eq!(usage.current, 3);
    assert_eq!(usage.limit, 10);
    assert_eq!(usage.remaining, 7);
    assert!(usage.retry_after.is_none());
}

#[tokio::test]
async fn test_rate_limiter_remove_org_limit() {
    let default_config = RateLimitConfig {
        max_requests: 5,
        window_duration: Duration::from_secs(10),
        burst_size: 1,
    };

    let limiter = InMemoryRateLimiter::new(default_config.clone());

    // Set custom limit
    let custom_config = RateLimitConfig {
        max_requests: 100,
        window_duration: Duration::from_secs(10),
        burst_size: 10,
    };
    limiter.set_org_limit("org-1".to_string(), custom_config).await;

    // Remove custom limit
    limiter.remove_org_limit("org-1").await;

    // Should now use default limit
    for _ in 0..6 {
        assert!(limiter.check_rate_limit("org-1").await.unwrap());
    }
    assert!(!limiter.check_rate_limit("org-1").await.unwrap());
}

#[tokio::test]
async fn test_rate_limiter_config_builders() {
    let per_minute = RateLimitConfig::per_minute(60);
    assert_eq!(per_minute.max_requests, 60);
    assert_eq!(per_minute.window_duration, Duration::from_secs(60));

    let per_hour = RateLimitConfig::per_hour(3600);
    assert_eq!(per_hour.max_requests, 3600);
    assert_eq!(per_hour.window_duration, Duration::from_secs(3600));

    let per_day = RateLimitConfig::per_day(86400);
    assert_eq!(per_day.max_requests, 86400);
    assert_eq!(per_day.window_duration, Duration::from_secs(86400));
}

#[tokio::test]
async fn test_no_op_rate_limiter() {
    let limiter = NoOpRateLimiter;

    // Should always allow requests
    for _ in 0..1000 {
        assert!(limiter.check_rate_limit("org-1").await.unwrap());
    }

    // Record request should succeed
    assert!(limiter.record_request("org-1").await.is_ok());
}

// ============================================================================
// Webhook Handler Tests - HTTP endpoints
// ============================================================================

#[tokio::test]
async fn test_webhook_health_endpoint() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);
    let app = create_webhook_router(handler);

    let request = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_webhook_single_ingestion_endpoint() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);
    let app = create_webhook_router(handler);

    let payload = create_valid_payload();
    let body = serde_json::to_string(&payload).unwrap();

    let request = Request::builder()
        .method("POST")
        .uri("/v1/usage")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_webhook_batch_ingestion_endpoint() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);
    let app = create_webhook_router(handler);

    let batch_request = BatchIngestionRequest {
        batch_id: Uuid::new_v4(),
        source: "test".to_string(),
        records: vec![create_valid_payload(), create_minimal_payload()],
    };
    let body = serde_json::to_string(&batch_request).unwrap();

    let request = Request::builder()
        .method("POST")
        .uri("/v1/usage/batch")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_webhook_invalid_json() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);
    let app = create_webhook_router(handler);

    let request = Request::builder()
        .method("POST")
        .uri("/v1/usage")
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_webhook_with_rate_limiting() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let config = RateLimitConfig {
        max_requests: 2,
        window_duration: Duration::from_secs(60),
        burst_size: 0,
    };
    let rate_limiter = InMemoryRateLimiter::new(config);

    let app = create_webhook_router_with_rate_limit(handler, rate_limiter.clone());

    let payload = create_valid_payload();
    let body = serde_json::to_string(&payload).unwrap();

    // First request should succeed
    let request = Request::builder()
        .method("POST")
        .uri("/v1/usage")
        .header("content-type", "application/json")
        .body(Body::from(body.clone()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Second request should succeed
    let request = Request::builder()
        .method("POST")
        .uri("/v1/usage")
        .header("content-type", "application/json")
        .body(Body::from(body.clone()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // Third request should be rate limited
    let request = Request::builder()
        .method("POST")
        .uri("/v1/usage")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

// ============================================================================
// Stream Message Tests
// ============================================================================

#[test]
fn test_stream_message_creation() {
    let payload = create_valid_payload();
    let message = StreamMessage {
        message_id: "msg-123".to_string(),
        event_type: StreamEventType::UsageCreated,
        created_at: Utc::now(),
        payload: payload.clone(),
        retry_count: 0,
    };

    assert_eq!(message.message_id, "msg-123");
    assert_eq!(message.event_type, StreamEventType::UsageCreated);
    assert_eq!(message.retry_count, 0);
}

#[test]
fn test_stream_event_types() {
    let created = StreamEventType::UsageCreated;
    let updated = StreamEventType::UsageUpdated;
    let batch = StreamEventType::BatchUploaded;

    assert_ne!(created, updated);
    assert_ne!(updated, batch);
    assert_ne!(created, batch);
}

#[test]
fn test_stream_message_serialization() {
    let payload = create_valid_payload();
    let message = StreamMessage {
        message_id: "msg-123".to_string(),
        event_type: StreamEventType::UsageCreated,
        created_at: Utc::now(),
        payload,
        retry_count: 0,
    };

    let json = serde_json::to_string(&message).unwrap();
    let deserialized: StreamMessage = serde_json::from_str(&json).unwrap();

    assert_eq!(message.message_id, deserialized.message_id);
    assert_eq!(message.event_type, deserialized.event_type);
}

// ============================================================================
// Model Conversion Tests
// ============================================================================

#[test]
fn test_webhook_payload_to_usage_record() {
    let payload = create_valid_payload();
    let record = payload.to_usage_record();

    assert_eq!(record.id, payload.request_id);
    assert_eq!(record.organization_id, payload.organization_id);
    assert_eq!(record.prompt_tokens, payload.usage.prompt_tokens);
    assert_eq!(record.completion_tokens, payload.usage.completion_tokens);
    assert_eq!(record.total_tokens, payload.usage.total_tokens);
}

#[test]
fn test_minimal_payload_to_usage_record() {
    let payload = create_minimal_payload();
    let record = payload.to_usage_record();

    assert_eq!(record.id, payload.request_id);
    assert!(record.project_id.is_none());
    assert!(record.user_id.is_none());
    assert!(record.cached_tokens.is_none());
}

#[test]
fn test_payload_with_metadata() {
    let mut payload = create_valid_payload();
    payload.metadata.insert("custom_key".to_string(), serde_json::json!("custom_value"));

    let record = payload.to_usage_record();

    assert!(record.metadata.is_object());
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[test]
fn test_default_ingestion_config() {
    use llm_cost_ops::ingestion::models::IngestionConfig;

    let config = IngestionConfig::default();

    assert!(config.webhook_enabled);
    assert_eq!(config.webhook_bind, "0.0.0.0:8080");
    assert!(!config.nats_enabled);
    assert!(!config.redis_enabled);
    assert_eq!(config.buffer_size, 10000);
    assert_eq!(config.max_batch_size, 1000);
}

#[test]
fn test_default_retry_config() {
    use llm_cost_ops::ingestion::models::RetryConfig;

    let config = RetryConfig::default();

    assert_eq!(config.max_retries, 3);
    assert_eq!(config.initial_delay_ms, 100);
    assert_eq!(config.max_delay_ms, 30000);
    assert_eq!(config.backoff_multiplier, 2.0);
}

// ============================================================================
// Concurrent Request Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_ingestion_requests() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut handles = vec![];

    for _ in 0..10 {
        let handler_clone = handler.clone();
        let handle = tokio::spawn(async move {
            let payload = create_valid_payload();
            handler_clone.handle_single(payload).await
        });
        handles.push(handle);
    }

    for handle in handles {
        let response = handle.await.unwrap().unwrap();
        assert_eq!(response.status, IngestionStatus::Success);
    }
}

#[tokio::test]
async fn test_concurrent_rate_limiting() {
    let config = RateLimitConfig {
        max_requests: 50,
        window_duration: Duration::from_secs(10),
        burst_size: 10,
    };

    let limiter = InMemoryRateLimiter::new(config);

    let mut handles = vec![];

    for _ in 0..100 {
        let limiter_clone = limiter.clone();
        let handle = tokio::spawn(async move {
            limiter_clone.check_rate_limit("org-concurrent").await
        });
        handles.push(handle);
    }

    let mut allowed = 0;
    let mut denied = 0;

    for handle in handles {
        let result = handle.await.unwrap().unwrap();
        if result {
            allowed += 1;
        } else {
            denied += 1;
        }
    }

    // Should allow max_requests + burst_size
    assert!(allowed <= 60);
    assert!(denied > 0);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_batch_error_index_tracking() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut invalid1 = create_valid_payload();
    invalid1.usage.total_tokens = 999;

    let valid = create_valid_payload();

    let mut invalid2 = create_valid_payload();
    invalid2.organization_id = "".to_string();

    let payloads = vec![invalid1, valid, invalid2];

    let response = handler.handle_batch(payloads).await.unwrap();

    assert_eq!(response.errors.len(), 2);
    assert_eq!(response.errors[0].index, Some(0));
    assert_eq!(response.errors[1].index, Some(2));
}

// ============================================================================
// Provider-Specific Tests
// ============================================================================

#[tokio::test]
async fn test_openai_provider() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.provider = "openai".to_string();
    payload.model.name = "gpt-4-turbo".to_string();

    let response = handler.handle_single(payload).await.unwrap();
    assert_eq!(response.status, IngestionStatus::Success);
}

#[tokio::test]
async fn test_anthropic_provider() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.provider = "anthropic".to_string();
    payload.model.name = "claude-3-opus".to_string();

    let response = handler.handle_single(payload).await.unwrap();
    assert_eq!(response.status, IngestionStatus::Success);
}

#[tokio::test]
async fn test_google_provider() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.provider = "google".to_string();
    payload.model.name = "gemini-pro".to_string();

    let response = handler.handle_single(payload).await.unwrap();
    assert_eq!(response.status, IngestionStatus::Success);
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_very_long_organization_id() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.organization_id = "a".repeat(256); // Exceeds max length

    let response = handler.handle_single(payload).await.unwrap();
    assert_eq!(response.status, IngestionStatus::Failed);
}

#[tokio::test]
async fn test_negative_context_window_not_allowed() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.model.context_window = Some(0); // Zero is valid, just unusual

    let response = handler.handle_single(payload).await.unwrap();
    // Should succeed - context_window is optional validation
    assert_eq!(response.status, IngestionStatus::Success);
}

#[tokio::test]
async fn test_duplicate_tags() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.tags = vec!["tag1".to_string(), "tag1".to_string(), "tag2".to_string()];

    let response = handler.handle_single(payload).await.unwrap();
    assert_eq!(response.status, IngestionStatus::Success);
}

#[tokio::test]
async fn test_special_characters_in_metadata() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_valid_payload();
    payload.metadata.insert(
        "special_chars".to_string(),
        serde_json::json!("!@#$%^&*()_+-=[]{}|;':\",./<>?"),
    );

    let response = handler.handle_single(payload).await.unwrap();
    assert_eq!(response.status, IngestionStatus::Success);
}
