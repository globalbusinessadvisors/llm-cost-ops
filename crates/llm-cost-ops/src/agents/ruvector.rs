//! RuVector Service Client
//!
//! This module provides a client for persisting DecisionEvents to ruvector-service.
//! LLM-CostOps NEVER connects directly to Google SQL - all persistence occurs
//! through ruvector-service client calls only.
//!
//! # Architecture
//!
//! - Async, non-blocking writes
//! - Automatic retry with exponential backoff
//! - Circuit breaker for fault tolerance
//! - Connection pooling
//!
//! # Usage
//!
//! ```ignore
//! let client = RuvectorClient::new(RuvectorConfig::from_env()?)?;
//! client.persist_decision_event(&event).await?;
//! ```

use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::contracts::{DecisionEvent, AgentTelemetryEvent};

/// RuVector service client errors
#[derive(Debug, Error)]
pub enum RuvectorError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Request failed with status {status}: {message}")]
    RequestFailed { status: u16, message: String },

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Circuit breaker open: too many failures")]
    CircuitBreakerOpen,

    #[error("Request timeout")]
    Timeout,

    #[error("Rate limited: retry after {retry_after_seconds} seconds")]
    RateLimited { retry_after_seconds: u64 },
}

/// RuVector service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuvectorConfig {
    /// Service endpoint URL
    pub endpoint: String,

    /// API key for authentication
    pub api_key: Option<String>,

    /// Request timeout in milliseconds
    pub timeout_ms: u64,

    /// Maximum retry attempts
    pub max_retries: u32,

    /// Initial retry delay in milliseconds
    pub retry_delay_ms: u64,

    /// Circuit breaker failure threshold
    pub circuit_breaker_threshold: u32,

    /// Circuit breaker reset timeout in seconds
    pub circuit_breaker_reset_seconds: u64,

    /// Enable request logging
    pub enable_logging: bool,
}

impl RuvectorConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Result<Self, RuvectorError> {
        Ok(Self {
            endpoint: std::env::var("RUVECTOR_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            api_key: std::env::var("RUVECTOR_API_KEY").ok(),
            timeout_ms: std::env::var("RUVECTOR_TIMEOUT_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5000),
            max_retries: std::env::var("RUVECTOR_MAX_RETRIES")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3),
            retry_delay_ms: std::env::var("RUVECTOR_RETRY_DELAY_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
            circuit_breaker_threshold: std::env::var("RUVECTOR_CIRCUIT_BREAKER_THRESHOLD")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(5),
            circuit_breaker_reset_seconds: std::env::var("RUVECTOR_CIRCUIT_BREAKER_RESET_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(30),
            enable_logging: std::env::var("RUVECTOR_ENABLE_LOGGING")
                .ok()
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), RuvectorError> {
        if self.endpoint.is_empty() {
            return Err(RuvectorError::ConfigError("endpoint cannot be empty".to_string()));
        }
        if self.timeout_ms == 0 {
            return Err(RuvectorError::ConfigError("timeout_ms must be > 0".to_string()));
        }
        Ok(())
    }
}

impl Default for RuvectorConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:8080".to_string(),
            api_key: None,
            timeout_ms: 5000,
            max_retries: 3,
            retry_delay_ms: 100,
            circuit_breaker_threshold: 5,
            circuit_breaker_reset_seconds: 30,
            enable_logging: true,
        }
    }
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker for fault tolerance
struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_count: AtomicU64,
    last_failure_time: RwLock<Option<DateTime<Utc>>>,
    threshold: u32,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    fn new(threshold: u32, reset_timeout_seconds: u64) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed),
            failure_count: AtomicU64::new(0),
            last_failure_time: RwLock::new(None),
            threshold,
            reset_timeout: Duration::from_secs(reset_timeout_seconds),
        }
    }

    async fn can_execute(&self) -> bool {
        let state = *self.state.read().await;
        match state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if reset timeout has passed
                if let Some(last_failure) = *self.last_failure_time.read().await {
                    let elapsed = Utc::now().signed_duration_since(last_failure);
                    if elapsed.to_std().unwrap_or(Duration::ZERO) >= self.reset_timeout {
                        *self.state.write().await = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    async fn record_success(&self) {
        self.failure_count.store(0, Ordering::SeqCst);
        *self.state.write().await = CircuitState::Closed;
    }

    async fn record_failure(&self) {
        let count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
        *self.last_failure_time.write().await = Some(Utc::now());

        if count >= self.threshold as u64 {
            *self.state.write().await = CircuitState::Open;
        }
    }
}

/// Persistence result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceResult {
    /// Event ID that was persisted
    pub event_id: Uuid,
    /// Timestamp when persisted
    pub persisted_at: DateTime<Utc>,
    /// Storage location/key
    pub storage_key: String,
    /// Whether this was a retry
    pub was_retry: bool,
    /// Retry count
    pub retry_count: u32,
}

/// Batch persistence result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPersistenceResult {
    /// Total events in batch
    pub total: usize,
    /// Successfully persisted
    pub succeeded: usize,
    /// Failed to persist
    pub failed: usize,
    /// Individual results
    pub results: Vec<Result<PersistenceResult, String>>,
}

