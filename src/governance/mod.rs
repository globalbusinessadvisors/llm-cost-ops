//! Governance & FinOps Layer 1 Module
//!
//! Phase 4 Implementation - Governance layer for LLM-CostOps.
//!
//! # Governance Rules (MANDATORY)
//!
//! Agents MUST:
//! - Emit cost signals via `emit_cost_risk_signal()`
//! - Emit policy evaluation signals via `emit_policy_violation_signal()`
//! - Emit approval requirements via `emit_approval_required_signal()`
//!
//! Agents MUST NOT:
//! - Auto-enforce policy (analysis only)
//! - Auto-approve actions (emit requirement, human decides)
//!
//! # Performance Budgets
//! - MAX_TOKENS: 1200
//! - MAX_LATENCY_MS: 2500

pub mod signals;
pub mod policy;
pub mod performance;
pub mod types;

pub use signals::{
    CostSignalEmitter, PolicySignalEmitter, ApprovalSignalEmitter,
    emit_cost_risk_signal, emit_budget_threshold_signal,
    emit_policy_violation_signal, emit_approval_required_signal,
};
pub use policy::{PolicyEvaluator, PolicyRule, PolicyResult, PolicySeverity};
pub use performance::{PerformanceBudget, PerformanceGuard, BudgetExceeded};
pub use types::{
    GovernanceDecisionEvent, GovernanceDecisionType,
    CostRiskSignal, BudgetThresholdSignal, PolicyViolationSignal, ApprovalRequiredSignal,
    RiskLevel as GovernanceRiskLevel, ApprovalType, ViolationType,
};

/// Phase 4 Layer 1 Constants
pub const AGENT_PHASE: &str = "phase4";
pub const AGENT_LAYER: &str = "layer1";
pub const MAX_TOKENS: usize = 1200;
pub const MAX_LATENCY_MS: u64 = 2500;

/// Governance configuration
#[derive(Debug, Clone)]
pub struct GovernanceConfig {
    pub phase: String,
    pub layer: String,
    pub max_tokens: usize,
    pub max_latency_ms: u64,
    pub ruvector_endpoint: String,
    pub enable_policy_signals: bool,
    pub enable_cost_signals: bool,
    pub enable_approval_signals: bool,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            phase: AGENT_PHASE.to_string(),
            layer: AGENT_LAYER.to_string(),
            max_tokens: MAX_TOKENS,
            max_latency_ms: MAX_LATENCY_MS,
            ruvector_endpoint: std::env::var("RUVECTOR_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8081".to_string()),
            enable_policy_signals: true,
            enable_cost_signals: true,
            enable_approval_signals: true,
        }
    }
}

impl GovernanceConfig {
    /// Create from environment variables
    pub fn from_env() -> Self {
        Self {
            phase: std::env::var("AGENT_PHASE").unwrap_or_else(|_| AGENT_PHASE.to_string()),
            layer: std::env::var("AGENT_LAYER").unwrap_or_else(|_| AGENT_LAYER.to_string()),
            max_tokens: std::env::var("MAX_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(MAX_TOKENS),
            max_latency_ms: std::env::var("MAX_LATENCY_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(MAX_LATENCY_MS),
            ruvector_endpoint: std::env::var("RUVECTOR_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8081".to_string()),
            enable_policy_signals: std::env::var("ENABLE_POLICY_SIGNALS")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            enable_cost_signals: std::env::var("ENABLE_COST_SIGNALS")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
            enable_approval_signals: std::env::var("ENABLE_APPROVAL_SIGNALS")
                .map(|v| v != "false" && v != "0")
                .unwrap_or(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_constants() {
        assert_eq!(AGENT_PHASE, "phase4");
        assert_eq!(AGENT_LAYER, "layer1");
        assert_eq!(MAX_TOKENS, 1200);
        assert_eq!(MAX_LATENCY_MS, 2500);
    }

    #[test]
    fn test_governance_config_default() {
        let config = GovernanceConfig::default();
        assert_eq!(config.phase, "phase4");
        assert_eq!(config.layer, "layer1");
        assert_eq!(config.max_tokens, 1200);
        assert_eq!(config.max_latency_ms, 2500);
        assert!(config.enable_policy_signals);
        assert!(config.enable_cost_signals);
        assert!(config.enable_approval_signals);
    }
}
