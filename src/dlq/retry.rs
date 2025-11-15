// Retry policies and backoff strategies for DLQ

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::types::DlqItem;

/// Backoff strategy for retries
pub trait BackoffStrategy: Send + Sync {
    /// Calculate the next retry time for a DLQ item
    fn next_retry_time(&self, item: &DlqItem, base_delay_secs: u64) -> DateTime<Utc>;

    /// Get the delay in seconds for a specific retry attempt
    fn get_delay_secs(&self, retry_count: u32, base_delay_secs: u64) -> u64;
}

/// Exponential backoff strategy
#[derive(Debug, Clone)]
pub struct ExponentialBackoff {
    /// Multiplier for each retry attempt
    pub multiplier: f64,

    /// Maximum delay in seconds
    pub max_delay_secs: u64,

    /// Add jitter to prevent thundering herd
    pub jitter: bool,
}

impl ExponentialBackoff {
    pub fn new(multiplier: f64, max_delay_secs: u64) -> Self {
        Self {
            multiplier,
            max_delay_secs,
            jitter: true,
        }
    }

    pub fn without_jitter(mut self) -> Self {
        self.jitter = false;
        self
    }
}

impl Default for ExponentialBackoff {
    fn default() -> Self {
        Self {
            multiplier: 2.0,
            max_delay_secs: 3600,
            jitter: true,
        }
    }
}

impl BackoffStrategy for ExponentialBackoff {
    fn next_retry_time(&self, item: &DlqItem, base_delay_secs: u64) -> DateTime<Utc> {
        let delay_secs = self.get_delay_secs(item.retry_count, base_delay_secs);
        Utc::now() + Duration::seconds(delay_secs as i64)
    }

    fn get_delay_secs(&self, retry_count: u32, base_delay_secs: u64) -> u64 {
        // Calculate exponential delay: base * (multiplier ^ retry_count)
        let delay = (base_delay_secs as f64) * self.multiplier.powi(retry_count as i32);
        let mut delay_secs = delay.min(self.max_delay_secs as f64) as u64;

        // Add jitter (±20% random variation)
        if self.jitter {
            let jitter_range = (delay_secs as f64 * 0.2) as u64;
            let jitter = (rand::random::<f64>() * (jitter_range as f64 * 2.0)) as u64;
            delay_secs = delay_secs.saturating_add(jitter).saturating_sub(jitter_range);
        }

        delay_secs
    }
}

/// Fixed backoff strategy (constant delay)
#[derive(Debug, Clone)]
pub struct FixedBackoff {
    /// Fixed delay in seconds
    pub delay_secs: u64,
}

impl FixedBackoff {
    pub fn new(delay_secs: u64) -> Self {
        Self { delay_secs }
    }
}

impl Default for FixedBackoff {
    fn default() -> Self {
        Self { delay_secs: 60 }
    }
}

impl BackoffStrategy for FixedBackoff {
    fn next_retry_time(&self, _item: &DlqItem, _base_delay_secs: u64) -> DateTime<Utc> {
        Utc::now() + Duration::seconds(self.delay_secs as i64)
    }

    fn get_delay_secs(&self, _retry_count: u32, _base_delay_secs: u64) -> u64 {
        self.delay_secs
    }
}

/// Linear backoff strategy
#[derive(Debug, Clone)]
pub struct LinearBackoff {
    /// Increment delay for each retry
    pub increment_secs: u64,

    /// Maximum delay in seconds
    pub max_delay_secs: u64,
}

impl LinearBackoff {
    pub fn new(increment_secs: u64, max_delay_secs: u64) -> Self {
        Self {
            increment_secs,
            max_delay_secs,
        }
    }
}

impl Default for LinearBackoff {
    fn default() -> Self {
        Self {
            increment_secs: 60,
            max_delay_secs: 3600,
        }
    }
}

impl BackoffStrategy for LinearBackoff {
    fn next_retry_time(&self, item: &DlqItem, base_delay_secs: u64) -> DateTime<Utc> {
        let delay_secs = self.get_delay_secs(item.retry_count, base_delay_secs);
        Utc::now() + Duration::seconds(delay_secs as i64)
    }

    fn get_delay_secs(&self, retry_count: u32, base_delay_secs: u64) -> u64 {
        let delay = base_delay_secs + (self.increment_secs * retry_count as u64);
        delay.min(self.max_delay_secs)
    }
}

/// Retry policy that combines backoff strategy with other retry logic
#[derive(Clone)]
pub struct RetryPolicy {
    /// Backoff strategy to use
    backoff: BackoffType,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Base delay in seconds
    pub base_delay_secs: u64,
}

/// Backoff type enum
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BackoffType {
    Exponential {
        multiplier: f64,
        max_delay_secs: u64,
        jitter: bool,
    },
    Fixed {
        delay_secs: u64,
    },
    Linear {
        increment_secs: u64,
        max_delay_secs: u64,
    },
}

