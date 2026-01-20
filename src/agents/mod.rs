//! LLM-CostOps Agent Infrastructure
//!
//! This module implements the agent framework for the LLM-CostOps platform.
//! All agents follow the LLM-CostOps Constitution:
//!
//! - Agents execute as Google Cloud Edge Functions
//! - All persistence via ruvector-service (NO direct SQL)
//! - Stateless execution with deterministic behavior
//! - Emit DecisionEvents for every invocation
//! - Analysis-only: NO execution interception or enforcement
//!
//! # Agent Types
//! - COST ANALYSIS: Cost attribution and breakdown
//! - FORECASTING: Cost prediction and projections
//! - FINANCIAL GOVERNANCE: Budget advisories and ROI analysis

pub mod contracts;
pub mod cost_forecasting;
pub mod ruvector_client;
pub mod telemetry;
pub mod edge_function;
pub mod registry;

pub use contracts::{
    AgentId, AgentVersion, DecisionEvent, DecisionType,
    ConstraintApplied, AgentInput, AgentOutput, ValidationError,
};
pub use cost_forecasting::{
    CostForecastingAgent, CostForecastInput, CostForecastOutput,
    ForecastProjection, RiskIndicator, RiskLevel,
};
pub use ruvector_client::{RuVectorClient, RuVectorConfig, RuVectorError};
pub use telemetry::{AgentTelemetry, TelemetryEvent, TelemetryEmitter};
pub use edge_function::{
    create_router, create_app, EdgeFunctionState,
    ForecastRequest, ForecastResponse, HealthResponse, AgentInfoResponse,
};
pub use registry::{AgentRegistry, AgentRegistryEntry, global_registry};

/// Agent classification per LLM-CostOps Constitution
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AgentClassification {
    /// Cost attribution and breakdown analysis
    CostAnalysis,
    /// Cost prediction and projections
    Forecasting,
    /// Budget advisories and ROI analysis
    FinancialGovernance,
}

impl std::fmt::Display for AgentClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CostAnalysis => write!(f, "COST_ANALYSIS"),
            Self::Forecasting => write!(f, "FORECASTING"),
            Self::FinancialGovernance => write!(f, "FINANCIAL_GOVERNANCE"),
        }
    }
}

/// Trait for all LLM-CostOps agents
#[async_trait::async_trait]
pub trait Agent: Send + Sync {
    /// The input type for this agent
    type Input: serde::de::DeserializeOwned + serde::Serialize + Send + Sync;

    /// The output type for this agent
    type Output: serde::de::DeserializeOwned + serde::Serialize + Send + Sync;

    /// Get the agent's unique identifier
    fn agent_id(&self) -> &AgentId;

    /// Get the agent's version
    fn agent_version(&self) -> &AgentVersion;

    /// Get the agent's classification
    fn classification(&self) -> AgentClassification;

    /// Get the decision type this agent produces
    fn decision_type(&self) -> DecisionType;

    /// Validate input before processing
    fn validate_input(&self, input: &Self::Input) -> Result<(), ValidationError>;

    /// Execute the agent's core logic
    /// Returns output and confidence score
    async fn execute(&self, input: Self::Input) -> Result<(Self::Output, f64), AgentError>;

    /// Generate a DecisionEvent from the execution
    fn create_decision_event(
        &self,
        input: &Self::Input,
        output: &Self::Output,
        confidence: f64,
        constraints: Vec<ConstraintApplied>,
        execution_ref: Option<String>,
    ) -> DecisionEvent;
}

/// Agent execution error types
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("Insufficient data for analysis: {0}")]
    InsufficientData(String),

    #[error("Forecast model error: {0}")]
    ModelError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("RuVector service error: {0}")]
    RuVectorError(#[from] RuVectorError),

    #[error("Telemetry emission error: {0}")]
    TelemetryError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<AgentError> for crate::domain::CostOpsError {
    fn from(err: AgentError) -> Self {
        crate::domain::CostOpsError::internal(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_classification_display() {
        assert_eq!(AgentClassification::CostAnalysis.to_string(), "COST_ANALYSIS");
        assert_eq!(AgentClassification::Forecasting.to_string(), "FORECASTING");
        assert_eq!(AgentClassification::FinancialGovernance.to_string(), "FINANCIAL_GOVERNANCE");
    }

    #[test]
    fn test_agent_classification_serialization() {
        let json = serde_json::to_string(&AgentClassification::Forecasting).unwrap();
        assert_eq!(json, "\"FORECASTING\"");

        let parsed: AgentClassification = serde_json::from_str("\"COST_ANALYSIS\"").unwrap();
        assert_eq!(parsed, AgentClassification::CostAnalysis);
    }
}
