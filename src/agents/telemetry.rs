//! Agent Telemetry Module
//!
//! Telemetry emission for LLM-Observatory integration.
//!
//! # LLM-CostOps Constitution Compliance
//! - All agents MUST emit telemetry compatible with LLM-Observatory
//! - Telemetry is for observability only, not enforcement

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::contracts::{AgentId, AgentVersion, DecisionType};

/// Telemetry event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetryEventType {
    /// Agent execution started
    ExecutionStarted,

    /// Agent execution completed successfully
    ExecutionCompleted,

    /// Agent execution failed
    ExecutionFailed,

    /// Input validation performed
    InputValidation,

    /// Output generated
    OutputGenerated,

    /// Decision event persisted
    DecisionPersisted,

    /// Constraint evaluated
    ConstraintEvaluated,

    /// Model inference performed (for forecasting)
    ModelInference,
}

/// Telemetry event for LLM-Observatory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Unique event ID
    pub id: Uuid,

    /// Event type
    pub event_type: TelemetryEventType,

    /// Agent identifier
    pub agent_id: AgentId,

    /// Agent version
    pub agent_version: AgentVersion,

    /// Decision type being processed
    pub decision_type: DecisionType,

    /// Execution reference (correlation ID)
    pub execution_ref: Option<String>,

    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Duration in milliseconds (if applicable)
    pub duration_ms: Option<u64>,

    /// Success indicator
    pub success: bool,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Additional metrics
    pub metrics: TelemetryMetrics,

    /// Organization context
    pub organization_id: Option<String>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// Telemetry metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetryMetrics {
    /// Input size in bytes
    pub input_size_bytes: Option<u64>,

    /// Output size in bytes
    pub output_size_bytes: Option<u64>,

    /// Number of data points processed
    pub data_points_processed: Option<u64>,

    /// Confidence score (for forecasting)
    pub confidence: Option<f64>,

    /// Model name used (if applicable)
    pub model_name: Option<String>,

    /// Number of constraints evaluated
    pub constraints_evaluated: Option<u32>,

    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,
}

impl TelemetryEvent {
    /// Create a new telemetry event
    pub fn new(
        event_type: TelemetryEventType,
        agent_id: AgentId,
        agent_version: AgentVersion,
        decision_type: DecisionType,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            agent_id,
            agent_version,
            decision_type,
            execution_ref: None,
            timestamp: Utc::now(),
            duration_ms: None,
            success: true,
            error: None,
            metrics: TelemetryMetrics::default(),
            organization_id: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Set execution reference
    pub fn with_execution_ref(mut self, exec_ref: impl Into<String>) -> Self {
        self.execution_ref = Some(exec_ref.into());
        self
    }

    /// Set duration
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Mark as failed
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error = Some(error.into());
        self
    }

