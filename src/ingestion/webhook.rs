// Webhook HTTP server for receiving usage data

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};
use tracing::{error, info, warn};

use crate::domain::Result;

use super::models::{BatchIngestionRequest, IngestionResponse, UsageWebhookPayload};
use super::traits::{IngestionHandler, RateLimiter};

/// Webhook server state
#[derive(Clone)]
pub struct WebhookServerState<H: IngestionHandler> {
    handler: Arc<H>,
}

impl<H: IngestionHandler> WebhookServerState<H> {
    pub fn new(handler: H) -> Self {
        Self {
            handler: Arc::new(handler),
        }
    }
}

/// Webhook server state with rate limiting
#[derive(Clone)]
pub struct WebhookServerStateWithRateLimit<H: IngestionHandler, R: RateLimiter> {
    handler: Arc<H>,
    rate_limiter: Arc<R>,
}

impl<H: IngestionHandler, R: RateLimiter> WebhookServerStateWithRateLimit<H, R> {
    pub fn new(handler: H, rate_limiter: R) -> Self {
        Self {
            handler: Arc::new(handler),
            rate_limiter: Arc::new(rate_limiter),
        }
    }
}

/// Create webhook router with all endpoints
pub fn create_webhook_router<H: IngestionHandler + 'static>(
    handler: H,
) -> Router {
    let state = WebhookServerState::new(handler);

    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/usage", post(ingest_single_handler::<H>))
        .route("/v1/usage/batch", post(ingest_batch_handler::<H>))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(crate::metrics::middleware::metrics_middleware))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO)),
                )
                .layer(CorsLayer::permissive()),
        )
}

/// Create webhook router with rate limiting enabled
pub fn create_webhook_router_with_rate_limit<H: IngestionHandler + 'static, R: RateLimiter + Clone + 'static>(
    handler: H,
    rate_limiter: R,
) -> Router {
    let state = WebhookServerStateWithRateLimit::new(handler, rate_limiter);

    Router::new()
        .route("/health", get(health_handler))
        .route("/v1/usage", post(ingest_single_handler_with_rate_limit))
        .route("/v1/usage/batch", post(ingest_batch_handler_with_rate_limit))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(axum::middleware::from_fn(crate::metrics::middleware::metrics_middleware))
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(tracing::Level::INFO)),
                )
                .layer(CorsLayer::permissive()),
        )
}

/// Single usage record ingestion endpoint with rate limiting
async fn ingest_single_handler_with_rate_limit<H: IngestionHandler, R: RateLimiter>(
    State(state): State<WebhookServerStateWithRateLimit<H, R>>,
    Json(payload): Json<UsageWebhookPayload>,
) -> std::result::Result<Json<IngestionResponse>, AppError> {
    use std::time::Instant;
    let start = Instant::now();
    let org_id = payload.organization_id.clone();

    info!(
        request_id = %payload.request_id,
        organization_id = %org_id,
        "Received single usage ingestion request"
    );

    // Check rate limit
    match state.rate_limiter.check_rate_limit(&org_id).await {
        Ok(allowed) => {
            if !allowed {
                warn!(
                    organization_id = %org_id,
                    "Rate limit exceeded"
                );
                let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
                crate::metrics::collectors::IngestionMetrics::record_failure(&org_id, "rate_limit", duration_ms);
                return Err(AppError::RateLimitExceeded(org_id));
            }
        }
        Err(e) => {
            error!(error = %e, "Rate limit check failed, allowing request");
            // On error, allow the request (fail open for availability)
        }
    }

    // Process request
    match state.handler.handle_single(payload).await {
        Ok(response) => {
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            crate::metrics::collectors::IngestionMetrics::record_success(&org_id, 1, duration_ms);
            Ok(Json(response))
        }
        Err(e) => {
            error!(error = %e, "Failed to handle ingestion request");
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            crate::metrics::collectors::IngestionMetrics::record_failure(&org_id, "processing_error", duration_ms);
            Err(AppError::InternalError(e.to_string()))
        }
    }
}

