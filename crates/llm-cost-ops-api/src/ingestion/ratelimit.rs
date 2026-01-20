// Rate limiting implementation for ingestion endpoints

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, warn};

use llm_cost_ops::{CostOpsError, Result};

use super::traits::RateLimiter;

/// Rate limit configuration per organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u64,

    /// Time window duration
    pub window_duration: Duration,

    /// Burst allowance (additional requests allowed in short bursts)
    pub burst_size: u64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 1000,
            window_duration: Duration::from_secs(60),
            burst_size: 100,
        }
    }
}

impl RateLimitConfig {
    /// Create a rate limit config for specific throughput
    pub fn per_minute(requests_per_minute: u64) -> Self {
        Self {
            max_requests: requests_per_minute,
            window_duration: Duration::from_secs(60),
            burst_size: requests_per_minute / 10, // 10% burst
        }
    }

    /// Create a rate limit config for hourly limits
    pub fn per_hour(requests_per_hour: u64) -> Self {
        Self {
            max_requests: requests_per_hour,
            window_duration: Duration::from_secs(3600),
            burst_size: requests_per_hour / 100, // 1% burst
        }
    }

    /// Create a rate limit config for daily limits
    pub fn per_day(requests_per_day: u64) -> Self {
        Self {
            max_requests: requests_per_day,
            window_duration: Duration::from_secs(86400),
            burst_size: requests_per_day / 1000, // 0.1% burst
        }
    }
}

/// Sliding window counter for rate limiting
#[derive(Debug, Clone)]
struct SlidingWindow {
    /// Request timestamps in the current window
    requests: Vec<Instant>,

    /// Last cleanup time
    last_cleanup: Instant,
}

impl SlidingWindow {
    fn new() -> Self {
        Self {
            requests: Vec::new(),
            last_cleanup: Instant::now(),
        }
    }

    /// Add a new request and return whether it should be allowed
    fn add_request(&mut self, config: &RateLimitConfig) -> bool {
        let now = Instant::now();

        // Cleanup old requests
        self.cleanup(now, config.window_duration);

        // Check if we're within limits
        let current_count = self.requests.len() as u64;

        if current_count < config.max_requests + config.burst_size {
            self.requests.push(now);
            true
        } else {
            false
        }
    }

    /// Get current request count in window
    fn current_count(&mut self, config: &RateLimitConfig) -> u64 {
        let now = Instant::now();
        self.cleanup(now, config.window_duration);
        self.requests.len() as u64
    }

    /// Remove requests outside the window
    fn cleanup(&mut self, now: Instant, window_duration: Duration) {
        // For short windows (< 1 second), always cleanup
        // For longer windows, only cleanup every second to avoid excessive operations
        let cleanup_interval = if window_duration < Duration::from_secs(1) {
            window_duration / 10 // Cleanup at 10% of window duration for short windows
        } else {
            Duration::from_secs(1)
        };

        if now.duration_since(self.last_cleanup) < cleanup_interval {
            return;
        }

        let cutoff = now - window_duration;
        self.requests.retain(|&req_time| req_time > cutoff);
        self.last_cleanup = now;
    }

    /// Get remaining requests in current window
    fn remaining(&mut self, config: &RateLimitConfig) -> u64 {
        let current = self.current_count(config);
        config.max_requests.saturating_sub(current)
    }

    /// Get time until next available request slot
    fn retry_after(&mut self, config: &RateLimitConfig) -> Option<Duration> {
        let current = self.current_count(config);

        if current < config.max_requests {
            return None;
        }

        // Find oldest request in window
        if let Some(&oldest) = self.requests.first() {
            let now = Instant::now();
            let elapsed = now.duration_since(oldest);

            if elapsed < config.window_duration {
                Some(config.window_duration - elapsed)
            } else {
                None
            }
        } else {
            None
        }
    }
}

/// In-memory rate limiter using sliding window algorithm
#[derive(Clone)]
pub struct InMemoryRateLimiter {
    /// Per-organization sliding windows
    windows: Arc<RwLock<HashMap<String, SlidingWindow>>>,

    /// Default rate limit configuration
    default_config: RateLimitConfig,

    /// Per-organization custom configurations
    org_configs: Arc<RwLock<HashMap<String, RateLimitConfig>>>,
}

