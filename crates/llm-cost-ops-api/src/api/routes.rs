// API route definitions

use axum::{
    routing::{get, post},
    Router,
};

use super::handlers;

/// Create API routes
pub fn create_routes() -> Router {
    Router::new()
        // Health and monitoring
        .route("/health", get(handlers::health_check))
        .route("/ready", get(handlers::readiness_check))
        // Service info
        .route("/info", get(handlers::service_info))
        // Usage endpoints
        .route("/api/v1/usage", post(handlers::submit_usage))
        .route("/api/v1/usage/history", get(handlers::get_usage_history))
        // Cost endpoints
        .route("/api/v1/costs", get(handlers::get_costs))
        // Pricing endpoints
        .route("/api/v1/pricing", get(handlers::list_pricing))
        .route("/api/v1/pricing", post(handlers::create_pricing))
        // Analytics endpoints
        .route("/api/v1/analytics", get(handlers::get_analytics))
        // Agent endpoints - Budget Enforcement
        .route("/api/v1/agents/budget-enforcement/info", get(handlers::budget_enforcement_info))
        .route("/api/v1/agents/budget-enforcement/inspect", get(handlers::budget_enforcement_inspect))
        .route("/api/v1/agents/budget-enforcement/analyze", post(handlers::budget_enforcement_analyze))
        .route("/api/v1/agents/budget-enforcement/health", get(handlers::health_check))
        // Agent endpoints - Cost Forecasting
        .route("/api/v1/agents/cost-forecasting/info", get(handlers::cost_forecasting_info))
        .route("/api/v1/agents/cost-forecasting/forecast", post(handlers::cost_forecasting_forecast))
        .route("/api/v1/agents/cost-forecasting/health", get(handlers::health_check))
        // Agent endpoints - ROI Estimation
        .route("/api/v1/agents/roi-estimation/info", get(handlers::roi_estimation_info))
        .route("/api/v1/agents/roi-estimation/health", get(handlers::health_check))
        // Agent endpoints - Cost Performance
        .route("/api/v1/agents/cost-performance/info", get(handlers::cost_performance_info))
        .route("/api/v1/agents/cost-performance/health", get(handlers::health_check))
        // Agent endpoints - Cost Attribution
        .route("/api/v1/agents/cost-attribution/info", get(handlers::cost_attribution_info))
        .route("/api/v1/agents/cost-attribution/health", get(handlers::health_check))
}
