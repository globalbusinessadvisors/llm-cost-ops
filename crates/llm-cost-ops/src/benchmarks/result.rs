/// Canonical benchmark result structure for CostOps
///
/// This provides a standardized format for all benchmark results across the system,
/// enabling consistent reporting and analysis.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Canonical benchmark result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// Unique identifier for the benchmark
    pub id: String,

    /// Human-readable name of the benchmark
    pub name: String,

    /// Category/module this benchmark belongs to
    pub category: String,

    /// Total duration of the benchmark execution
    pub duration: Duration,

    /// Number of iterations/operations performed
    pub iterations: u64,

    /// Operations per second (throughput)
    pub ops_per_sec: f64,

    /// Average time per operation
    pub avg_time_per_op: Duration,

    /// Minimum execution time observed
    pub min_time: Duration,

    /// Maximum execution time observed
    pub max_time: Duration,

    /// Standard deviation of execution times
    pub std_dev: Duration,

    /// Additional metadata specific to the benchmark
    pub metadata: serde_json::Value,

    /// Timestamp when the benchmark was run
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Whether the benchmark passed any validation checks
    pub passed: bool,

    /// Optional error message if benchmark failed
    pub error: Option<String>,
}

impl BenchmarkResult {
    /// Create a new benchmark result
    pub fn new(id: String, name: String, category: String) -> Self {
        Self {
            id,
            name,
            category,
            duration: Duration::ZERO,
            iterations: 0,
            ops_per_sec: 0.0,
            avg_time_per_op: Duration::ZERO,
            min_time: Duration::MAX,
            max_time: Duration::ZERO,
            std_dev: Duration::ZERO,
            metadata: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
            passed: true,
            error: None,
        }
    }

    /// Create a successful result with timing data
    pub fn success(
        id: String,
        name: String,
        category: String,
        duration: Duration,
        iterations: u64,
    ) -> Self {
        let avg_time_per_op = if iterations > 0 {
            duration / iterations as u32
        } else {
            Duration::ZERO
        };

        let ops_per_sec = if duration.as_secs_f64() > 0.0 {
            iterations as f64 / duration.as_secs_f64()
        } else {
            0.0
        };

        Self {
            id,
            name,
            category,
            duration,
            iterations,
            ops_per_sec,
            avg_time_per_op,
            min_time: avg_time_per_op,
            max_time: avg_time_per_op,
            std_dev: Duration::ZERO,
            metadata: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
            passed: true,
            error: None,
        }
    }

    /// Create a failed result with error message
    pub fn failure(
        id: String,
        name: String,
        category: String,
        error: String,
    ) -> Self {
        Self {
            id,
            name,
            category,
            duration: Duration::ZERO,
            iterations: 0,
            ops_per_sec: 0.0,
            avg_time_per_op: Duration::ZERO,
            min_time: Duration::ZERO,
            max_time: Duration::ZERO,
            std_dev: Duration::ZERO,
            metadata: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
            passed: false,
            error: Some(error),
        }
    }

    /// Set detailed timing statistics
    pub fn with_stats(mut self, min: Duration, max: Duration, std_dev: Duration) -> Self {
        self.min_time = min;
        self.max_time = max;
        self.std_dev = std_dev;
        self
    }

    /// Add metadata to the result
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }

    /// Get formatted throughput string
    pub fn throughput_string(&self) -> String {
        if self.ops_per_sec >= 1_000_000.0 {
            format!("{:.2}M ops/sec", self.ops_per_sec / 1_000_000.0)
        } else if self.ops_per_sec >= 1_000.0 {
            format!("{:.2}K ops/sec", self.ops_per_sec / 1_000.0)
        } else {
            format!("{:.2} ops/sec", self.ops_per_sec)
        }
    }

    /// Get formatted average time string
    pub fn avg_time_string(&self) -> String {
        let nanos = self.avg_time_per_op.as_nanos();

        if nanos < 1_000 {
            format!("{} ns", nanos)
        } else if nanos < 1_000_000 {
            format!("{:.2} μs", nanos as f64 / 1_000.0)
        } else if nanos < 1_000_000_000 {
            format!("{:.2} ms", nanos as f64 / 1_000_000.0)
        } else {
            format!("{:.2} s", nanos as f64 / 1_000_000_000.0)
        }
    }
}

/// Summary of multiple benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    /// Total number of benchmarks run
    pub total_count: usize,

    /// Number of successful benchmarks
    pub passed_count: usize,

    /// Number of failed benchmarks
    pub failed_count: usize,

    /// Total execution time for all benchmarks
    pub total_duration: Duration,

    /// Results grouped by category
    pub by_category: std::collections::HashMap<String, Vec<BenchmarkResult>>,

    /// Timestamp of summary generation
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl BenchmarkSummary {
    /// Create a new summary from a list of results
    pub fn from_results(results: Vec<BenchmarkResult>) -> Self {
        let total_count = results.len();
        let passed_count = results.iter().filter(|r| r.passed).count();
        let failed_count = total_count - passed_count;

        let total_duration = results.iter()
            .map(|r| r.duration)
            .sum();

        let mut by_category = std::collections::HashMap::new();
        for result in results {
            by_category
                .entry(result.category.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        Self {
            total_count,
            passed_count,
            failed_count,
            total_duration,
            by_category,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            (self.passed_count as f64 / self.total_count as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult::success(
            "test-1".to_string(),
            "Test Benchmark".to_string(),
            "engine".to_string(),
            Duration::from_secs(1),
            1000,
        );

        assert!(result.passed);
        assert_eq!(result.iterations, 1000);
        assert!(result.ops_per_sec > 0.0);
    }

    #[test]
    fn test_benchmark_result_failure() {
        let result = BenchmarkResult::failure(
            "test-2".to_string(),
            "Failed Benchmark".to_string(),
            "engine".to_string(),
            "Test error".to_string(),
        );

        assert!(!result.passed);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_throughput_string() {
        let mut result = BenchmarkResult::new(
            "test".to_string(),
            "Test".to_string(),
            "test".to_string(),
        );

        result.ops_per_sec = 1_500_000.0;
        assert!(result.throughput_string().contains("M ops/sec"));

        result.ops_per_sec = 1_500.0;
        assert!(result.throughput_string().contains("K ops/sec"));

        result.ops_per_sec = 100.0;
        assert!(result.throughput_string().contains("ops/sec"));
    }

    #[test]
    fn test_summary_creation() {
        let results = vec![
            BenchmarkResult::success(
                "1".to_string(),
                "Test 1".to_string(),
                "engine".to_string(),
                Duration::from_secs(1),
                100,
            ),
            BenchmarkResult::failure(
                "2".to_string(),
                "Test 2".to_string(),
                "storage".to_string(),
                "Error".to_string(),
            ),
        ];

        let summary = BenchmarkSummary::from_results(results);

        assert_eq!(summary.total_count, 2);
        assert_eq!(summary.passed_count, 1);
        assert_eq!(summary.failed_count, 1);
        assert_eq!(summary.success_rate(), 50.0);
        assert_eq!(summary.by_category.len(), 2);
    }
}
