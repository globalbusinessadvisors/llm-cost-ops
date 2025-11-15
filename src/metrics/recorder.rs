// Prometheus metrics recorder initialization

use metrics_exporter_prometheus::PrometheusBuilder;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::domain::{CostOpsError, Result};

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable metrics collection
    pub enabled: bool,

    /// Prometheus exporter bind address
    pub prometheus_addr: String,

    /// Metrics export interval in seconds
    pub export_interval_secs: u64,

    /// Include detailed labels (may increase cardinality)
    pub detailed_labels: bool,

    /// Histogram buckets for latency metrics (milliseconds)
    pub latency_buckets: Vec<f64>,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            prometheus_addr: "0.0.0.0:9090".to_string(),
            export_interval_secs: 15,
            detailed_labels: false,
            latency_buckets: vec![
                1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0, 5000.0,
            ],
        }
    }
}

impl MetricsConfig {
    /// Create a production configuration
    pub fn production() -> Self {
        Self {
            enabled: true,
            prometheus_addr: "0.0.0.0:9090".to_string(),
            export_interval_secs: 15,
            detailed_labels: false,
            latency_buckets: vec![
                1.0, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 5000.0,
            ],
        }
    }

    /// Create a development configuration with more detailed metrics
    pub fn development() -> Self {
        Self {
            enabled: true,
            prometheus_addr: "127.0.0.1:9090".to_string(),
            export_interval_secs: 5,
            detailed_labels: true,
            latency_buckets: vec![
                0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0, 2500.0,
                5000.0, 10000.0,
            ],
        }
    }

    /// Create a disabled configuration for testing
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            prometheus_addr: "127.0.0.1:9090".to_string(),
            export_interval_secs: 60,
            detailed_labels: false,
            latency_buckets: vec![],
        }
    }
}

/// Initialize Prometheus metrics exporter
pub fn init_metrics(config: &MetricsConfig) -> Result<()> {
    if !config.enabled {
        tracing::info!("Metrics collection is disabled");
        return Ok(());
    }

    let addr: SocketAddr = config
        .prometheus_addr
        .parse()
        .map_err(|e| CostOpsError::Integration(format!("Invalid Prometheus address: {}", e)))?;

    tracing::info!(
        addr = %addr,
        interval_secs = config.export_interval_secs,
        "Initializing Prometheus metrics exporter"
    );

    let builder = PrometheusBuilder::new();

    // Install the recorder
    builder
        .with_http_listener(addr)
        .install()
        .map_err(|e| CostOpsError::Integration(format!("Failed to install Prometheus exporter: {}", e)))?;

    tracing::info!("Prometheus metrics exporter initialized successfully");

    // Register application info metric
    metrics::counter!("llm_cost_ops_info", 1);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_config_default() {
        let config = MetricsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.prometheus_addr, "0.0.0.0:9090");
        assert_eq!(config.export_interval_secs, 15);
        assert!(!config.detailed_labels);
        assert!(!config.latency_buckets.is_empty());
    }

    #[test]
    fn test_metrics_config_production() {
        let config = MetricsConfig::production();
        assert!(config.enabled);
        assert!(!config.detailed_labels);
    }

    #[test]
    fn test_metrics_config_development() {
        let config = MetricsConfig::development();
        assert!(config.enabled);
        assert!(config.detailed_labels);
        assert_eq!(config.export_interval_secs, 5);
    }

    #[test]
    fn test_metrics_config_disabled() {
        let config = MetricsConfig::disabled();
        assert!(!config.enabled);
    }
}
