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
}
