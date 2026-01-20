//! Agent Contract Schemas (agentics-contracts)
//!
//! This module defines all schemas used by LLM-CostOps agents. These schemas
//! are the authoritative contract definitions for agent inputs, outputs, and
//! persistence events.
//!
//! # Schema Requirements
//!
//! All agents MUST:
//! - Validate inputs/outputs against these contracts
//! - Emit exactly ONE DecisionEvent per invocation
//! - Include all required fields in DecisionEvent
//!
//! # DecisionEvent Schema
//!
//! Every agent invocation produces a DecisionEvent with:
//! - `agent_id`: Unique agent identifier
//! - `agent_version`: Semantic version (major.minor.patch)
//! - `decision_type`: Type of decision made
//! - `inputs_hash`: SHA-256 hash of inputs for auditability
//! - `outputs`: Structured output data
//! - `confidence`: Estimation certainty (0.0 - 1.0)
//! - `constraints_applied`: Budget, ROI, or cost caps applied
//! - `execution_ref`: Reference to triggering execution
//! - `timestamp`: UTC timestamp

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use uuid::Uuid;

/// Agent identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AgentId(pub String);

impl AgentId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn budget_enforcement() -> Self {
        Self::new("llm-costops.budget-enforcement")
    }

    pub fn cost_attribution() -> Self {
        Self::new("llm-costops.cost-attribution")
    }

    pub fn spend_forecaster() -> Self {
        Self::new("llm-costops.spend-forecaster")
    }

    pub fn roi_analyzer() -> Self {
        Self::new("llm-costops.roi-analyzer")
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Agent version following semantic versioning
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl AgentVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    pub fn v1_0_0() -> Self {
        Self::new(1, 0, 0)
    }
}

impl std::fmt::Display for AgentVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Agent classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentClassification {
    /// Budget enforcement, constraint evaluation
    FinancialGovernance,
    /// Attribution, cost breakdown analysis
    CostAnalysis,
    /// Spend projection, trend analysis
    Forecasting,
}

impl std::fmt::Display for AgentClassification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FinancialGovernance => write!(f, "financial_governance"),
            Self::CostAnalysis => write!(f, "cost_analysis"),
            Self::Forecasting => write!(f, "forecasting"),
        }
    }
}

/// Decision type indicating the kind of analysis performed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionType {
    /// Cost attribution to executions, agents, workflows, or tenants
    Attribution,
    /// Future spend forecast based on historical usage
    Forecast,
    /// Budget threshold evaluation and constraint signal
    BudgetConstraintEvaluation,
    /// ROI and cost-efficiency analysis
    RoiAnalysis,
    /// Cost vs performance tradeoff evaluation
    CostPerformanceTradeoff,
}

impl std::fmt::Display for DecisionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Attribution => write!(f, "attribution"),
            Self::Forecast => write!(f, "forecast"),
            Self::BudgetConstraintEvaluation => write!(f, "budget_constraint_evaluation"),
            Self::RoiAnalysis => write!(f, "roi_analysis"),
            Self::CostPerformanceTradeoff => write!(f, "cost_performance_tradeoff"),
        }
    }
}

/// Signal type for budget enforcement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    /// Informational signal, no action required
    Advisory,
    /// Warning signal, approaching limit
    Warning,
    /// Gating signal, recommend blocking or throttling
    Gating,
}

/// Constraint type applied during evaluation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintType {
    /// Budget limit constraint
    BudgetCap {
        limit: Decimal,
        currency: String,
    },
    /// ROI threshold constraint
    RoiThreshold {
        minimum_roi: f64,
    },
    /// Cost cap per execution
    CostCap {
        max_cost_per_execution: Decimal,
        currency: String,
    },
    /// Token budget constraint
    TokenBudget {
        max_tokens: u64,
    },
    /// Rate limit constraint
    RateLimit {
        max_requests_per_period: u64,
        period_seconds: u64,
    },
}

/// Constraint application result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedConstraint {
    /// Constraint type that was applied
    pub constraint_type: ConstraintType,
    /// Whether the constraint was violated
    pub violated: bool,
    /// Current value being evaluated
    pub current_value: String,
    /// Threshold value
    pub threshold_value: String,
    /// Utilization percentage (0.0 - 100.0)
    pub utilization_percent: f64,
}

/// Execution reference for tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRef {
    /// Unique execution identifier
    pub execution_id: Uuid,
    /// Workflow ID (if part of a workflow)
    pub workflow_id: Option<String>,
    /// Agent ID (if triggered by an agent)
    pub agent_id: Option<String>,
    /// Tenant/organization ID
    pub tenant_id: String,
    /// Project ID
    pub project_id: Option<String>,
    /// Correlation ID for distributed tracing
    pub correlation_id: Option<String>,
}

impl ExecutionRef {
    pub fn new(execution_id: Uuid, tenant_id: impl Into<String>) -> Self {
        Self {
            execution_id,
            workflow_id: None,
            agent_id: None,
            tenant_id: tenant_id.into(),
            project_id: None,
            correlation_id: None,
        }
    }
}

