//! LLM-Observatory Integration
//!
//! Thin adapter for consuming telemetry, token usage traces, and time-series
//! cost events from LLM-Observatory.
//!
//! This module provides a "consumes-from" integration that receives data from
//! the Observatory and converts it into CostOps domain types for processing.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

// Import from llm-observatory-core
use llm_observatory_core as observatory;

use crate::domain::{Provider, UsageRecord, ModelIdentifier, IngestionSource};
use crate::forecasting::{DataPoint, TimeSeriesData};

/// Errors that can occur during Observatory integration
#[derive(Debug, Error)]
pub enum ObservatoryError {
    #[error("Failed to connect to Observatory: {0}")]
    ConnectionError(String),

    #[error("Failed to parse telemetry event: {0}")]
    ParseError(String),

    #[error("Invalid event format: {0}")]
    InvalidFormat(String),

    #[error("Provider mapping not found: {0}")]
    UnknownProvider(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),
}

/// Configuration for Observatory integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservatoryConfig {
    /// Whether the integration is enabled
    pub enabled: bool,

    /// Observatory endpoint URL (if remote)
    pub endpoint: Option<String>,

    /// Batch size for consuming events
    pub batch_size: usize,

    /// Provider name mapping (observatory name -> CostOps provider)
    pub provider_mapping: HashMap<String, String>,

    /// Whether to validate incoming events
    pub validate_events: bool,
}

impl Default for ObservatoryConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: None,
            batch_size: 100,
            provider_mapping: default_provider_mapping(),
            validate_events: true,
        }
    }
}

fn default_provider_mapping() -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("openai".to_string(), "openai".to_string());
    map.insert("anthropic".to_string(), "anthropic".to_string());
    map.insert("google".to_string(), "google".to_string());
    map.insert("azure".to_string(), "azure".to_string());
    map.insert("aws".to_string(), "aws".to_string());
    map.insert("cohere".to_string(), "cohere".to_string());
    map.insert("mistral".to_string(), "mistral".to_string());
    map
}

/// Telemetry event consumed from Observatory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Event identifier
    pub event_id: Uuid,

    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,

    /// Event type (e.g., "llm_request", "token_usage", "cost_event")
    pub event_type: String,

    /// Provider identifier
    pub provider: String,

    /// Model identifier
    pub model: String,

    /// Organization/tenant ID
    pub organization_id: String,

    /// Token counts
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,

    /// Latency metrics
    pub latency_ms: Option<u64>,
    pub time_to_first_token_ms: Option<u64>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Token usage trace from Observatory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsageTrace {
    /// Trace identifier
    pub trace_id: Uuid,

    /// Parent span ID (for distributed tracing)
    pub parent_span_id: Option<String>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Provider
    pub provider: String,

    /// Model
    pub model: String,

    /// Token breakdown
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cached_tokens: Option<u64>,
    pub reasoning_tokens: Option<u64>,

    /// Request metadata
    pub request_id: Option<String>,
    pub organization_id: String,
    pub project_id: Option<String>,
    pub user_id: Option<String>,
}

/// Time-series cost event from Observatory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTimeSeriesEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,

    /// Cost value
    pub cost: Decimal,

    /// Currency code
    pub currency: String,

    /// Provider
    pub provider: String,

    /// Model (optional for aggregated events)
    pub model: Option<String>,

    /// Organization
    pub organization_id: String,

    /// Aggregation window (in seconds, None for point events)
    pub window_seconds: Option<u64>,

    /// Event tags
    pub tags: Vec<String>,
}

/// Consumer for Observatory data
pub struct ObservatoryConsumer {
    config: ObservatoryConfig,
}

