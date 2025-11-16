// API request and response types

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use validator::Validate;

use llm_cost_ops::{Currency, Provider};

/// API version
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ApiVersion {
    V1,
}

/// Standard API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMetadata>,
}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub request_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

impl<T> ApiResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data, meta: None }
    }

    pub fn with_meta(data: T, meta: ResponseMetadata) -> Self {
        Self {
            data,
            meta: Some(meta),
        }
    }
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

// ===== Usage API Types =====

/// Submit usage request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SubmitUsageRequest {
    #[validate(length(min = 1, max = 100))]
    pub organization_id: String,

    pub provider: Provider,

    #[validate(length(min = 1, max = 255))]
    pub model_id: String,

    #[validate(range(min = 0))]
    pub input_tokens: u64,

    #[validate(range(min = 0))]
    pub output_tokens: u64,

    #[validate(range(min = 0))]
    pub total_tokens: u64,

    pub timestamp: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// Submit usage response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitUsageResponse {
    pub usage_id: String,
    pub organization_id: String,
    pub estimated_cost: Decimal,
    pub currency: Currency,
    pub processed_at: DateTime<Utc>,
}

// ===== Cost API Types =====

/// Get costs request query parameters
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct GetCostsQuery {
    #[validate(length(min = 1))]
    pub organization_id: Option<String>,

    pub provider: Option<Provider>,

    pub model_id: Option<String>,

    pub start_date: Option<DateTime<Utc>>,

    pub end_date: Option<DateTime<Utc>>,

    #[serde(default)]
    pub group_by: CostGroupBy,
}

/// Cost grouping options
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CostGroupBy {
    #[default]
    None,
    Provider,
    Model,
    Day,
    Week,
    Month,
}

/// Cost summary response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_cost: Decimal,
    pub currency: Currency,
    pub total_tokens: u64,
    pub total_requests: u64,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<CostBreakdown>>,
}

/// Cost breakdown by dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub dimension: String,
    pub value: String,
    pub cost: Decimal,
    pub tokens: u64,
    pub requests: u64,
}

// ===== Pricing API Types =====

/// Create pricing request
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CreatePricingRequest {
    pub provider: Provider,

    #[validate(length(min = 1, max = 255))]
    pub model_id: String,

    pub input_price_per_1k: Decimal,

    pub output_price_per_1k: Decimal,

    pub currency: Currency,

    pub effective_date: Option<DateTime<Utc>>,
}

/// Pricing response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingResponse {
    pub id: String,
    pub provider: Provider,
    pub model_id: String,
    pub input_price_per_1k: Decimal,
    pub output_price_per_1k: Decimal,
    pub currency: Currency,
    pub effective_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ===== Analytics API Types =====

/// Analytics query
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct AnalyticsQuery {
    #[validate(length(min = 1))]
    pub organization_id: Option<String>,

    pub start_date: DateTime<Utc>,

    pub end_date: DateTime<Utc>,

    #[serde(default)]
    pub metrics: Vec<AnalyticsMetric>,

    #[serde(default)]
    pub group_by: Vec<AnalyticsDimension>,

    #[serde(default)]
    pub interval: AnalyticsInterval,
}

/// Analytics metrics
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticsMetric {
    TotalCost,
    TotalTokens,
    TotalRequests,
    AverageCostPerRequest,
    AverageTokensPerRequest,
}

/// Analytics dimensions
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsDimension {
    Provider,
    Model,
    Organization,
}

/// Analytics interval
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnalyticsInterval {
    Hour,
    #[default]
    Day,
    Week,
    Month,
}

/// Analytics response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResponse {
    pub time_series: Vec<TimeSeriesPoint>,
    pub summary: AnalyticsSummary,
}

/// Time series data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesPoint {
    pub timestamp: DateTime<Utc>,
    pub metrics: serde_json::Value,
}

/// Analytics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_cost: Decimal,
    pub total_tokens: u64,
    pub total_requests: u64,
    pub average_cost_per_request: Decimal,
    pub average_tokens_per_request: f64,
}

// ===== Health Check Types =====

/// Health check response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub version: String,
    pub uptime_seconds: u64,
    pub components: Vec<ComponentHealth>,
}

/// Health status
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Component health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_creation() {
        let data = "test data";
        let response = ApiResponse::new(data);

        assert_eq!(response.data, data);
        assert!(response.meta.is_none());
    }

    #[test]
    fn test_submit_usage_validation() {
        let valid_request = SubmitUsageRequest {
            organization_id: "org-123".to_string(),
            provider: Provider::OpenAI,
            model_id: "gpt-4".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
            timestamp: None,
            metadata: None,
        };

        assert!(valid_request.validate().is_ok());

        let invalid_request = SubmitUsageRequest {
            organization_id: "".to_string(), // Invalid: empty
            provider: Provider::OpenAI,
            model_id: "gpt-4".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            total_tokens: 150,
            timestamp: None,
            metadata: None,
        };

        assert!(invalid_request.validate().is_err());
    }

    #[test]
    fn test_health_response_serialization() {
        let health = HealthResponse {
            status: HealthStatus::Healthy,
            version: "1.0.0".to_string(),
            uptime_seconds: 3600,
            components: vec![
                ComponentHealth {
                    name: "database".to_string(),
                    status: HealthStatus::Healthy,
                    message: None,
                },
            ],
        };

        let json = serde_json::to_string(&health).unwrap();
        assert!(json.contains("\"status\":\"healthy\""));
    }
}
