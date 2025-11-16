// Observability configuration

use serde::{Deserialize, Serialize};

/// Observability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ObservabilityConfig {
    /// Metrics configuration
    pub metrics: MetricsConfig,

    /// Tracing configuration
    pub tracing: TracingConfig,

    /// Logging configuration
    pub logging: LoggingConfig,

    /// Health check configuration
    pub health: HealthConfig,
}


/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Prometheus endpoint path
    pub endpoint: String,

    /// Metrics port (if different from main API)
    pub port: Option<u16>,

    /// Metrics push interval for push-based systems
    pub push_interval_secs: Option<u64>,

    /// Include process metrics
    pub include_process_metrics: bool,

    /// Histogram buckets for latency metrics (in seconds)
    pub latency_buckets: Vec<f64>,

    /// Histogram buckets for cost metrics (in dollars)
    pub cost_buckets: Vec<f64>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/metrics".to_string(),
            port: None,
            push_interval_secs: None,
            include_process_metrics: true,
            latency_buckets: vec![
                0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
            ],
            cost_buckets: vec![
                0.0001, 0.001, 0.01, 0.1, 1.0, 10.0, 100.0, 1000.0, 10000.0,
            ],
        }
    }
}

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable tracing
    pub enabled: bool,

    /// Tracing level (trace, debug, info, warn, error)
    pub level: String,

    /// Output format (text, json, pretty)
    pub format: TracingFormat,

    /// Enable ANSI colors
    pub ansi: bool,

    /// Include file and line numbers
    pub include_location: bool,

    /// Include thread names/IDs
    pub include_thread: bool,

    /// Include timestamps
    pub include_timestamp: bool,

    /// Export to OTLP (OpenTelemetry Protocol)
    pub otlp: Option<OtlpConfig>,

    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
            format: TracingFormat::Text,
            ansi: true,
            include_location: true,
            include_thread: false,
            include_timestamp: true,
            otlp: None,
            sampling_rate: 1.0,
        }
    }
}

/// Tracing output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TracingFormat {
    /// Human-readable text format
    Text,

    /// JSON format
    Json,

    /// Pretty-printed format
    Pretty,

    /// Compact format
    Compact,
}

/// OpenTelemetry Protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpConfig {
    /// OTLP endpoint URL
    pub endpoint: String,

    /// Service name
    pub service_name: String,

    /// Service version
    pub service_version: String,

    /// Environment (dev, staging, prod)
    pub environment: String,

    /// Additional attributes
    #[serde(default)]
    pub attributes: std::collections::HashMap<String, String>,

    /// Batch size
    pub batch_size: usize,

    /// Export timeout in seconds
    pub timeout_secs: u64,
}

impl Default for OtlpConfig {
    fn default() -> Self {
        Self {
            endpoint: "http://localhost:4317".to_string(),
            service_name: "llm-cost-ops".to_string(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
            environment: "development".to_string(),
            attributes: std::collections::HashMap::new(),
            batch_size: 512,
            timeout_secs: 10,
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable logging
    pub enabled: bool,

    /// Log level
    pub level: String,

    /// Output format
    pub format: LoggingFormat,

    /// Log to file
    pub file: Option<LogFileConfig>,

    /// Include span context
    pub include_span_context: bool,

    /// Filter targets (e.g., "my_crate=debug,other_crate=info")
    pub filter: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            level: "info".to_string(),
            format: LoggingFormat::Text,
            file: None,
            include_span_context: true,
            filter: None,
        }
    }
}

/// Logging output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoggingFormat {
    /// Human-readable text
    Text,

    /// JSON format
    Json,

    /// Logfmt format
    Logfmt,
}

/// Log file configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogFileConfig {
    /// File path
    pub path: String,

    /// Max file size in MB before rotation
    pub max_size_mb: u64,

    /// Number of rotated files to keep
    pub max_files: usize,

    /// Compress rotated files
    pub compress: bool,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Enable health checks
    pub enabled: bool,

    /// Health check endpoint
    pub endpoint: String,

    /// Readiness check endpoint
    pub readiness_endpoint: String,

    /// Liveness check endpoint
    pub liveness_endpoint: String,

    /// Check interval in seconds
    pub check_interval_secs: u64,

    /// Timeout for health checks in seconds
    pub timeout_secs: u64,

    /// Include detailed status
    pub include_details: bool,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: "/health".to_string(),
            readiness_endpoint: "/ready".to_string(),
            liveness_endpoint: "/live".to_string(),
            check_interval_secs: 30,
            timeout_secs: 5,
            include_details: true,
        }
    }
}

