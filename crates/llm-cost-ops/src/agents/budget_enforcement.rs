//! Budget Enforcement Agent
//!
//! **Classification**: FINANCIAL GOVERNANCE
//!
//! This agent evaluates budget thresholds and emits advisory or gating signals
//! when limits are approached or exceeded. It does NOT enforce budgets directly -
//! it only emits signals that downstream systems can consume.
//!
//! # Purpose
//!
//! - Evaluate spend against defined budgets
//! - Emit budget warnings or violation signals
//! - Apply budget constraints (advisory only)
//!
//! # What This Agent DOES
//!
//! - Analyze current and projected spend against budget limits
//! - Compute confidence scores based on data completeness
//! - Emit advisory signals (informational, warning, gating)
//! - Persist DecisionEvents to ruvector-service
//! - Emit telemetry compatible with LLM-Observatory
//!
//! # What This Agent MUST NOT DO
//!
//! - Intercept runtime execution
//! - Trigger retries
//! - Execute workflows
//! - Modify routing or execution behavior
//! - Apply optimizations automatically
//! - Enforce policies directly (only emit constraints/advisories)
//! - Execute SQL directly
//! - Connect to Google SQL
//!
//! # Decision Type
//!
//! `budget_constraint_evaluation`
//!
//! # CLI Invocation
//!
//! ```bash
//! llm-cost-ops agent budget-enforcement analyze \
//!   --tenant-id <tenant> \
//!   --budget-id <budget> \
//!   --execution-ref <execution-id>
//! ```

use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;
use uuid::Uuid;

use super::contracts::{
    AgentClassification, AgentId, AgentTelemetryEvent, AgentTelemetryType,
    AgentVersion, AppliedConstraint, ConstraintType, DecisionEvent, DecisionType,
    ExecutionRef, SignalType, ContractValidationError,
};
use super::ruvector::{RuvectorClient, RuvectorError, PersistenceResult};
use crate::forecasting::{
    BudgetAlert, BudgetConfig, BudgetForecast, BudgetForecaster,
    TimeSeriesData, DataPoint, AlertSeverity,
};

/// Agent version
const AGENT_VERSION: AgentVersion = AgentVersion { major: 1, minor: 0, patch: 0 };