/// Decision event persisted to ruvector-service
///
/// This is the canonical schema for all agent decision outputs.
/// Every agent invocation MUST produce exactly ONE DecisionEvent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEvent {
    /// Unique event identifier
    pub event_id: Uuid,

    /// Agent that produced this event
    pub agent_id: AgentId,

    /// Agent version
    pub agent_version: AgentVersion,

    /// Agent classification
    pub classification: AgentClassification,

    /// Type of decision made
    pub decision_type: DecisionType,

    /// SHA-256 hash of inputs for auditability
    pub inputs_hash: String,

    /// Structured output data (JSON serializable)
    pub outputs: serde_json::Value,

    /// Estimation certainty (0.0 - 1.0)
    ///
    /// For budget enforcement:
    /// - 1.0: Exact current spend known
    /// - 0.8-0.99: Based on recent complete data
    /// - 0.5-0.79: Based on partial or estimated data
    /// - <0.5: Low confidence, may be inaccurate
    pub confidence: f64,

    /// Constraints that were applied during evaluation
    pub constraints_applied: Vec<AppliedConstraint>,

    /// Reference to the triggering execution
    pub execution_ref: ExecutionRef,

    /// Event timestamp (UTC)
    pub timestamp: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl DecisionEvent {
    /// Create a new DecisionEvent builder
    pub fn builder() -> DecisionEventBuilder {
        DecisionEventBuilder::new()
    }

    /// Validate the event meets contract requirements
    pub fn validate(&self) -> Result<(), ContractValidationError> {
        // Validate confidence range
        if self.confidence < 0.0 || self.confidence > 1.0 {
            return Err(ContractValidationError::InvalidConfidence(self.confidence));
        }

        // Validate inputs hash format (SHA-256 = 64 hex chars)
        if self.inputs_hash.len() != 64 || !self.inputs_hash.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ContractValidationError::InvalidInputsHash(
                "inputs_hash must be a valid SHA-256 hex string (64 characters)".to_string()
            ));
        }

        // Validate timestamp is not in the future
        if self.timestamp > Utc::now() {
            return Err(ContractValidationError::FutureTimestamp);
        }

        Ok(())
    }
}

/// Builder for DecisionEvent
pub struct DecisionEventBuilder {
    event_id: Option<Uuid>,
    agent_id: Option<AgentId>,
    agent_version: Option<AgentVersion>,
    classification: Option<AgentClassification>,
    decision_type: Option<DecisionType>,
    inputs_hash: Option<String>,
    outputs: Option<serde_json::Value>,
    confidence: Option<f64>,
    constraints_applied: Vec<AppliedConstraint>,
    execution_ref: Option<ExecutionRef>,
    timestamp: Option<DateTime<Utc>>,
    metadata: HashMap<String, serde_json::Value>,
}

impl DecisionEventBuilder {
    pub fn new() -> Self {
        Self {
            event_id: None,
            agent_id: None,
            agent_version: None,
            classification: None,
            decision_type: None,
            inputs_hash: None,
            outputs: None,
            confidence: None,
            constraints_applied: Vec::new(),
            execution_ref: None,
            timestamp: None,
            metadata: HashMap::new(),
        }
    }

    pub fn event_id(mut self, id: Uuid) -> Self {
        self.event_id = Some(id);
        self
    }

    pub fn agent_id(mut self, id: AgentId) -> Self {
        self.agent_id = Some(id);
        self
    }

    pub fn agent_version(mut self, version: AgentVersion) -> Self {
        self.agent_version = Some(version);
        self
    }

    pub fn classification(mut self, classification: AgentClassification) -> Self {
        self.classification = Some(classification);
        self
    }

    pub fn decision_type(mut self, decision_type: DecisionType) -> Self {
        self.decision_type = Some(decision_type);
        self
    }

    pub fn inputs_hash(mut self, hash: String) -> Self {
        self.inputs_hash = Some(hash);
        self
    }

    /// Compute inputs hash from serializable data
    pub fn inputs_hash_from<T: Serialize>(mut self, inputs: &T) -> Result<Self, serde_json::Error> {
        let json = serde_json::to_string(inputs)?;
        let mut hasher = Sha256::new();
        hasher.update(json.as_bytes());
        let hash = format!("{:x}", hasher.finalize());
        self.inputs_hash = Some(hash);
        Ok(self)
    }

    pub fn outputs(mut self, outputs: serde_json::Value) -> Self {
        self.outputs = Some(outputs);
        self
    }

    /// Set outputs from serializable data
    pub fn outputs_from<T: Serialize>(mut self, outputs: &T) -> Result<Self, serde_json::Error> {
        self.outputs = Some(serde_json::to_value(outputs)?);
        Ok(self)
    }

    pub fn confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    pub fn add_constraint(mut self, constraint: AppliedConstraint) -> Self {
        self.constraints_applied.push(constraint);
        self
    }

    pub fn constraints(mut self, constraints: Vec<AppliedConstraint>) -> Self {
        self.constraints_applied = constraints;
        self
    }

