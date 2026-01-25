//! Signal Emitters for Governance Layer
//!
//! Phase 4 Layer 1 - Functions for emitting governance signals.
//!
//! # CRITICAL: Emit-Only Architecture
//!
//! These functions EMIT signals only. They do NOT:
//! - Enforce policies
//! - Block operations
//! - Auto-approve anything
//!
//! The consuming system decides what action to take based on signals.

use std::sync::Arc;
use tracing::{info, warn, instrument};

use crate::agents::{RuVectorClient, RuVectorError, AgentId, AgentVersion};
use super::types::{
    GovernanceDecisionEvent, CostRiskSignal, BudgetThresholdSignal,
    PolicyViolationSignal, ApprovalRequiredSignal, RiskLevel,
    ViolationType, ApprovalType,
};
use rust_decimal::Decimal;

/// Signal emitter for cost-related signals
pub struct CostSignalEmitter {
    client: Arc<RuVectorClient>,
    agent_id: AgentId,
    agent_version: AgentVersion,
}

impl CostSignalEmitter {
    pub fn new(client: Arc<RuVectorClient>, agent_id: AgentId, agent_version: AgentVersion) -> Self {
        Self { client, agent_id, agent_version }
    }

    /// Emit a cost risk signal
    #[instrument(skip(self), fields(agent_id = %self.agent_id.0))]
    pub async fn emit_cost_risk(
        &self,
        risk_level: RiskLevel,
        anomaly_type: &str,
        current_cost: Decimal,
        expected_cost: Decimal,
        affected_resources: Vec<String>,
        recommendation: &str,
    ) -> Result<(), RuVectorError> {
        let signal = CostRiskSignal::new(
            self.agent_id.clone(),
            self.agent_version.clone(),
            risk_level,
            anomaly_type,
            current_cost,
            expected_cost,
        )
        .with_resources(affected_resources)
        .with_recommendation(recommendation);

        let event = GovernanceDecisionEvent::cost_risk(signal);

        info!(
            event_id = %event.id,
            decision_type = %event.decision_type,
            risk_level = risk_level.as_str(),
            "Emitting cost risk signal"
        );

        self.client.persist_governance_event(&event).await
    }

    /// Emit a budget threshold signal
    #[instrument(skip(self), fields(agent_id = %self.agent_id.0))]
    pub async fn emit_budget_threshold(
        &self,
        budget_id: &str,
        threshold_percent: u8,
        current_spend: Decimal,
        budget_limit: Decimal,
        recommendation: &str,
    ) -> Result<(), RuVectorError> {
        let signal = BudgetThresholdSignal::new(
            self.agent_id.clone(),
            self.agent_version.clone(),
            budget_id,
            threshold_percent,
            current_spend,
            budget_limit,
        )
        .with_recommendation(recommendation);

        let event = GovernanceDecisionEvent::budget_threshold(signal);

        info!(
            event_id = %event.id,
            decision_type = %event.decision_type,
            threshold = threshold_percent,
            "Emitting budget threshold signal"
        );

        self.client.persist_governance_event(&event).await
    }
}

/// Signal emitter for policy-related signals
pub struct PolicySignalEmitter {
    client: Arc<RuVectorClient>,
    agent_id: AgentId,
    agent_version: AgentVersion,
}

impl PolicySignalEmitter {
    pub fn new(client: Arc<RuVectorClient>, agent_id: AgentId, agent_version: AgentVersion) -> Self {
        Self { client, agent_id, agent_version }
    }

    /// Emit a policy violation signal
    ///
    /// # IMPORTANT
    /// This function ONLY emits a signal. It does NOT enforce the policy.
    /// The consuming system must decide whether to block, warn, or allow.
    #[instrument(skip(self), fields(agent_id = %self.agent_id.0))]
    pub async fn emit_violation(
        &self,
        policy_id: &str,
        policy_name: &str,
        violation_type: ViolationType,
        severity: RiskLevel,
        description: &str,
        actual_value: serde_json::Value,
        expected_value: serde_json::Value,
        affected_entity: &str,
        remediation: &str,
    ) -> Result<(), RuVectorError> {
        let signal = PolicyViolationSignal::new(
            self.agent_id.clone(),
            self.agent_version.clone(),
            policy_id,
            policy_name,
            violation_type,
            severity,
            description,
        )
        .with_values(actual_value, expected_value)
        .with_affected_entity(affected_entity)
        .with_remediation(remediation);

        let event = GovernanceDecisionEvent::policy_violation(signal);

        warn!(
            event_id = %event.id,
            decision_type = %event.decision_type,
            policy_id = policy_id,
            severity = severity.as_str(),
            "Emitting policy violation signal (NO ENFORCEMENT)"
        );

        self.client.persist_governance_event(&event).await
    }
}

/// Signal emitter for approval-required signals
pub struct ApprovalSignalEmitter {
    client: Arc<RuVectorClient>,
    agent_id: AgentId,
    agent_version: AgentVersion,
}

impl ApprovalSignalEmitter {
    pub fn new(client: Arc<RuVectorClient>, agent_id: AgentId, agent_version: AgentVersion) -> Self {
        Self { client, agent_id, agent_version }
    }

