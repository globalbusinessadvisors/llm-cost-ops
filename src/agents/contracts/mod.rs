//! Agent Contracts Module
//!
//! This module defines all agent contracts following the agentics-contracts schema format.
//! All inputs, outputs, and events are validated against these contracts.
//!
//! # Contract Requirements (per LLM-CostOps Constitution)
//! - All schemas MUST be imported from this module
//! - All inputs/outputs MUST be validated against contracts
//! - DecisionEvents MUST include all required fields
//! - Versioning rules MUST be followed

mod decision_event;
mod agent_types;
mod validation;

pub use decision_event::{
    DecisionEvent, DecisionType, ConstraintApplied, ConstraintType,
};
pub use agent_types::{
    AgentId, AgentVersion, AgentInput, AgentOutput,
    InputsHash, OutputsHash,
};
pub use validation::{ValidationError, ValidationResult, Validator};

/// Contract version for schema compatibility
pub const CONTRACT_VERSION: &str = "1.0.0";

/// Namespace for cost forecasting contracts
pub const COST_FORECAST_NAMESPACE: &str = "llm-costops.agents.cost-forecasting";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_version() {
        assert!(!CONTRACT_VERSION.is_empty());
    }
}
