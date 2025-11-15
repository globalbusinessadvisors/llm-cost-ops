// Comprehensive metrics system with Prometheus

use prometheus::{
    Gauge, GaugeVec, Histogram, HistogramOpts, HistogramVec, Opts, Registry,
    TextEncoder, Encoder, IntCounter, IntCounterVec, IntGauge, IntGaugeVec,
};
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::config::MetricsConfig;

/// Error type for metrics operations
#[derive(Debug, thiserror::Error)]
pub enum MetricsError {
    #[error("Failed to register metric: {0}")]
    RegistrationError(String),

    #[error("Failed to record metric: {0}")]
    RecordError(String),

    #[error("Failed to export metrics: {0}")]
    ExportError(String),

    #[error("Metric not found: {0}")]
    NotFound(String),
}

/// Global metrics registry
pub struct MetricsRegistry {
    registry: Arc<Registry>,
    config: MetricsConfig,

    // Request metrics
    pub http_requests_total: IntCounterVec,
    pub http_request_duration_seconds: HistogramVec,
    pub http_request_size_bytes: HistogramVec,
    pub http_response_size_bytes: HistogramVec,

    // Cost metrics
    pub cost_calculations_total: IntCounterVec,
    pub cost_calculation_duration_seconds: HistogramVec,
    pub total_cost_calculated: HistogramVec,
    pub cost_by_provider: GaugeVec,
    pub cost_by_model: GaugeVec,

    // Usage metrics
    pub token_usage_total: IntCounterVec,
    pub requests_by_model: IntCounterVec,
    pub requests_by_provider: IntCounterVec,

    // Database metrics
    pub db_queries_total: IntCounterVec,
    pub db_query_duration_seconds: HistogramVec,
    pub db_connections_active: IntGauge,
    pub db_connections_idle: IntGauge,

    // Cache metrics
    pub cache_hits_total: IntCounterVec,
    pub cache_misses_total: IntCounterVec,
    pub cache_size_bytes: IntGauge,
    pub cache_evictions_total: IntCounter,

    // Authentication metrics
    pub auth_attempts_total: IntCounterVec,
    pub auth_failures_total: IntCounterVec,
    pub active_sessions: IntGauge,
    pub api_key_usage: IntCounterVec,

    // RBAC metrics
    pub permission_checks_total: IntCounterVec,
    pub access_denied_total: IntCounterVec,

    // Ingestion metrics
    pub ingestion_requests_total: IntCounterVec,
    pub ingestion_duration_seconds: HistogramVec,
    pub ingestion_errors_total: IntCounterVec,
    pub webhook_payloads_received: IntCounterVec,

    // DLQ metrics
    pub dlq_items_total: IntGaugeVec,
    pub dlq_retry_attempts_total: IntCounterVec,
    pub dlq_success_total: IntCounterVec,
    pub dlq_failures_total: IntCounterVec,

    // Forecasting metrics
    pub forecast_requests_total: IntCounterVec,
    pub forecast_duration_seconds: HistogramVec,
    pub forecast_accuracy: GaugeVec,
    pub anomalies_detected_total: IntCounterVec,

    // System metrics
    pub uptime_seconds: Gauge,
    pub build_info: IntCounterVec,
}

