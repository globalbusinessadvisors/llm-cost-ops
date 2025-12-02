/// Canonical benchmark interface for LLM-CostOps
///
/// This module provides a standardized benchmarking system for all CostOps operations,
/// with consistent result formats, reporting, and I/O capabilities.

pub mod result;
pub mod io;
pub mod markdown;

pub use result::{BenchmarkResult, BenchmarkSummary};
pub use io::{BenchmarkIo, BenchmarkIoError, IoResult};
pub use markdown::MarkdownGenerator;

use crate::adapters;
use std::path::Path;

/// Run all available benchmarks and save results
///
/// This is the main entry point for running the canonical benchmark suite.
/// It executes all registered benchmarks, saves individual results, generates
/// a summary, and creates a markdown report.
///
/// # Arguments
///
/// * `output_dir` - Directory where benchmark results will be saved
///
/// # Returns
///
/// A `BenchmarkSummary` containing all results and statistics
///
/// # Example
///
/// ```no_run
/// use llm_cost_ops::benchmarks::run_all_benchmarks;
///
/// let summary = run_all_benchmarks("benchmarks/output").unwrap();
/// println!("Ran {} benchmarks with {:.2}% success rate",
///     summary.total_count,
///     summary.success_rate()
/// );
/// ```
pub fn run_all_benchmarks<P: AsRef<Path>>(output_dir: P) -> IoResult<BenchmarkSummary> {
    println!("Starting LLM-CostOps Benchmark Suite...");
    println!();

    // Initialize I/O handler
    let io = BenchmarkIo::new(output_dir.as_ref())?;

    // Clear previous results
    io.clear_results()?;

    // Get all benchmark targets
    let registry = adapters::all_targets();
    let targets = registry.targets();

    println!("Registered {} benchmark targets", targets.len());
    println!();

    // Run all benchmarks
    let results = registry.run_all();

    // Print progress
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("Completed: {} passed, {} failed", passed, failed);
    println!();

    // Save individual results
    println!("Saving results...");
    io.write_all_results(&results)?;

    // Create summary
    let summary = BenchmarkSummary::from_results(results.clone());

    // Save summary as JSON
    io.write_summary(&summary)?;

    // Generate and save markdown report
    let report = MarkdownGenerator::generate_report(&summary);
    let report_path = io.output_dir().join("summary.md");
    std::fs::write(&report_path, report)?;

    println!("Results saved to: {}", io.output_dir().display());
    println!("  - Raw results: {}", io.raw_dir().display());
    println!("  - Summary JSON: {}", io.output_dir().join("summary.json").display());
    println!("  - Summary Markdown: {}", report_path.display());
    println!();

    Ok(summary)
}

/// Run benchmarks for a specific category only
///
/// # Arguments
///
/// * `category` - The category to run benchmarks for (e.g., "engine", "compression", "forecasting")
/// * `output_dir` - Directory where benchmark results will be saved
///
/// # Returns
///
/// A `BenchmarkSummary` containing results for the specified category
pub fn run_category_benchmarks<P: AsRef<Path>>(
    category: &str,
    output_dir: P,
) -> IoResult<BenchmarkSummary> {
    println!("Starting benchmarks for category: {}", category);
    println!();

    // Initialize I/O handler
    let io = BenchmarkIo::new(output_dir.as_ref())?;

    // Get all benchmark targets
    let registry = adapters::all_targets();

    // Run category benchmarks
    let results = registry.run_category(category);

    if results.is_empty() {
        println!("Warning: No benchmarks found for category '{}'", category);
        return Ok(BenchmarkSummary::from_results(vec![]));
    }

    // Print progress
    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("Completed: {} passed, {} failed", passed, failed);
    println!();

    // Save individual results
    io.write_all_results(&results)?;

    // Create summary
    let summary = BenchmarkSummary::from_results(results);

    // Save summary
    io.write_summary(&summary)?;

    // Generate and save markdown report
    let report = MarkdownGenerator::generate_report(&summary);
    let report_path = io.output_dir().join(format!("{}_summary.md", category));
    std::fs::write(&report_path, report)?;

    println!("Results saved to: {}", io.output_dir().display());
    println!();

    Ok(summary)
}

/// Run a quick benchmark test with minimal output
///
/// This is useful for quick validation and testing. It runs all benchmarks
/// but only prints a compact summary without saving detailed results.
///
/// # Returns
///
/// A `BenchmarkSummary` containing all results
pub fn run_quick_benchmark() -> BenchmarkSummary {
    println!("Running quick benchmark test...");
    println!();

    let registry = adapters::all_targets();
    let results = registry.run_all();

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;

    println!("Completed: {} passed, {} failed", passed, failed);

    let summary = BenchmarkSummary::from_results(results);

    println!("Success rate: {:.2}%", summary.success_rate());
    println!("Total duration: {:.2}s", summary.total_duration.as_secs_f64());
    println!();

    summary
}

/// Get available benchmark categories
pub fn available_categories() -> Vec<String> {
    let registry = adapters::all_targets();
    let mut categories: Vec<String> = registry.targets()
        .iter()
        .map(|t| t.category())
        .collect();

    categories.sort();
    categories.dedup();
    categories
}

/// Get benchmark targets for a specific category
pub fn targets_in_category(category: &str) -> Vec<String> {
    let registry = adapters::all_targets();
    registry.targets_by_category(category)
        .iter()
        .map(|t| t.name())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_available_categories() {
        let categories = available_categories();
        assert!(!categories.is_empty());
        assert!(categories.contains(&"engine".to_string()));
        assert!(categories.contains(&"compression".to_string()));
        assert!(categories.contains(&"forecasting".to_string()));
    }

    #[test]
    fn test_targets_in_category() {
        let targets = targets_in_category("engine");
        assert!(!targets.is_empty());
    }

    #[test]
    fn test_run_all_benchmarks() {
        let temp_dir = TempDir::new().unwrap();
        let summary = run_all_benchmarks(temp_dir.path()).unwrap();

        assert!(summary.total_count > 0);
        assert!(temp_dir.path().join("summary.json").exists());
        assert!(temp_dir.path().join("summary.md").exists());
    }

    #[test]
    fn test_run_category_benchmarks() {
        let temp_dir = TempDir::new().unwrap();
        let summary = run_category_benchmarks("engine", temp_dir.path()).unwrap();

        assert!(summary.total_count > 0);
        assert_eq!(summary.by_category.len(), 1);
        assert!(summary.by_category.contains_key("engine"));
    }

    #[test]
    fn test_run_quick_benchmark() {
        let summary = run_quick_benchmark();
        assert!(summary.total_count > 0);
    }
}
