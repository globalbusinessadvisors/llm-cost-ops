//! SDK configuration with builder pattern and validation

use crate::sdk::error::{SdkError, SdkResult};
use std::time::Duration;
use url::Url;

/// Main SDK configuration
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL for the API
    pub base_url: Url,

    /// API key for authentication
    pub api_key: String,

    /// Request timeout
    pub timeout: Duration,

    /// Connection pool configuration
    pub pool_config: PoolConfig,

    /// Retry configuration
    pub retry_config: RetryConfig,

    /// Rate limiting configuration
    pub rate_limit_config: RateLimitConfig,

    /// Telemetry configuration
    pub telemetry_config: TelemetryConfig,

    /// Custom headers to include in all requests
    pub default_headers: Vec<(String, String)>,

    /// User agent string
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: Url::parse("http://localhost:8080").unwrap(),
            api_key: String::new(),
            timeout: Duration::from_secs(30),
            pool_config: PoolConfig::default(),
            retry_config: RetryConfig::default(),
            rate_limit_config: RateLimitConfig::default(),
            telemetry_config: TelemetryConfig::default(),
            default_headers: Vec::new(),
            user_agent: format!("llm-cost-ops-sdk/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl ClientConfig {
    /// Create a new builder
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::default()
    }

    /// Validate the configuration
    pub fn validate(&self) -> SdkResult<()> {
        if self.api_key.is_empty() {
            return Err(SdkError::config("API key is required"));
        }

        if self.timeout.as_secs() == 0 {
            return Err(SdkError::config("Timeout must be greater than zero"));
        }

        self.pool_config.validate()?;
        self.retry_config.validate()?;
        self.rate_limit_config.validate()?;

        Ok(())
    }
}

/// Builder for ClientConfig with fluent API
#[derive(Debug, Default)]
pub struct ClientConfigBuilder {
    base_url: Option<Url>,
    api_key: Option<String>,
    timeout: Option<Duration>,
    pool_config: Option<PoolConfig>,
    retry_config: Option<RetryConfig>,
    rate_limit_config: Option<RateLimitConfig>,
    telemetry_config: Option<TelemetryConfig>,
    default_headers: Vec<(String, String)>,
    user_agent: Option<String>,
}

impl ClientConfigBuilder {
    /// Set the base URL
    pub fn base_url(mut self, url: impl AsRef<str>) -> SdkResult<Self> {
        self.base_url = Some(
            Url::parse(url.as_ref())
                .map_err(|e| SdkError::config(format!("Invalid URL: {}", e)))?,
        );
        Ok(self)
    }

    /// Set the API key
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set the pool configuration
    pub fn pool_config(mut self, config: PoolConfig) -> Self {
        self.pool_config = Some(config);
        self
    }

    /// Set the retry configuration
    pub fn retry_config(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }

    /// Set the rate limit configuration
    pub fn rate_limit_config(mut self, config: RateLimitConfig) -> Self {
        self.rate_limit_config = Some(config);
        self
    }

    /// Set the telemetry configuration
    pub fn telemetry_config(mut self, config: TelemetryConfig) -> Self {
        self.telemetry_config = Some(config);
        self
    }

    /// Add a default header
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.default_headers.push((key.into(), value.into()));
        self
    }

    /// Set the user agent
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    /// Build the configuration
    pub fn build(self) -> SdkResult<ClientConfig> {
        let config = ClientConfig {
            base_url: self
                .base_url
                .ok_or_else(|| SdkError::config("base_url is required"))?,
            api_key: self
                .api_key
                .ok_or_else(|| SdkError::config("api_key is required"))?,
            timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
            pool_config: self.pool_config.unwrap_or_default(),
            retry_config: self.retry_config.unwrap_or_default(),
            rate_limit_config: self.rate_limit_config.unwrap_or_default(),
            telemetry_config: self.telemetry_config.unwrap_or_default(),
            default_headers: self.default_headers,
            user_agent: self
                .user_agent
                .unwrap_or_else(|| format!("llm-cost-ops-sdk/{}", env!("CARGO_PKG_VERSION"))),
        };

        config.validate()?;
        Ok(config)
    }
}

