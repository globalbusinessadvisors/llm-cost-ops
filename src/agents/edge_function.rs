//! Edge Function Handler
//!
//! Google Cloud Edge Function handler for the Cost Forecasting Agent.
//!
//! # Deployment Model (per LLM-CostOps Constitution)
//! - Deployed as Google Cloud Edge Function
//! - Part of the unified CostOps service
//! - Stateless execution
//! - Deterministic behavior

use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{
    cost_forecasting::{CostForecastInput, CostForecastOutput, CostForecastingAgent},
    contracts::ValidationError,
    ruvector_client::RuVectorClient,
    telemetry::TelemetryEmitter,
    AgentError,
};

/// Edge function application state
#[derive(Clone)]
pub struct EdgeFunctionState {
    /// Cost Forecasting Agent instance
    pub cost_forecasting_agent: Arc<CostForecastingAgent>,
}

impl EdgeFunctionState {
    /// Create new state with default configuration
    pub fn new() -> Result<Self, AgentError> {
        let ruvector_client = Arc::new(
            RuVectorClient::with_defaults()
                .map_err(|e| AgentError::ConfigError(e.to_string()))?
        );
        let telemetry_emitter = TelemetryEmitter::from_env();

        let agent = CostForecastingAgent::new(
            ruvector_client,
            telemetry_emitter,
        );

        Ok(Self {
            cost_forecasting_agent: Arc::new(agent),
        })
    }

    /// Create with custom configuration
    pub fn with_config(
        ruvector_client: Arc<RuVectorClient>,
        telemetry_emitter: TelemetryEmitter,
    ) -> Self {
        let agent = CostForecastingAgent::new(
            ruvector_client,
            telemetry_emitter,
        );

        Self {
            cost_forecasting_agent: Arc::new(agent),
        }
    }
}

impl Default for EdgeFunctionState {
    fn default() -> Self {
        Self::new().expect("Failed to create default EdgeFunctionState")
    }
}

/// Edge function error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    /// Error code
    pub code: String,

    /// Error message
    pub message: String,

    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ErrorResponse {
    fn validation_error(err: &ValidationError) -> Self {
        Self {
            code: "VALIDATION_ERROR".to_string(),
            message: err.to_string(),
            details: None,
        }
    }

    fn agent_error(err: &AgentError) -> Self {
        let code = match err {
            AgentError::Validation(_) => "VALIDATION_ERROR",
            AgentError::InsufficientData(_) => "INSUFFICIENT_DATA",
            AgentError::ModelError(_) => "MODEL_ERROR",
            AgentError::ConfigError(_) => "CONFIG_ERROR",
            AgentError::RuVectorError(_) => "PERSISTENCE_ERROR",
            AgentError::TelemetryError(_) => "TELEMETRY_ERROR",
            AgentError::InternalError(_) => "INTERNAL_ERROR",
        };

        Self {
            code: code.to_string(),
            message: err.to_string(),
            details: None,
        }
    }

    fn internal_error(message: impl Into<String>) -> Self {
        Self {
            code: "INTERNAL_ERROR".to_string(),
            message: message.into(),
            details: None,
        }
    }
}

/// Edge function API error
pub struct ApiError {
    status: StatusCode,
    body: ErrorResponse,
}

impl ApiError {
    fn bad_request(err: ErrorResponse) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            body: err,
        }
    }

    fn unprocessable_entity(err: ErrorResponse) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            body: err,
        }
    }

    fn internal_server_error(err: ErrorResponse) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            body: err,
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(self.body)).into_response()
    }
}

impl From<ValidationError> for ApiError {
    fn from(err: ValidationError) -> Self {
        Self::bad_request(ErrorResponse::validation_error(&err))
    }
}

impl From<AgentError> for ApiError {
    fn from(err: AgentError) -> Self {
        match &err {
            AgentError::Validation(_) => Self::bad_request(ErrorResponse::agent_error(&err)),
            AgentError::InsufficientData(_) => Self::unprocessable_entity(ErrorResponse::agent_error(&err)),
            _ => Self::internal_server_error(ErrorResponse::agent_error(&err)),
        }
    }
}

/// Forecast request wrapper
#[derive(Debug, Deserialize)]
pub struct ForecastRequest {
    #[serde(flatten)]
    pub input: CostForecastInput,
}

/// Forecast response wrapper
#[derive(Debug, Serialize)]
pub struct ForecastResponse {
    /// Success indicator
    pub success: bool,

    /// Forecast output
    pub data: CostForecastOutput,

    /// Request metadata
    pub request_id: String,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub agent_id: String,
    pub agent_version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Agent info response
#[derive(Debug, Serialize)]
pub struct AgentInfoResponse {
    pub agent_id: String,
    pub agent_version: String,
    pub classification: String,
    pub decision_type: String,
    pub description: String,
    pub input_schema_version: String,
    pub output_schema_version: String,
}

/// Query parameters for analyze endpoint
#[derive(Debug, Deserialize)]
pub struct AnalyzeParams {
    /// Organization ID filter
    pub organization_id: Option<String>,

