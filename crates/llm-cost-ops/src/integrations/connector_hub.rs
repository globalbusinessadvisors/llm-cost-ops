//! LLM-Connector-Hub Integration
//!
//! Thin adapter for consuming provider metadata, pricing tables, and backend
//! capabilities from LLM-Connector-Hub.
//!
//! **Note**: LLM-Connector-Hub is a TypeScript/Node.js project. This module
//! provides a bridge interface for consuming data via:
//! - REST API calls to Connector Hub service
//! - JSON file imports from Connector Hub exports
//! - WebSocket subscriptions for real-time updates
//!
//! This is a "consumes-from" integration - CostOps receives data from
//! Connector Hub but never exports data back to it.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::domain::{Provider, PricingTable, PricingStructure, Currency};

/// Errors that can occur during Connector Hub integration
#[derive(Debug, Error)]
pub enum ConnectorHubError {
    #[error("Failed to connect to Connector Hub: {0}")]
    ConnectionError(String),

    #[error("Failed to parse connector data: {0}")]
    ParseError(String),

    #[error("Provider not supported: {0}")]
    UnsupportedProvider(String),

    #[error("Invalid pricing format: {0}")]
    InvalidPricing(String),

    #[error("API error: {code} - {message}")]
    ApiError { code: u16, message: String },

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Bridge not available: {0}")]
    BridgeUnavailable(String),
}

/// Configuration for Connector Hub integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorHubConfig {
    /// Whether the integration is enabled
    pub enabled: bool,

    /// Connector Hub REST API endpoint
    pub api_endpoint: Option<String>,

    /// API key for authentication (if required)
    pub api_key: Option<String>,

    /// WebSocket endpoint for real-time updates
    pub websocket_endpoint: Option<String>,

    /// Local file path for JSON imports (fallback)
    pub local_data_path: Option<String>,

    /// Cache TTL for provider metadata (in seconds)
    pub cache_ttl_seconds: u64,

    /// Whether to auto-refresh pricing on startup
    pub auto_refresh_pricing: bool,

    /// Timeout for API requests (in milliseconds)
    pub request_timeout_ms: u64,
}

impl Default for ConnectorHubConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_endpoint: None,
            api_key: None,
            websocket_endpoint: None,
            local_data_path: None,
            cache_ttl_seconds: 3600,
            auto_refresh_pricing: false,
            request_timeout_ms: 30000,
        }
    }
}

/// Provider metadata from Connector Hub
///
/// Contains comprehensive information about an LLM provider including
/// supported models, capabilities, and API details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderMetadata {
    /// Provider identifier (matches CostOps Provider enum)
    pub provider_id: String,

    /// Human-readable provider name
    pub display_name: String,

    /// Provider description
    pub description: Option<String>,

    /// Base API URL
    pub api_base_url: String,

    /// Supported models
    pub models: Vec<ModelMetadata>,

    /// Provider capabilities
    pub capabilities: Vec<String>,

    /// Authentication type (e.g., "api_key", "oauth", "bearer")
    pub auth_type: String,

    /// Rate limiting information
    pub rate_limits: Option<RateLimitInfo>,

    /// Whether the provider is currently active/available
    pub is_active: bool,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Model metadata from Connector Hub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelMetadata {
    /// Model identifier
    pub model_id: String,

    /// Human-readable model name
    pub display_name: String,

    /// Model description
    pub description: Option<String>,

    /// Context window size
    pub context_window: u64,

    /// Maximum output tokens
    pub max_output_tokens: Option<u64>,

    /// Supported features (e.g., "streaming", "function_calling", "vision")
    pub features: Vec<String>,

    /// Model family (e.g., "gpt-4", "claude-3", "gemini")
    pub family: Option<String>,

    /// Model version
    pub version: Option<String>,

    /// Training data cutoff date
    pub training_cutoff: Option<String>,

    /// Whether the model is deprecated
    pub deprecated: bool,

    /// Deprecation date (if applicable)
    pub deprecation_date: Option<DateTime<Utc>>,
}

