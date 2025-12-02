/// Markdown report generation for benchmark results
///
/// Generates human-readable markdown reports from benchmark results.

use super::result::{BenchmarkResult, BenchmarkSummary};
use std::fmt::Write as FmtWrite;

/// Markdown report generator
pub struct MarkdownGenerator;

impl MarkdownGenerator {
    /// Generate a full markdown report from a summary
    pub fn generate_report(summary: &BenchmarkSummary) -> String {
        let mut output = String::new();

        // Header
        writeln!(&mut output, "# LLM-CostOps Benchmark Results").unwrap();
        writeln!(&mut output).unwrap();
        writeln!(&mut output, "**Generated:** {}", summary.timestamp.format("%Y-%m-%d %H:%M:%S UTC")).unwrap();
        writeln!(&mut output).unwrap();

        // Summary section
        Self::write_summary(&mut output, summary);
        writeln!(&mut output).unwrap();

        // Results by category
        let mut categories: Vec<_> = summary.by_category.keys().collect();
        categories.sort();

        for category in categories {
            if let Some(results) = summary.by_category.get(category) {
                Self::write_category(&mut output, category, results);
                writeln!(&mut output).unwrap();
            }
        }

        // Failed benchmarks section
        let failed: Vec<_> = summary.by_category.values()
            .flat_map(|v| v.iter())
            .filter(|r| !r.passed)
            .collect();

        if !failed.is_empty() {
            Self::write_failures(&mut output, &failed);
        }

        output
    }

    /// Write the summary section
    fn write_summary(output: &mut String, summary: &BenchmarkSummary) {
        writeln!(output, "## Summary").unwrap();
        writeln!(output).unwrap();
        writeln!(output, "| Metric | Value |").unwrap();
        writeln!(output, "|--------|-------|").unwrap();
        writeln!(output, "| Total Benchmarks | {} |", summary.total_count).unwrap();
        writeln!(output, "| Passed | {} |", summary.passed_count).unwrap();
        writeln!(output, "| Failed | {} |", summary.failed_count).unwrap();
        writeln!(output, "| Success Rate | {:.2}% |", summary.success_rate()).unwrap();
        writeln!(output, "| Total Duration | {:.2}s |", summary.total_duration.as_secs_f64()).unwrap();
    }

    /// Write a category section
    fn write_category(output: &mut String, category: &str, results: &[BenchmarkResult]) {
        writeln!(output, "## Category: {}", category).unwrap();
        writeln!(output).unwrap();

        // Only include passed benchmarks in the main table
        let passed: Vec<_> = results.iter().filter(|r| r.passed).collect();

        if passed.is_empty() {
            writeln!(output, "*No successful benchmarks in this category.*").unwrap();
            return;
        }

        writeln!(output, "| Benchmark | Iterations | Throughput | Avg Time | Duration |").unwrap();
        writeln!(output, "|-----------|-----------|------------|----------|----------|").unwrap();

        for result in passed {
            writeln!(
                output,
                "| {} | {} | {} | {} | {:.2}s |",
                result.name,
                result.iterations,
                result.throughput_string(),
                result.avg_time_string(),
                result.duration.as_secs_f64()
            ).unwrap();
        }
    }

    /// Write the failures section
    fn write_failures(output: &mut String, failures: &[&BenchmarkResult]) {
        writeln!(output, "## Failed Benchmarks").unwrap();
        writeln!(output).unwrap();

        writeln!(output, "| Category | Benchmark | Error |").unwrap();
        writeln!(output, "|----------|-----------|-------|").unwrap();

        for result in failures {
            let error = result.error.as_deref().unwrap_or("Unknown error");
            writeln!(
                output,
                "| {} | {} | {} |",
                result.category,
                result.name,
                error
            ).unwrap();
        }
    }

    /// Generate a compact summary (single line per benchmark)
    pub fn generate_compact_summary(results: &[BenchmarkResult]) -> String {
        let mut output = String::new();

        writeln!(&mut output, "# Benchmark Summary").unwrap();
        writeln!(&mut output).unwrap();

        for result in results {
            if result.passed {
                writeln!(
                    &mut output,
                    "- **{}** ({}): {} - {}",
                    result.name,
                    result.category,
                    result.throughput_string(),
                    result.avg_time_string()
                ).unwrap();
            } else {
                writeln!(
                    &mut output,
                    "- **{}** ({}): FAILED - {}",
                    result.name,
                    result.category,
                    result.error.as_deref().unwrap_or("Unknown error")
                ).unwrap();
            }
        }

        output
    }