/// RuVector service client
pub struct RuvectorClient {
    config: RuvectorConfig,
    http_client: Client,
    circuit_breaker: Arc<CircuitBreaker>,
}

impl RuvectorClient {
    /// Create a new RuVector client
    pub fn new(config: RuvectorConfig) -> Result<Self, RuvectorError> {
        config.validate()?;

        let http_client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .pool_max_idle_per_host(10)
            .build()
            .map_err(|e| RuvectorError::ConfigError(format!("Failed to create HTTP client: {}", e)))?;

        let circuit_breaker = Arc::new(CircuitBreaker::new(
            config.circuit_breaker_threshold,
            config.circuit_breaker_reset_seconds,
        ));

        Ok(Self {
            config,
            http_client,
            circuit_breaker,
        })
    }

    /// Create a client from environment configuration
    pub fn from_env() -> Result<Self, RuvectorError> {
        Self::new(RuvectorConfig::from_env()?)
    }

    /// Persist a DecisionEvent to ruvector-service
    pub async fn persist_decision_event(
        &self,
        event: &DecisionEvent,
    ) -> Result<PersistenceResult, RuvectorError> {
        self.persist_with_retry(event, 0).await
    }

    /// Persist a batch of DecisionEvents
    pub async fn persist_batch(
        &self,
        events: &[DecisionEvent],
    ) -> BatchPersistenceResult {
        let mut results = Vec::with_capacity(events.len());

        for event in events {
            let result = self.persist_decision_event(event).await;
            results.push(result.map_err(|e| e.to_string()));
        }

        let succeeded = results.iter().filter(|r| r.is_ok()).count();
        let failed = results.len() - succeeded;

        BatchPersistenceResult {
            total: events.len(),
            succeeded,
            failed,
            results,
        }
    }

    /// Persist agent telemetry event
    pub async fn persist_telemetry(
        &self,
        event: &AgentTelemetryEvent,
    ) -> Result<PersistenceResult, RuvectorError> {
        if !self.circuit_breaker.can_execute().await {
            return Err(RuvectorError::CircuitBreakerOpen);
        }

        let url = format!("{}/api/v1/telemetry", self.config.endpoint);
        let body = serde_json::to_string(event)?;

        let mut request = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body(body);

        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await?;

        match response.status() {
            StatusCode::OK | StatusCode::CREATED => {
                self.circuit_breaker.record_success().await;
                Ok(PersistenceResult {
                    event_id: event.event_id,
                    persisted_at: Utc::now(),
                    storage_key: format!("telemetry/{}", event.event_id),
                    was_retry: false,
                    retry_count: 0,
                })
            }
            status => {
                self.circuit_breaker.record_failure().await;
                let message = response.text().await.unwrap_or_default();
                Err(RuvectorError::RequestFailed {
                    status: status.as_u16(),
                    message,
                })
            }
        }
    }

