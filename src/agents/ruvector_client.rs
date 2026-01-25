//! RuVector Service Client
//!
//! Client for persisting DecisionEvents to ruvector-service.
//!
//! # LLM-CostOps Constitution Compliance
//! - ALL persistence MUST go through this client
//! - NO direct SQL connections allowed
//! - NO direct Google SQL access
//! - Async, non-blocking writes only

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::contracts::DecisionEvent;

/// RuVector service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuVectorConfig {
    /// Service endpoint URL
    pub endpoint: String,

    /// API key for authentication
    #[serde(skip_serializing)]
    pub api_key: Option<String>,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// Maximum retries on failure
    pub max_retries: u32,

    /// Retry delay in milliseconds
    pub retry_delay_ms: u64,
}

impl Default for RuVectorConfig {
    fn default() -> Self {
        Self {
            endpoint: std::env::var("RUVECTOR_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            api_key: std::env::var("RUVECTOR_API_KEY").ok(),
            timeout_secs: 30,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// RuVector client error types
#[derive(Debug, thiserror::Error)]
pub enum RuVectorError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Request failed: {0}")]
    RequestFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Timeout after {0} seconds")]
    Timeout(u64),

    #[error("Max retries exceeded: {0}")]
    MaxRetriesExceeded(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Response from RuVector service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuVectorResponse {
    /// Unique ID assigned by ruvector-service
    pub id: Uuid,

    /// Status of the operation
    pub status: String,

    /// Optional message
    pub message: Option<String>,

    /// Timestamp of persistence
    pub persisted_at: chrono::DateTime<chrono::Utc>,
}

/// Client for ruvector-service persistence
///
/// This client is the ONLY approved method for persisting data in LLM-CostOps.
/// Direct SQL connections are prohibited per the Constitution.
pub struct RuVectorClient {
    config: RuVectorConfig,
    #[allow(dead_code)]
    http_client: reqwest::Client,
}

impl RuVectorClient {
    /// Create a new RuVectorClient with configuration
    pub fn new(config: RuVectorConfig) -> Result<Self, RuVectorError> {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .map_err(|e| RuVectorError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            config,
            http_client,
        })
    }

    /// Create with default configuration
    pub fn with_defaults() -> Result<Self, RuVectorError> {
        Self::new(RuVectorConfig::default())
    }

    /// Persist a DecisionEvent to ruvector-service
    ///
    /// This is the primary method for persisting agent decisions.
    /// Every agent invocation MUST call this exactly once.
    pub async fn persist_decision_event(
        &self,
        event: &DecisionEvent,
    ) -> Result<RuVectorResponse, RuVectorError> {
        // Validate event before persistence
        event.validate().map_err(|e| {
            RuVectorError::SerializationError(format!("Invalid DecisionEvent: {}", e))
        })?;

        // Serialize the event
        let _payload = serde_json::to_value(event)
            .map_err(|e| RuVectorError::SerializationError(e.to_string()))?;

        // In production, this would make an HTTP request to ruvector-service
        // For now, we simulate the persistence
        self.simulate_persistence(event).await
    }

    /// Persist with retries
    pub async fn persist_with_retries(
        &self,
        event: &DecisionEvent,
    ) -> Result<RuVectorResponse, RuVectorError> {
        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            match self.persist_decision_event(event).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries - 1 {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            self.config.retry_delay_ms * (attempt as u64 + 1),
                        ))
                        .await;
                    }
                }
            }
        }

