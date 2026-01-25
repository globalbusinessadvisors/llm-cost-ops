//! Governance Decision Event Types
//!
//! Phase 4 Layer 1 - DecisionEvent types for governance signals.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::agents::{AgentId, AgentVersion, InputsHash};

/// Governance-specific decision types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceDecisionType {
    /// Cost risk signal - emitted when cost anomalies detected
    CostRiskSignal,

    /// Budget threshold signal - emitted when budget thresholds approached/exceeded
    BudgetThresholdSignal,

    /// Policy violation signal - emitted when policy rules violated (NO ENFORCEMENT)
    PolicyViolationSignal,

    /// Approval required signal - emitted when human approval needed (NO AUTO-APPROVE)
    ApprovalRequiredSignal,
}

impl std::fmt::Display for GovernanceDecisionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CostRiskSignal => write!(f, "cost_risk_signal"),
            Self::BudgetThresholdSignal => write!(f, "budget_threshold_signal"),
            Self::PolicyViolationSignal => write!(f, "policy_violation_signal"),
            Self::ApprovalRequiredSignal => write!(f, "approval_required_signal"),
        }
    }
}

/// Risk level for governance signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Low risk - informational
    Low,
    /// Medium risk - attention needed
    Medium,
    /// High risk - action recommended
    High,
    /// Critical risk - immediate attention required
    Critical,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Violation types for policy signals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    /// Budget policy violation
    BudgetPolicy,
    /// Usage policy violation
    UsagePolicy,
    /// Rate limit policy violation
    RateLimitPolicy,
    /// Resource allocation policy violation
    ResourcePolicy,
    /// Compliance policy violation
    CompliancePolicy,
    /// Cost optimization policy violation
    CostOptimizationPolicy,
}

/// Approval types for approval signals
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalType {
    /// Budget override approval needed
    BudgetOverride,
    /// Policy exception approval needed
    PolicyException,
    /// High-cost operation approval needed
    HighCostOperation,
    /// Resource allocation approval needed
    ResourceAllocation,
    /// Configuration change approval needed
    ConfigurationChange,
}

