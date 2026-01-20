//! DecisionEvent Contract
//!
//! Defines the DecisionEvent schema that MUST be emitted by every agent invocation.
//! This is persisted to ruvector-service for audit and governance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{AgentId, AgentVersion, InputsHash, OutputsHash};

/// Decision types for LLM-CostOps agents
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    /// Cost attribution decision (COST_ANALYSIS agents)
    Attribution,

    /// Cost forecast projection (FORECASTING agents)
    CostForecast,

    /// Budget signal/advisory (FINANCIAL_GOVERNANCE agents)
    BudgetSignal,

    /// ROI analysis result (FINANCIAL_GOVERNANCE agents)
    RoiAnalysis,

    /// Budget threshold evaluation
    BudgetEvaluation,

    /// Cost efficiency analysis
    EfficiencyAnalysis,
}

impl std::fmt::Display for DecisionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Attribution => write!(f, "attribution"),
            Self::CostForecast => write!(f, "cost_forecast"),
            Self::BudgetSignal => write!(f, "budget_signal"),
            Self::RoiAnalysis => write!(f, "roi_analysis"),
            Self::BudgetEvaluation => write!(f, "budget_evaluation"),
            Self::EfficiencyAnalysis => write!(f, "efficiency_analysis"),
        }
    }
}

/// Constraint types that can be applied during agent execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    /// Budget cap constraint
    BudgetCap,

    /// ROI threshold constraint
    RoiThreshold,

    /// Cost cap per period
    CostCap,

    /// Growth rate limit
    GrowthLimit,

    /// Minimum confidence requirement
    MinConfidence,

    /// Maximum forecast horizon
    MaxHorizon,

    /// Data quality threshold
    DataQuality,
}

/// A constraint that was applied during agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintApplied {
    /// Type of constraint
    pub constraint_type: ConstraintType,

    /// Constraint name/identifier
    pub name: String,

    /// Constraint value (serialized)
    pub value: serde_json::Value,

    /// Whether the constraint was satisfied
    pub satisfied: bool,

    /// Optional description of constraint impact
    pub impact: Option<String>,
}

impl ConstraintApplied {
    /// Create a new budget cap constraint
    pub fn budget_cap(name: impl Into<String>, value: f64, satisfied: bool) -> Self {
        Self {
            constraint_type: ConstraintType::BudgetCap,
            name: name.into(),
            value: serde_json::json!(value),
            satisfied,
            impact: None,
        }
    }

    /// Create a new ROI threshold constraint
    pub fn roi_threshold(name: impl Into<String>, value: f64, satisfied: bool) -> Self {
        Self {
            constraint_type: ConstraintType::RoiThreshold,
            name: name.into(),
            value: serde_json::json!(value),
            satisfied,
            impact: None,
        }
    }

    /// Create a new cost cap constraint
    pub fn cost_cap(name: impl Into<String>, value: f64, satisfied: bool) -> Self {
        Self {
            constraint_type: ConstraintType::CostCap,
            name: name.into(),
            value: serde_json::json!(value),
            satisfied,
            impact: None,
        }
    }

    /// Add impact description
    pub fn with_impact(mut self, impact: impl Into<String>) -> Self {
        self.impact = Some(impact.into());
        self
    }
}