impl InMemoryRateLimiter {
    /// Create a new in-memory rate limiter
    pub fn new(default_config: RateLimitConfig) -> Self {
        Self {
            windows: Arc::new(RwLock::new(HashMap::new())),
            default_config,
            org_configs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set custom rate limit for a specific organization
    pub async fn set_org_limit(&self, org_id: String, config: RateLimitConfig) {
        let mut configs = self.org_configs.write().await;
        configs.insert(org_id, config);
    }

    /// Remove custom rate limit for an organization
    pub async fn remove_org_limit(&self, org_id: &str) {
        let mut configs = self.org_configs.write().await;
        configs.remove(org_id);
    }

    /// Get configuration for an organization
    async fn get_config(&self, org_id: &str) -> RateLimitConfig {
        let configs = self.org_configs.read().await;
        configs.get(org_id).cloned().unwrap_or_else(|| self.default_config.clone())
    }

    /// Get current usage statistics for an organization
    pub async fn get_usage(&self, org_id: &str) -> RateLimitUsage {
        let config = self.get_config(org_id).await;
        let mut windows = self.windows.write().await;

        let window = windows.entry(org_id.to_string()).or_insert_with(SlidingWindow::new);

        RateLimitUsage {
            current: window.current_count(&config),
            limit: config.max_requests,
            remaining: window.remaining(&config),
            retry_after: window.retry_after(&config),
        }
    }
}

#[async_trait]
impl RateLimiter for InMemoryRateLimiter {
    async fn check_rate_limit(&self, organization_id: &str) -> Result<bool> {
        let config = self.get_config(organization_id).await;
        let mut windows = self.windows.write().await;

        let window = windows.entry(organization_id.to_string()).or_insert_with(SlidingWindow::new);

        let allowed = window.add_request(&config);
        let current = window.current_count(&config);
        let remaining = window.remaining(&config);

        // Record metrics
        llm_cost_ops::metrics::collectors::RateLimitMetrics::record_check(organization_id, allowed);
        llm_cost_ops::metrics::collectors::RateLimitMetrics::record_usage(
            organization_id,
            current,
            config.max_requests,
            remaining,
        );

        if !allowed {
            warn!(
                organization_id = %organization_id,
                current = current,
                limit = config.max_requests,
                "Rate limit exceeded"
            );
        } else {
            debug!(
                organization_id = %organization_id,
                current = current,
                limit = config.max_requests,
                "Rate limit check passed"
            );
        }

        Ok(allowed)
    }

    async fn record_request(&self, organization_id: &str) -> Result<()> {
        // For in-memory limiter, recording happens in check_rate_limit
        // This is here for compatibility with other implementations
        debug!(organization_id = %organization_id, "Request recorded");
        Ok(())
    }
}

/// Rate limit usage statistics
#[derive(Debug, Clone)]
pub struct RateLimitUsage {
    /// Current request count in window
    pub current: u64,

    /// Maximum allowed requests
    pub limit: u64,

    /// Remaining requests
    pub remaining: u64,

    /// Time to wait before retry (if rate limited)
    pub retry_after: Option<Duration>,
}

/// Redis-backed rate limiter for distributed systems
#[derive(Clone)]
pub struct RedisRateLimiter {
    /// Redis client
    client: redis::Client,

    /// Default rate limit configuration
    default_config: RateLimitConfig,

    /// Key prefix for Redis keys
    key_prefix: String,
}

impl RedisRateLimiter {
    /// Create a new Redis-backed rate limiter
    pub fn new(
        redis_url: &str,
        default_config: RateLimitConfig,
    ) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| CostOpsError::Integration(format!("Redis connection failed: {}", e)))?;

        Ok(Self {
            client,
            default_config,
            key_prefix: "llm_cost_ops:ratelimit".to_string(),
        })
    }

    /// Set custom key prefix
    pub fn with_prefix(mut self, prefix: String) -> Self {
        self.key_prefix = prefix;
        self
    }

    /// Get Redis key for organization
    fn get_key(&self, org_id: &str) -> String {
        format!("{}:{}", self.key_prefix, org_id)
    }

    /// Get configuration key for organization
    fn get_config_key(&self, org_id: &str) -> String {
        format!("{}:config:{}", self.key_prefix, org_id)
    }

    /// Set custom rate limit for a specific organization
    pub async fn set_org_limit(&self, org_id: String, config: RateLimitConfig) -> Result<()> {
        use redis::AsyncCommands;

        let mut con = self.client.get_async_connection().await
            .map_err(|e| CostOpsError::Integration(format!("Redis connection failed: {}", e)))?;

        let key = self.get_config_key(&org_id);
        let value = serde_json::to_string(&config)?;

        con.set::<_, _, ()>(&key, value).await
            .map_err(|e| CostOpsError::Integration(format!("Redis set failed: {}", e)))?;

        Ok(())
    }