    /// Generate a comparison table for multiple benchmark runs
    pub fn generate_comparison_table(
        baseline: &[BenchmarkResult],
        current: &[BenchmarkResult],
    ) -> String {
        let mut output = String::new();

        writeln!(&mut output, "# Benchmark Comparison").unwrap();
        writeln!(&mut output).unwrap();
        writeln!(&mut output, "| Benchmark | Baseline | Current | Change |").unwrap();
        writeln!(&mut output, "|-----------|----------|---------|--------|").unwrap();

        for baseline_result in baseline {
            if let Some(current_result) = current.iter().find(|r| r.id == baseline_result.id) {
                let baseline_ops = baseline_result.ops_per_sec;
                let current_ops = current_result.ops_per_sec;
                let change = if baseline_ops > 0.0 {
                    ((current_ops - baseline_ops) / baseline_ops) * 100.0
                } else {
                    0.0
                };

                let change_str = if change > 0.0 {
                    format!("+{:.2}%", change)
                } else {
                    format!("{:.2}%", change)
                };

                writeln!(
                    &mut output,
                    "| {} | {} | {} | {} |",
                    baseline_result.name,
                    baseline_result.throughput_string(),
                    current_result.throughput_string(),
                    change_str
                ).unwrap();
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_generate_report() {
        let results = vec![
            BenchmarkResult::success(
                "1".to_string(),
                "Test 1".to_string(),
                "engine".to_string(),
                Duration::from_secs(1),
                1000,
            ),
            BenchmarkResult::success(
                "2".to_string(),
                "Test 2".to_string(),
                "storage".to_string(),
                Duration::from_millis(500),
                500,
            ),
        ];

        let summary = BenchmarkSummary::from_results(results);
        let report = MarkdownGenerator::generate_report(&summary);

        assert!(report.contains("# LLM-CostOps Benchmark Results"));
        assert!(report.contains("## Summary"));
        assert!(report.contains("## Category: engine"));
        assert!(report.contains("## Category: storage"));
    }

    #[test]
    fn test_generate_compact_summary() {
        let results = vec![
            BenchmarkResult::success(
                "1".to_string(),
                "Test 1".to_string(),
                "engine".to_string(),
                Duration::from_secs(1),
                1000,
            ),
        ];

        let summary = MarkdownGenerator::generate_compact_summary(&results);

        assert!(summary.contains("# Benchmark Summary"));
        assert!(summary.contains("Test 1"));
        assert!(summary.contains("engine"));
    }

    #[test]
    fn test_generate_comparison_table() {
        let baseline = vec![
            BenchmarkResult::success(
                "1".to_string(),
                "Test 1".to_string(),
                "engine".to_string(),
                Duration::from_secs(1),
                1000,
            ),
        ];

        let current = vec![
            BenchmarkResult::success(
                "1".to_string(),
                "Test 1".to_string(),
                "engine".to_string(),
                Duration::from_secs(1),
                1200,
            ),
        ];

        let comparison = MarkdownGenerator::generate_comparison_table(&baseline, &current);

        assert!(comparison.contains("# Benchmark Comparison"));
        assert!(comparison.contains("Test 1"));
        assert!(comparison.contains("%")); // Should show percentage change
    }

    #[test]
    fn test_report_with_failures() {
        let results = vec![
            BenchmarkResult::success(
                "1".to_string(),
                "Test 1".to_string(),
                "engine".to_string(),
                Duration::from_secs(1),
                1000,
            ),
            BenchmarkResult::failure(
                "2".to_string(),
                "Test 2".to_string(),
                "storage".to_string(),
                "Test error".to_string(),
            ),
        ];

        let summary = BenchmarkSummary::from_results(results);
        let report = MarkdownGenerator::generate_report(&summary);

        assert!(report.contains("## Failed Benchmarks"));
        assert!(report.contains("Test error"));
    }
}
