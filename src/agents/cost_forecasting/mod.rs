//! Cost Forecasting Agent
//!
//! # Agent Classification: FORECASTING
//!
//! Forecasts future LLM spend based on historical usage patterns and growth trends.
//!
//! ## Purpose
//! - Analyze historical cost data
//! - Model future spend projections
//! - Emit forecast ranges and risk indicators
//!
//! ## Decision Type: `cost_forecast`
//!
//! ## LLM-CostOps Constitution Compliance
//! - ✅ Imports schemas from agentics-contracts (contracts module)
//! - ✅ Validates all inputs/outputs against contracts
//! - ✅ Emits telemetry compatible with LLM-Observatory
//! - ✅ Emits exactly ONE DecisionEvent per invocation
//! - ✅ Deployable as Google Edge Function
//! - ✅ Returns deterministic, machine-readable output
//!
//! ## Non-Responsibilities (MUST NOT)
//! - ❌ Intercept runtime execution
//! - ❌ Trigger retries
//! - ❌ Execute workflows
//! - ❌ Modify routing or execution behavior
//! - ❌ Apply optimizations automatically
//! - ❌ Enforce policies directly (only emit advisories)

mod types;
mod agent;

pub use types::{
    CostForecastInput, CostForecastOutput, ForecastProjection,
    RiskIndicator, RiskLevel, ForecastGranularity, GrowthPattern,
    HistoricalDataPoint, ForecastConstraints, ForecastMetadata,
};
pub use agent::CostForecastingAgent;

/// Agent identifier constant
pub const AGENT_ID: &str = "cost-forecasting-agent";

/// Agent version (semver)
pub const AGENT_VERSION: &str = "1.0.0";

/// Minimum data points required for forecasting
pub const MIN_DATA_POINTS: usize = 7;

/// Maximum forecast horizon in days
pub const MAX_FORECAST_DAYS: u64 = 365;

/// Default confidence level (95%)
pub const DEFAULT_CONFIDENCE_LEVEL: f64 = 0.95;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert!(!AGENT_ID.is_empty());
        assert!(!AGENT_VERSION.is_empty());
        assert!(MIN_DATA_POINTS > 0);
        assert!(MAX_FORECAST_DAYS > 0);
        assert!(DEFAULT_CONFIDENCE_LEVEL > 0.0 && DEFAULT_CONFIDENCE_LEVEL <= 1.0);
    }
}