impl RetryPolicy {
    /// Create a new retry policy with exponential backoff
    pub fn exponential(
        max_retries: u32,
        base_delay_secs: u64,
        multiplier: f64,
        max_delay_secs: u64,
    ) -> Self {
        Self {
            backoff: BackoffType::Exponential {
                multiplier,
                max_delay_secs,
                jitter: true,
            },
            max_retries,
            base_delay_secs,
        }
    }

    /// Create a new retry policy with fixed backoff
    pub fn fixed(max_retries: u32, delay_secs: u64) -> Self {
        Self {
            backoff: BackoffType::Fixed { delay_secs },
            max_retries,
            base_delay_secs: delay_secs,
        }
    }

    /// Create a new retry policy with linear backoff
    pub fn linear(max_retries: u32, base_delay_secs: u64, increment_secs: u64, max_delay_secs: u64) -> Self {
        Self {
            backoff: BackoffType::Linear {
                increment_secs,
                max_delay_secs,
            },
            max_retries,
            base_delay_secs,
        }
    }

    /// Calculate the next retry time for an item
    pub fn next_retry_time(&self, item: &DlqItem) -> DateTime<Utc> {
        match &self.backoff {
            BackoffType::Exponential {
                multiplier,
                max_delay_secs,
                jitter,
            } => {
                let strategy = ExponentialBackoff {
                    multiplier: *multiplier,
                    max_delay_secs: *max_delay_secs,
                    jitter: *jitter,
                };
                strategy.next_retry_time(item, self.base_delay_secs)
            }
            BackoffType::Fixed { delay_secs } => {
                let strategy = FixedBackoff::new(*delay_secs);
                strategy.next_retry_time(item, self.base_delay_secs)
            }
            BackoffType::Linear {
                increment_secs,
                max_delay_secs,
            } => {
                let strategy = LinearBackoff::new(*increment_secs, *max_delay_secs);
                strategy.next_retry_time(item, self.base_delay_secs)
            }
        }
    }

    /// Check if an item should be retried
    pub fn should_retry(&self, item: &DlqItem) -> bool {
        item.retry_count < self.max_retries && item.failure_reason.is_retryable()
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::exponential(3, 60, 2.0, 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dlq::types::FailureReason;

    fn create_test_item(retry_count: u32) -> DlqItem {
        let mut item = DlqItem::new(
            "org-123".to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::NetworkError,
            "Test error".to_string(),
            5,
        );
        item.retry_count = retry_count;
        item
    }

    #[test]
    fn test_exponential_backoff() {
        let backoff = ExponentialBackoff::new(2.0, 3600).without_jitter();

        assert_eq!(backoff.get_delay_secs(0, 60), 60);
        assert_eq!(backoff.get_delay_secs(1, 60), 120);
        assert_eq!(backoff.get_delay_secs(2, 60), 240);
        assert_eq!(backoff.get_delay_secs(3, 60), 480);

        // Test max delay
        assert_eq!(backoff.get_delay_secs(10, 60), 3600);
    }

    #[test]
    fn test_fixed_backoff() {
        let backoff = FixedBackoff::new(120);

        assert_eq!(backoff.get_delay_secs(0, 60), 120);
        assert_eq!(backoff.get_delay_secs(1, 60), 120);
        assert_eq!(backoff.get_delay_secs(5, 60), 120);
    }

    #[test]
    fn test_linear_backoff() {
        let backoff = LinearBackoff::new(30, 300);

        assert_eq!(backoff.get_delay_secs(0, 60), 60);
        assert_eq!(backoff.get_delay_secs(1, 60), 90);
        assert_eq!(backoff.get_delay_secs(2, 60), 120);
        assert_eq!(backoff.get_delay_secs(3, 60), 150);

        // Test max delay
        assert_eq!(backoff.get_delay_secs(10, 60), 300);
    }

    #[test]
    fn test_retry_policy_exponential() {
        let policy = RetryPolicy::exponential(3, 60, 2.0, 3600);
        let item = create_test_item(0);

        assert!(policy.should_retry(&item));

        let next_retry = policy.next_retry_time(&item);
        assert!(next_retry > Utc::now());
    }

    #[test]
    fn test_retry_policy_should_not_retry_max_attempts() {
        let policy = RetryPolicy::exponential(3, 60, 2.0, 3600);
        let item = create_test_item(3);

        assert!(!policy.should_retry(&item));
    }

    #[test]
    fn test_retry_policy_should_not_retry_non_retryable() {
        let policy = RetryPolicy::exponential(3, 60, 2.0, 3600);
        let mut item = create_test_item(0);
        item.failure_reason = FailureReason::ValidationError;

        assert!(!policy.should_retry(&item));
    }

    #[test]
    fn test_exponential_backoff_with_jitter() {
        let backoff = ExponentialBackoff::new(2.0, 3600);

        // With jitter, results should vary slightly
        let delay1 = backoff.get_delay_secs(1, 60);
        let delay2 = backoff.get_delay_secs(1, 60);

        // Both should be around 120 seconds but may differ due to jitter
        assert!(delay1 >= 96 && delay1 <= 144); // ±20% of 120
        assert!(delay2 >= 96 && delay2 <= 144);
    }
}
