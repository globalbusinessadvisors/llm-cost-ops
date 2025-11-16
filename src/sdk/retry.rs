//! Retry logic with exponential backoff and jitter

use crate::sdk::config::RetryConfig;
use crate::sdk::error::{SdkError, SdkResult};
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry policy for handling transient failures
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    config: RetryConfig,
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute an operation with retry logic
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> SdkResult<T>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = SdkResult<T>>,
    {
        let mut attempts = 0;
        let mut last_error = None;

        while attempts < self.config.max_attempts {
            attempts += 1;

            match operation().await {
                Ok(result) => {
                    if attempts > 1 {
                        debug!(
                            "Operation succeeded after {} attempt(s)",
                            attempts
                        );
                    }
                    return Ok(result);
                }
                Err(err) => {
                    if !err.is_retryable() {
                        debug!(
                            "Non-retryable error encountered: {}",
                            err
                        );
                        return Err(err);
                    }

                    warn!(
                        "Attempt {}/{} failed: {}",
                        attempts, self.config.max_attempts, err
                    );

                    last_error = Some(err);

                    if attempts < self.config.max_attempts {
                        let backoff = self.calculate_backoff(attempts);
                        debug!("Retrying after {:?}", backoff);
                        sleep(backoff).await;
                    }
                }
            }
        }

        Err(SdkError::RetryExhausted {
            attempts,
            last_error: Box::new(last_error.unwrap()),
        })
    }

    /// Calculate backoff duration for a given attempt
    fn calculate_backoff(&self, attempt: usize) -> Duration {
        let base_backoff = self.config.initial_backoff.as_secs_f64()
            * self.config.multiplier.powi((attempt - 1) as i32);

        let backoff = Duration::from_secs_f64(base_backoff.min(
            self.config.max_backoff.as_secs_f64(),
        ));

        if self.config.jitter {
            self.add_jitter(backoff)
        } else {
            backoff
        }
    }

    /// Add jitter to backoff to avoid thundering herd
    fn add_jitter(&self, duration: Duration) -> Duration {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let jitter: f64 = rng.gen_range(0.0..=0.3); // 0-30% jitter
        let multiplier = 1.0 - jitter;
        Duration::from_secs_f64(duration.as_secs_f64() * multiplier)
    }
}

/// Backoff strategy for retry logic
#[derive(Debug, Clone, Copy)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed(Duration),

    /// Exponential backoff with optional jitter
    Exponential {
        initial: Duration,
        max: Duration,
        multiplier: f64,
        jitter: bool,
    },

    /// Linear backoff
    Linear {
        initial: Duration,
        increment: Duration,
        max: Duration,
    },
}

impl BackoffStrategy {
    /// Calculate the backoff duration for a given attempt
    pub fn calculate(&self, attempt: usize) -> Duration {
        match self {
            BackoffStrategy::Fixed(duration) => *duration,
            BackoffStrategy::Exponential {
                initial,
                max,
                multiplier,
                jitter,
            } => {
                let base = initial.as_secs_f64() * multiplier.powi((attempt - 1) as i32);
                let backoff = Duration::from_secs_f64(base.min(max.as_secs_f64()));

                if *jitter {
                    add_jitter(backoff)
                } else {
                    backoff
                }
            }
            BackoffStrategy::Linear {
                initial,
                increment,
                max,
            } => {
                let total = initial.as_secs_f64() + increment.as_secs_f64() * (attempt - 1) as f64;
                Duration::from_secs_f64(total.min(max.as_secs_f64()))
            }
        }
    }
}

fn add_jitter(duration: Duration) -> Duration {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let jitter: f64 = rng.gen_range(0.0..=0.3);
    let multiplier = 1.0 - jitter;
    Duration::from_secs_f64(duration.as_secs_f64() * multiplier)
}

/// Trait for retry-able operations
#[async_trait::async_trait]
pub trait Retryable {
    type Output;
    type Error;

    /// Execute the operation
    async fn execute(&mut self) -> Result<Self::Output, Self::Error>;

    /// Check if the error is retryable
    fn is_retryable(error: &Self::Error) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let config = RetryConfig {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(10),
            max_backoff: Duration::from_millis(100),
            multiplier: 2.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config);

        let result = policy
            .execute(|| async {
                let count = counter_clone.fetch_add(1, Ordering::SeqCst);
                if count < 2 {
                    Err(SdkError::api(500, "Server error".to_string(), None))
                } else {
                    Ok("Success")
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_exhausted() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(10),
            max_backoff: Duration::from_millis(100),
            multiplier: 2.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config);

        let result = policy
            .execute(|| async {
                Err::<(), _>(SdkError::api(500, "Server error".to_string(), None))
            })
            .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SdkError::RetryExhausted { .. }));
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let config = RetryConfig {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(10),
            max_backoff: Duration::from_millis(100),
            multiplier: 2.0,
            jitter: false,
        };

        let policy = RetryPolicy::new(config);

        let result = policy
            .execute(|| async {
                counter_clone.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(SdkError::api(404, "Not found".to_string(), None))
            })
            .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Should not retry
    }

    #[test]
    fn test_backoff_strategies() {
        let fixed = BackoffStrategy::Fixed(Duration::from_secs(1));
        assert_eq!(fixed.calculate(1), Duration::from_secs(1));
        assert_eq!(fixed.calculate(10), Duration::from_secs(1));

        let exponential = BackoffStrategy::Exponential {
            initial: Duration::from_secs(1),
            max: Duration::from_secs(60),
            multiplier: 2.0,
            jitter: false,
        };
        assert_eq!(exponential.calculate(1), Duration::from_secs(1));
        assert_eq!(exponential.calculate(2), Duration::from_secs(2));
        assert_eq!(exponential.calculate(3), Duration::from_secs(4));

        let linear = BackoffStrategy::Linear {
            initial: Duration::from_secs(1),
            increment: Duration::from_secs(1),
            max: Duration::from_secs(10),
        };
        assert_eq!(linear.calculate(1), Duration::from_secs(1));
        assert_eq!(linear.calculate(2), Duration::from_secs(2));
        assert_eq!(linear.calculate(3), Duration::from_secs(3));
        assert_eq!(linear.calculate(20), Duration::from_secs(10)); // Capped at max
    }
}