    pub fn execution_ref(mut self, execution_ref: ExecutionRef) -> Self {
        self.execution_ref = Some(execution_ref);
        self
    }

    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn add_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Build the DecisionEvent
    pub fn build(self) -> Result<DecisionEvent, ContractValidationError> {
        let event = DecisionEvent {
            event_id: self.event_id.unwrap_or_else(Uuid::new_v4),
            agent_id: self.agent_id.ok_or(ContractValidationError::MissingField("agent_id"))?,
            agent_version: self.agent_version.ok_or(ContractValidationError::MissingField("agent_version"))?,
            classification: self.classification.ok_or(ContractValidationError::MissingField("classification"))?,
            decision_type: self.decision_type.ok_or(ContractValidationError::MissingField("decision_type"))?,
            inputs_hash: self.inputs_hash.ok_or(ContractValidationError::MissingField("inputs_hash"))?,
            outputs: self.outputs.ok_or(ContractValidationError::MissingField("outputs"))?,
            confidence: self.confidence.ok_or(ContractValidationError::MissingField("confidence"))?,
            constraints_applied: self.constraints_applied,
            execution_ref: self.execution_ref.ok_or(ContractValidationError::MissingField("execution_ref"))?,
            timestamp: self.timestamp.unwrap_or_else(Utc::now),
            metadata: self.metadata,
        };

        event.validate()?;
        Ok(event)
    }
}

impl Default for DecisionEventBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Contract validation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum ContractValidationError {
    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Invalid confidence value: {0} (must be 0.0 - 1.0)")]
    InvalidConfidence(f64),

    #[error("Invalid inputs_hash: {0}")]
    InvalidInputsHash(String),

    #[error("Timestamp cannot be in the future")]
    FutureTimestamp,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl From<serde_json::Error> for ContractValidationError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

/// Telemetry event for LLM-Observatory compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTelemetryEvent {
    /// Event identifier
    pub event_id: Uuid,

    /// Agent identifier
    pub agent_id: String,

    /// Event type
    pub event_type: AgentTelemetryType,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Whether the evaluation succeeded
    pub success: bool,

    /// Error message if failed
    pub error: Option<String>,

    /// Decision type
    pub decision_type: String,

    /// Confidence score
    pub confidence: f64,

    /// Number of constraints evaluated
    pub constraints_evaluated: usize,

    /// Number of constraints violated
    pub constraints_violated: usize,

    /// Additional attributes
    pub attributes: HashMap<String, serde_json::Value>,
}

/// Telemetry event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentTelemetryType {
    /// Agent invocation started
    InvocationStart,
    /// Agent invocation completed
    InvocationComplete,
    /// Agent invocation failed
    InvocationFailed,
    /// Constraint evaluated
    ConstraintEvaluated,
    /// Signal emitted
    SignalEmitted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id_creation() {
        let id = AgentId::budget_enforcement();
        assert_eq!(id.0, "llm-costops.budget-enforcement");
    }

    #[test]
    fn test_agent_version_display() {
        let version = AgentVersion::new(1, 2, 3);
        assert_eq!(version.to_string(), "1.2.3");
    }

    #[test]
    fn test_decision_event_builder() {
        let inputs = serde_json::json!({"budget": 1000, "current_spend": 500});
        let outputs = serde_json::json!({"signal": "advisory", "utilization": 0.5});

        let event = DecisionEvent::builder()
            .agent_id(AgentId::budget_enforcement())
            .agent_version(AgentVersion::v1_0_0())
            .classification(AgentClassification::FinancialGovernance)
            .decision_type(DecisionType::BudgetConstraintEvaluation)
            .inputs_hash_from(&inputs)
            .unwrap()
            .outputs(outputs)
            .confidence(0.95)
            .execution_ref(ExecutionRef::new(Uuid::new_v4(), "tenant-123"))
            .build();

        assert!(event.is_ok());
        let event = event.unwrap();
        assert_eq!(event.agent_id, AgentId::budget_enforcement());
        assert_eq!(event.confidence, 0.95);
    }

    #[test]
    fn test_decision_event_validation_invalid_confidence() {
        let event = DecisionEvent::builder()
            .agent_id(AgentId::budget_enforcement())
            .agent_version(AgentVersion::v1_0_0())
            .classification(AgentClassification::FinancialGovernance)
            .decision_type(DecisionType::BudgetConstraintEvaluation)
            .inputs_hash("a".repeat(64))
            .outputs(serde_json::json!({}))
            .confidence(1.5) // Invalid
            .execution_ref(ExecutionRef::new(Uuid::new_v4(), "tenant-123"))
            .build();

        assert!(event.is_err());
        assert!(matches!(
            event.unwrap_err(),
            ContractValidationError::InvalidConfidence(_)
        ));
    }

    #[test]
    fn test_constraint_types() {
        let budget_cap = ConstraintType::BudgetCap {
            limit: Decimal::from(1000),
            currency: "USD".to_string(),
        };

        let json = serde_json::to_string(&budget_cap).unwrap();
        assert!(json.contains("budget_cap"));
        assert!(json.contains("1000"));
    }
}