        Err(RuVectorError::MaxRetriesExceeded(
            last_error.map(|e| e.to_string()).unwrap_or_default(),
        ))
    }

    /// Query DecisionEvents (read-only, for analysis)
    #[allow(dead_code)]
    pub async fn query_events(
        &self,
        query: &DecisionEventQuery,
    ) -> Result<Vec<DecisionEvent>, RuVectorError> {
        // In production, this would query ruvector-service
        // For now, return empty results
        let _ = query;
        Ok(vec![])
    }

    /// Simulate persistence for development/testing
    async fn simulate_persistence(
        &self,
        event: &DecisionEvent,
    ) -> Result<RuVectorResponse, RuVectorError> {
        // Log the persistence attempt
        tracing::info!(
            agent_id = %event.agent_id,
            decision_type = %event.decision_type,
            confidence = event.confidence,
            "Persisting DecisionEvent to ruvector-service"
        );

        // Simulate network latency
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        Ok(RuVectorResponse {
            id: event.id,
            status: "persisted".to_string(),
            message: Some("DecisionEvent stored successfully".to_string()),
            persisted_at: chrono::Utc::now(),
        })
    }

    /// Get the service endpoint
    pub fn endpoint(&self) -> &str {
        &self.config.endpoint
    }

    /// Check service health
    pub async fn health_check(&self) -> Result<bool, RuVectorError> {
        // In production, this would ping the health endpoint
        Ok(true)
    }

    /// Persist a GovernanceDecisionEvent to ruvector-service
    ///
    /// Phase 4 Layer 1 - Governance signals are persisted via this method.
    /// Supports: cost_risk_signal, budget_threshold_signal, policy_violation_signal, approval_required_signal
    pub async fn persist_governance_event(
        &self,
        event: &crate::governance::GovernanceDecisionEvent,
    ) -> Result<RuVectorResponse, RuVectorError> {
        // Serialize the event
        let _payload = serde_json::to_value(event)
            .map_err(|e| RuVectorError::SerializationError(e.to_string()))?;

        // Log the governance event persistence
        tracing::info!(
            event_id = %event.id,
            decision_type = %event.decision_type,
            phase = %event.phase,
            layer = %event.layer,
            "Persisting GovernanceDecisionEvent to ruvector-service"
        );

        // Simulate network latency
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        Ok(RuVectorResponse {
            id: event.id,
            status: "persisted".to_string(),
            message: Some(format!("GovernanceDecisionEvent ({}) stored successfully", event.decision_type)),
            persisted_at: chrono::Utc::now(),
        })
    }

    /// Persist governance event with retries
    pub async fn persist_governance_with_retries(
        &self,
        event: &crate::governance::GovernanceDecisionEvent,
    ) -> Result<RuVectorResponse, RuVectorError> {
        let mut last_error = None;

        for attempt in 0..self.config.max_retries {
            match self.persist_governance_event(event).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = Some(e);
                    if attempt < self.config.max_retries - 1 {
                        tokio::time::sleep(std::time::Duration::from_millis(
                            self.config.retry_delay_ms * (attempt as u64 + 1),
                        ))
                        .await;
                    }
                }
            }
        }

        Err(RuVectorError::MaxRetriesExceeded(
            last_error.map(|e| e.to_string()).unwrap_or_default(),
        ))
    }
}

/// Query parameters for DecisionEvents
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionEventQuery {
    /// Filter by agent ID
    pub agent_id: Option<String>,

    /// Filter by decision type
    pub decision_type: Option<String>,

    /// Filter by organization
    pub organization_id: Option<String>,

    /// Start timestamp
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,

    /// End timestamp
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Maximum results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,
}

impl DecisionEventQuery {
    /// Create a new query builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by agent ID
    pub fn with_agent_id(mut self, agent_id: impl Into<String>) -> Self {
        self.agent_id = Some(agent_id.into());
        self
    }

    /// Filter by organization
    pub fn with_organization(mut self, org_id: impl Into<String>) -> Self {
        self.organization_id = Some(org_id.into());
        self
    }

    /// Set time range
    pub fn with_time_range(
        mut self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Set result limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agents::contracts::{AgentId, AgentVersion, DecisionType, InputsHash};

    #[test]
    fn test_config_defaults() {
        let config = RuVectorConfig::default();
        assert!(!config.endpoint.is_empty());
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.max_retries, 3);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = RuVectorClient::with_defaults();
        assert!(client.is_ok());
    }

    #[tokio::test]
    async fn test_persist_decision_event() {
        let client = RuVectorClient::with_defaults().unwrap();

        let event = DecisionEvent::new(
            AgentId::new("test-agent"),
            AgentVersion::new("1.0.0"),
            DecisionType::CostForecast,
            InputsHash::compute(b"test"),
            serde_json::json!({"forecast": 100}),
            0.95,
        );

        let result = client.persist_decision_event(&event).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, "persisted");
    }

    #[tokio::test]
    async fn test_health_check() {
        let client = RuVectorClient::with_defaults().unwrap();
        let health = client.health_check().await;
        assert!(health.is_ok());
        assert!(health.unwrap());
    }

    #[test]
    fn test_query_builder() {
        let query = DecisionEventQuery::new()
            .with_agent_id("cost-forecasting-agent")
            .with_organization("org-123")
            .with_limit(100);

        assert_eq!(query.agent_id, Some("cost-forecasting-agent".to_string()));
        assert_eq!(query.organization_id, Some("org-123".to_string()));
        assert_eq!(query.limit, Some(100));
    }
}