/// Rate limit information from Connector Hub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitInfo {
    /// Requests per minute
    pub requests_per_minute: Option<u64>,

    /// Tokens per minute
    pub tokens_per_minute: Option<u64>,

    /// Tokens per day
    pub tokens_per_day: Option<u64>,

    /// Concurrent request limit
    pub concurrent_requests: Option<u64>,
}

/// Pricing table update from Connector Hub
///
/// Represents pricing information for a specific provider/model combination
/// that can be converted into CostOps PricingTable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTableUpdate {
    /// Provider identifier
    pub provider_id: String,

    /// Model identifier
    pub model_id: String,

    /// Currency code
    pub currency: String,

    /// Input token price (per 1M tokens)
    pub input_price_per_million: Decimal,

    /// Output token price (per 1M tokens)
    pub output_price_per_million: Decimal,

    /// Cached token price (per 1M tokens, if applicable)
    pub cached_price_per_million: Option<Decimal>,

    /// Cache discount percentage (0.0-1.0)
    pub cache_discount: Option<Decimal>,

    /// Effective date for this pricing
    pub effective_date: DateTime<Utc>,

    /// End date for this pricing (None = currently active)
    pub end_date: Option<DateTime<Utc>>,

    /// Pricing tier information (for tiered pricing)
    pub tiers: Option<Vec<PricingTierUpdate>>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
}

/// Pricing tier update for tiered pricing models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTierUpdate {
    /// Tier name
    pub tier_name: String,

    /// Minimum tokens for this tier
    pub min_tokens: u64,

    /// Maximum tokens for this tier (None = unlimited)
    pub max_tokens: Option<u64>,

    /// Input price per 1M tokens at this tier
    pub input_price_per_million: Decimal,

    /// Output price per 1M tokens at this tier
    pub output_price_per_million: Decimal,
}

/// Backend capability information from Connector Hub
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendCapability {
    /// Capability identifier
    pub capability_id: String,

    /// Capability name
    pub name: String,

    /// Description
    pub description: Option<String>,

    /// Providers that support this capability
    pub supported_providers: Vec<String>,

    /// Models that support this capability (grouped by provider)
    pub supported_models: HashMap<String, Vec<String>>,

    /// Whether this capability affects pricing
    pub affects_pricing: bool,

    /// Pricing modifier (multiplier) if affects_pricing is true
    pub pricing_modifier: Option<Decimal>,
}

/// Bridge for consuming Connector Hub data
///
/// Since Connector Hub is a TypeScript project, this bridge provides
/// methods to consume data via REST API, local JSON files, or WebSocket.
pub struct ConnectorHubBridge {
    config: ConnectorHubConfig,
}

impl ConnectorHubBridge {
    /// Create a new Connector Hub bridge with the given configuration
    pub fn new(config: ConnectorHubConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(ConnectorHubConfig::default())
    }

    /// Check if the integration is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Check if REST API is configured
    pub fn has_api(&self) -> bool {
        self.config.api_endpoint.is_some()
    }

    /// Check if local data is configured
    pub fn has_local_data(&self) -> bool {
        self.config.local_data_path.is_some()
    }