impl ObservabilityConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            metrics: MetricsConfig::from_env(),
            tracing: TracingConfig::from_env(),
            logging: LoggingConfig::from_env(),
            health: HealthConfig::from_env(),
        }
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate sampling rate
        if self.tracing.sampling_rate < 0.0 || self.tracing.sampling_rate > 1.0 {
            return Err("Tracing sampling rate must be between 0.0 and 1.0".to_string());
        }

        // Validate histogram buckets are sorted
        for i in 1..self.metrics.latency_buckets.len() {
            if self.metrics.latency_buckets[i] <= self.metrics.latency_buckets[i - 1] {
                return Err("Latency buckets must be in ascending order".to_string());
            }
        }

        for i in 1..self.metrics.cost_buckets.len() {
            if self.metrics.cost_buckets[i] <= self.metrics.cost_buckets[i - 1] {
                return Err("Cost buckets must be in ascending order".to_string());
            }
        }

        Ok(())
    }
}

impl MetricsConfig {
    /// Create from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("METRICS_ENABLED") {
            config.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var("METRICS_ENDPOINT") {
            config.endpoint = val;
        }

        if let Ok(val) = std::env::var("METRICS_PORT") {
            config.port = val.parse().ok();
        }

        if let Ok(val) = std::env::var("METRICS_PROCESS") {
            config.include_process_metrics = val.parse().unwrap_or(true);
        }

        config
    }
}

impl TracingConfig {
    /// Create from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("TRACING_ENABLED") {
            config.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var("RUST_LOG") {
            config.level = val;
        } else if let Ok(val) = std::env::var("TRACING_LEVEL") {
            config.level = val;
        }

        if let Ok(val) = std::env::var("TRACING_FORMAT") {
            config.format = match val.to_lowercase().as_str() {
                "json" => TracingFormat::Json,
                "pretty" => TracingFormat::Pretty,
                "compact" => TracingFormat::Compact,
                _ => TracingFormat::Text,
            };
        }

        if let Ok(val) = std::env::var("TRACING_ANSI") {
            config.ansi = val.parse().unwrap_or(true);
        }

        if let Ok(endpoint) = std::env::var("OTLP_ENDPOINT") {
            let mut otlp = OtlpConfig {
                endpoint,
                ..Default::default()
            };

            if let Ok(name) = std::env::var("OTEL_SERVICE_NAME") {
                otlp.service_name = name;
            }

            if let Ok(env) = std::env::var("ENVIRONMENT") {
                otlp.environment = env;
            }

            config.otlp = Some(otlp);
        }

        config
    }
}

impl LoggingConfig {
    /// Create from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("LOGGING_ENABLED") {
            config.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var("LOG_LEVEL") {
            config.level = val;
        }

        if let Ok(val) = std::env::var("LOG_FORMAT") {
            config.format = match val.to_lowercase().as_str() {
                "json" => LoggingFormat::Json,
                "logfmt" => LoggingFormat::Logfmt,
                _ => LoggingFormat::Text,
            };
        }

        if let Ok(path) = std::env::var("LOG_FILE") {
            config.file = Some(LogFileConfig {
                path,
                max_size_mb: std::env::var("LOG_FILE_MAX_SIZE_MB")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(100),
                max_files: std::env::var("LOG_FILE_MAX_FILES")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(10),
                compress: std::env::var("LOG_FILE_COMPRESS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(false),
            });
        }

        config
    }
}

impl HealthConfig {
    /// Create from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(val) = std::env::var("HEALTH_ENABLED") {
            config.enabled = val.parse().unwrap_or(true);
        }

        if let Ok(val) = std::env::var("HEALTH_ENDPOINT") {
            config.endpoint = val;
        }

        if let Ok(val) = std::env::var("HEALTH_CHECK_INTERVAL") {
            config.check_interval_secs = val.parse().unwrap_or(30);
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ObservabilityConfig::default();
        assert!(config.metrics.enabled);
        assert!(config.tracing.enabled);
        assert!(config.logging.enabled);
        assert!(config.health.enabled);
    }

    #[test]
    fn test_validate_sampling_rate() {
        let mut config = ObservabilityConfig::default();
        assert!(config.validate().is_ok());

        config.tracing.sampling_rate = 1.5;
        assert!(config.validate().is_err());

        config.tracing.sampling_rate = -0.1;
        assert!(config.validate().is_err());

        config.tracing.sampling_rate = 0.5;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_histogram_buckets() {
        let mut config = ObservabilityConfig::default();
        assert!(config.validate().is_ok());

        // Unsorted buckets
        config.metrics.latency_buckets = vec![1.0, 0.5, 2.0];
        assert!(config.validate().is_err());

        // Sorted buckets
        config.metrics.latency_buckets = vec![0.1, 0.5, 1.0, 2.0];
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_tracing_format() {
        let config = TracingConfig {
            format: TracingFormat::Json,
            ..Default::default()
        };
        assert_eq!(config.format, TracingFormat::Json);
    }

    #[test]
    fn test_otlp_config() {
        let otlp = OtlpConfig::default();
        assert_eq!(otlp.service_name, "llm-cost-ops");
        assert_eq!(otlp.environment, "development");
    }
}
