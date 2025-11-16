//! Type-safe request and response types for the SDK

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Request to submit usage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRequest {
    /// Organization ID
    pub organization_id: Uuid,

    /// Provider name
    pub provider: String,

    /// Model identifier
    pub model: String,

    /// Number of prompt tokens
    pub prompt_tokens: u64,

    /// Number of completion tokens
    pub completion_tokens: u64,

    /// Total tokens
    pub total_tokens: u64,

    /// Timestamp of usage
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,

    /// Request ID for idempotency
    pub request_id: Option<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Response from usage submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageResponse {
    /// Record ID
    pub id: Uuid,

    /// Processing status
    pub status: String,

    /// Cost calculation if available
    pub cost: Option<CostCalculation>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
}

/// Cost calculation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostCalculation {
    /// Prompt cost
    pub prompt_cost: Decimal,

    /// Completion cost
    pub completion_cost: Decimal,

    /// Total cost
    pub total_cost: Decimal,

    /// Currency
    pub currency: String,
}

/// Request for cost data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRequest {
    /// Organization ID
    pub organization_id: Uuid,

    /// Start date
    #[serde(with = "chrono::serde::ts_seconds")]
    pub start_date: DateTime<Utc>,

    /// End date
    #[serde(with = "chrono::serde::ts_seconds")]
    pub end_date: DateTime<Utc>,

    /// Filter by provider
    pub provider: Option<String>,

    /// Filter by model
    pub model: Option<String>,

    /// Aggregation level
    pub aggregation: Option<AggregationLevel>,
}

/// Aggregation level for cost queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AggregationLevel {
    /// Hourly aggregation
    Hourly,
    /// Daily aggregation
    Daily,
    /// Weekly aggregation
    Weekly,
    /// Monthly aggregation
    Monthly,
}

/// Response containing cost data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostResponse {
    /// Total cost
    pub total_cost: Decimal,

    /// Currency
    pub currency: String,

    /// Cost breakdown by period
    pub breakdown: Vec<CostBreakdown>,

    /// Query metadata
    pub metadata: QueryMetadata,
}

/// Cost breakdown for a specific period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    /// Period start
    pub period_start: DateTime<Utc>,

    /// Period end
    pub period_end: DateTime<Utc>,

    /// Cost for this period
    pub cost: Decimal,

    /// Token count
    pub tokens: u64,

    /// Request count
    pub requests: u64,

    /// Breakdown by provider
    pub by_provider: HashMap<String, Decimal>,

    /// Breakdown by model
    pub by_model: HashMap<String, Decimal>,
}

/// Request for cost forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastRequest {
    /// Organization ID
    pub organization_id: Uuid,

    /// Forecast horizon in days
    pub horizon_days: u32,

    /// Historical data period in days
    pub lookback_days: u32,

    /// Confidence level (0.0 - 1.0)
    pub confidence_level: Option<f64>,

    /// Include seasonality
    pub include_seasonality: bool,
}

/// Response containing forecast data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResponse {
    /// Forecasted costs by day
    pub forecasts: Vec<ForecastPoint>,

    /// Total forecasted cost
    pub total_forecast: Decimal,

    /// Currency
    pub currency: String,

    /// Model accuracy metrics
    pub metrics: ForecastMetrics,

    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
}

/// Single forecast data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastPoint {
    /// Date
    pub date: DateTime<Utc>,

    /// Forecasted cost
    pub cost: Decimal,

    /// Lower confidence bound
    pub lower_bound: Decimal,

    /// Upper confidence bound
    pub upper_bound: Decimal,

    /// Trend indicator
    pub trend: String,
}

/// Forecast accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastMetrics {
    /// Mean Absolute Error
    pub mae: f64,

    /// Root Mean Squared Error
    pub rmse: f64,

    /// Mean Absolute Percentage Error
    pub mape: f64,

    /// R-squared score
    pub r_squared: f64,
}

/// Query parameters for list operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryParams {
    /// Pagination parameters
    #[serde(flatten)]
    pub pagination: Pagination,

    /// Sorting field
    pub sort_by: Option<String>,

    /// Sort order
    pub sort_order: Option<SortOrder>,

    /// Filter parameters
    #[serde(flatten)]
    pub filters: HashMap<String, String>,
}

/// Pagination parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    /// Page number (1-indexed)
    pub page: Option<u32>,

    /// Items per page
    pub per_page: Option<u32>,

    /// Cursor for cursor-based pagination
    pub cursor: Option<String>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(50),
            cursor: None,
        }
    }
}

/// Sort order
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Ascending order
    Asc,
    /// Descending order
    Desc,
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Data items
    pub data: Vec<T>,

    /// Pagination metadata
    pub pagination: PaginationMeta,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMeta {
    /// Current page
    pub page: u32,

    /// Items per page
    pub per_page: u32,

    /// Total items
    pub total_items: u64,

    /// Total pages
    pub total_pages: u32,

    /// Has next page
    pub has_next: bool,

    /// Has previous page
    pub has_previous: bool,

    /// Next cursor
    pub next_cursor: Option<String>,
}

/// Query metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryMetadata {
    /// Query execution time in milliseconds
    pub execution_time_ms: u64,

    /// Number of records processed
    pub records_processed: u64,

    /// Cache hit indicator
    pub cached: bool,
}

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// Service status
    pub status: String,

    /// Version
    pub version: String,

    /// Uptime in seconds
    pub uptime_seconds: u64,

    /// Component health
    pub components: HashMap<String, ComponentHealth>,
}

/// Component health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Status
    pub status: String,

    /// Message
    pub message: Option<String>,

    /// Last check timestamp
    pub last_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
            request_id: Some("test-123".to_string()),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UsageRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.provider, deserialized.provider);
        assert_eq!(request.model, deserialized.model);
    }

    #[test]
    fn test_pagination_default() {
        let pagination = Pagination::default();
        assert_eq!(pagination.page, Some(1));
        assert_eq!(pagination.per_page, Some(50));
    }

    #[test]
    fn test_aggregation_level() {
        let json = r#""daily""#;
        let level: AggregationLevel = serde_json::from_str(json).unwrap();
        assert!(matches!(level, AggregationLevel::Daily));
    }

    #[test]
    fn test_sort_order() {
        let asc: SortOrder = serde_json::from_str(r#""asc""#).unwrap();
        let desc: SortOrder = serde_json::from_str(r#""desc""#).unwrap();
        assert!(matches!(asc, SortOrder::Asc));
        assert!(matches!(desc, SortOrder::Desc));
    }
}