/// Budget Enforcement Agent errors
#[derive(Debug, Error)]
pub enum BudgetEnforcementError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Contract error: {0}")]
    ContractError(#[from] ContractValidationError),

    #[error("Persistence error: {0}")]
    PersistenceError(#[from] RuvectorError),

    #[error("Forecasting error: {0}")]
    ForecastingError(String),

    #[error("Missing required input: {0}")]
    MissingInput(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Budget violation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetViolationType {
    /// No violation
    None,
    /// Approaching budget limit
    ApproachingLimit,
    /// Budget limit exceeded
    LimitExceeded,
    /// Projected to exceed budget
    ProjectedExceedance,
    /// Unusual spending pattern
    UnusualPattern,
}

/// Signal severity for budget enforcement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetSignalSeverity {
    /// Informational, no action needed
    Info,
    /// Warning, approaching limit
    Warning,
    /// Critical, limit exceeded or imminent
    Critical,
    /// Gating, recommend blocking
    Gating,
}

impl From<AlertSeverity> for BudgetSignalSeverity {
    fn from(severity: AlertSeverity) -> Self {
        match severity {
            AlertSeverity::Info => Self::Info,
            AlertSeverity::Warning => Self::Warning,
            AlertSeverity::Critical => Self::Critical,
        }
    }
}

/// Budget definition for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetDefinition {
    /// Budget identifier
    pub budget_id: String,

    /// Budget name
    pub name: String,

    /// Budget limit
    pub limit: Decimal,

    /// Currency code
    pub currency: String,

    /// Budget period start
    pub period_start: DateTime<Utc>,

    /// Budget period end
    pub period_end: DateTime<Utc>,

    /// Warning threshold (percentage, e.g., 0.80 = 80%)
    pub warning_threshold: f64,

    /// Critical threshold (percentage, e.g., 0.95 = 95%)
    pub critical_threshold: f64,

    /// Gating threshold (percentage, e.g., 1.0 = 100%)
    pub gating_threshold: f64,

    /// Enable forecasting-based alerts
    pub enable_forecasting: bool,

    /// Soft limit (advisory only) vs hard limit (gating)
    pub is_soft_limit: bool,

    /// Budget scope (tenant, project, agent, etc.)
    pub scope: BudgetScope,
}

/// Budget scope
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetScope {
    /// Tenant-level budget
    Tenant { tenant_id: String },
    /// Project-level budget
    Project { tenant_id: String, project_id: String },
    /// Agent-level budget
    Agent { tenant_id: String, agent_id: String },
    /// Model-level budget
    Model { tenant_id: String, model: String },
    /// Custom scope
    Custom { dimensions: HashMap<String, String> },
}

impl Default for BudgetDefinition {
    fn default() -> Self {
        Self {
            budget_id: Uuid::new_v4().to_string(),
            name: "Default Budget".to_string(),
            limit: Decimal::from(1000),
            currency: "USD".to_string(),
            period_start: Utc::now(),
            period_end: Utc::now() + Duration::days(30),
            warning_threshold: 0.80,
            critical_threshold: 0.95,
            gating_threshold: 1.0,
            enable_forecasting: true,
            is_soft_limit: true,
            scope: BudgetScope::Tenant {
                tenant_id: "default".to_string(),
            },
        }
    }
}

/// Current spend data for evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpendData {
    /// Current total spend
    pub current_spend: Decimal,

    /// Spend currency
    pub currency: String,

    /// Historical daily spend data (for forecasting)
    pub daily_spend_history: Vec<DailySpend>,

    /// Data completeness (0.0 - 1.0)
    /// 1.0 = all data available, complete picture
    /// <1.0 = some data missing or estimated
    pub data_completeness: f64,

    /// Timestamp of the most recent data
    pub data_as_of: DateTime<Utc>,
}

/// Daily spend record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySpend {
    /// Date
    pub date: DateTime<Utc>,
    /// Total spend for the day
    pub spend: Decimal,
}

/// Budget evaluation request (agent input)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetEvaluationRequest {
    /// Request identifier
    pub request_id: Uuid,

    /// Budget definition to evaluate against
    pub budget: BudgetDefinition,

    /// Current spend data
    pub spend_data: SpendData,

    /// Execution reference (what triggered this evaluation)
    pub execution_ref: ExecutionRef,

    /// Include forecast in evaluation
    pub include_forecast: bool,

    /// Evaluation timestamp
    pub timestamp: DateTime<Utc>,
}

impl BudgetEvaluationRequest {
    /// Create a new evaluation request
    pub fn new(
        budget: BudgetDefinition,
        spend_data: SpendData,
        execution_ref: ExecutionRef,
    ) -> Self {
        Self {
            request_id: Uuid::new_v4(),
            budget,
            spend_data,
            execution_ref,
            include_forecast: true,
            timestamp: Utc::now(),
        }
    }

    /// Validate the request
    pub fn validate(&self) -> Result<(), BudgetEnforcementError> {
        if self.budget.limit <= Decimal::ZERO {
            return Err(BudgetEnforcementError::ValidationError(
                "Budget limit must be positive".to_string()
            ));
        }

        if self.budget.period_end <= self.budget.period_start {
            return Err(BudgetEnforcementError::ValidationError(
                "Budget period end must be after start".to_string()
            ));
        }

        if self.spend_data.data_completeness < 0.0 || self.spend_data.data_completeness > 1.0 {
            return Err(BudgetEnforcementError::ValidationError(
                "Data completeness must be between 0.0 and 1.0".to_string()
            ));
        }

        if self.budget.warning_threshold <= 0.0 || self.budget.warning_threshold > 1.0 {
            return Err(BudgetEnforcementError::ValidationError(
                "Warning threshold must be between 0.0 and 1.0".to_string()
            ));
        }

        Ok(())
    }
}