/// Cost risk signal payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRiskSignal {
    /// Unique signal ID
    pub signal_id: Uuid,

    /// Agent that emitted the signal
    pub agent_id: AgentId,

    /// Agent version
    pub agent_version: AgentVersion,

    /// Risk level assessment
    pub risk_level: RiskLevel,

    /// Cost anomaly type
    pub anomaly_type: String,

    /// Current cost value
    pub current_cost: Decimal,

    /// Expected cost value
    pub expected_cost: Decimal,

    /// Deviation percentage
    pub deviation_percent: f64,

    /// Time period for the cost
    pub period: String,

    /// Affected resources/models
    pub affected_resources: Vec<String>,

    /// Recommended action (advisory only)
    pub recommended_action: String,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Organization context
    pub organization_id: Option<String>,

    /// Project context
    pub project_id: Option<String>,

    /// UTC timestamp
    pub timestamp: DateTime<Utc>,

    /// Execution reference
    pub execution_ref: Option<String>,

    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl CostRiskSignal {
    pub fn new(
        agent_id: AgentId,
        agent_version: AgentVersion,
        risk_level: RiskLevel,
        anomaly_type: impl Into<String>,
        current_cost: Decimal,
        expected_cost: Decimal,
    ) -> Self {
        let deviation = if expected_cost.is_zero() {
            100.0
        } else {
            ((current_cost - expected_cost) / expected_cost * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        };

        Self {
            signal_id: Uuid::new_v4(),
            agent_id,
            agent_version,
            risk_level,
            anomaly_type: anomaly_type.into(),
            current_cost,
            expected_cost,
            deviation_percent: deviation,
            period: "daily".to_string(),
            affected_resources: Vec::new(),
            recommended_action: String::new(),
            confidence: 0.8,
            organization_id: None,
            project_id: None,
            timestamp: Utc::now(),
            execution_ref: None,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_period(mut self, period: impl Into<String>) -> Self {
        self.period = period.into();
        self
    }

    pub fn with_resources(mut self, resources: Vec<String>) -> Self {
        self.affected_resources = resources;
        self
    }

    pub fn with_recommendation(mut self, action: impl Into<String>) -> Self {
        self.recommended_action = action.into();
        self
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    pub fn with_organization(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn with_execution_ref(mut self, exec_ref: impl Into<String>) -> Self {
        self.execution_ref = Some(exec_ref.into());
        self
    }
}

/// Budget threshold signal payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetThresholdSignal {
    /// Unique signal ID
    pub signal_id: Uuid,

    /// Agent that emitted the signal
    pub agent_id: AgentId,

    /// Agent version
    pub agent_version: AgentVersion,

    /// Budget name/identifier
    pub budget_id: String,

    /// Threshold percentage (e.g., 80, 90, 100)
    pub threshold_percent: u8,

    /// Current spend
    pub current_spend: Decimal,

    /// Budget limit
    pub budget_limit: Decimal,

    /// Utilization percentage
    pub utilization_percent: f64,

    /// Projected overage (if any)
    pub projected_overage: Option<Decimal>,

    /// Days until budget exhaustion (if at current rate)
    pub days_until_exhaustion: Option<u32>,

    /// Risk level
    pub risk_level: RiskLevel,

    /// Recommended action (advisory only)
    pub recommended_action: String,

    /// Organization context
    pub organization_id: Option<String>,

    /// Project context
    pub project_id: Option<String>,

    /// UTC timestamp
    pub timestamp: DateTime<Utc>,

    /// Execution reference
    pub execution_ref: Option<String>,

    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl BudgetThresholdSignal {
    pub fn new(
        agent_id: AgentId,
        agent_version: AgentVersion,
        budget_id: impl Into<String>,
        threshold_percent: u8,
        current_spend: Decimal,
        budget_limit: Decimal,
    ) -> Self {
        let utilization = if budget_limit.is_zero() {
            100.0
        } else {
            (current_spend / budget_limit * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        };

        let risk_level = match threshold_percent {
            0..=50 => RiskLevel::Low,
            51..=75 => RiskLevel::Medium,
            76..=90 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        Self {
            signal_id: Uuid::new_v4(),
            agent_id,
            agent_version,
            budget_id: budget_id.into(),
            threshold_percent,
            current_spend,
            budget_limit,
            utilization_percent: utilization,
            projected_overage: None,
            days_until_exhaustion: None,
            risk_level,
            recommended_action: String::new(),
            organization_id: None,
            project_id: None,
            timestamp: Utc::now(),
            execution_ref: None,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_projected_overage(mut self, overage: Decimal) -> Self {
        self.projected_overage = Some(overage);
        self
    }

    pub fn with_days_until_exhaustion(mut self, days: u32) -> Self {
        self.days_until_exhaustion = Some(days);
        self
    }

    pub fn with_recommendation(mut self, action: impl Into<String>) -> Self {
        self.recommended_action = action.into();
        self
    }

    pub fn with_organization(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn with_execution_ref(mut self, exec_ref: impl Into<String>) -> Self {
        self.execution_ref = Some(exec_ref.into());
        self
    }
}

/// Policy violation signal payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolationSignal {
    /// Unique signal ID
    pub signal_id: Uuid,

    /// Agent that emitted the signal
    pub agent_id: AgentId,

    /// Agent version
    pub agent_version: AgentVersion,

    /// Policy rule that was violated
    pub policy_id: String,

    /// Policy rule name
    pub policy_name: String,

    /// Violation type
    pub violation_type: ViolationType,

    /// Severity level
    pub severity: RiskLevel,

    /// Description of the violation
    pub description: String,

    /// Actual value that violated the policy
    pub actual_value: serde_json::Value,

    /// Expected/allowed value per policy
    pub expected_value: serde_json::Value,

    /// Affected entity (user, project, model, etc.)
    pub affected_entity: String,

    /// Recommended remediation (advisory only - NO ENFORCEMENT)
    pub recommended_remediation: String,

    /// Whether this violation is blocking (for UI purposes only)
    /// NOTE: Agents MUST NOT enforce this - it's informational
    pub is_blocking: bool,

    /// Organization context
    pub organization_id: Option<String>,

    /// Project context
    pub project_id: Option<String>,

    /// UTC timestamp
    pub timestamp: DateTime<Utc>,

    /// Execution reference
    pub execution_ref: Option<String>,

    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl PolicyViolationSignal {
    pub fn new(
        agent_id: AgentId,
        agent_version: AgentVersion,
        policy_id: impl Into<String>,
        policy_name: impl Into<String>,
        violation_type: ViolationType,
        severity: RiskLevel,
        description: impl Into<String>,
    ) -> Self {
        Self {
            signal_id: Uuid::new_v4(),
            agent_id,
            agent_version,
            policy_id: policy_id.into(),
            policy_name: policy_name.into(),
            violation_type,
            severity,
            description: description.into(),
            actual_value: serde_json::Value::Null,
            expected_value: serde_json::Value::Null,
            affected_entity: String::new(),
            recommended_remediation: String::new(),
            is_blocking: false,
            organization_id: None,
            project_id: None,
            timestamp: Utc::now(),
            execution_ref: None,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_values(
        mut self,
        actual: serde_json::Value,
        expected: serde_json::Value,
    ) -> Self {
        self.actual_value = actual;
        self.expected_value = expected;
        self
    }

    pub fn with_affected_entity(mut self, entity: impl Into<String>) -> Self {
        self.affected_entity = entity.into();
        self
    }

    pub fn with_remediation(mut self, remediation: impl Into<String>) -> Self {
        self.recommended_remediation = remediation.into();
        self
    }

    pub fn with_blocking(mut self, is_blocking: bool) -> Self {
        self.is_blocking = is_blocking;
        self
    }

    pub fn with_organization(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn with_execution_ref(mut self, exec_ref: impl Into<String>) -> Self {
        self.execution_ref = Some(exec_ref.into());
        self
    }
}

/// Approval required signal payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequiredSignal {
    /// Unique signal ID
    pub signal_id: Uuid,

    /// Agent that emitted the signal
    pub agent_id: AgentId,

    /// Agent version
    pub agent_version: AgentVersion,

    /// Approval type required
    pub approval_type: ApprovalType,

    /// Description of what needs approval
    pub description: String,

    /// Reason approval is required
    pub reason: String,

    /// Requestor (user, service, etc.)
    pub requestor: String,

    /// Requested action details
    pub requested_action: serde_json::Value,

    /// Risk level of the action
    pub risk_level: RiskLevel,

    /// Estimated impact (cost, resources, etc.)
    pub estimated_impact: Option<String>,

    /// Suggested approvers (for routing purposes only - NO AUTO-APPROVE)
    pub suggested_approvers: Vec<String>,

    /// Approval deadline (if time-sensitive)
    pub deadline: Option<DateTime<Utc>>,

    /// Organization context
    pub organization_id: Option<String>,

    /// Project context
    pub project_id: Option<String>,

    /// UTC timestamp
    pub timestamp: DateTime<Utc>,

    /// Execution reference
    pub execution_ref: Option<String>,

    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl ApprovalRequiredSignal {
    pub fn new(
        agent_id: AgentId,
        agent_version: AgentVersion,
        approval_type: ApprovalType,
        description: impl Into<String>,
        reason: impl Into<String>,
        requestor: impl Into<String>,
    ) -> Self {
        Self {
            signal_id: Uuid::new_v4(),
            agent_id,
            agent_version,
            approval_type,
            description: description.into(),
            reason: reason.into(),
            requestor: requestor.into(),
            requested_action: serde_json::Value::Null,
            risk_level: RiskLevel::Medium,
            estimated_impact: None,
            suggested_approvers: Vec::new(),
            deadline: None,
            organization_id: None,
            project_id: None,
            timestamp: Utc::now(),
            execution_ref: None,
            metadata: serde_json::Value::Null,
        }
    }

    pub fn with_action(mut self, action: serde_json::Value) -> Self {
        self.requested_action = action;
        self
    }

    pub fn with_risk_level(mut self, level: RiskLevel) -> Self {
        self.risk_level = level;
        self
    }

    pub fn with_impact(mut self, impact: impl Into<String>) -> Self {
        self.estimated_impact = Some(impact.into());
        self
    }

    pub fn with_approvers(mut self, approvers: Vec<String>) -> Self {
        self.suggested_approvers = approvers;
        self
    }

    pub fn with_deadline(mut self, deadline: DateTime<Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    pub fn with_organization(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    pub fn with_execution_ref(mut self, exec_ref: impl Into<String>) -> Self {
        self.execution_ref = Some(exec_ref.into());
        self
    }
}

/// Governance decision event wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceDecisionEvent {
    /// Unique event ID
    pub id: Uuid,

    /// Decision type
    pub decision_type: GovernanceDecisionType,

    /// Inputs hash for deduplication
    pub inputs_hash: InputsHash,

    /// Signal payload (one of the signal types)
    pub signal: GovernanceSignal,

    /// UTC timestamp
    pub timestamp: DateTime<Utc>,

    /// Phase identifier
    pub phase: String,

    /// Layer identifier
    pub layer: String,
}

/// Union type for governance signals
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum GovernanceSignal {
    CostRisk(CostRiskSignal),
    BudgetThreshold(BudgetThresholdSignal),
    PolicyViolation(PolicyViolationSignal),
    ApprovalRequired(ApprovalRequiredSignal),
}

impl GovernanceDecisionEvent {
    pub fn cost_risk(signal: CostRiskSignal) -> Self {
        Self {
            id: Uuid::new_v4(),
            decision_type: GovernanceDecisionType::CostRiskSignal,
            inputs_hash: InputsHash::compute(
                serde_json::to_string(&signal).unwrap_or_default().as_bytes()
            ),
            signal: GovernanceSignal::CostRisk(signal),
            timestamp: Utc::now(),
            phase: super::AGENT_PHASE.to_string(),
            layer: super::AGENT_LAYER.to_string(),
        }
    }

    pub fn budget_threshold(signal: BudgetThresholdSignal) -> Self {
        Self {
            id: Uuid::new_v4(),
            decision_type: GovernanceDecisionType::BudgetThresholdSignal,
            inputs_hash: InputsHash::compute(
                serde_json::to_string(&signal).unwrap_or_default().as_bytes()
            ),
            signal: GovernanceSignal::BudgetThreshold(signal),
            timestamp: Utc::now(),
            phase: super::AGENT_PHASE.to_string(),
            layer: super::AGENT_LAYER.to_string(),
        }
    }

    pub fn policy_violation(signal: PolicyViolationSignal) -> Self {
        Self {
            id: Uuid::new_v4(),
            decision_type: GovernanceDecisionType::PolicyViolationSignal,
            inputs_hash: InputsHash::compute(
                serde_json::to_string(&signal).unwrap_or_default().as_bytes()
            ),
            signal: GovernanceSignal::PolicyViolation(signal),
            timestamp: Utc::now(),
            phase: super::AGENT_PHASE.to_string(),
            layer: super::AGENT_LAYER.to_string(),
        }
    }

    pub fn approval_required(signal: ApprovalRequiredSignal) -> Self {
        Self {
            id: Uuid::new_v4(),
            decision_type: GovernanceDecisionType::ApprovalRequiredSignal,
            inputs_hash: InputsHash::compute(
                serde_json::to_string(&signal).unwrap_or_default().as_bytes()
            ),
            signal: GovernanceSignal::ApprovalRequired(signal),
            timestamp: Utc::now(),
            phase: super::AGENT_PHASE.to_string(),
            layer: super::AGENT_LAYER.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_governance_decision_type_display() {
        assert_eq!(GovernanceDecisionType::CostRiskSignal.to_string(), "cost_risk_signal");
        assert_eq!(GovernanceDecisionType::BudgetThresholdSignal.to_string(), "budget_threshold_signal");
        assert_eq!(GovernanceDecisionType::PolicyViolationSignal.to_string(), "policy_violation_signal");
        assert_eq!(GovernanceDecisionType::ApprovalRequiredSignal.to_string(), "approval_required_signal");
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::Low < RiskLevel::Medium);
        assert!(RiskLevel::Medium < RiskLevel::High);
        assert!(RiskLevel::High < RiskLevel::Critical);
    }

    #[test]
    fn test_cost_risk_signal_creation() {
        let signal = CostRiskSignal::new(
            AgentId::new("test-agent"),
            AgentVersion::new("1.0.0"),
            RiskLevel::High,
            "cost_spike",
            Decimal::from(150),
            Decimal::from(100),
        );

        assert_eq!(signal.risk_level, RiskLevel::High);
        assert_eq!(signal.deviation_percent, 50.0);
    }

    #[test]
    fn test_budget_threshold_signal_creation() {
        let signal = BudgetThresholdSignal::new(
            AgentId::new("test-agent"),
            AgentVersion::new("1.0.0"),
            "monthly-budget",
            85,
            Decimal::from(8500),
            Decimal::from(10000),
        );

        assert_eq!(signal.threshold_percent, 85);
        assert_eq!(signal.risk_level, RiskLevel::High);
    }
}