    /// Get configuration for an organization
    async fn get_config(&self, org_id: &str) -> Result<RateLimitConfig> {
        use redis::AsyncCommands;

        let mut con = self.client.get_async_connection().await
            .map_err(|e| CostOpsError::Integration(format!("Redis connection failed: {}", e)))?;

        let key = self.get_config_key(org_id);

        let value: Option<String> = con.get(&key).await
            .map_err(|e| CostOpsError::Integration(format!("Redis get failed: {}", e)))?;

        if let Some(v) = value {
            let config: RateLimitConfig = serde_json::from_str(&v)?;
            Ok(config)
        } else {
            Ok(self.default_config.clone())
        }
    }

    /// Get current usage statistics for an organization
    pub async fn get_usage(&self, org_id: &str) -> Result<RateLimitUsage> {
        use redis::AsyncCommands;

        let config = self.get_config(org_id).await?;
        let mut con = self.client.get_async_connection().await
            .map_err(|e| CostOpsError::Integration(format!("Redis connection failed: {}", e)))?;

        let key = self.get_key(org_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now - config.window_duration.as_secs();

        // Count requests in current window
        let current: u64 = con.zcount(&key, window_start, now).await
            .map_err(|e| CostOpsError::Integration(format!("Redis zcount failed: {}", e)))?;

        let remaining = config.max_requests.saturating_sub(current);

        // Calculate retry_after if rate limited
        let retry_after = if current >= config.max_requests {
            // Get oldest request timestamp
            let oldest: Vec<(String, f64)> = con.zrange_withscores(&key, 0, 0).await
                .map_err(|e| CostOpsError::Integration(format!("Redis zrange failed: {}", e)))?;

            let oldest = oldest.into_iter().next();

            if let Some((_, score)) = oldest {
                let oldest_time = score as u64;
                let window_end = oldest_time + config.window_duration.as_secs();

                if window_end > now {
                    Some(Duration::from_secs(window_end - now))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Ok(RateLimitUsage {
            current,
            limit: config.max_requests,
            remaining,
            retry_after,
        })
    }
}

#[async_trait]
impl RateLimiter for RedisRateLimiter {
    async fn check_rate_limit(&self, organization_id: &str) -> Result<bool> {
        use redis::AsyncCommands;

        let config = self.get_config(organization_id).await?;
        let mut con = self.client.get_async_connection().await
            .map_err(|e| CostOpsError::Integration(format!("Redis connection failed: {}", e)))?;

        let key = self.get_key(organization_id);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let window_start = now - config.window_duration.as_secs();

        // Remove old entries
        let _: () = con.zrembyscore(&key, 0, window_start).await
            .map_err(|e| CostOpsError::Integration(format!("Redis zrembyscore failed: {}", e)))?;

        // Count requests in current window
        let current: u64 = con.zcard(&key).await
            .map_err(|e| CostOpsError::Integration(format!("Redis zcard failed: {}", e)))?;

        // Check if we're within limits (including burst)
        let allowed = current < config.max_requests + config.burst_size;
        let final_current = if allowed { current + 1 } else { current };
        let remaining = config.max_requests.saturating_sub(final_current);

        if allowed {
            // Add current request
            let request_id = uuid::Uuid::new_v4().to_string();
            let _: () = con.zadd(&key, &request_id, now).await
                .map_err(|e| CostOpsError::Integration(format!("Redis zadd failed: {}", e)))?;

            // Set expiry on the key
            let expiry = (config.window_duration.as_secs() + 60) as usize;
            let _: () = con.expire(&key, expiry).await
                .map_err(|e| CostOpsError::Integration(format!("Redis expire failed: {}", e)))?;

            debug!(
                organization_id = %organization_id,
                current = final_current,
                limit = config.max_requests,
                "Rate limit check passed"
            );
        } else {
            warn!(
                organization_id = %organization_id,
                current = current,
                limit = config.max_requests,
                "Rate limit exceeded"
            );
        }

        // Record metrics
        llm_cost_ops::metrics::collectors::RateLimitMetrics::record_check(organization_id, allowed);
        llm_cost_ops::metrics::collectors::RateLimitMetrics::record_usage(
            organization_id,
            final_current,
            config.max_requests,
            remaining,
        );

        Ok(allowed)
    }

    async fn record_request(&self, organization_id: &str) -> Result<()> {
        // For Redis limiter, recording happens in check_rate_limit
        debug!(organization_id = %organization_id, "Request recorded");
        Ok(())
    }
}

/// No-op rate limiter for testing or disabled rate limiting
#[derive(Clone)]
pub struct NoOpRateLimiter;

#[async_trait]
impl RateLimiter for NoOpRateLimiter {
    async fn check_rate_limit(&self, _organization_id: &str) -> Result<bool> {
        Ok(true)
    }

    async fn record_request(&self, _organization_id: &str) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_rate_limiter_basic() {
        let config = RateLimitConfig {
            max_requests: 5,
            window_duration: Duration::from_secs(10),
            burst_size: 2,
        };

        let limiter = InMemoryRateLimiter::new(config);

        // First 5 requests should pass
        for _ in 0..5 {
            assert!(limiter.check_rate_limit("org-1").await.unwrap());
        }

        // Next 2 requests should pass (burst)
        for _ in 0..2 {
            assert!(limiter.check_rate_limit("org-1").await.unwrap());
        }

        // 8th request should fail
        assert!(!limiter.check_rate_limit("org-1").await.unwrap());
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_per_org() {
        let config = RateLimitConfig {
            max_requests: 3,
            window_duration: Duration::from_secs(10),
            burst_size: 1,
        };

        let limiter = InMemoryRateLimiter::new(config);

        // org-1 uses its quota
        for _ in 0..4 {
            assert!(limiter.check_rate_limit("org-1").await.unwrap());
        }
        assert!(!limiter.check_rate_limit("org-1").await.unwrap());

        // org-2 should still have its full quota
        for _ in 0..4 {
            assert!(limiter.check_rate_limit("org-2").await.unwrap());
        }
    }

    #[tokio::test]
    async fn test_in_memory_rate_limiter_custom_limits() {
        let default_config = RateLimitConfig {
            max_requests: 5,
            window_duration: Duration::from_secs(10),
            burst_size: 1,
        };

        let limiter = InMemoryRateLimiter::new(default_config.clone());

        // Set custom limit for org-premium
        let premium_config = RateLimitConfig {
            max_requests: 100,
            window_duration: Duration::from_secs(10),
            burst_size: 10,
        };
        limiter.set_org_limit("org-premium".to_string(), premium_config).await;

        // org-basic should have default limits
        for _ in 0..6 {
            assert!(limiter.check_rate_limit("org-basic").await.unwrap());
        }
        assert!(!limiter.check_rate_limit("org-basic").await.unwrap());

        // org-premium should have higher limits
        for _ in 0..100 {
            assert!(limiter.check_rate_limit("org-premium").await.unwrap());
        }
    }

    #[tokio::test]
    async fn test_usage_stats() {
        let config = RateLimitConfig {
            max_requests: 10,
            window_duration: Duration::from_secs(60),
            burst_size: 2,
        };

        let limiter = InMemoryRateLimiter::new(config);

        // Make 3 requests
        for _ in 0..3 {
            limiter.check_rate_limit("org-1").await.unwrap();
        }

        let usage = limiter.get_usage("org-1").await;
        assert_eq!(usage.current, 3);
        assert_eq!(usage.limit, 10);
        assert_eq!(usage.remaining, 7);
        assert!(usage.retry_after.is_none());
    }

    #[test]
    fn test_rate_limit_config_builders() {
        let per_minute = RateLimitConfig::per_minute(60);
        assert_eq!(per_minute.max_requests, 60);
        assert_eq!(per_minute.window_duration, Duration::from_secs(60));

        let per_hour = RateLimitConfig::per_hour(3600);
        assert_eq!(per_hour.max_requests, 3600);
        assert_eq!(per_hour.window_duration, Duration::from_secs(3600));

        let per_day = RateLimitConfig::per_day(86400);
        assert_eq!(per_day.max_requests, 86400);
        assert_eq!(per_day.window_duration, Duration::from_secs(86400));
    }

    #[tokio::test]
    async fn test_no_op_rate_limiter() {
        let limiter = NoOpRateLimiter;

        // Should always allow requests
        for _ in 0..1000 {
            assert!(limiter.check_rate_limit("org-1").await.unwrap());
        }
    }
}