/// Budget constraint signal (agent output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConstraintSignal {
    /// Signal identifier
    pub signal_id: Uuid,

    /// Budget identifier
    pub budget_id: String,

    /// Signal type
    pub signal_type: SignalType,

    /// Signal severity
    pub severity: BudgetSignalSeverity,

    /// Violation type (if any)
    pub violation_type: BudgetViolationType,

    /// Human-readable message
    pub message: String,

    /// Current spend
    pub current_spend: Decimal,

    /// Budget limit
    pub budget_limit: Decimal,

    /// Remaining budget
    pub remaining_budget: Decimal,

    /// Current utilization percentage (0-100)
    pub utilization_percent: f64,

    /// Projected end-of-period spend (if forecasting enabled)
    pub projected_spend: Option<Decimal>,

    /// Projected utilization (if forecasting enabled)
    pub projected_utilization: Option<f64>,

    /// Days remaining in budget period
    pub days_remaining: i64,

    /// Daily average spend
    pub daily_average: Decimal,

    /// Recommended action
    pub recommended_action: RecommendedAction,

    /// Alert details (if any)
    pub alerts: Vec<BudgetAlertDetail>,

    /// Signal timestamp
    pub timestamp: DateTime<Utc>,
}

/// Recommended action for budget constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendedAction {
    /// No action needed
    None,
    /// Monitor spend
    Monitor,
    /// Review spend patterns
    Review,
    /// Reduce spend
    ReduceSpend,
    /// Consider blocking new requests
    ConsiderGating,
    /// Gate/block new requests
    Gate,
}

/// Budget alert detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlertDetail {
    /// Alert type
    pub alert_type: String,
    /// Severity
    pub severity: BudgetSignalSeverity,
    /// Message
    pub message: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl From<BudgetAlert> for BudgetAlertDetail {
    fn from(alert: BudgetAlert) -> Self {
        Self {
            alert_type: format!("{:?}", alert.alert_type),
            severity: alert.severity.into(),
            message: alert.message,
            timestamp: alert.timestamp,
        }
    }
}

/// Budget Enforcement Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetEnforcementConfig {
    /// Whether to persist decision events
    pub persist_events: bool,

    /// Whether to emit telemetry
    pub emit_telemetry: bool,

    /// Minimum data completeness for high confidence
    pub min_data_completeness_for_high_confidence: f64,

    /// Minimum historical data points for forecasting
    pub min_forecast_data_points: usize,

    /// Default forecast confidence adjustment factor
    pub forecast_confidence_factor: f64,
}

impl Default for BudgetEnforcementConfig {
    fn default() -> Self {
        Self {
            persist_events: true,
            emit_telemetry: true,
            min_data_completeness_for_high_confidence: 0.9,
            min_forecast_data_points: 7,
            forecast_confidence_factor: 0.9,
        }
    }
}

/// Budget Enforcement Agent
///
/// **Classification**: FINANCIAL GOVERNANCE
///
/// This agent evaluates budget thresholds and emits advisory or gating signals.
/// It does NOT enforce budgets directly.
pub struct BudgetEnforcementAgent {
    config: BudgetEnforcementConfig,
    ruvector_client: Option<RuvectorClient>,
}