    fn persist_with_retry<'a>(
        &'a self,
        event: &'a DecisionEvent,
        retry_count: u32,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<PersistenceResult, RuvectorError>> + Send + 'a>> {
        Box::pin(async move {
        if !self.circuit_breaker.can_execute().await {
            return Err(RuvectorError::CircuitBreakerOpen);
        }

        let url = format!("{}/api/v1/decisions", self.config.endpoint);
        let body = serde_json::to_string(event)?;

        if self.config.enable_logging {
            tracing::debug!(
                event_id = %event.event_id,
                agent_id = %event.agent_id,
                decision_type = %event.decision_type,
                retry = retry_count,
                "Persisting decision event"
            );
        }

        let mut request = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("X-Request-ID", Uuid::new_v4().to_string())
            .body(body);

        if let Some(ref api_key) = self.config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request.send().await;

        match response {
            Ok(resp) => {
                match resp.status() {
                    StatusCode::OK | StatusCode::CREATED => {
                        self.circuit_breaker.record_success().await;
                        Ok(PersistenceResult {
                            event_id: event.event_id,
                            persisted_at: Utc::now(),
                            storage_key: format!("decisions/{}/{}", event.agent_id, event.event_id),
                            was_retry: retry_count > 0,
                            retry_count,
                        })
                    }
                    StatusCode::TOO_MANY_REQUESTS => {
                        let retry_after = resp
                            .headers()
                            .get("Retry-After")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.parse().ok())
                            .unwrap_or(60);

                        if retry_count < self.config.max_retries {
                            tokio::time::sleep(Duration::from_secs(retry_after)).await;
                            self.persist_with_retry(event, retry_count + 1).await
                        } else {
                            Err(RuvectorError::RateLimited {
                                retry_after_seconds: retry_after,
                            })
                        }
                    }
                    StatusCode::SERVICE_UNAVAILABLE | StatusCode::BAD_GATEWAY | StatusCode::GATEWAY_TIMEOUT => {
                        self.circuit_breaker.record_failure().await;
                        if retry_count < self.config.max_retries {
                            let delay = self.config.retry_delay_ms * (2_u64.pow(retry_count));
                            tokio::time::sleep(Duration::from_millis(delay)).await;
                            self.persist_with_retry(event, retry_count + 1).await
                        } else {
                            let message = resp.text().await.unwrap_or_default();
                            Err(RuvectorError::RequestFailed {
                                status: StatusCode::SERVICE_UNAVAILABLE.as_u16(),
                                message,
                            })
                        }
                    }
                    status => {
                        self.circuit_breaker.record_failure().await;
                        let message = resp.text().await.unwrap_or_default();
                        Err(RuvectorError::RequestFailed {
                            status: status.as_u16(),
                            message,
                        })
                    }
                }
            }
            Err(e) if e.is_timeout() => {
                self.circuit_breaker.record_failure().await;
                if retry_count < self.config.max_retries {
                    let delay = self.config.retry_delay_ms * (2_u64.pow(retry_count));
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    self.persist_with_retry(event, retry_count + 1).await
                } else {
                    Err(RuvectorError::Timeout)
                }
            }
            Err(e) => {
                self.circuit_breaker.record_failure().await;
                Err(RuvectorError::HttpError(e))
            }
        }
        })
    }

    /// Check if the service is healthy
    pub async fn health_check(&self) -> Result<bool, RuvectorError> {
        let url = format!("{}/health", self.config.endpoint);

        let response = self.http_client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;

        Ok(response.status().is_success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RuvectorConfig::default();
        assert_eq!(config.endpoint, "http://localhost:8080");
        assert_eq!(config.timeout_ms, 5000);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_config_validation() {
        let mut config = RuvectorConfig::default();
        assert!(config.validate().is_ok());

        config.endpoint = String::new();
        assert!(config.validate().is_err());
    }

    #[tokio::test]
    async fn test_circuit_breaker() {
        let breaker = CircuitBreaker::new(3, 30);

        // Initial state should allow execution
        assert!(breaker.can_execute().await);

        // Record failures
        breaker.record_failure().await;
        breaker.record_failure().await;
        assert!(breaker.can_execute().await); // Still closed

        breaker.record_failure().await;
        // Circuit should now be open
        assert!(!breaker.can_execute().await);

        // Record success should reset
        breaker.record_success().await;
        assert!(breaker.can_execute().await);
    }
}
