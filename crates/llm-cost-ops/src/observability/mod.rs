// Observability stack - metrics, tracing, logging, and health checks

pub mod config;
pub mod metrics;
pub mod tracing;
pub mod logging;
pub mod health;

// Re-export commonly used types
pub use config::{
    ObservabilityConfig,
    MetricsConfig as ObservabilityMetricsConfig,
    TracingConfig, TracingFormat,
    LoggingConfig, LoggingFormat, HealthConfig, OtlpConfig,
};

pub use metrics::{
    MetricsRegistry, MetricsError, Timer, start_timer,
};

pub use tracing::{
    CorrelationId, RequestId, TraceContext,
    init_tracing as init_tracing_with_config, create_span_with_context,
    info_span_with_context, debug_span_with_context, trace_span_with_context,
    warn_span_with_context, error_span_with_context,
    extract_trace_context_from_headers, inject_trace_context_into_headers,
    TraceContextLayer,
};

pub use logging::{
    LogLevel, LogEntry, StructuredLogger, PerformanceLogger,
};

pub use health::{
    HealthStatus, ComponentHealth, SystemHealth,
    HealthCheck, HealthChecker,
    DatabaseHealthCheck, CacheHealthCheck, ExternalServiceHealthCheck,
    FunctionHealthCheck,
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize basic tracing (legacy function for backward compatibility)
pub fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Initialize JSON tracing (legacy function for backward compatibility)
pub fn init_tracing_json() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .init();
}

/// Initialize observability stack with configuration
pub fn init_observability(config: &ObservabilityConfig) -> Result<ObservabilityStack, String> {
    ObservabilityStack::init(config)
}

/// Complete observability stack
pub struct ObservabilityStack {
    pub metrics: Option<MetricsRegistry>,
    pub health: Option<HealthChecker>,
    config: ObservabilityConfig,
}

impl ObservabilityStack {
    /// Initialize the observability stack
    pub fn init(config: &ObservabilityConfig) -> Result<Self, String> {
        // Validate configuration
        config.validate()?;

        // Initialize tracing
        if config.tracing.enabled {
            tracing::init_tracing(&config.tracing)?;
        }

        // Initialize metrics
        let metrics = if config.metrics.enabled {
            Some(
                metrics::MetricsRegistry::new(config.metrics.clone())
                    .map_err(|e| format!("Failed to initialize metrics: {}", e))?,
            )
        } else {
            None
        };

        // Initialize health checker
        let health = if config.health.enabled {
            Some(HealthChecker::new(config.health.clone()))
        } else {
            None
        };

        Ok(Self {
            metrics,
            health,
            config: config.clone(),
        })
    }

    /// Get metrics registry
    pub fn metrics(&self) -> Option<&MetricsRegistry> {
        self.metrics.as_ref()
    }

    /// Get health checker
    pub fn health(&self) -> Option<&HealthChecker> {
        self.health.as_ref()
    }

    /// Get configuration
    pub fn config(&self) -> &ObservabilityConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observability_stack_init() {
        let config = ObservabilityConfig::default();
        let stack = ObservabilityStack::init(&config);
        assert!(stack.is_ok());

        let obs = stack.unwrap();
        assert!(obs.metrics.is_some());
        assert!(obs.health.is_some());
    }

    #[test]
    fn test_observability_stack_disabled() {
        let mut config = ObservabilityConfig::default();
        config.metrics.enabled = false;
        config.health.enabled = false;

        let stack = ObservabilityStack::init(&config).unwrap();
        assert!(stack.metrics.is_none());
        assert!(stack.health.is_none());
    }
}