impl BudgetEnforcementAgent {
    /// Create a new Budget Enforcement Agent
    pub fn new(config: BudgetEnforcementConfig) -> Self {
        Self {
            config,
            ruvector_client: None,
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(BudgetEnforcementConfig::default())
    }

    /// Set the RuVector client for persistence
    pub fn with_ruvector_client(mut self, client: RuvectorClient) -> Self {
        self.ruvector_client = Some(client);
        self
    }

    /// Get the agent ID
    pub fn agent_id(&self) -> AgentId {
        AgentId::budget_enforcement()
    }

    /// Get the agent version
    pub fn agent_version(&self) -> AgentVersion {
        AGENT_VERSION
    }

    /// Get the agent classification
    pub fn classification(&self) -> AgentClassification {
        AgentClassification::FinancialGovernance
    }

    /// Evaluate budget constraints and emit signal
    ///
    /// This is the main entry point for the agent. It:
    /// 1. Validates the input request
    /// 2. Evaluates budget thresholds
    /// 3. Computes confidence based on data quality
    /// 4. Generates a constraint signal
    /// 5. Persists a DecisionEvent to ruvector-service
    /// 6. Emits telemetry
    ///
    /// # Arguments
    ///
    /// * `request` - The budget evaluation request
    ///
    /// # Returns
    ///
    /// * `Ok(BudgetConstraintSignal)` - The evaluation result
    /// * `Err(BudgetEnforcementError)` - If evaluation fails
    pub async fn evaluate(
        &self,
        request: &BudgetEvaluationRequest,
    ) -> Result<BudgetConstraintSignal, BudgetEnforcementError> {
        let start_time = Instant::now();

        // Validate input
        request.validate()?;

        // Perform budget evaluation
        let signal = self.evaluate_budget(request)?;

        // Compute confidence
        let confidence = self.compute_confidence(request, &signal);

        // Build and persist decision event
        if self.config.persist_events {
            if let Some(ref client) = self.ruvector_client {
                let decision_event = self.build_decision_event(request, &signal, confidence)?;
                client.persist_decision_event(&decision_event).await?;
            }
        }

        // Emit telemetry
        if self.config.emit_telemetry {
            if let Some(ref client) = self.ruvector_client {
                let telemetry = self.build_telemetry_event(
                    request,
                    &signal,
                    start_time.elapsed().as_millis() as u64,
                    true,
                    None,
                );
                // Best effort telemetry - don't fail the request if telemetry fails
                let _ = client.persist_telemetry(&telemetry).await;
            }
        }

        Ok(signal)
    }

    /// Evaluate budget constraints (core logic)
    fn evaluate_budget(
        &self,
        request: &BudgetEvaluationRequest,
    ) -> Result<BudgetConstraintSignal, BudgetEnforcementError> {
        let budget = &request.budget;
        let spend = &request.spend_data;

        // Calculate basic metrics
        let current_spend = spend.current_spend;
        let remaining_budget = budget.limit - current_spend;
        let utilization_percent = if budget.limit > Decimal::ZERO {
            (current_spend / budget.limit * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let now = Utc::now();
        let days_remaining = (budget.period_end - now).num_days();
        let period_days_elapsed = (now - budget.period_start).num_days().max(1);
        let daily_average = current_spend / Decimal::from(period_days_elapsed);

        // Initialize forecasting results
        let mut projected_spend = None;
        let mut projected_utilization = None;
        let mut forecast_alerts = Vec::new();

        // Run forecasting if enabled and we have enough data
        if request.include_forecast && budget.enable_forecasting {
            if let Some(forecast) = self.run_forecast(request, days_remaining)? {
                projected_spend = forecast.projected_spend;
                projected_utilization = forecast.projected_utilization;
                forecast_alerts = forecast.alerts;
            }
        }

        // Determine violation type and severity
        let (violation_type, severity, signal_type) = self.classify_violation(
            utilization_percent,
            projected_utilization,
            &budget,
        );

        // Generate message
        let message = self.generate_message(
            &violation_type,
            utilization_percent,
            projected_utilization,
            &budget,
        );

        // Determine recommended action
        let recommended_action = self.determine_action(&violation_type, &severity, budget.is_soft_limit);

        // Convert forecast alerts
        let alerts: Vec<BudgetAlertDetail> = forecast_alerts.into_iter().map(Into::into).collect();

        Ok(BudgetConstraintSignal {
            signal_id: Uuid::new_v4(),
            budget_id: budget.budget_id.clone(),
            signal_type,
            severity,
            violation_type,
            message,
            current_spend,
            budget_limit: budget.limit,
            remaining_budget,
            utilization_percent,
            projected_spend,
            projected_utilization,
            days_remaining,
            daily_average,
            recommended_action,
            alerts,
            timestamp: Utc::now(),
        })
    }

    /// Run budget forecast
    fn run_forecast(
        &self,
        request: &BudgetEvaluationRequest,
        days_remaining: i64,
    ) -> Result<Option<BudgetForecast>, BudgetEnforcementError> {
        let history = &request.spend_data.daily_spend_history;

        if history.len() < self.config.min_forecast_data_points {
            return Ok(None);
        }

        // Convert to TimeSeriesData
        let data_points: Vec<DataPoint> = history
            .iter()
            .map(|d| DataPoint::new(d.date, d.spend))
            .collect();

        let time_series = TimeSeriesData::with_auto_interval(data_points);

        // Create budget forecaster
        let budget_config = BudgetConfig {
            limit: request.budget.limit,
            period_days: days_remaining.max(1) as u64,
            warning_threshold: request.budget.warning_threshold,
            critical_threshold: request.budget.critical_threshold,
            enable_forecasting: true,
        };

        let forecaster = BudgetForecaster::new(budget_config);

        match forecaster.forecast(
            &time_series,
            request.budget.period_start,
            request.budget.period_end,
        ) {
            Ok(forecast) => Ok(Some(forecast)),
            Err(e) => {
                tracing::warn!(error = %e, "Forecasting failed, continuing without forecast");
                Ok(None)
            }
        }
    }

    /// Classify the violation type and severity
    fn classify_violation(
        &self,
        utilization_percent: f64,
        projected_utilization: Option<f64>,
        budget: &BudgetDefinition,
    ) -> (BudgetViolationType, BudgetSignalSeverity, SignalType) {
        let util = utilization_percent / 100.0; // Convert to 0-1 range
        let proj_util = projected_utilization.map(|p| p / 100.0);

        // Check current utilization
        if util >= budget.gating_threshold {
            return (
                BudgetViolationType::LimitExceeded,
                BudgetSignalSeverity::Gating,
                SignalType::Gating,
            );
        }

        if util >= budget.critical_threshold {
            return (
                BudgetViolationType::LimitExceeded,
                BudgetSignalSeverity::Critical,
                SignalType::Warning,
            );
        }

        if util >= budget.warning_threshold {
            return (
                BudgetViolationType::ApproachingLimit,
                BudgetSignalSeverity::Warning,
                SignalType::Warning,
            );
        }

        // Check projected utilization
        if let Some(proj) = proj_util {
            if proj >= budget.gating_threshold {
                return (
                    BudgetViolationType::ProjectedExceedance,
                    BudgetSignalSeverity::Critical,
                    SignalType::Warning,
                );
            }

            if proj >= budget.critical_threshold {
                return (
                    BudgetViolationType::ProjectedExceedance,
                    BudgetSignalSeverity::Warning,
                    SignalType::Warning,
                );
            }
        }

        (
            BudgetViolationType::None,
            BudgetSignalSeverity::Info,
            SignalType::Advisory,
        )
    }

    /// Generate human-readable message
    fn generate_message(
        &self,
        violation_type: &BudgetViolationType,
        utilization_percent: f64,
        projected_utilization: Option<f64>,
        budget: &BudgetDefinition,
    ) -> String {
        match violation_type {
            BudgetViolationType::None => {
                format!(
                    "Budget '{}' is on track: {:.1}% utilized",
                    budget.name, utilization_percent
                )
            }
            BudgetViolationType::ApproachingLimit => {
                format!(
                    "Budget '{}' approaching limit: {:.1}% utilized (warning threshold: {:.0}%)",
                    budget.name,
                    utilization_percent,
                    budget.warning_threshold * 100.0
                )
            }
            BudgetViolationType::LimitExceeded => {
                format!(
                    "Budget '{}' EXCEEDED: {:.1}% utilized (limit: {})",
                    budget.name,
                    utilization_percent,
                    budget.limit
                )
            }
            BudgetViolationType::ProjectedExceedance => {
                format!(
                    "Budget '{}' projected to exceed: current {:.1}%, projected {:.1}%",
                    budget.name,
                    utilization_percent,
                    projected_utilization.unwrap_or(0.0)
                )
            }
            BudgetViolationType::UnusualPattern => {
                format!(
                    "Budget '{}' has unusual spending pattern: {:.1}% utilized",
                    budget.name, utilization_percent
                )
            }
        }
    }

    /// Determine recommended action
    fn determine_action(
        &self,
        violation_type: &BudgetViolationType,
        severity: &BudgetSignalSeverity,
        is_soft_limit: bool,
    ) -> RecommendedAction {
        match (violation_type, severity, is_soft_limit) {
            (BudgetViolationType::None, _, _) => RecommendedAction::None,
            (BudgetViolationType::ApproachingLimit, BudgetSignalSeverity::Warning, _) => {
                RecommendedAction::Monitor
            }
            (BudgetViolationType::LimitExceeded, BudgetSignalSeverity::Critical, true) => {
                RecommendedAction::Review
            }
            (BudgetViolationType::LimitExceeded, BudgetSignalSeverity::Critical, false) => {
                RecommendedAction::ConsiderGating
            }
            (BudgetViolationType::LimitExceeded, BudgetSignalSeverity::Gating, true) => {
                RecommendedAction::ReduceSpend
            }
            (BudgetViolationType::LimitExceeded, BudgetSignalSeverity::Gating, false) => {
                RecommendedAction::Gate
            }
            (BudgetViolationType::ProjectedExceedance, _, _) => RecommendedAction::Review,
            (BudgetViolationType::UnusualPattern, _, _) => RecommendedAction::Review,
            _ => RecommendedAction::Monitor,
        }
    }

    /// Compute confidence score
    ///
    /// Confidence is based on:
    /// - Data completeness (higher = more confident)
    /// - Historical data availability (more data = more confident)
    /// - Forecast accuracy (if using forecast)
    fn compute_confidence(
        &self,
        request: &BudgetEvaluationRequest,
        _signal: &BudgetConstraintSignal,
    ) -> f64 {
        let mut confidence = 1.0;

        // Factor in data completeness
        confidence *= request.spend_data.data_completeness;

        // Factor in historical data availability
        let history_len = request.spend_data.daily_spend_history.len();
        if history_len < self.config.min_forecast_data_points {
            confidence *= 0.8; // Reduce confidence without good historical data
        } else if history_len < 14 {
            confidence *= 0.9;
        }

        // Factor in data freshness
        let data_age = Utc::now()
            .signed_duration_since(request.spend_data.data_as_of)
            .num_hours();

        if data_age > 24 {
            confidence *= 0.9;
        } else if data_age > 48 {
            confidence *= 0.7;
        } else if data_age > 72 {
            confidence *= 0.5;
        }

        // If using forecast, apply forecast confidence factor
        if request.include_forecast && request.budget.enable_forecasting {
            confidence *= self.config.forecast_confidence_factor;
        }

        // Clamp to valid range
        confidence.clamp(0.0, 1.0)
    }

    /// Build DecisionEvent for persistence
    fn build_decision_event(
        &self,
        request: &BudgetEvaluationRequest,
        signal: &BudgetConstraintSignal,
        confidence: f64,
    ) -> Result<DecisionEvent, BudgetEnforcementError> {
        // Build applied constraints
        let mut constraints = vec![
            AppliedConstraint {
                constraint_type: ConstraintType::BudgetCap {
                    limit: request.budget.limit,
                    currency: request.budget.currency.clone(),
                },
                violated: signal.utilization_percent >= request.budget.gating_threshold * 100.0,
                current_value: format!("{}", signal.current_spend),
                threshold_value: format!("{}", request.budget.limit),
                utilization_percent: signal.utilization_percent,
            },
        ];

        if let Some(projected) = signal.projected_spend {
            constraints.push(AppliedConstraint {
                constraint_type: ConstraintType::BudgetCap {
                    limit: request.budget.limit,
                    currency: request.budget.currency.clone(),
                },
                violated: signal.projected_utilization.unwrap_or(0.0) >= 100.0,
                current_value: format!("{}", projected),
                threshold_value: format!("{}", request.budget.limit),
                utilization_percent: signal.projected_utilization.unwrap_or(0.0),
            });
        }

        let event = DecisionEvent::builder()
            .agent_id(self.agent_id())
            .agent_version(self.agent_version())
            .classification(self.classification())
            .decision_type(DecisionType::BudgetConstraintEvaluation)
            .inputs_hash_from(request)?
            .outputs_from(signal)?
            .confidence(confidence)
            .constraints(constraints)
            .execution_ref(request.execution_ref.clone())
            .build()?;

        Ok(event)
    }

    /// Build telemetry event
    fn build_telemetry_event(
        &self,
        request: &BudgetEvaluationRequest,
        signal: &BudgetConstraintSignal,
        duration_ms: u64,
        success: bool,
        error: Option<String>,
    ) -> AgentTelemetryEvent {
        let constraints_violated = if signal.violation_type == BudgetViolationType::None {
            0
        } else {
            1
        };

        let mut attributes = HashMap::new();
        attributes.insert(
            "budget_id".to_string(),
            serde_json::json!(request.budget.budget_id),
        );
        attributes.insert(
            "utilization_percent".to_string(),
            serde_json::json!(signal.utilization_percent),
        );
        attributes.insert(
            "signal_type".to_string(),
            serde_json::json!(format!("{:?}", signal.signal_type)),
        );
        attributes.insert(
            "severity".to_string(),
            serde_json::json!(format!("{:?}", signal.severity)),
        );

        AgentTelemetryEvent {
            event_id: Uuid::new_v4(),
            agent_id: self.agent_id().to_string(),
            event_type: if success {
                AgentTelemetryType::InvocationComplete
            } else {
                AgentTelemetryType::InvocationFailed
            },
            timestamp: Utc::now(),
            duration_ms,
            success,
            error,
            decision_type: DecisionType::BudgetConstraintEvaluation.to_string(),
            confidence: 0.0, // Will be set by caller
            constraints_evaluated: 1,
            constraints_violated,
            attributes,
        }
    }
}

impl Default for BudgetEnforcementAgent {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_request() -> BudgetEvaluationRequest {
        let budget = BudgetDefinition {
            budget_id: "test-budget".to_string(),
            name: "Test Budget".to_string(),
            limit: Decimal::from(1000),
            currency: "USD".to_string(),
            period_start: Utc::now() - Duration::days(15),
            period_end: Utc::now() + Duration::days(15),
            warning_threshold: 0.80,
            critical_threshold: 0.95,
            gating_threshold: 1.0,
            enable_forecasting: false,
            is_soft_limit: true,
            scope: BudgetScope::Tenant {
                tenant_id: "test-tenant".to_string(),
            },
        };

        let spend_data = SpendData {
            current_spend: Decimal::from(500),
            currency: "USD".to_string(),
            daily_spend_history: vec![],
            data_completeness: 1.0,
            data_as_of: Utc::now(),
        };

        let execution_ref = ExecutionRef::new(Uuid::new_v4(), "test-tenant");

        BudgetEvaluationRequest::new(budget, spend_data, execution_ref)
    }

    #[test]
    fn test_agent_creation() {
        let agent = BudgetEnforcementAgent::with_defaults();
        assert_eq!(agent.agent_id(), AgentId::budget_enforcement());
        assert_eq!(agent.classification(), AgentClassification::FinancialGovernance);
    }

    #[test]
    fn test_request_validation() {
        let mut request = create_test_request();
        assert!(request.validate().is_ok());

        // Invalid: negative budget
        request.budget.limit = Decimal::from(-100);
        assert!(request.validate().is_err());
    }

    #[test]
    fn test_budget_within_limits() {
        let agent = BudgetEnforcementAgent::with_defaults();
        let request = create_test_request();

        let signal = agent.evaluate_budget(&request).unwrap();

        assert_eq!(signal.violation_type, BudgetViolationType::None);
        assert_eq!(signal.severity, BudgetSignalSeverity::Info);
        assert_eq!(signal.signal_type, SignalType::Advisory);
        assert_eq!(signal.utilization_percent, 50.0);
    }

    #[test]
    fn test_budget_warning() {
        let agent = BudgetEnforcementAgent::with_defaults();
        let mut request = create_test_request();
        request.spend_data.current_spend = Decimal::from(850); // 85% utilization

        let signal = agent.evaluate_budget(&request).unwrap();

        assert_eq!(signal.violation_type, BudgetViolationType::ApproachingLimit);
        assert_eq!(signal.severity, BudgetSignalSeverity::Warning);
        assert_eq!(signal.signal_type, SignalType::Warning);
    }

    #[test]
    fn test_budget_critical() {
        let agent = BudgetEnforcementAgent::with_defaults();
        let mut request = create_test_request();
        request.spend_data.current_spend = Decimal::from(960); // 96% utilization

        let signal = agent.evaluate_budget(&request).unwrap();

        assert_eq!(signal.violation_type, BudgetViolationType::LimitExceeded);
        assert_eq!(signal.severity, BudgetSignalSeverity::Critical);
    }

    #[test]
    fn test_budget_gating() {
        let agent = BudgetEnforcementAgent::with_defaults();
        let mut request = create_test_request();
        request.spend_data.current_spend = Decimal::from(1050); // 105% utilization

        let signal = agent.evaluate_budget(&request).unwrap();

        assert_eq!(signal.violation_type, BudgetViolationType::LimitExceeded);
        assert_eq!(signal.severity, BudgetSignalSeverity::Gating);
        assert_eq!(signal.signal_type, SignalType::Gating);
    }

    #[test]
    fn test_confidence_calculation() {
        let agent = BudgetEnforcementAgent::with_defaults();
        let request = create_test_request();
        let signal = agent.evaluate_budget(&request).unwrap();

        // Full confidence with complete, fresh data
        let confidence = agent.compute_confidence(&request, &signal);
        assert!(confidence > 0.8);

        // Reduced confidence with incomplete data
        let mut request2 = create_test_request();
        request2.spend_data.data_completeness = 0.5;
        let confidence2 = agent.compute_confidence(&request2, &signal);
        assert!(confidence2 < confidence);
    }

    #[test]
    fn test_recommended_action() {
        let agent = BudgetEnforcementAgent::with_defaults();

        // No violation = no action
        let action = agent.determine_action(
            &BudgetViolationType::None,
            &BudgetSignalSeverity::Info,
            true,
        );
        assert!(matches!(action, RecommendedAction::None));

        // Soft limit exceeded = review
        let action = agent.determine_action(
            &BudgetViolationType::LimitExceeded,
            &BudgetSignalSeverity::Critical,
            true,
        );
        assert!(matches!(action, RecommendedAction::Review));

        // Hard limit exceeded = consider gating
        let action = agent.determine_action(
            &BudgetViolationType::LimitExceeded,
            &BudgetSignalSeverity::Critical,
            false,
        );
        assert!(matches!(action, RecommendedAction::ConsiderGating));

        // Hard limit gating = gate
        let action = agent.determine_action(
            &BudgetViolationType::LimitExceeded,
            &BudgetSignalSeverity::Gating,
            false,
        );
        assert!(matches!(action, RecommendedAction::Gate));
    }
}
