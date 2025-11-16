// Observatory integration module for real-time data ingestion

pub mod handler;
pub mod middleware;
pub mod models;
pub mod ratelimit;
pub mod stream;
pub mod traits;
pub mod webhook;

pub use handler::{DefaultIngestionHandler, StorageAdapter};
pub use middleware::{RateLimitMiddleware, rate_limit_middleware};
pub use models::{
    BatchIngestionRequest, IngestionConfig, IngestionError, IngestionResponse, IngestionStatus,
    RetryConfig, StreamEventType, StreamMessage, UsageWebhookPayload,
};
pub use ratelimit::{
    InMemoryRateLimiter, NoOpRateLimiter, RateLimitConfig, RateLimitUsage, RedisRateLimiter,
};
pub use stream::{NatsConsumer, RedisConsumer};
pub use traits::{IngestionHandler, IngestionStorage, PayloadValidator, RateLimiter, RecordBuffer};
pub use webhook::{create_webhook_router, create_webhook_router_with_rate_limit, start_webhook_server, WebhookServerState};