    /// Convert ProviderMetadata to CostOps Provider
    pub fn metadata_to_provider(
        &self,
        metadata: &ProviderMetadata,
    ) -> Result<Provider, ConnectorHubError> {
        match metadata.provider_id.to_lowercase().as_str() {
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

    /// Convert PricingTableUpdate to CostOps PricingTable
    pub fn pricing_update_to_table(
        &self,
        update: &PricingTableUpdate,
    ) -> Result<PricingTable, ConnectorHubError> {
        let provider = self.parse_provider(&update.provider_id)?;

        // Convert per-million pricing to per-token pricing
        let input_per_token = update.input_price_per_million / Decimal::from(1_000_000);
        let output_per_token = update.output_price_per_million / Decimal::from(1_000_000);

        let pricing_structure = if let Some(cache_discount) = update.cache_discount {
            PricingStructure::per_token_with_cache(
                input_per_token,
                output_per_token,
                cache_discount,
            )
        } else {
            PricingStructure::simple_per_token(input_per_token, output_per_token)
        };

        Ok(PricingTable::new(
            provider,
            update.model_id.clone(),
            pricing_structure,
        ))
    }

    /// Batch convert pricing updates to pricing tables
    pub fn batch_pricing_to_tables(
        &self,
        updates: &[PricingTableUpdate],
    ) -> Vec<Result<PricingTable, ConnectorHubError>> {
        updates
            .iter()
            .map(|u| self.pricing_update_to_table(u))
            .collect()
    }

    /// Extract model context windows from provider metadata
    pub fn extract_context_windows(
        &self,
        metadata: &ProviderMetadata,
    ) -> HashMap<String, u64> {
        metadata
            .models
            .iter()
            .map(|m| (m.model_id.clone(), m.context_window))
            .collect()
    }

    /// Extract model features from provider metadata
    pub fn extract_model_features(
        &self,
        metadata: &ProviderMetadata,
    ) -> HashMap<String, Vec<String>> {
        metadata
            .models
            .iter()
            .map(|m| (m.model_id.clone(), m.features.clone()))
            .collect()
    }

    /// Check if a model supports a specific capability
    pub fn model_supports_capability(
        &self,
        capability: &BackendCapability,
        provider_id: &str,
        model_id: &str,
    ) -> bool {
        capability
            .supported_models
            .get(provider_id)
            .map(|models| models.contains(&model_id.to_string()))
            .unwrap_or(false)
    }

    /// Get pricing modifier for a capability
    pub fn capability_pricing_modifier(
        &self,
        capability: &BackendCapability,
    ) -> Option<Decimal> {
        if capability.affects_pricing {
            capability.pricing_modifier
        } else {
            None
        }
    }

    /// Parse currency code to CostOps Currency
    pub fn parse_currency(&self, code: &str) -> Currency {
        match code.to_uppercase().as_str() {
            "USD" => Currency::USD,
            "EUR" => Currency::EUR,
            "GBP" => Currency::GBP,
            "JPY" => Currency::JPY,
            other => Currency::Custom(other.to_string()),
        }
    }

    /// Parse provider string to CostOps Provider
    fn parse_provider(&self, provider_id: &str) -> Result<Provider, ConnectorHubError> {
        match provider_id.to_lowercase().as_str() {
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

    /// Validate a pricing update
    pub fn validate_pricing_update(
        &self,
        update: &PricingTableUpdate,
    ) -> Result<(), ConnectorHubError> {
        // Check for valid prices
        if update.input_price_per_million < Decimal::ZERO {
            return Err(ConnectorHubError::InvalidPricing(
                "Input price cannot be negative".to_string(),
            ));
        }

        if update.output_price_per_million < Decimal::ZERO {
            return Err(ConnectorHubError::InvalidPricing(
                "Output price cannot be negative".to_string(),
            ));
        }

        // Check cache discount range
        if let Some(discount) = update.cache_discount {
            if discount < Decimal::ZERO || discount > Decimal::ONE {
                return Err(ConnectorHubError::InvalidPricing(
                    "Cache discount must be between 0 and 1".to_string(),
                ));
            }
        }

        // Check dates
        if let Some(end_date) = update.end_date {
            if end_date < update.effective_date {
                return Err(ConnectorHubError::InvalidPricing(
                    "End date cannot be before effective date".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Default for ConnectorHubBridge {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pricing_update() -> PricingTableUpdate {
        PricingTableUpdate {
            provider_id: "openai".to_string(),
            model_id: "gpt-4-turbo".to_string(),
            currency: "USD".to_string(),
            input_price_per_million: Decimal::from(10),
            output_price_per_million: Decimal::from(30),
            cached_price_per_million: Some(Decimal::from(5)),
            cache_discount: Some(Decimal::new(5, 1)), // 0.5
            effective_date: Utc::now(),
            end_date: None,
            tiers: None,
            updated_at: Utc::now(),
        }
    }

    fn create_test_provider_metadata() -> ProviderMetadata {
        ProviderMetadata {
            provider_id: "openai".to_string(),
            display_name: "OpenAI".to_string(),
            description: Some("OpenAI API".to_string()),
            api_base_url: "https://api.openai.com/v1".to_string(),
            models: vec![
                ModelMetadata {
                    model_id: "gpt-4-turbo".to_string(),
                    display_name: "GPT-4 Turbo".to_string(),
                    description: Some("Latest GPT-4 model".to_string()),
                    context_window: 128000,
                    max_output_tokens: Some(4096),
                    features: vec!["streaming".to_string(), "function_calling".to_string()],
                    family: Some("gpt-4".to_string()),
                    version: Some("turbo-2024-04-09".to_string()),
                    training_cutoff: Some("2023-12".to_string()),
                    deprecated: false,
                    deprecation_date: None,
                },
                ModelMetadata {
                    model_id: "gpt-3.5-turbo".to_string(),
                    display_name: "GPT-3.5 Turbo".to_string(),
                    description: None,
                    context_window: 16385,
                    max_output_tokens: Some(4096),
                    features: vec!["streaming".to_string()],
                    family: Some("gpt-3.5".to_string()),
                    version: None,
                    training_cutoff: None,
                    deprecated: false,
                    deprecation_date: None,
                },
            ],
            capabilities: vec!["text".to_string(), "chat".to_string()],
            auth_type: "api_key".to_string(),
            rate_limits: Some(RateLimitInfo {
                requests_per_minute: Some(3500),
                tokens_per_minute: Some(90000),
                tokens_per_day: None,
                concurrent_requests: None,
            }),
            is_active: true,
            updated_at: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_pricing_update_to_table() {
        let bridge = ConnectorHubBridge::with_defaults();
        let update = create_test_pricing_update();

        let result = bridge.pricing_update_to_table(&update);
        assert!(result.is_ok());

        let table = result.unwrap();
        assert!(matches!(table.provider, Provider::OpenAI));
        assert_eq!(table.model, "gpt-4-turbo");
    }

    #[test]
    fn test_metadata_to_provider() {
        let bridge = ConnectorHubBridge::with_defaults();
        let metadata = create_test_provider_metadata();

        let result = bridge.metadata_to_provider(&metadata);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Provider::OpenAI));
    }

    #[test]
    fn test_extract_context_windows() {
        let bridge = ConnectorHubBridge::with_defaults();
        let metadata = create_test_provider_metadata();

        let windows = bridge.extract_context_windows(&metadata);
        assert_eq!(windows.get("gpt-4-turbo"), Some(&128000));
        assert_eq!(windows.get("gpt-3.5-turbo"), Some(&16385));
    }

    #[test]
    fn test_validate_pricing_update() {
        let bridge = ConnectorHubBridge::with_defaults();
        let update = create_test_pricing_update();

        let result = bridge.validate_pricing_update(&update);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_pricing() {
        let bridge = ConnectorHubBridge::with_defaults();
        let mut update = create_test_pricing_update();
        update.input_price_per_million = Decimal::from(-10);

        let result = bridge.validate_pricing_update(&update);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_currency() {
        let bridge = ConnectorHubBridge::with_defaults();

        assert!(matches!(bridge.parse_currency("USD"), Currency::USD));
        assert!(matches!(bridge.parse_currency("eur"), Currency::EUR));
        assert!(matches!(bridge.parse_currency("GBP"), Currency::GBP));
        assert!(matches!(bridge.parse_currency("XYZ"), Currency::Custom(_)));
    }

    #[test]
    fn test_capability_check() {
        let bridge = ConnectorHubBridge::with_defaults();

        let capability = BackendCapability {
            capability_id: "function_calling".to_string(),
            name: "Function Calling".to_string(),
            description: None,
            supported_providers: vec!["openai".to_string()],
            supported_models: {
                let mut map = HashMap::new();
                map.insert("openai".to_string(), vec!["gpt-4-turbo".to_string()]);
                map
            },
            affects_pricing: false,
            pricing_modifier: None,
        };

        assert!(bridge.model_supports_capability(&capability, "openai", "gpt-4-turbo"));
        assert!(!bridge.model_supports_capability(&capability, "openai", "gpt-3.5-turbo"));
        assert!(!bridge.model_supports_capability(&capability, "anthropic", "claude-3"));
    }
}