impl MetricsRegistry {
    /// Create a new metrics registry
    pub fn new(config: MetricsConfig) -> Result<Self, MetricsError> {
        let registry = Registry::new();

        // HTTP metrics
        let http_requests_total = IntCounterVec::new(
            Opts::new("http_requests_total", "Total HTTP requests")
                .namespace("llm_cost_ops"),
            &["method", "path", "status"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(http_requests_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let http_request_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "http_request_duration_seconds",
                "HTTP request duration in seconds",
            )
            .namespace("llm_cost_ops")
            .buckets(config.latency_buckets.clone()),
            &["method", "path"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(http_request_duration_seconds.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let http_request_size_bytes = HistogramVec::new(
            HistogramOpts::new("http_request_size_bytes", "HTTP request size in bytes")
                .namespace("llm_cost_ops")
                .buckets(vec![
                    100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 10000000.0,
                ]),
            &["method", "path"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(http_request_size_bytes.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let http_response_size_bytes = HistogramVec::new(
            HistogramOpts::new("http_response_size_bytes", "HTTP response size in bytes")
                .namespace("llm_cost_ops")
                .buckets(vec![
                    100.0, 1000.0, 10000.0, 100000.0, 1000000.0, 10000000.0,
                ]),
            &["method", "path"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(http_response_size_bytes.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // Cost metrics
        let cost_calculations_total = IntCounterVec::new(
            Opts::new("cost_calculations_total", "Total cost calculations")
                .namespace("llm_cost_ops"),
            &["provider", "model"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(cost_calculations_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let cost_calculation_duration_seconds = HistogramVec::new(
            HistogramOpts::new(
                "cost_calculation_duration_seconds",
                "Cost calculation duration",
            )
            .namespace("llm_cost_ops")
            .buckets(config.latency_buckets.clone()),
            &["provider"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(cost_calculation_duration_seconds.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let total_cost_calculated = HistogramVec::new(
            HistogramOpts::new("total_cost_calculated", "Total cost calculated in USD")
                .namespace("llm_cost_ops")
                .buckets(config.cost_buckets.clone()),
            &["provider", "model", "organization"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(total_cost_calculated.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let cost_by_provider = GaugeVec::new(
            Opts::new("cost_by_provider", "Current cost by provider")
                .namespace("llm_cost_ops"),
            &["provider", "organization"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(cost_by_provider.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let cost_by_model = GaugeVec::new(
            Opts::new("cost_by_model", "Current cost by model").namespace("llm_cost_ops"),
            &["provider", "model", "organization"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(cost_by_model.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // Usage metrics
        let token_usage_total = IntCounterVec::new(
            Opts::new("token_usage_total", "Total tokens used").namespace("llm_cost_ops"),
            &["provider", "model", "token_type", "organization"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(token_usage_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let requests_by_model = IntCounterVec::new(
            Opts::new("requests_by_model", "Requests by model").namespace("llm_cost_ops"),
            &["provider", "model", "organization"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(requests_by_model.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let requests_by_provider = IntCounterVec::new(
            Opts::new("requests_by_provider", "Requests by provider")
                .namespace("llm_cost_ops"),
            &["provider", "organization"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(requests_by_provider.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // Database metrics
        let db_queries_total = IntCounterVec::new(
            Opts::new("db_queries_total", "Total database queries")
                .namespace("llm_cost_ops"),
            &["operation", "table", "status"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(db_queries_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let db_query_duration_seconds = HistogramVec::new(
            HistogramOpts::new("db_query_duration_seconds", "Database query duration")
                .namespace("llm_cost_ops")
                .buckets(config.latency_buckets.clone()),
            &["operation", "table"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(db_query_duration_seconds.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let db_connections_active = IntGauge::new(
            "llm_cost_ops_db_connections_active",
            "Active database connections",
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(db_connections_active.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let db_connections_idle = IntGauge::new(
            "llm_cost_ops_db_connections_idle",
            "Idle database connections",
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(db_connections_idle.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // Cache metrics
        let cache_hits_total = IntCounterVec::new(
            Opts::new("cache_hits_total", "Total cache hits").namespace("llm_cost_ops"),
            &["cache_name"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(cache_hits_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let cache_misses_total = IntCounterVec::new(
            Opts::new("cache_misses_total", "Total cache misses").namespace("llm_cost_ops"),
            &["cache_name"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(cache_misses_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let cache_size_bytes =
            IntGauge::new("llm_cost_ops_cache_size_bytes", "Cache size in bytes")
                .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(cache_size_bytes.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let cache_evictions_total = IntCounter::new(
            "llm_cost_ops_cache_evictions_total",
            "Total cache evictions",
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(cache_evictions_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // Authentication metrics
        let auth_attempts_total = IntCounterVec::new(
            Opts::new("auth_attempts_total", "Total authentication attempts")
                .namespace("llm_cost_ops"),
            &["method", "status"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(auth_attempts_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let auth_failures_total = IntCounterVec::new(
            Opts::new("auth_failures_total", "Total authentication failures")
                .namespace("llm_cost_ops"),
            &["method", "reason"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(auth_failures_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let active_sessions =
            IntGauge::new("llm_cost_ops_active_sessions", "Active user sessions")
                .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(active_sessions.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let api_key_usage = IntCounterVec::new(
            Opts::new("api_key_usage", "API key usage").namespace("llm_cost_ops"),
            &["key_id", "organization"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(api_key_usage.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // RBAC metrics
        let permission_checks_total = IntCounterVec::new(
            Opts::new("permission_checks_total", "Total permission checks")
                .namespace("llm_cost_ops"),
            &["resource", "action", "result"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(permission_checks_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let access_denied_total = IntCounterVec::new(
            Opts::new("access_denied_total", "Total access denied").namespace("llm_cost_ops"),
            &["resource", "action", "user"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(access_denied_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // Ingestion metrics
        let ingestion_requests_total = IntCounterVec::new(
            Opts::new("ingestion_requests_total", "Total ingestion requests")
                .namespace("llm_cost_ops"),
            &["source", "status"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(ingestion_requests_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let ingestion_duration_seconds = HistogramVec::new(
            HistogramOpts::new("ingestion_duration_seconds", "Ingestion duration")
                .namespace("llm_cost_ops")
                .buckets(config.latency_buckets.clone()),
            &["source"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(ingestion_duration_seconds.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let ingestion_errors_total = IntCounterVec::new(
            Opts::new("ingestion_errors_total", "Total ingestion errors")
                .namespace("llm_cost_ops"),
            &["source", "error_type"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(ingestion_errors_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let webhook_payloads_received = IntCounterVec::new(
            Opts::new("webhook_payloads_received", "Webhook payloads received")
                .namespace("llm_cost_ops"),
            &["provider", "organization"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(webhook_payloads_received.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // DLQ metrics
        let dlq_items_total = IntGaugeVec::new(
            Opts::new("dlq_items_total", "Total items in DLQ").namespace("llm_cost_ops"),
            &["status"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(dlq_items_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let dlq_retry_attempts_total = IntCounterVec::new(
            Opts::new("dlq_retry_attempts_total", "Total DLQ retry attempts")
                .namespace("llm_cost_ops"),
            &["attempt"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(dlq_retry_attempts_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let dlq_success_total = IntCounterVec::new(
            Opts::new("dlq_success_total", "Total DLQ successes").namespace("llm_cost_ops"),
            &["source"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(dlq_success_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let dlq_failures_total = IntCounterVec::new(
            Opts::new("dlq_failures_total", "Total DLQ failures").namespace("llm_cost_ops"),
            &["source", "reason"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(dlq_failures_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // Forecasting metrics
        let forecast_requests_total = IntCounterVec::new(
            Opts::new("forecast_requests_total", "Total forecast requests")
                .namespace("llm_cost_ops"),
            &["model", "horizon"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(forecast_requests_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let forecast_duration_seconds = HistogramVec::new(
            HistogramOpts::new("forecast_duration_seconds", "Forecast duration")
                .namespace("llm_cost_ops")
                .buckets(config.latency_buckets.clone()),
            &["model"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(forecast_duration_seconds.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let forecast_accuracy = GaugeVec::new(
            Opts::new("forecast_accuracy", "Forecast accuracy").namespace("llm_cost_ops"),
            &["model", "metric"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(forecast_accuracy.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let anomalies_detected_total = IntCounterVec::new(
            Opts::new("anomalies_detected_total", "Total anomalies detected")
                .namespace("llm_cost_ops"),
            &["method", "severity"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(anomalies_detected_total.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // System metrics
        let uptime_seconds = Gauge::new("llm_cost_ops_uptime_seconds", "System uptime in seconds")
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(uptime_seconds.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        let build_info = IntCounterVec::new(
            Opts::new("build_info", "Build information").namespace("llm_cost_ops"),
            &["version", "rustc_version"],
        )
        .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        registry
            .register(Box::new(build_info.clone()))
            .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;

        // Set build info
        build_info
            .with_label_values(&[env!("CARGO_PKG_VERSION"), "unknown"])
            .inc();

        // Register process metrics if enabled
        if config.include_process_metrics {
            let process_collector = prometheus::process_collector::ProcessCollector::for_self();
            registry
                .register(Box::new(process_collector))
                .map_err(|e| MetricsError::RegistrationError(e.to_string()))?;
        }

        Ok(Self {
            registry: Arc::new(registry),
            config,
            http_requests_total,
            http_request_duration_seconds,
            http_request_size_bytes,
            http_response_size_bytes,
            cost_calculations_total,
            cost_calculation_duration_seconds,
            total_cost_calculated,
            cost_by_provider,
            cost_by_model,
            token_usage_total,
            requests_by_model,
            requests_by_provider,
            db_queries_total,
            db_query_duration_seconds,
            db_connections_active,
            db_connections_idle,
            cache_hits_total,
            cache_misses_total,
            cache_size_bytes,
            cache_evictions_total,
            auth_attempts_total,
            auth_failures_total,
            active_sessions,
            api_key_usage,
            permission_checks_total,
            access_denied_total,
            ingestion_requests_total,
            ingestion_duration_seconds,
            ingestion_errors_total,
            webhook_payloads_received,
            dlq_items_total,
            dlq_retry_attempts_total,
            dlq_success_total,
            dlq_failures_total,
            forecast_requests_total,
            forecast_duration_seconds,
            forecast_accuracy,
            anomalies_detected_total,
            uptime_seconds,
            build_info,
        })
    }

    /// Export metrics in Prometheus text format
    pub fn export(&self) -> Result<String, MetricsError> {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        let mut buffer = Vec::new();
        encoder
            .encode(&metric_families, &mut buffer)
            .map_err(|e| MetricsError::ExportError(e.to_string()))?;

        String::from_utf8(buffer).map_err(|e| MetricsError::ExportError(e.to_string()))
    }

    /// Get the underlying registry
    pub fn registry(&self) -> Arc<Registry> {
        self.registry.clone()
    }
}

/// Timer for measuring operation duration
pub struct Timer {
    start: Instant,
    histogram: Histogram,
}

impl Timer {
    /// Create a new timer
    pub fn new(histogram: Histogram) -> Self {
        Self {
            start: Instant::now(),
            histogram,
        }
    }

    /// Observe the elapsed time and record it
    pub fn observe_duration(self) -> Duration {
        let duration = self.start.elapsed();
        self.histogram.observe(duration.as_secs_f64());
        duration
    }
}

/// Helper to create a timer from a histogram
pub fn start_timer(histogram: &Histogram) -> Timer {
    Timer::new(histogram.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_registry_creation() {
        let config = MetricsConfig::default();
        let registry = MetricsRegistry::new(config);
        assert!(registry.is_ok());
    }

    #[test]
    fn test_metrics_export() {
        let config = MetricsConfig::default();
        let registry = MetricsRegistry::new(config).unwrap();

        // Record some metrics
        registry
            .http_requests_total
            .with_label_values(&["GET", "/api/v1/usage", "200"])
            .inc();

        let exported = registry.export();
        assert!(exported.is_ok());

        let output = exported.unwrap();
        assert!(output.contains("llm_cost_ops_http_requests_total"));
    }

    #[test]
    fn test_cost_metrics() {
        let config = MetricsConfig::default();
        let registry = MetricsRegistry::new(config).unwrap();

        // Record cost calculation
        registry
            .cost_calculations_total
            .with_label_values(&["openai", "gpt-4"])
            .inc();

        registry
            .total_cost_calculated
            .with_label_values(&["openai", "gpt-4", "org1"])
            .observe(1.50);

        let exported = registry.export().unwrap();
        assert!(exported.contains("llm_cost_ops_cost_calculations_total"));
        assert!(exported.contains("llm_cost_ops_total_cost_calculated"));
    }

    #[test]
    fn test_timer() {
        let config = MetricsConfig::default();
        let registry = MetricsRegistry::new(config).unwrap();

        let histogram = registry
            .http_request_duration_seconds
            .with_label_values(&["GET", "/test"]);

        let timer = start_timer(&histogram);
        std::thread::sleep(Duration::from_millis(10));
        let duration = timer.observe_duration();

        assert!(duration.as_millis() >= 10);
    }

    #[test]
    fn test_auth_metrics() {
        let config = MetricsConfig::default();
        let registry = MetricsRegistry::new(config).unwrap();

        registry
            .auth_attempts_total
            .with_label_values(&["jwt", "success"])
            .inc();

        registry
            .auth_failures_total
            .with_label_values(&["api_key", "invalid"])
            .inc();

        registry.active_sessions.set(42);

        let exported = registry.export().unwrap();
        assert!(exported.contains("llm_cost_ops_auth_attempts_total"));
        assert!(exported.contains("llm_cost_ops_active_sessions"));
    }

    #[test]
    fn test_database_metrics() {
        let config = MetricsConfig::default();
        let registry = MetricsRegistry::new(config).unwrap();

        registry
            .db_queries_total
            .with_label_values(&["select", "usage_records", "success"])
            .inc();

        registry.db_connections_active.set(10);
        registry.db_connections_idle.set(5);

        let exported = registry.export().unwrap();
        assert!(exported.contains("llm_cost_ops_db_queries_total"));
        assert!(exported.contains("llm_cost_ops_db_connections_active"));
    }

    #[test]
    fn test_cache_metrics() {
        let config = MetricsConfig::default();
        let registry = MetricsRegistry::new(config).unwrap();

        registry
            .cache_hits_total
            .with_label_values(&["pricing"])
            .inc();

        registry
            .cache_misses_total
            .with_label_values(&["pricing"])
            .inc();

        registry.cache_size_bytes.set(1024 * 1024);
        registry.cache_evictions_total.inc();

        let exported = registry.export().unwrap();
        assert!(exported.contains("llm_cost_ops_cache_hits_total"));
    }

    #[test]
    fn test_ingestion_metrics() {
        let config = MetricsConfig::default();
        let registry = MetricsRegistry::new(config).unwrap();

        registry
            .ingestion_requests_total
            .with_label_values(&["webhook", "success"])
            .inc();

        registry
            .webhook_payloads_received
            .with_label_values(&["openai", "org1"])
            .inc();

        let exported = registry.export().unwrap();
        assert!(exported.contains("llm_cost_ops_ingestion_requests_total"));
    }

    #[test]
    fn test_forecast_metrics() {
        let config = MetricsConfig::default();
        let registry = MetricsRegistry::new(config).unwrap();

        registry
            .forecast_requests_total
            .with_label_values(&["linear_trend", "7days"])
            .inc();

        registry
            .forecast_accuracy
            .with_label_values(&["linear_trend", "mae"])
            .set(0.95);

        registry
            .anomalies_detected_total
            .with_label_values(&["zscore", "high"])
            .inc();

        let exported = registry.export().unwrap();
        assert!(exported.contains("llm_cost_ops_forecast_requests_total"));
        assert!(exported.contains("llm_cost_ops_anomalies_detected_total"));
    }
}