    /// Set metrics
    pub fn with_metrics(mut self, metrics: TelemetryMetrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Set organization
    pub fn with_organization(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Agent telemetry collector
#[derive(Debug, Clone)]
pub struct AgentTelemetry {
    agent_id: AgentId,
    agent_version: AgentVersion,
    decision_type: DecisionType,
    execution_ref: Option<String>,
    start_time: std::time::Instant,
    events: Vec<TelemetryEvent>,
}

impl AgentTelemetry {
    /// Create a new telemetry collector for an agent execution
    pub fn new(
        agent_id: AgentId,
        agent_version: AgentVersion,
        decision_type: DecisionType,
    ) -> Self {
        Self {
            agent_id,
            agent_version,
            decision_type,
            execution_ref: None,
            start_time: std::time::Instant::now(),
            events: Vec::new(),
        }
    }

    /// Set execution reference
    pub fn with_execution_ref(mut self, exec_ref: impl Into<String>) -> Self {
        self.execution_ref = Some(exec_ref.into());
        self
    }

    /// Record execution started
    pub fn record_start(&mut self) {
        let event = TelemetryEvent::new(
            TelemetryEventType::ExecutionStarted,
            self.agent_id.clone(),
            self.agent_version.clone(),
            self.decision_type,
        );
        let event = if let Some(ref exec_ref) = self.execution_ref {
            event.with_execution_ref(exec_ref.clone())
        } else {
            event
        };
        self.events.push(event);
    }

    /// Record execution completed
    pub fn record_completion(&mut self, metrics: TelemetryMetrics) {
        let duration_ms = self.start_time.elapsed().as_millis() as u64;
        let event = TelemetryEvent::new(
            TelemetryEventType::ExecutionCompleted,
            self.agent_id.clone(),
            self.agent_version.clone(),
            self.decision_type,
        )
        .with_duration(duration_ms)
        .with_metrics(metrics);

        let event = if let Some(ref exec_ref) = self.execution_ref {
            event.with_execution_ref(exec_ref.clone())
        } else {
            event
        };
        self.events.push(event);
    }

    /// Record execution failure
    pub fn record_failure(&mut self, error: impl Into<String>) {
        let duration_ms = self.start_time.elapsed().as_millis() as u64;
        let event = TelemetryEvent::new(
            TelemetryEventType::ExecutionFailed,
            self.agent_id.clone(),
            self.agent_version.clone(),
            self.decision_type,
        )
        .with_duration(duration_ms)
        .with_error(error);

        let event = if let Some(ref exec_ref) = self.execution_ref {
            event.with_execution_ref(exec_ref.clone())
        } else {
            event
        };
        self.events.push(event);
    }

    /// Record a custom event
    pub fn record_event(&mut self, event_type: TelemetryEventType) {
        let event = TelemetryEvent::new(
            event_type,
            self.agent_id.clone(),
            self.agent_version.clone(),
            self.decision_type,
        );
        let event = if let Some(ref exec_ref) = self.execution_ref {
            event.with_execution_ref(exec_ref.clone())
        } else {
            event
        };
        self.events.push(event);
    }

    /// Get elapsed time since start
    pub fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    /// Get all recorded events
    pub fn events(&self) -> &[TelemetryEvent] {
        &self.events
    }

    /// Consume and return all events
    pub fn into_events(self) -> Vec<TelemetryEvent> {
        self.events
    }
}

/// Telemetry emitter for LLM-Observatory
#[derive(Debug, Clone)]
pub struct TelemetryEmitter {
    /// Observatory endpoint
    endpoint: String,

    /// Whether telemetry is enabled
    enabled: bool,
}

impl TelemetryEmitter {
    /// Create a new telemetry emitter
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
            enabled: true,
        }
    }

    /// Create from environment
    pub fn from_env() -> Self {
        let endpoint = std::env::var("OBSERVATORY_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:9090".to_string());
        let enabled = std::env::var("TELEMETRY_ENABLED")
            .map(|v| v != "false" && v != "0")
            .unwrap_or(true);

        Self { endpoint, enabled }
    }

    /// Disable telemetry
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Enable telemetry
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Emit a single telemetry event
    pub async fn emit(&self, event: &TelemetryEvent) -> Result<(), TelemetryError> {
        if !self.enabled {
            return Ok(());
        }

        // Log the event using tracing
        tracing::debug!(
            event_type = ?event.event_type,
            agent_id = %event.agent_id,
            success = event.success,
            duration_ms = ?event.duration_ms,
            "Emitting telemetry event to Observatory"
        );

        // In production, this would send to LLM-Observatory
        // For now, we just log it
        self.simulate_emit(event).await
    }

    /// Emit multiple telemetry events
    pub async fn emit_batch(&self, events: &[TelemetryEvent]) -> Result<(), TelemetryError> {
        if !self.enabled {
            return Ok(());
        }

        for event in events {
            self.emit(event).await?;
        }

        Ok(())
    }

    /// Simulate emission for development
    async fn simulate_emit(&self, _event: &TelemetryEvent) -> Result<(), TelemetryError> {
        // Simulate network latency
        tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        Ok(())
    }

    /// Get the endpoint
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Check if telemetry is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

impl Default for TelemetryEmitter {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Telemetry error types
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    #[error("Failed to emit telemetry: {0}")]
    EmissionFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_event_creation() {
        let event = TelemetryEvent::new(
            TelemetryEventType::ExecutionStarted,
            AgentId::new("test-agent"),
            AgentVersion::new("1.0.0"),
            DecisionType::CostForecast,
        );

        assert!(event.success);
        assert!(event.error.is_none());
        assert_eq!(event.event_type, TelemetryEventType::ExecutionStarted);
    }

    #[test]
    fn test_telemetry_event_with_error() {
        let event = TelemetryEvent::new(
            TelemetryEventType::ExecutionFailed,
            AgentId::new("test-agent"),
            AgentVersion::new("1.0.0"),
            DecisionType::CostForecast,
        )
        .with_error("Something went wrong");

        assert!(!event.success);
        assert_eq!(event.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_agent_telemetry() {
        let mut telemetry = AgentTelemetry::new(
            AgentId::new("test-agent"),
            AgentVersion::new("1.0.0"),
            DecisionType::CostForecast,
        );

        telemetry.record_start();
        assert_eq!(telemetry.events().len(), 1);

        telemetry.record_completion(TelemetryMetrics::default());
        assert_eq!(telemetry.events().len(), 2);
    }

    #[test]
    fn test_telemetry_emitter() {
        let emitter = TelemetryEmitter::new("http://test:9090");
        assert!(emitter.is_enabled());
        assert_eq!(emitter.endpoint(), "http://test:9090");
    }

    #[tokio::test]
    async fn test_emit_telemetry() {
        let emitter = TelemetryEmitter::new("http://test:9090");
        let event = TelemetryEvent::new(
            TelemetryEventType::ExecutionStarted,
            AgentId::new("test-agent"),
            AgentVersion::new("1.0.0"),
            DecisionType::CostForecast,
        );

        let result = emitter.emit(&event).await;
        assert!(result.is_ok());
    }
}
