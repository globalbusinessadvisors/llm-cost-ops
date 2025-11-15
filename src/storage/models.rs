// Database models (SQLx)

use serde_json::Value as JsonValue;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct UsageRecordRow {
    pub id: String,
    pub timestamp: String,
    pub provider: String,
    pub model_name: String,
    pub model_version: Option<String>,
    pub context_window: i64,
    pub organization_id: String,
    pub project_id: Option<String>,
    pub user_id: Option<String>,
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
    pub cached_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub latency_ms: Option<i64>,
    pub time_to_first_token_ms: Option<i64>,
    pub tags: JsonValue,
    pub metadata: JsonValue,
    pub ingested_at: String,
    pub source_type: String,
    pub source_metadata: JsonValue,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CostRecordRow {
    pub id: String,
    pub usage_id: String,
    pub timestamp: String,
    pub provider: String,
    pub model_name: String,
    pub input_cost: String,
    pub output_cost: String,
    pub total_cost: String,
    pub currency: String,
    pub cost_model_id: String,
    pub pricing_structure: JsonValue,
    pub organization_id: String,
    pub project_id: Option<String>,
    pub tags: JsonValue,
    pub calculated_at: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct PricingTableRow {
    pub id: String,
    pub provider: String,
    pub model_name: String,
    pub effective_date: String,
    pub end_date: Option<String>,
    pub pricing_structure: JsonValue,
    pub currency: String,
    pub region: Option<String>,
    pub metadata: JsonValue,
    pub created_at: String,
    pub updated_at: String,
}