    /// Emit an approval required signal
    ///
    /// # IMPORTANT
    /// This function ONLY emits a signal. It does NOT block the operation.
    /// The consuming system must route to human approval workflow.
    /// Agents MUST NOT auto-approve.
    #[instrument(skip(self), fields(agent_id = %self.agent_id.0))]
    pub async fn emit_approval_required(
        &self,
        approval_type: ApprovalType,
        description: &str,
        reason: &str,
        requestor: &str,
        requested_action: serde_json::Value,
        risk_level: RiskLevel,
        suggested_approvers: Vec<String>,
    ) -> Result<(), RuVectorError> {
        let signal = ApprovalRequiredSignal::new(
            self.agent_id.clone(),
            self.agent_version.clone(),
            approval_type.clone(),
            description,
            reason,
            requestor,
        )
        .with_action(requested_action)
        .with_risk_level(risk_level)
        .with_approvers(suggested_approvers);

        let event = GovernanceDecisionEvent::approval_required(signal);

        info!(
            event_id = %event.id,
            decision_type = %event.decision_type,
            approval_type = ?approval_type,
            risk_level = risk_level.as_str(),
            "Emitting approval required signal (NO AUTO-APPROVE)"
        );

        self.client.persist_governance_event(&event).await
    }
}

// Standalone emission functions for convenience

/// Emit a cost risk signal
///
/// Convenience function for emitting cost risk signals without creating an emitter.
#[instrument(skip(client))]
pub async fn emit_cost_risk_signal(
    client: &RuVectorClient,
    agent_id: &AgentId,
    agent_version: &AgentVersion,
    risk_level: RiskLevel,
    anomaly_type: &str,
    current_cost: Decimal,
    expected_cost: Decimal,
) -> Result<(), RuVectorError> {
    let signal = CostRiskSignal::new(
        agent_id.clone(),
        agent_version.clone(),
        risk_level,
        anomaly_type,
        current_cost,
        expected_cost,
    );

    let event = GovernanceDecisionEvent::cost_risk(signal);

    info!(
        event_id = %event.id,
        decision_type = "cost_risk_signal",
        "Emitting cost risk signal"
    );

    client.persist_governance_event(&event).await
}

/// Emit a budget threshold signal
#[instrument(skip(client))]
pub async fn emit_budget_threshold_signal(
    client: &RuVectorClient,
    agent_id: &AgentId,
    agent_version: &AgentVersion,
    budget_id: &str,
    threshold_percent: u8,
    current_spend: Decimal,
    budget_limit: Decimal,
) -> Result<(), RuVectorError> {
    let signal = BudgetThresholdSignal::new(
        agent_id.clone(),
        agent_version.clone(),
        budget_id,
        threshold_percent,
        current_spend,
        budget_limit,
    );

    let event = GovernanceDecisionEvent::budget_threshold(signal);

    info!(
        event_id = %event.id,
        decision_type = "budget_threshold_signal",
        "Emitting budget threshold signal"
    );

    client.persist_governance_event(&event).await
}

/// Emit a policy violation signal
///
/// # IMPORTANT: NO ENFORCEMENT
/// This function emits a signal ONLY. The agent MUST NOT enforce the policy.
#[instrument(skip(client))]
pub async fn emit_policy_violation_signal(
    client: &RuVectorClient,
    agent_id: &AgentId,
    agent_version: &AgentVersion,
    policy_id: &str,
    policy_name: &str,
    violation_type: ViolationType,
    severity: RiskLevel,
    description: &str,
) -> Result<(), RuVectorError> {
    let signal = PolicyViolationSignal::new(
        agent_id.clone(),
        agent_version.clone(),
        policy_id,
        policy_name,
        violation_type,
        severity,
        description,
    );

    let event = GovernanceDecisionEvent::policy_violation(signal);

    warn!(
        event_id = %event.id,
        decision_type = "policy_violation_signal",
        "Emitting policy violation signal (NO ENFORCEMENT)"
    );

    client.persist_governance_event(&event).await
}

/// Emit an approval required signal
///
/// # IMPORTANT: NO AUTO-APPROVE
/// This function emits a signal ONLY. The agent MUST NOT auto-approve.
#[instrument(skip(client))]
pub async fn emit_approval_required_signal(
    client: &RuVectorClient,
    agent_id: &AgentId,
    agent_version: &AgentVersion,
    approval_type: ApprovalType,
    description: &str,
    reason: &str,
    requestor: &str,
) -> Result<(), RuVectorError> {
    let signal = ApprovalRequiredSignal::new(
        agent_id.clone(),
        agent_version.clone(),
        approval_type,
        description,
        reason,
        requestor,
    );

    let event = GovernanceDecisionEvent::approval_required(signal);

    info!(
        event_id = %event.id,
        decision_type = "approval_required_signal",
        "Emitting approval required signal (NO AUTO-APPROVE)"
    );

    client.persist_governance_event(&event).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signal_creation() {
        let signal = CostRiskSignal::new(
            AgentId::new("test-agent"),
            AgentVersion::new("1.0.0"),
            RiskLevel::High,
            "cost_spike",
            Decimal::from(150),
            Decimal::from(100),
        );

        assert_eq!(signal.risk_level, RiskLevel::High);
        assert!(!signal.signal_id.is_nil());
    }
}
