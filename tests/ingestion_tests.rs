// Integration tests for ingestion module

use chrono::Utc;
use llm_cost_ops::{
    ingestion::{
        DefaultIngestionHandler, IngestionHandler, IngestionStatus,
        UsageWebhookPayload,
        models::{ModelWebhook, PerformanceMetrics, TokenUsageWebhook},
    },
    storage::SqliteUsageRepository,
};
use sqlx::sqlite::SqlitePoolOptions;
use uuid::Uuid;

async fn setup_test_db() -> SqliteUsageRepository {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to create in-memory database");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    SqliteUsageRepository::new(pool)
}

fn create_test_payload() -> UsageWebhookPayload {
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
        metadata: std::collections::HashMap::new(),
    }
}

#[tokio::test]
async fn test_single_ingestion_success() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let payload = create_test_payload();
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
async fn test_single_ingestion_validation_error() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_test_payload();
    // Create token mismatch
    payload.usage.total_tokens = 999;

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Failed);
    assert_eq!(response.accepted, 0);
    assert_eq!(response.rejected, 1);
    assert_eq!(response.errors.len(), 1);
    assert_eq!(response.errors[0].code, "VALIDATION_ERROR");
}

#[tokio::test]
async fn test_batch_ingestion_success() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let payloads = vec![
        create_test_payload(),
        create_test_payload(),
        create_test_payload(),
    ];

    let response = handler.handle_batch(payloads).await
        .expect("Failed to handle batch ingestion");

    assert_eq!(response.status, IngestionStatus::Success);
    assert_eq!(response.accepted, 3);
    assert_eq!(response.rejected, 0);
    assert!(response.errors.is_empty());
}

#[tokio::test]
async fn test_batch_ingestion_partial_success() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let valid1 = create_test_payload();
    let mut invalid = create_test_payload();
    invalid.usage.total_tokens = 999; // Mismatch
    let valid2 = create_test_payload();

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
async fn test_cached_tokens_validation() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let mut payload = create_test_payload();
    // Cached tokens exceed prompt tokens
    payload.usage.cached_tokens = Some(200);

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Failed);
    assert_eq!(response.accepted, 0);
    assert_eq!(response.rejected, 1);
}

#[tokio::test]
async fn test_ingestion_with_minimal_payload() {
    let repository = setup_test_db().await;
    let handler = DefaultIngestionHandler::new(repository);

    let payload = UsageWebhookPayload {
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
        metadata: std::collections::HashMap::new(),
    };

    let response = handler.handle_single(payload).await
        .expect("Failed to handle ingestion");

    assert_eq!(response.status, IngestionStatus::Success);
    assert_eq!(response.accepted, 1);
    assert_eq!(response.rejected, 0);
}
