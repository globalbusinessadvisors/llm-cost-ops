//! LLM-CostOps API - REST API and data ingestion endpoints
//!
//! This crate provides the HTTP API server and data ingestion functionality
//! for LLM Cost Operations.

pub mod api;
pub mod ingestion;

// Re-export commonly used types from api module
pub use api::{
    // Error types
    ApiError, ApiResult,
    // Server
    ApiServer, ApiServerConfig, create_api_router,
    // Types
    ApiVersion, ApiResponse, PaginationParams, PaginatedResponse,
    // Constants
    API_VERSION, API_PREFIX,
};

// Re-export commonly used types from ingestion module
pub use ingestion::{
    // Handler
    DefaultIngestionHandler, StorageAdapter,
    IngestionHandler, IngestionStorage, PayloadValidator,
    // Models
    BatchIngestionRequest, IngestionConfig, IngestionError, IngestionResponse, IngestionStatus,
    StreamEventType, StreamMessage, UsageWebhookPayload,
    // Rate limiting
    RateLimiter, RateLimitConfig, RateLimitUsage,
    InMemoryRateLimiter, NoOpRateLimiter, RedisRateLimiter,
    RateLimitMiddleware, rate_limit_middleware,
    // Streaming
    NatsConsumer, RedisConsumer,
    RecordBuffer,
    // Webhook
    create_webhook_router, create_webhook_router_with_rate_limit,
    start_webhook_server, WebhookServerState,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