impl ObservatoryConsumer {
    /// Create a new Observatory consumer with the given configuration
    pub fn new(config: ObservatoryConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ObservatoryConfig::default())
    }

    /// Check if the integration is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Convert a TelemetryEvent into a UsageRecord
    pub fn telemetry_to_usage_record(
        &self,
        event: &TelemetryEvent,
    ) -> Result<UsageRecord, ObservatoryError> {
        let provider = self.map_provider(&event.provider)?;

        let prompt_tokens = event.prompt_tokens.unwrap_or(0);
        let completion_tokens = event.completion_tokens.unwrap_or(0);
        let total_tokens = event.total_tokens.unwrap_or(prompt_tokens + completion_tokens);

        let mut record = UsageRecord::new(
            provider,
            ModelIdentifier::new(event.model.clone(), 0),
            event.organization_id.clone(),
            prompt_tokens,
            completion_tokens,
        );

        record.id = event.event_id;
        record.timestamp = event.timestamp;
        record.total_tokens = total_tokens;

        if let Some(latency) = event.latency_ms {
            record.latency_ms = Some(latency);
        }

        if let Some(ttft) = event.time_to_first_token_ms {
            record.time_to_first_token_ms = Some(ttft);
        }

        record.source = IngestionSource::Stream {
            topic: "observatory.telemetry".to_string(),
        };

        if self.config.validate_events {
            record.validate().map_err(|e| {
                ObservatoryError::InvalidFormat(format!("Validation failed: {}", e))
            })?;
        }

        Ok(record)
    }

    /// Convert a TokenUsageTrace into a UsageRecord
    pub fn trace_to_usage_record(
        &self,
        trace: &TokenUsageTrace,
    ) -> Result<UsageRecord, ObservatoryError> {
        let provider = self.map_provider(&trace.provider)?;

        let mut record = UsageRecord::new(
            provider,
            ModelIdentifier::new(trace.model.clone(), 0),
            trace.organization_id.clone(),
            trace.prompt_tokens,
            trace.completion_tokens,
        );

        record.id = trace.trace_id;
        record.timestamp = trace.timestamp;
        record.cached_tokens = trace.cached_tokens;
        record.reasoning_tokens = trace.reasoning_tokens;
        record.project_id = trace.project_id.clone();
        record.user_id = trace.user_id.clone();

        record.source = IngestionSource::Stream {
            topic: "observatory.traces".to_string(),
        };

        if self.config.validate_events {
            record.validate().map_err(|e| {
                ObservatoryError::InvalidFormat(format!("Validation failed: {}", e))
            })?;
        }

        Ok(record)
    }

    /// Convert CostTimeSeriesEvents into TimeSeriesData for forecasting
    pub fn cost_events_to_time_series(
        &self,
        events: &[CostTimeSeriesEvent],
    ) -> Result<TimeSeriesData, ObservatoryError> {
        if events.is_empty() {
            return Ok(TimeSeriesData::new(vec![]));
        }

        let mut data_points: Vec<DataPoint> = events
            .iter()
            .map(|event| DataPoint::new(event.timestamp, event.cost))
            .collect();

        // Sort by timestamp
        data_points.sort_by_key(|p| p.timestamp);

        Ok(TimeSeriesData::with_auto_interval(data_points))
    }

    /// Batch convert telemetry events
    pub fn batch_telemetry_to_usage(
        &self,
        events: &[TelemetryEvent],
    ) -> Vec<Result<UsageRecord, ObservatoryError>> {
        events
            .iter()
            .map(|e| self.telemetry_to_usage_record(e))
            .collect()
    }

    /// Batch convert token traces
    pub fn batch_traces_to_usage(
        &self,
        traces: &[TokenUsageTrace],
    ) -> Vec<Result<UsageRecord, ObservatoryError>> {
        traces
            .iter()
            .map(|t| self.trace_to_usage_record(t))
            .collect()
    }

    /// Map Observatory provider name to CostOps Provider
    fn map_provider(&self, observatory_provider: &str) -> Result<Provider, ObservatoryError> {
        let normalized = observatory_provider.to_lowercase();

        // Check custom mapping first
        if let Some(mapped) = self.config.provider_mapping.get(&normalized) {
            return self.parse_provider(mapped);
        }

        // Try direct mapping
        self.parse_provider(&normalized)
    }

    fn parse_provider(&self, name: &str) -> Result<Provider, ObservatoryError> {
        match name.to_lowercase().as_str() {
            "openai" => Ok(Provider::OpenAI),
            "anthropic" => Ok(Provider::Anthropic),
            "google" | "gemini" | "vertex" => Ok(Provider::GoogleVertexAI),
            "azure" | "azure_openai" => Ok(Provider::AzureOpenAI),
            "aws" | "bedrock" => Ok(Provider::AWSBedrock),
            "cohere" => Ok(Provider::Cohere),
            "mistral" => Ok(Provider::Mistral),
            other => Ok(Provider::Custom(other.to_string())),
        }
    }
}

