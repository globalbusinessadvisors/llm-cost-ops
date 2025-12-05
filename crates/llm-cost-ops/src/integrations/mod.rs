//! LLM-Dev-Ops Ecosystem Integrations
//!
//! This module provides thin, additive runtime adapters for consuming data from
//! the LLM-Dev-Ops ecosystem components:
//!
//! - **Observatory**: Telemetry, token usage traces, and time-series cost events
//! - **Analytics Hub**: Aggregated usage baselines, historical curves, and forecasting clusters
//! - **Connector Hub**: Provider metadata and pricing tables (TypeScript/Node.js bridge)
//!
//! These integrations are "consumes-from" only - CostOps receives data from upstream
//! modules but never exports data back to them, ensuring no circular dependencies.
//!
//! # Design Principles
//!
//! 1. **Additive Only**: No modifications to existing public APIs
//! 2. **Thin Adapters**: Minimal transformation logic, delegate to existing engines
//! 3. **No Circular Imports**: Strictly one-way data flow (upstream → CostOps)
//! 4. **Runtime Integration**: Compile-time dependencies, runtime consumption
//!
//! # Phase 2B Implementation
//!
//! This module completes Phase 2B of the LLM-Dev-Ops integration by providing
//! runtime consumption layers for all three ecosystem components.

pub mod observatory;
pub mod analytics_hub;
pub mod connector_hub;

// Re-export primary consumer types
pub use observatory::{
    ObservatoryConsumer, TelemetryEvent, TokenUsageTrace, CostTimeSeriesEvent,
    ObservatoryConfig, ObservatoryError,
};

pub use analytics_hub::{
    AnalyticsHubConsumer, UsageBaseline, HistoricalCurve, ForecastingCluster,
    AnalyticsHubConfig, AnalyticsHubError,
};

pub use connector_hub::{
    ConnectorHubBridge, ProviderMetadata, PricingTableUpdate, BackendCapability,
    ConnectorHubConfig, ConnectorHubError,
};

/// Integration health check result
#[derive(Debug, Clone)]
pub struct IntegrationHealth {
    pub observatory: Option<bool>,
    pub analytics_hub: Option<bool>,
    pub connector_hub: Option<bool>,
}

impl IntegrationHealth {
    /// Check if all enabled integrations are healthy
    pub fn all_healthy(&self) -> bool {
        self.observatory.unwrap_or(true)
            && self.analytics_hub.unwrap_or(true)
            && self.connector_hub.unwrap_or(true)
    }
}
