// Compression metrics and monitoring

use super::{CompressionAlgorithm, CompressionStats};
use prometheus::{
    register_histogram_vec, register_int_counter_vec, register_int_gauge_vec, HistogramVec,
    IntCounterVec, IntGaugeVec,
};
use std::sync::Arc;
use tracing::{debug, info};

lazy_static::lazy_static! {
    /// Compression operations counter
    static ref COMPRESSION_OPERATIONS: IntCounterVec = register_int_counter_vec!(
        "compression_operations_total",
        "Total number of compression operations",
        &["algorithm", "operation", "status"]
    )
    .unwrap();

    /// Compression ratio histogram
    static ref COMPRESSION_RATIO: HistogramVec = register_histogram_vec!(
        "compression_ratio",
        "Compression ratio (compressed/original)",
        &["algorithm"],
        vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
    )
    .unwrap();

    /// Compression duration histogram
    static ref COMPRESSION_DURATION: HistogramVec = register_histogram_vec!(
        "compression_duration_milliseconds",
        "Compression duration in milliseconds",
        &["algorithm", "operation"],
        vec![0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 25.0, 50.0, 100.0, 250.0, 500.0, 1000.0]
    )
    .unwrap();

    /// Bytes processed counter
    static ref BYTES_PROCESSED: IntCounterVec = register_int_counter_vec!(
        "compression_bytes_processed_total",
        "Total bytes processed",
        &["algorithm", "type"]
    )
    .unwrap();

    /// Bytes saved gauge
    static ref BYTES_SAVED: IntGaugeVec = register_int_gauge_vec!(
        "compression_bytes_saved",
        "Bytes saved by compression",
        &["algorithm"]
    )
    .unwrap();
}

/// Compression metrics collector
#[derive(Debug, Clone)]
pub struct CompressionMetrics {
    enabled: bool,
}

impl CompressionMetrics {
    /// Create a new metrics collector
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Record compression operation
    pub fn record_compression(&self, stats: &CompressionStats) {
        if !self.enabled {
            return;
        }

        let algorithm = stats
            .algorithm
            .map(|a| a.as_str())
            .unwrap_or("unknown");

        // Increment operation counter
        COMPRESSION_OPERATIONS
            .with_label_values(&[algorithm, "compress", "success"])
            .inc();

        // Record compression ratio
        COMPRESSION_RATIO
            .with_label_values(&[algorithm])
            .observe(stats.compression_ratio);

        // Record duration
        COMPRESSION_DURATION
            .with_label_values(&[algorithm, "compress"])
            .observe(stats.duration_ms);

        // Record bytes processed
        BYTES_PROCESSED
            .with_label_values(&[algorithm, "original"])
            .inc_by(stats.original_size as u64);

        BYTES_PROCESSED
            .with_label_values(&[algorithm, "compressed"])
            .inc_by(stats.compressed_size as u64);

        // Update bytes saved
        BYTES_SAVED
            .with_label_values(&[algorithm])
            .add(stats.bytes_saved as i64);

        debug!(
            algorithm = algorithm,
            original_size = stats.original_size,
            compressed_size = stats.compressed_size,
            ratio = stats.compression_ratio,
            duration_ms = stats.duration_ms,
            "Compression completed"
        );

        // Log significant compression
        if stats.bytes_saved > 10_000 {
            info!(
                algorithm = algorithm,
                bytes_saved = stats.bytes_saved,
                compression_pct = stats.compression_percentage(),
                "Significant compression achieved"
            );
        }
    }

    /// Record decompression operation
    pub fn record_decompression(&self, stats: &CompressionStats) {
        if !self.enabled {
            return;
        }

        let algorithm = stats
            .algorithm
            .map(|a| a.as_str())
            .unwrap_or("unknown");

        // Increment operation counter
        COMPRESSION_OPERATIONS
            .with_label_values(&[algorithm, "decompress", "success"])
            .inc();

        // Record duration
        COMPRESSION_DURATION
            .with_label_values(&[algorithm, "decompress"])
            .observe(stats.duration_ms);

        debug!(
            algorithm = algorithm,
            compressed_size = stats.compressed_size,
            decompressed_size = stats.original_size,
            duration_ms = stats.duration_ms,
            "Decompression completed"
        );
    }

    /// Record compression error
    pub fn record_error(&self, algorithm: Option<CompressionAlgorithm>, operation: &str) {
        if !self.enabled {
            return;
        }

        let algorithm = algorithm
            .map(|a| a.as_str())
            .unwrap_or("unknown");

        COMPRESSION_OPERATIONS
            .with_label_values(&[algorithm, operation, "error"])
            .inc();
    }
}

impl Default for CompressionMetrics {
    fn default() -> Self {
        Self::new(true)
    }
}

/// Global metrics instance
static GLOBAL_METRICS: once_cell::sync::Lazy<Arc<CompressionMetrics>> =
    once_cell::sync::Lazy::new(|| Arc::new(CompressionMetrics::default()));

/// Get global compression metrics instance
pub fn get_metrics() -> Arc<CompressionMetrics> {
    GLOBAL_METRICS.clone()
}

/// Initialize compression metrics
pub fn init_compression_metrics(enabled: bool) -> Arc<CompressionMetrics> {
    Arc::new(CompressionMetrics::new(enabled))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_metrics_creation() {
        let metrics = CompressionMetrics::new(true);
        assert!(metrics.enabled);

        let metrics = CompressionMetrics::new(false);
        assert!(!metrics.enabled);
    }

    #[test]
    fn test_record_compression() {
        let metrics = CompressionMetrics::new(true);
        let stats = CompressionStats::new(1000, 300, CompressionAlgorithm::Gzip, 10.5);

        metrics.record_compression(&stats);
        // Should not panic
    }

    #[test]
    fn test_record_decompression() {
        let metrics = CompressionMetrics::new(true);
        let stats = CompressionStats::new(1000, 300, CompressionAlgorithm::Gzip, 5.0);

        metrics.record_decompression(&stats);
        // Should not panic
    }

    #[test]
    fn test_record_error() {
        let metrics = CompressionMetrics::new(true);
        metrics.record_error(Some(CompressionAlgorithm::Gzip), "compress");
        // Should not panic
    }

    #[test]
    fn test_disabled_metrics() {
        let metrics = CompressionMetrics::new(false);
        let stats = CompressionStats::new(1000, 300, CompressionAlgorithm::Gzip, 10.5);

        // Should be no-ops
        metrics.record_compression(&stats);
        metrics.record_decompression(&stats);
        metrics.record_error(Some(CompressionAlgorithm::Gzip), "compress");
    }

    #[test]
    fn test_get_metrics() {
        let metrics = get_metrics();
        assert!(metrics.enabled);
    }
}
