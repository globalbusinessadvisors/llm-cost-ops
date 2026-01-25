//! Performance Budget Enforcement
//!
//! Phase 4 Layer 1 - Performance budgets for agent execution.
//!
//! # Performance Budgets
//! - MAX_TOKENS: 1200
//! - MAX_LATENCY_MS: 2500

use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::{warn, instrument};

use super::{MAX_TOKENS, MAX_LATENCY_MS};

/// Performance budget configuration
#[derive(Debug, Clone)]
pub struct PerformanceBudget {
    /// Maximum tokens allowed
    pub max_tokens: usize,

    /// Maximum latency in milliseconds
    pub max_latency_ms: u64,

    /// Whether to enforce strict limits (fail on exceed) or soft limits (warn only)
    pub strict: bool,
}

impl Default for PerformanceBudget {
    fn default() -> Self {
        Self {
            max_tokens: MAX_TOKENS,
            max_latency_ms: MAX_LATENCY_MS,
            strict: false, // Advisory by default (per governance rules)
        }
    }
}

impl PerformanceBudget {
    /// Create a new performance budget with custom limits
    pub fn new(max_tokens: usize, max_latency_ms: u64) -> Self {
        Self {
            max_tokens,
            max_latency_ms,
            strict: false,
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Self {
        Self {
            max_tokens: std::env::var("MAX_TOKENS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(MAX_TOKENS),
            max_latency_ms: std::env::var("MAX_LATENCY_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(MAX_LATENCY_MS),
            strict: std::env::var("STRICT_PERFORMANCE_BUDGET")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
        }
    }

    /// Set strict mode
    pub fn with_strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }

    /// Check if token count is within budget
    pub fn check_tokens(&self, tokens: usize) -> Result<(), BudgetExceeded> {
        if tokens > self.max_tokens {
            let err = BudgetExceeded::Tokens {
                used: tokens,
                max: self.max_tokens,
            };

            if self.strict {
                return Err(err);
            } else {
                warn!(
                    used = tokens,
                    max = self.max_tokens,
                    "Token budget exceeded (advisory)"
                );
            }
        }
        Ok(())
    }

    /// Check if latency is within budget
    pub fn check_latency(&self, latency_ms: u64) -> Result<(), BudgetExceeded> {
        if latency_ms > self.max_latency_ms {
            let err = BudgetExceeded::Latency {
                used_ms: latency_ms,
                max_ms: self.max_latency_ms,
            };

            if self.strict {
                return Err(err);
            } else {
                warn!(
                    used_ms = latency_ms,
                    max_ms = self.max_latency_ms,
                    "Latency budget exceeded (advisory)"
                );
            }
        }
        Ok(())
    }

    /// Create a performance guard for measuring execution
    pub fn guard(&self) -> PerformanceGuard {
        PerformanceGuard::new(self.clone())
    }
}

/// Error type for budget exceedances
#[derive(Debug, Error)]
pub enum BudgetExceeded {
    #[error("Token budget exceeded: used {used}, max {max}")]
    Tokens { used: usize, max: usize },

    #[error("Latency budget exceeded: {used_ms}ms, max {max_ms}ms")]
    Latency { used_ms: u64, max_ms: u64 },

    #[error("Multiple budgets exceeded: {0}")]
    Multiple(String),
}

/// Performance guard for measuring execution time and resources
pub struct PerformanceGuard {
    budget: PerformanceBudget,
    start_time: Instant,
    tokens_used: usize,
}

impl PerformanceGuard {
    pub fn new(budget: PerformanceBudget) -> Self {
        Self {
            budget,
            start_time: Instant::now(),
            tokens_used: 0,
        }
    }

    /// Record tokens used
    pub fn record_tokens(&mut self, tokens: usize) {
        self.tokens_used += tokens;
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> u64 {
        self.elapsed().as_millis() as u64
    }

    /// Check all budgets
    #[instrument(skip(self))]
    pub fn check(&self) -> Result<PerformanceMetrics, BudgetExceeded> {
        let elapsed_ms = self.elapsed_ms();
        let mut violations = Vec::new();

        if self.tokens_used > self.budget.max_tokens {
            violations.push(format!(
                "tokens: {} > {}",
                self.tokens_used, self.budget.max_tokens
            ));
        }

        if elapsed_ms > self.budget.max_latency_ms {
            violations.push(format!(
                "latency: {}ms > {}ms",
                elapsed_ms, self.budget.max_latency_ms
            ));
        }

        let metrics = PerformanceMetrics {
            tokens_used: self.tokens_used,
            latency_ms: elapsed_ms,
            max_tokens: self.budget.max_tokens,
            max_latency_ms: self.budget.max_latency_ms,
            within_budget: violations.is_empty(),
        };

        if !violations.is_empty() && self.budget.strict {
            return Err(BudgetExceeded::Multiple(violations.join(", ")));
        }

        if !violations.is_empty() {
            warn!(
                violations = ?violations,
                "Performance budget exceeded (advisory)"
            );
        }

        Ok(metrics)
    }

    /// Finish and get metrics (does not fail on budget exceed in non-strict mode)
    pub fn finish(self) -> PerformanceMetrics {
        let elapsed_ms = self.elapsed_ms();

        PerformanceMetrics {
            tokens_used: self.tokens_used,
            latency_ms: elapsed_ms,
            max_tokens: self.budget.max_tokens,
            max_latency_ms: self.budget.max_latency_ms,
            within_budget: self.tokens_used <= self.budget.max_tokens
                && elapsed_ms <= self.budget.max_latency_ms,
        }
    }
}

/// Performance metrics from execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PerformanceMetrics {
    /// Tokens used in execution
    pub tokens_used: usize,

    /// Latency in milliseconds
    pub latency_ms: u64,

    /// Maximum tokens budget
    pub max_tokens: usize,

    /// Maximum latency budget
    pub max_latency_ms: u64,

    /// Whether execution was within budget
    pub within_budget: bool,
}

impl PerformanceMetrics {
    /// Get token utilization as percentage
    pub fn token_utilization(&self) -> f64 {
        if self.max_tokens == 0 {
            0.0
        } else {
            (self.tokens_used as f64 / self.max_tokens as f64) * 100.0
        }
    }

    /// Get latency utilization as percentage
    pub fn latency_utilization(&self) -> f64 {
        if self.max_latency_ms == 0 {
            0.0
        } else {
            (self.latency_ms as f64 / self.max_latency_ms as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_performance_budget_defaults() {
        let budget = PerformanceBudget::default();
        assert_eq!(budget.max_tokens, 1200);
        assert_eq!(budget.max_latency_ms, 2500);
        assert!(!budget.strict);
    }

    #[test]
    fn test_token_check_within_budget() {
        let budget = PerformanceBudget::default();
        assert!(budget.check_tokens(1000).is_ok());
    }

    #[test]
    fn test_token_check_over_budget_advisory() {
        let budget = PerformanceBudget::default();
        // In advisory mode, should not return error
        assert!(budget.check_tokens(2000).is_ok());
    }

    #[test]
    fn test_token_check_over_budget_strict() {
        let budget = PerformanceBudget::default().with_strict(true);
        assert!(budget.check_tokens(2000).is_err());
    }

    #[test]
    fn test_performance_guard() {
        let budget = PerformanceBudget::new(1000, 5000);
        let mut guard = budget.guard();

        guard.record_tokens(500);

        let metrics = guard.finish();
        assert_eq!(metrics.tokens_used, 500);
        assert!(metrics.within_budget);
    }

    #[test]
    fn test_performance_metrics_utilization() {
        let metrics = PerformanceMetrics {
            tokens_used: 600,
            latency_ms: 1250,
            max_tokens: 1200,
            max_latency_ms: 2500,
            within_budget: true,
        };

        assert_eq!(metrics.token_utilization(), 50.0);
        assert_eq!(metrics.latency_utilization(), 50.0);
    }
}
