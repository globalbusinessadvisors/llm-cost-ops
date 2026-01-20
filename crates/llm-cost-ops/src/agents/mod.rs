//! LLM-CostOps Agent Infrastructure
//!
//! This module provides the agent infrastructure for the LLM-CostOps platform.
//! Agents are stateless, deterministic components that evaluate cost data and
//! emit advisory signals. They do NOT enforce policies directly.
//!
//! # Agent Types
//!
//! - **Financial Governance**: Budget enforcement, constraint evaluation
//! - **Cost Analysis**: Attribution, cost breakdown analysis
//! - **Forecasting**: Spend projection, trend analysis
//!
//! # Architecture
//!
//! - All agents are deployed as Google Cloud Edge Functions
//! - Agents persist DecisionEvents via ruvector-service (no direct SQL)
//! - Agents emit telemetry compatible with LLM-Observatory
//! - Agents are stateless at runtime

pub mod contracts;
pub mod ruvector;
pub mod budget_enforcement;
pub mod registry;

// Re-export commonly used types
pub use contracts::{
    AgentId, AgentVersion, DecisionType, DecisionEvent,
    AgentClassification, ConstraintType, SignalType,
};

pub use ruvector::{RuvectorClient, RuvectorConfig, RuvectorError};

pub use budget_enforcement::{
    BudgetEnforcementAgent, BudgetEvaluationRequest, BudgetConstraintSignal,
    BudgetEnforcementConfig, BudgetSignalSeverity, BudgetViolationType,
};

pub use registry::{AgentRegistry, AgentMetadata};
