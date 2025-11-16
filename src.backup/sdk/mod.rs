//! Enterprise-grade Rust SDK for LLM-CostOps
//!
//! This module provides a production-ready, type-safe SDK for interacting with the
//! LLM-CostOps platform. It features:
//!
//! - Zero-cost abstractions with full type safety
//! - Async/await support with tokio runtime
//! - Automatic retry logic with exponential backoff
//! - Connection pooling and rate limiting
//! - Structured logging and distributed tracing
//! - Metrics and telemetry collection
//! - Trait-based extensibility

pub mod client;
pub mod config;
pub mod error;
pub mod retry;
pub mod telemetry;
pub mod types;

pub use client::{CostOpsClient, ClientBuilder};
pub use config::{ClientConfig, RetryConfig, TelemetryConfig, PoolConfig, RateLimitConfig};
pub use error::{SdkError, SdkResult};
pub use retry::{RetryPolicy, BackoffStrategy};
pub use telemetry::{SdkMetrics, TelemetryCollector};
pub use types::{
    UsageRequest, UsageResponse, CostRequest, CostResponse,
    ForecastRequest, ForecastResponse, QueryParams, Pagination,
    PaginatedResponse,
};