    /// Project ID filter
    pub project_id: Option<String>,
}

/// Create the Edge Function router
pub fn create_router() -> Router<EdgeFunctionState> {
    Router::new()
        // Health and info endpoints
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .route("/info", get(info_handler))

        // Agent endpoints
        .route("/forecast", post(forecast_handler))
        .route("/analyze", post(analyze_handler))
        .route("/inspect", get(inspect_handler))

        // CLI-compatible endpoint
        .route("/api/v1/agents/cost-forecasting/forecast", post(forecast_handler))
}

/// Create router with state
pub fn create_app() -> Result<Router, AgentError> {
    let state = EdgeFunctionState::new()?;
    Ok(create_router().with_state(state))
}

/// Health check handler
///
/// GET /health
async fn health_handler(
    State(state): State<EdgeFunctionState>,
) -> Json<HealthResponse> {
    use super::Agent;

    Json(HealthResponse {
        status: "healthy".to_string(),
        agent_id: state.cost_forecasting_agent.agent_id().to_string(),
        agent_version: state.cost_forecasting_agent.agent_version().to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// Readiness check handler
///
/// GET /ready
async fn ready_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "ready": true,
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

/// Agent info handler
///
/// GET /info
async fn info_handler(
    State(state): State<EdgeFunctionState>,
) -> Json<AgentInfoResponse> {
    use super::Agent;

    Json(AgentInfoResponse {
        agent_id: state.cost_forecasting_agent.agent_id().to_string(),
        agent_version: state.cost_forecasting_agent.agent_version().to_string(),
        classification: state.cost_forecasting_agent.classification().to_string(),
        decision_type: state.cost_forecasting_agent.decision_type().to_string(),
        description: "Forecasts future LLM spend based on historical usage patterns and growth trends".to_string(),
        input_schema_version: super::contracts::CONTRACT_VERSION.to_string(),
        output_schema_version: super::contracts::CONTRACT_VERSION.to_string(),
    })
}

/// Forecast handler
///
/// POST /forecast
///
/// Main entry point for cost forecasting. This endpoint:
/// 1. Validates input against contracts
/// 2. Executes the forecasting agent
/// 3. Persists DecisionEvent to ruvector-service
/// 4. Returns deterministic output
async fn forecast_handler(
    State(state): State<EdgeFunctionState>,
    Json(request): Json<ForecastRequest>,
) -> Result<Json<ForecastResponse>, ApiError> {
    use super::contracts::AgentInput;

    // Validate input
    request.input.validate()?;

    // Generate request ID
    let request_id = uuid::Uuid::new_v4().to_string();

    // Execute agent
    let output = state.cost_forecasting_agent
        .run(request.input)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(ForecastResponse {
        success: true,
        data: output,
        request_id,
    }))
}

/// Analyze handler (alias for forecast)
///
/// POST /analyze
///
/// CLI-invokable endpoint for cost analysis.
async fn analyze_handler(
    State(state): State<EdgeFunctionState>,
    Json(request): Json<ForecastRequest>,
) -> Result<Json<ForecastResponse>, ApiError> {
    forecast_handler(State(state), Json(request)).await
}

/// Inspect handler
///
/// GET /inspect
///
/// Returns information about the agent's current state and capabilities.
async fn inspect_handler(
    State(state): State<EdgeFunctionState>,
    Query(_params): Query<AnalyzeParams>,
) -> Json<serde_json::Value> {
    use super::Agent;

    Json(serde_json::json!({
        "agent_id": state.cost_forecasting_agent.agent_id().to_string(),
        "agent_version": state.cost_forecasting_agent.agent_version().to_string(),
        "classification": state.cost_forecasting_agent.classification().to_string(),
        "decision_type": state.cost_forecasting_agent.decision_type().to_string(),
        "capabilities": {
            "forecast_horizons": ["hourly", "daily", "weekly", "monthly"],
            "max_forecast_days": super::cost_forecasting::MAX_FORECAST_DAYS,
            "min_data_points": super::cost_forecasting::MIN_DATA_POINTS,
            "supported_models": ["linear_trend", "moving_average", "exponential_smoothing", "auto"],
            "constraint_types": ["budget_cap", "roi_threshold", "max_cost_per_period", "max_growth_rate", "min_confidence"]
        },
        "compliance": {
            "persists_to": "ruvector-service",
            "emits_telemetry_to": "llm-observatory",
            "stateless": true,
            "deterministic": true
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use rust_decimal_macros::dec;

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = create_app().unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_ready_endpoint() {
        let app = create_app().unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/ready")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_info_endpoint() {
        let app = create_app().unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/info")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_inspect_endpoint() {
        let app = create_app().unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/inspect")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    fn create_test_forecast_request() -> serde_json::Value {
        let now = chrono::Utc::now();
        let historical_data: Vec<serde_json::Value> = (0..14)
            .map(|i| {
                serde_json::json!({
                    "timestamp": (now - chrono::Duration::days(14 - i)).to_rfc3339(),
                    "total_cost": 100 + i * 5,
                    "total_tokens": 1000000,
                    "request_count": 1000
                })
            })
            .collect();

        serde_json::json!({
            "historical_data": historical_data,
            "forecast_horizon_days": 30,
            "confidence_level": 0.95,
            "constraints": {
                "budget_cap": 5000,
                "max_growth_rate": 15.0
            },
            "metadata": {
                "organization_id": "org-123",
                "execution_ref": "test-exec-001"
            }
        })
    }

    #[tokio::test]
    async fn test_forecast_endpoint() {
        let app = create_app().unwrap();
        let request_body = create_test_forecast_request();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/forecast")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_forecast_validation_error() {
        let app = create_app().unwrap();

        // Empty historical data
        let request_body = serde_json::json!({
            "historical_data": [],
            "forecast_horizon_days": 30
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/forecast")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
