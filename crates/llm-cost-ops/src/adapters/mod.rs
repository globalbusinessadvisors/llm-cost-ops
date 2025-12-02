/// Benchmark adapters for CostOps operations
///
/// This module provides a standardized interface for benchmarking different
/// components of the CostOps system through the BenchTarget trait.

pub mod engine_adapters;
pub mod compression_adapters;
pub mod forecasting_adapters;

use crate::benchmarks::result::BenchmarkResult;
use std::time::{Duration, Instant};

/// Trait for benchmarkable targets
pub trait BenchTarget: Send + Sync {
    /// Unique identifier for this benchmark target
    fn id(&self) -> String;

    /// Human-readable name of the benchmark
    fn name(&self) -> String;

    /// Category this benchmark belongs to
    fn category(&self) -> String;

    /// Run the benchmark and return the result
    fn run(&self) -> BenchmarkResult;

    /// Optional setup before running the benchmark
    fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }

    /// Optional teardown after running the benchmark
    fn teardown(&self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

/// Helper function to time an operation
pub fn time_operation<F, R>(f: F) -> (Duration, R)
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let result = f();
    let duration = start.elapsed();
    (duration, result)
}

/// Helper function to run a benchmark with iterations
pub fn run_iterations<F>(iterations: u64, mut operation: F) -> (Duration, Vec<Duration>)
where
    F: FnMut(),
{
    let mut timings = Vec::with_capacity(iterations as usize);
    let start = Instant::now();

    for _ in 0..iterations {
        let iter_start = Instant::now();
        operation();
        timings.push(iter_start.elapsed());
    }

    let total_duration = start.elapsed();
    (total_duration, timings)
}

/// Calculate statistics from a list of durations
pub fn calculate_stats(timings: &[Duration]) -> (Duration, Duration, Duration) {
    if timings.is_empty() {
        return (Duration::ZERO, Duration::ZERO, Duration::ZERO);
    }

    let min = *timings.iter().min().unwrap();
    let max = *timings.iter().max().unwrap();

    // Calculate standard deviation
    let mean_nanos: f64 = timings.iter().map(|d| d.as_nanos() as f64).sum::<f64>()
        / timings.len() as f64;

    let variance: f64 = timings.iter()
        .map(|d| {
            let diff = d.as_nanos() as f64 - mean_nanos;
            diff * diff
        })
        .sum::<f64>() / timings.len() as f64;

    let std_dev = Duration::from_nanos(variance.sqrt() as u64);

    (min, max, std_dev)
}

/// Registry of all available benchmark targets
pub struct BenchmarkRegistry {
    targets: Vec<Box<dyn BenchTarget>>,
}

impl BenchmarkRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
        }
    }

    /// Add a benchmark target to the registry
    pub fn register(&mut self, target: Box<dyn BenchTarget>) {
        self.targets.push(target);
    }

    /// Get all registered targets
    pub fn targets(&self) -> &[Box<dyn BenchTarget>] {
        &self.targets
    }

    /// Get targets filtered by category
    pub fn targets_by_category(&self, category: &str) -> Vec<&Box<dyn BenchTarget>> {
        self.targets.iter()
            .filter(|t| t.category() == category)
            .collect()
    }

    /// Run all benchmarks and collect results
    pub fn run_all(&self) -> Vec<BenchmarkResult> {
        self.targets.iter()
            .map(|target| {
                // Run setup
                if let Err(e) = target.setup() {
                    return BenchmarkResult::failure(
                        target.id(),
                        target.name(),
                        target.category(),
                        format!("Setup failed: {}", e),
                    );
                }

                // Run benchmark
                let result = target.run();

                // Run teardown
                if let Err(e) = target.teardown() {
                    eprintln!("Warning: Teardown failed for {}: {}", target.id(), e);
                }

                result
            })
            .collect()
    }

    /// Run benchmarks for a specific category
    pub fn run_category(&self, category: &str) -> Vec<BenchmarkResult> {
        self.targets_by_category(category).iter()
            .map(|target| {
                // Run setup
                if let Err(e) = target.setup() {
                    return BenchmarkResult::failure(
                        target.id(),
                        target.name(),
                        target.category(),
                        format!("Setup failed: {}", e),
                    );
                }

                // Run benchmark
                let result = target.run();

                // Run teardown
                if let Err(e) = target.teardown() {
                    eprintln!("Warning: Teardown failed for {}: {}", target.id(), e);
                }

                result
            })
            .collect()
    }
}

impl Default for BenchmarkRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Get all available benchmark targets
pub fn all_targets() -> BenchmarkRegistry {
    let mut registry = BenchmarkRegistry::new();

    // Register engine benchmarks
    for target in engine_adapters::create_targets() {
        registry.register(target);
    }

    // Register compression benchmarks
    for target in compression_adapters::create_targets() {
        registry.register(target);
    }

    // Register forecasting benchmarks
    for target in forecasting_adapters::create_targets() {
        registry.register(target);
    }

    registry
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestBenchmark {
        id: String,
        should_fail: bool,
    }

    impl BenchTarget for TestBenchmark {
        fn id(&self) -> String {
            self.id.clone()
        }

        fn name(&self) -> String {
            "Test Benchmark".to_string()
        }

        fn category(&self) -> String {
            "test".to_string()
        }

        fn run(&self) -> BenchmarkResult {
            if self.should_fail {
                BenchmarkResult::failure(
                    self.id(),
                    self.name(),
                    self.category(),
                    "Test failure".to_string(),
                )
            } else {
                BenchmarkResult::success(
                    self.id(),
                    self.name(),
                    self.category(),
                    Duration::from_secs(1),
                    100,
                )
            }
        }
    }

    #[test]
    fn test_registry_creation() {
        let registry = BenchmarkRegistry::new();
        assert_eq!(registry.targets().len(), 0);
    }

    #[test]
    fn test_registry_register() {
        let mut registry = BenchmarkRegistry::new();
        registry.register(Box::new(TestBenchmark {
            id: "test-1".to_string(),
            should_fail: false,
        }));

        assert_eq!(registry.targets().len(), 1);
    }

    #[test]
    fn test_run_all() {
        let mut registry = BenchmarkRegistry::new();
        registry.register(Box::new(TestBenchmark {
            id: "test-1".to_string(),
            should_fail: false,
        }));
        registry.register(Box::new(TestBenchmark {
            id: "test-2".to_string(),
            should_fail: false,
        }));

        let results = registry.run_all();
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn test_calculate_stats() {
        let timings = vec![
            Duration::from_millis(100),
            Duration::from_millis(150),
            Duration::from_millis(200),
        ];

        let (min, max, _std_dev) = calculate_stats(&timings);
        assert_eq!(min, Duration::from_millis(100));
        assert_eq!(max, Duration::from_millis(200));
    }
}
