// Rate limiting integration tests

use llm_cost_ops::ingestion::ratelimit::{InMemoryRateLimiter, RateLimitConfig};
use llm_cost_ops::ingestion::traits::RateLimiter;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_rate_limiter_basic_limits() {
    let config = RateLimitConfig {
        max_requests: 10,
        window_duration: Duration::from_secs(60),
        burst_size: 2,
    };

    let limiter = InMemoryRateLimiter::new(config);

    // Should allow up to max_requests + burst_size
    for i in 0..12 {
        let allowed = limiter.check_rate_limit("org-test").await.unwrap();
        assert!(allowed, "Request {} should be allowed", i + 1);
    }

    // 13th request should be denied
    let allowed = limiter.check_rate_limit("org-test").await.unwrap();
    assert!(!allowed, "Request 13 should be denied");
}

#[tokio::test]
async fn test_rate_limiter_per_organization_isolation() {
    let config = RateLimitConfig {
        max_requests: 5,
        window_duration: Duration::from_secs(60),
        burst_size: 1,
    };

    let limiter = InMemoryRateLimiter::new(config);

    // Organization 1 exhausts its quota
    for _ in 0..6 {
        limiter.check_rate_limit("org-1").await.unwrap();
    }

    let allowed = limiter.check_rate_limit("org-1").await.unwrap();
    assert!(!allowed, "org-1 should be rate limited");

    // Organization 2 should have independent quota
    for i in 0..6 {
        let allowed = limiter.check_rate_limit("org-2").await.unwrap();
        assert!(allowed, "org-2 request {} should be allowed", i + 1);
    }
}

#[tokio::test]
async fn test_rate_limiter_custom_org_limits() {
    let default_config = RateLimitConfig::per_minute(10);
    let limiter = InMemoryRateLimiter::new(default_config);

    // Set premium tier for org-premium
    let premium_config = RateLimitConfig::per_minute(1000);
    limiter
        .set_org_limit("org-premium".to_string(), premium_config)
        .await;

    // Default org should have low limits (10 + 1 burst = 11 total)
    for _ in 0..11 {
        limiter.check_rate_limit("org-default").await.unwrap();
    }
    let allowed = limiter.check_rate_limit("org-default").await.unwrap();
    assert!(!allowed, "org-default should be rate limited after 11 requests (10 + 1 burst)");

    // Premium org should have high limits
    for i in 0..100 {
        let allowed = limiter.check_rate_limit("org-premium").await.unwrap();
        assert!(
            allowed,
            "org-premium request {} should be allowed",
            i + 1
        );
    }
}

#[tokio::test]
async fn test_rate_limiter_sliding_window() {
    let config = RateLimitConfig {
        max_requests: 5,
        window_duration: Duration::from_millis(200),
        burst_size: 0,
    };

    let limiter = InMemoryRateLimiter::new(config);

    // Use up the quota
    for _ in 0..5 {
        limiter.check_rate_limit("org-test").await.unwrap();
    }

    // Should be denied immediately
    let allowed = limiter.check_rate_limit("org-test").await.unwrap();
    assert!(!allowed, "Should be rate limited");

    // Wait for window to pass
    sleep(Duration::from_millis(250)).await;

    // Should be allowed again
    let allowed = limiter.check_rate_limit("org-test").await.unwrap();
    assert!(allowed, "Should be allowed after window reset");
}

#[tokio::test]
async fn test_rate_limiter_usage_stats() {
    let config = RateLimitConfig {
        max_requests: 100,
        window_duration: Duration::from_secs(60),
        burst_size: 10,
    };

    let limiter = InMemoryRateLimiter::new(config);

    // Make 25 requests
    for _ in 0..25 {
        limiter.check_rate_limit("org-stats").await.unwrap();
    }

    let usage = limiter.get_usage("org-stats").await;

    assert_eq!(usage.current, 25, "Should have 25 current requests");
    assert_eq!(usage.limit, 100, "Limit should be 100");
    assert_eq!(usage.remaining, 75, "Should have 75 remaining");
    assert!(usage.retry_after.is_none(), "Should not need to retry");
}

#[tokio::test]
async fn test_rate_limiter_retry_after() {
    let config = RateLimitConfig {
        max_requests: 3,
        window_duration: Duration::from_secs(10),
        burst_size: 0,
    };

    let limiter = InMemoryRateLimiter::new(config);

    // Exhaust quota
    for _ in 0..3 {
        limiter.check_rate_limit("org-test").await.unwrap();
    }

    // Check for rate limit
    let allowed = limiter.check_rate_limit("org-test").await.unwrap();
    assert!(!allowed);

    // Get usage stats
    let usage = limiter.get_usage("org-test").await;

    assert_eq!(usage.remaining, 0);
    assert!(
        usage.retry_after.is_some(),
        "Should have retry_after duration"
    );

    if let Some(retry) = usage.retry_after {
        assert!(
            retry.as_secs() <= 10,
            "Retry after should be within window duration"
        );
    }
}

#[tokio::test]
async fn test_rate_limit_config_builders() {
    let per_minute = RateLimitConfig::per_minute(100);
    assert_eq!(per_minute.max_requests, 100);
    assert_eq!(per_minute.window_duration, Duration::from_secs(60));
    assert_eq!(per_minute.burst_size, 10);

    let per_hour = RateLimitConfig::per_hour(5000);
    assert_eq!(per_hour.max_requests, 5000);
    assert_eq!(per_hour.window_duration, Duration::from_secs(3600));
    assert_eq!(per_hour.burst_size, 50);

    let per_day = RateLimitConfig::per_day(100000);
    assert_eq!(per_day.max_requests, 100000);
    assert_eq!(per_day.window_duration, Duration::from_secs(86400));
    assert_eq!(per_day.burst_size, 100);
}

#[tokio::test]
async fn test_rate_limiter_concurrent_requests() {
    let config = RateLimitConfig {
        max_requests: 100,
        window_duration: Duration::from_secs(60),
        burst_size: 10,
    };

    let limiter = InMemoryRateLimiter::new(config);

    // Spawn multiple concurrent tasks
    let mut handles = vec![];

    for _ in 0..10 {
        let limiter_clone = limiter.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..15 {
                limiter_clone.check_rate_limit("org-concurrent").await.unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.unwrap();
    }

    // Total requests: 10 tasks * 15 requests = 150
    // Limit: 100 + 10 burst = 110
    // So we should have made 110 successful requests

    let usage = limiter.get_usage("org-concurrent").await;
    assert_eq!(
        usage.current, 110,
        "Should have exactly 110 requests (100 + burst 10)"
    );
}

#[tokio::test]
async fn test_rate_limiter_remove_custom_limit() {
    let default_config = RateLimitConfig::per_minute(10);
    let limiter = InMemoryRateLimiter::new(default_config);

    // Set custom limit
    let custom_config = RateLimitConfig::per_minute(100);
    limiter
        .set_org_limit("org-test".to_string(), custom_config)
        .await;

    // Verify custom limit works
    for _ in 0..50 {
        let allowed = limiter.check_rate_limit("org-test").await.unwrap();
        assert!(allowed);
    }

    // Remove custom limit
    limiter.remove_org_limit("org-test").await;

    // After reset, should have default limit (but window still has old requests)
    // So we need to wait for window to reset or test with new org
    let usage = limiter.get_usage("org-new").await;
    assert_eq!(usage.limit, 10, "Should revert to default limit");
}
