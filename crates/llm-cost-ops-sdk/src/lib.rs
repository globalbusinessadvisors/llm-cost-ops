//! LLM-CostOps SDK - Client library for integrating with LLM Cost Ops
//!
//! This crate provides a high-level SDK for interacting with the LLM Cost Ops
//! API, including automatic retries, telemetry, and type-safe client methods.

pub mod sdk;

// Re-export commonly used types
pub use sdk::{
    // Client
    CostOpsClient, ClientBuilder,
    // Configuration
    ClientConfig, RetryConfig, TelemetryConfig, PoolConfig, RateLimitConfig,
    // Error types
    SdkError, SdkResult,
    // Types
    UsageRequest, UsageResponse, CostRequest, CostResponse,
    ForecastRequest, ForecastResponse, QueryParams, Pagination,
    PaginatedResponse,
    // Retry
    RetryPolicy, BackoffStrategy,
    // Telemetry
    SdkMetrics, TelemetryCollector,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