/// DecisionEvent emitted by every agent invocation
///
/// This schema is persisted to ruvector-service and MUST include all required fields.
///
/// # Required Fields (per Constitution)
/// - agent_id: Unique agent identifier
/// - agent_version: Semantic version of the agent
/// - decision_type: Type of decision made
/// - inputs_hash: SHA-256 hash of inputs for deduplication
/// - outputs: Serialized output data
/// - confidence: Estimation certainty (0.0-1.0)
/// - constraints_applied: Budget, ROI, or cost caps applied
/// - execution_ref: Reference to execution context
/// - timestamp: UTC timestamp of decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEvent {
    /// Unique event identifier
    pub id: Uuid,

    /// Agent identifier (e.g., "cost-forecasting-agent")
    pub agent_id: AgentId,

    /// Agent semantic version (e.g., "1.0.0")
    pub agent_version: AgentVersion,

    /// Type of decision made
    pub decision_type: DecisionType,

    /// SHA-256 hash of serialized inputs for deduplication/audit
    pub inputs_hash: InputsHash,

    /// Serialized output data
    pub outputs: serde_json::Value,

    /// Estimation certainty (0.0 to 1.0)
    /// For forecasts: statistical confidence
    /// For analysis: data quality score
    pub confidence: f64,

    /// Constraints that were applied during execution
    pub constraints_applied: Vec<ConstraintApplied>,

    /// Optional reference to execution context (workflow ID, request ID, etc.)
    pub execution_ref: Option<String>,

    /// UTC timestamp of decision
    pub timestamp: DateTime<Utc>,

    /// Organization context
    pub organization_id: Option<String>,

    /// Project context
    pub project_id: Option<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl DecisionEvent {
    /// Create a new DecisionEvent
    pub fn new(
        agent_id: AgentId,
        agent_version: AgentVersion,
        decision_type: DecisionType,
        inputs_hash: InputsHash,
        outputs: serde_json::Value,
        confidence: f64,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            agent_version,
            decision_type,
            inputs_hash,
            outputs,
            confidence: confidence.clamp(0.0, 1.0),
            constraints_applied: Vec::new(),
            execution_ref: None,
            timestamp: Utc::now(),
            organization_id: None,
            project_id: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Add constraints to the event
    pub fn with_constraints(mut self, constraints: Vec<ConstraintApplied>) -> Self {
        self.constraints_applied = constraints;
        self
    }

    /// Add execution reference
    pub fn with_execution_ref(mut self, exec_ref: impl Into<String>) -> Self {
        self.execution_ref = Some(exec_ref.into());
        self
    }

    /// Add organization context
    pub fn with_organization(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    /// Add project context
    pub fn with_project(mut self, project_id: impl Into<String>) -> Self {
        self.project_id = Some(project_id.into());
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Validate the DecisionEvent
    pub fn validate(&self) -> Result<(), super::ValidationError> {
        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err(super::ValidationError::InvalidField {
                field: "confidence".to_string(),
                reason: "must be between 0.0 and 1.0".to_string(),
            });
        }

        if self.agent_id.0.is_empty() {
            return Err(super::ValidationError::RequiredField("agent_id".to_string()));
        }

        if self.agent_version.0.is_empty() {
            return Err(super::ValidationError::RequiredField("agent_version".to_string()));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_event_creation() {
        let event = DecisionEvent::new(
            AgentId::new("test-agent"),
            AgentVersion::new("1.0.0"),
            DecisionType::CostForecast,
            InputsHash::compute(b"test input"),
            serde_json::json!({"forecast": 100}),
            0.95,
        );

        assert!(!event.id.is_nil());
        assert_eq!(event.confidence, 0.95);
        assert_eq!(event.decision_type, DecisionType::CostForecast);
    }

    #[test]
    fn test_decision_event_validation() {
        let valid_event = DecisionEvent::new(
            AgentId::new("test-agent"),
            AgentVersion::new("1.0.0"),
            DecisionType::CostForecast,
            InputsHash::compute(b"test"),
            serde_json::json!({}),
            0.5,
        );
        assert!(valid_event.validate().is_ok());

        let invalid_event = DecisionEvent::new(
            AgentId::new(""),
            AgentVersion::new("1.0.0"),
            DecisionType::CostForecast,
            InputsHash::compute(b"test"),
            serde_json::json!({}),
            0.5,
        );
        assert!(invalid_event.validate().is_err());
    }

    #[test]
    fn test_confidence_clamping() {
        let event = DecisionEvent::new(
            AgentId::new("test"),
            AgentVersion::new("1.0.0"),
            DecisionType::CostForecast,
            InputsHash::compute(b"test"),
            serde_json::json!({}),
            1.5, // Over 1.0
        );
        assert_eq!(event.confidence, 1.0);

        let event2 = DecisionEvent::new(
            AgentId::new("test"),
            AgentVersion::new("1.0.0"),
            DecisionType::CostForecast,
            InputsHash::compute(b"test"),
            serde_json::json!({}),
            -0.5, // Under 0.0
        );
        assert_eq!(event2.confidence, 0.0);
    }

    #[test]
    fn test_constraint_applied() {
        let constraint = ConstraintApplied::budget_cap("monthly_limit", 10000.0, true)
            .with_impact("Within budget");

        assert_eq!(constraint.constraint_type, ConstraintType::BudgetCap);
        assert!(constraint.satisfied);
        assert_eq!(constraint.impact, Some("Within budget".to_string()));
    }

    #[test]
    fn test_decision_type_display() {
        assert_eq!(DecisionType::CostForecast.to_string(), "cost_forecast");
        assert_eq!(DecisionType::Attribution.to_string(), "attribution");
        assert_eq!(DecisionType::BudgetSignal.to_string(), "budget_signal");
    }
}