impl Default for ObservatoryConsumer {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_to_usage_record() {
        let consumer = ObservatoryConsumer::with_defaults();

        let event = TelemetryEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            event_type: "llm_request".to_string(),
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            organization_id: "org-123".to_string(),
            prompt_tokens: Some(100),
            completion_tokens: Some(50),
            total_tokens: Some(150),
            latency_ms: Some(250),
            time_to_first_token_ms: Some(50),
            metadata: HashMap::new(),
        };

        let result = consumer.telemetry_to_usage_record(&event);
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.prompt_tokens, 100);
        assert_eq!(record.completion_tokens, 50);
        assert_eq!(record.total_tokens, 150);
        assert_eq!(record.latency_ms, Some(250));
    }

    #[test]
    fn test_trace_to_usage_record() {
        let consumer = ObservatoryConsumer::with_defaults();

        let trace = TokenUsageTrace {
            trace_id: Uuid::new_v4(),
            parent_span_id: None,
            timestamp: Utc::now(),
            provider: "anthropic".to_string(),
            model: "claude-3".to_string(),
            prompt_tokens: 200,
            completion_tokens: 100,
            cached_tokens: Some(50),
            reasoning_tokens: None,
            request_id: Some("req-123".to_string()),
            organization_id: "org-456".to_string(),
            project_id: Some("proj-789".to_string()),
            user_id: None,
        };

        let result = consumer.trace_to_usage_record(&trace);
        assert!(result.is_ok());

        let record = result.unwrap();
        assert_eq!(record.prompt_tokens, 200);
        assert_eq!(record.cached_tokens, Some(50));
        assert!(matches!(record.provider, Provider::Anthropic));
    }

    #[test]
    fn test_cost_events_to_time_series() {
        let consumer = ObservatoryConsumer::with_defaults();

        let events = vec![
            CostTimeSeriesEvent {
                timestamp: Utc::now(),
                cost: Decimal::from(100),
                currency: "USD".to_string(),
                provider: "openai".to_string(),
                model: Some("gpt-4".to_string()),
                organization_id: "org-123".to_string(),
                window_seconds: None,
                tags: vec![],
            },
            CostTimeSeriesEvent {
                timestamp: Utc::now(),
                cost: Decimal::from(150),
                currency: "USD".to_string(),
                provider: "openai".to_string(),
                model: Some("gpt-4".to_string()),
                organization_id: "org-123".to_string(),
                window_seconds: None,
                tags: vec![],
            },
        ];

        let result = consumer.cost_events_to_time_series(&events);
        assert!(result.is_ok());

        let time_series = result.unwrap();
        assert_eq!(time_series.len(), 2);
    }

    #[test]
    fn test_provider_mapping() {
        let consumer = ObservatoryConsumer::with_defaults();

        assert!(matches!(consumer.map_provider("openai"), Ok(Provider::OpenAI)));
        assert!(matches!(consumer.map_provider("ANTHROPIC"), Ok(Provider::Anthropic)));
        assert!(matches!(consumer.map_provider("google"), Ok(Provider::GoogleVertexAI)));
        assert!(matches!(consumer.map_provider("gemini"), Ok(Provider::GoogleVertexAI)));
        assert!(matches!(consumer.map_provider("bedrock"), Ok(Provider::AWSBedrock)));
    }
}