/// Connection pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Maximum number of idle connections
    pub max_idle: usize,

    /// Maximum number of connections per host
    pub max_per_host: usize,

    /// Idle timeout for connections
    pub idle_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_idle: 10,
            max_per_host: 20,
            idle_timeout: Duration::from_secs(90),
        }
    }
}

impl PoolConfig {
    fn validate(&self) -> SdkResult<()> {
        if self.max_idle == 0 {
            return Err(SdkError::config("max_idle must be greater than zero"));
        }
        if self.max_per_host == 0 {
            return Err(SdkError::config("max_per_host must be greater than zero"));
        }
        Ok(())
    }
}

/// Retry configuration with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: usize,

    /// Initial backoff duration
    pub initial_backoff: Duration,

    /// Maximum backoff duration
    pub max_backoff: Duration,

    /// Backoff multiplier
    pub multiplier: f64,

    /// Enable jitter to avoid thundering herd
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    fn validate(&self) -> SdkResult<()> {
        if self.max_attempts == 0 {
            return Err(SdkError::config("max_attempts must be greater than zero"));
        }
        if self.multiplier <= 0.0 {
            return Err(SdkError::config("multiplier must be greater than zero"));
        }
        if self.initial_backoff > self.max_backoff {
            return Err(SdkError::config(
                "initial_backoff cannot be greater than max_backoff",
            ));
        }
        Ok(())
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum number of requests per second
    pub requests_per_second: Option<usize>,

    /// Maximum burst size
    pub burst_size: Option<usize>,

    /// Enable rate limiting
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: Some(100),
            burst_size: Some(10),
            enabled: true,
        }
    }
}

impl RateLimitConfig {
    fn validate(&self) -> SdkResult<()> {
        if self.enabled {
            if let Some(rps) = self.requests_per_second {
                if rps == 0 {
                    return Err(SdkError::config(
                        "requests_per_second must be greater than zero",
                    ));
                }
            }
            if let Some(burst) = self.burst_size {
                if burst == 0 {
                    return Err(SdkError::config("burst_size must be greater than zero"));
                }
            }
        }
        Ok(())
    }
}

/// Telemetry configuration
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Enable metrics collection
    pub metrics_enabled: bool,

    /// Enable tracing
    pub tracing_enabled: bool,

    /// Enable request logging
    pub logging_enabled: bool,

    /// Metrics export endpoint
    pub metrics_endpoint: Option<String>,

    /// Trace export endpoint
    pub trace_endpoint: Option<String>,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            tracing_enabled: true,
            logging_enabled: true,
            metrics_endpoint: None,
            trace_endpoint: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = ClientConfig::builder()
            .base_url("https://api.example.com")
            .unwrap()
            .api_key("test-key")
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap();

        assert_eq!(config.base_url.as_str(), "https://api.example.com/");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_config_validation() {
        let result = ClientConfig::builder()
            .base_url("https://api.example.com")
            .unwrap()
            .build();

        assert!(result.is_err()); // Missing API key
    }

    #[test]
    fn test_default_config() {
        let config = ClientConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.pool_config.max_idle, 10);
        assert_eq!(config.retry_config.max_attempts, 3);
    }

    #[test]
    fn test_retry_config_validation() {
        let mut config = RetryConfig::default();
        config.max_attempts = 0;
        assert!(config.validate().is_err());

        let mut config = RetryConfig::default();
        config.multiplier = -1.0;
        assert!(config.validate().is_err());

        let mut config = RetryConfig::default();
        config.initial_backoff = Duration::from_secs(100);
        config.max_backoff = Duration::from_secs(10);
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_pool_config_validation() {
        let mut config = PoolConfig::default();
        config.max_idle = 0;
        assert!(config.validate().is_err());

        let mut config = PoolConfig::default();
        config.max_per_host = 0;
        assert!(config.validate().is_err());
    }
}
