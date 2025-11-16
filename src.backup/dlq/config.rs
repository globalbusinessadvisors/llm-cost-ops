// Dead Letter Queue configuration

use serde::{Deserialize, Serialize};

/// DLQ configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqConfig {
    /// Enable DLQ processing
    pub enabled: bool,

    /// Maximum number of retries per item
    pub max_retries: u32,

    /// Initial retry delay in seconds
    pub initial_retry_delay_secs: u64,

    /// Maximum retry delay in seconds
    pub max_retry_delay_secs: u64,

    /// Retry delay multiplier for exponential backoff
    pub backoff_multiplier: f64,

    /// DLQ item expiration in hours
    pub item_expiration_hours: u64,

    /// Maximum number of items to process concurrently
    pub max_concurrent_processing: usize,

    /// Interval for processing DLQ items (in seconds)
    pub processing_interval_secs: u64,

    /// Maximum number of items to fetch per batch
    pub batch_size: usize,

    /// Enable automatic retry for retryable failures
    pub auto_retry_enabled: bool,

    /// Alert threshold for DLQ size
    pub alert_threshold: usize,

    /// Automatically archive processed items after N days
    pub auto_archive_after_days: Option<u64>,
}

impl Default for DlqConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            initial_retry_delay_secs: 60,
            max_retry_delay_secs: 3600,
            backoff_multiplier: 2.0,
            item_expiration_hours: 168, // 7 days
            max_concurrent_processing: 10,
            processing_interval_secs: 60,
            batch_size: 100,
            auto_retry_enabled: true,
            alert_threshold: 1000,
            auto_archive_after_days: Some(30),
        }
    }
}

impl DlqConfig {
    /// Create a production configuration
    pub fn production() -> Self {
        Self {
            enabled: true,
            max_retries: 5,
            initial_retry_delay_secs: 60,
            max_retry_delay_secs: 7200, // 2 hours
            backoff_multiplier: 2.0,
            item_expiration_hours: 168, // 7 days
            max_concurrent_processing: 20,
            processing_interval_secs: 30,
            batch_size: 50,
            auto_retry_enabled: true,
            alert_threshold: 5000,
            auto_archive_after_days: Some(30),
        }
    }

    /// Create a development configuration
    pub fn development() -> Self {
        Self {
            enabled: true,
            max_retries: 3,
            initial_retry_delay_secs: 10,
            max_retry_delay_secs: 300, // 5 minutes
            backoff_multiplier: 1.5,
            item_expiration_hours: 24, // 1 day
            max_concurrent_processing: 5,
            processing_interval_secs: 60,
            batch_size: 20,
            auto_retry_enabled: true,
            alert_threshold: 100,
            auto_archive_after_days: Some(7),
        }
    }

    /// Create a disabled configuration
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_retries == 0 {
            return Err("max_retries must be greater than 0".to_string());
        }

        if self.initial_retry_delay_secs == 0 {
            return Err("initial_retry_delay_secs must be greater than 0".to_string());
        }

        if self.max_retry_delay_secs < self.initial_retry_delay_secs {
            return Err(
                "max_retry_delay_secs must be greater than or equal to initial_retry_delay_secs"
                    .to_string(),
            );
        }

        if self.backoff_multiplier < 1.0 {
            return Err("backoff_multiplier must be >= 1.0".to_string());
        }

        if self.max_concurrent_processing == 0 {
            return Err("max_concurrent_processing must be greater than 0".to_string());
        }

        if self.batch_size == 0 {
            return Err("batch_size must be greater than 0".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DlqConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_retries, 3);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_production_config() {
        let config = DlqConfig::production();
        assert!(config.enabled);
        assert_eq!(config.max_retries, 5);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_development_config() {
        let config = DlqConfig::development();
        assert!(config.enabled);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_disabled_config() {
        let config = DlqConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_invalid_config() {
        let mut config = DlqConfig::default();
        config.max_retries = 0;
        assert!(config.validate().is_err());

        let mut config = DlqConfig::default();
        config.backoff_multiplier = 0.5;
        assert!(config.validate().is_err());
    }
}