/// Batch usage records ingestion endpoint with rate limiting
async fn ingest_batch_handler_with_rate_limit<H: IngestionHandler, R: RateLimiter>(
    State(state): State<WebhookServerStateWithRateLimit<H, R>>,
    Json(request): Json<BatchIngestionRequest>,
) -> std::result::Result<Json<IngestionResponse>, AppError> {
    use std::time::Instant;
    let start = Instant::now();
    let batch_size = request.records.len();

    info!(
        batch_id = %request.batch_id,
        batch_size = batch_size,
        source = %request.source,
        "Received batch usage ingestion request"
    );

    // Record batch size metric
    crate::metrics::collectors::IngestionMetrics::record_batch_size(batch_size);

    // Extract organization ID from first record
    let org_id = request.records.first()
        .map(|r| r.organization_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    // Check rate limit
    match state.rate_limiter.check_rate_limit(&org_id).await {
        Ok(allowed) => {
            if !allowed {
                warn!(
                    organization_id = %org_id,
                    "Rate limit exceeded for batch request"
                );
                let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
                crate::metrics::collectors::IngestionMetrics::record_failure(&org_id, "rate_limit", duration_ms);
                return Err(AppError::RateLimitExceeded(org_id));
            }
        }
        Err(e) => {
            error!(error = %e, "Rate limit check failed, allowing request");
        }
    }

    // Process request
    match state.handler.handle_batch(request.records).await {
        Ok(response) => {
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            crate::metrics::collectors::IngestionMetrics::record_success(&org_id, response.accepted, duration_ms);
            if response.rejected > 0 {
                crate::metrics::collectors::IngestionMetrics::record_rejected(&org_id, response.rejected);
            }
            Ok(Json(response))
        }
        Err(e) => {
            error!(error = %e, "Failed to handle batch ingestion request");
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            crate::metrics::collectors::IngestionMetrics::record_failure(&org_id, "processing_error", duration_ms);
            Err(AppError::InternalError(e.to_string()))
        }
    }
}

/// Health check endpoint
async fn health_handler() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "llm-cost-ops-ingestion",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Single usage record ingestion endpoint
async fn ingest_single_handler<H: IngestionHandler>(
    State(state): State<WebhookServerState<H>>,
    Json(payload): Json<UsageWebhookPayload>,
) -> std::result::Result<Json<IngestionResponse>, AppError> {
    use std::time::Instant;
    let start = Instant::now();
    let org_id = payload.organization_id.clone();

    info!(
        request_id = %payload.request_id,
        organization_id = %org_id,
        "Received single usage ingestion request"
    );

    match state.handler.handle_single(payload).await {
        Ok(response) => {
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            crate::metrics::collectors::IngestionMetrics::record_success(&org_id, 1, duration_ms);
            Ok(Json(response))
        }
        Err(e) => {
            error!(error = %e, "Failed to handle ingestion request");
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            crate::metrics::collectors::IngestionMetrics::record_failure(&org_id, "processing_error", duration_ms);
            Err(AppError::InternalError(e.to_string()))
        }
    }
}

/// Batch usage records ingestion endpoint
async fn ingest_batch_handler<H: IngestionHandler>(
    State(state): State<WebhookServerState<H>>,
    Json(request): Json<BatchIngestionRequest>,
) -> std::result::Result<Json<IngestionResponse>, AppError> {
    use std::time::Instant;
    let start = Instant::now();
    let batch_size = request.records.len();

    // Extract organization ID from first record
    let org_id = request.records.first()
        .map(|r| r.organization_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    info!(
        batch_id = %request.batch_id,
        batch_size = batch_size,
        source = %request.source,
        organization_id = %org_id,
        "Received batch usage ingestion request"
    );

    // Record batch size metric
    crate::metrics::collectors::IngestionMetrics::record_batch_size(batch_size);

    match state.handler.handle_batch(request.records).await {
        Ok(response) => {
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            crate::metrics::collectors::IngestionMetrics::record_success(&org_id, response.accepted, duration_ms);
            if response.rejected > 0 {
                crate::metrics::collectors::IngestionMetrics::record_rejected(&org_id, response.rejected);
            }
            Ok(Json(response))
        }
        Err(e) => {
            error!(error = %e, "Failed to handle batch ingestion request");
            let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
            crate::metrics::collectors::IngestionMetrics::record_failure(&org_id, "processing_error", duration_ms);
            Err(AppError::InternalError(e.to_string()))
        }
    }
}

/// Application error types
#[derive(Debug)]
pub enum AppError {
    ValidationError(String),
    InternalError(String),
    RateLimitExceeded(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, org_id) = match self {
            AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg, None),
            AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg, None),
            AppError::RateLimitExceeded(org) => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
                Some(org),
            ),
        };

        let mut body_json = serde_json::json!({
            "error": error_message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        if let Some(org) = org_id {
            body_json["organization_id"] = serde_json::json!(org);
        }

        let body = Json(body_json);

        let mut response = (status, body).into_response();

        // Add rate limit headers for rate limit errors
        if status == StatusCode::TOO_MANY_REQUESTS {
            response.headers_mut().insert(
                "Retry-After",
                "60".parse().unwrap(),
            );
            response.headers_mut().insert(
                "X-RateLimit-Limit",
                "1000".parse().unwrap(),
            );
            response.headers_mut().insert(
                "X-RateLimit-Remaining",
                "0".parse().unwrap(),
            );
        }

        response
    }
}

/// Start webhook server
pub async fn start_webhook_server<H: IngestionHandler + 'static>(
    bind_addr: &str,
    handler: H,
) -> Result<()> {
    info!(bind_addr = %bind_addr, "Starting webhook server");

    let app = create_webhook_router(handler);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;

    info!(
        addr = %listener.local_addr()?,
        "Webhook server listening"
    );

    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::UsageRecord;
    use async_trait::async_trait;
    use chrono::Utc;
    use uuid::Uuid;

    #[derive(Clone)]
    struct MockHandler;

    #[async_trait]
    impl IngestionHandler for MockHandler {
        async fn handle_single(
            &self,
            payload: UsageWebhookPayload,
        ) -> Result<IngestionResponse> {
            Ok(IngestionResponse {
                request_id: payload.request_id,
                status: super::super::models::IngestionStatus::Success,
                accepted: 1,
                rejected: 0,
                errors: vec![],
                processed_at: Utc::now(),
            })
        }

        async fn handle_batch(
            &self,
            payloads: Vec<UsageWebhookPayload>,
        ) -> Result<IngestionResponse> {
            Ok(IngestionResponse {
                request_id: Uuid::new_v4(),
                status: super::super::models::IngestionStatus::Success,
                accepted: payloads.len(),
                rejected: 0,
                errors: vec![],
                processed_at: Utc::now(),
            })
        }

        fn name(&self) -> &str {
            "mock_handler"
        }

        async fn health_check(&self) -> Result<bool> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_create_router() {
        let handler = MockHandler;
        let router = create_webhook_router(handler);
        // Router creation should succeed
        assert!(true);
    }
}
