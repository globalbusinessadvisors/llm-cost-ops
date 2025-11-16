//! Advanced SDK usage example
//!
//! This example demonstrates:
//! - Custom retry configuration
//! - Rate limiting
//! - Telemetry and metrics
//! - Custom headers
//! - Error handling patterns

use llm_cost_ops::sdk::{
    ClientConfig, CostOpsClient, PoolConfig, RateLimitConfig, RetryConfig, TelemetryConfig,
    UsageRequest,
};
use std::time::Duration;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing with custom configuration
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_target(false)
        .init();

    println!("Advanced SDK Usage Example\n");

    // Example 1: Custom retry configuration
    let retry_config = RetryConfig {
        max_attempts: 5,
        initial_backoff: Duration::from_millis(100),
        max_backoff: Duration::from_secs(30),
        multiplier: 2.0,
        jitter: true, // Add jitter to avoid thundering herd
    };

    // Example 2: Custom pool configuration
    let pool_config = PoolConfig {
        max_idle: 20,
        max_per_host: 50,
        idle_timeout: Duration::from_secs(120),
    };

    // Example 3: Rate limiting configuration
    let rate_limit_config = RateLimitConfig {
        requests_per_second: Some(100),
        burst_size: Some(20),
        enabled: true,
    };

    // Example 4: Telemetry configuration
    let telemetry_config = TelemetryConfig {
        metrics_enabled: true,
        tracing_enabled: true,
        logging_enabled: true,
        metrics_endpoint: Some("http://localhost:9090".to_string()),
        trace_endpoint: Some("http://localhost:4317".to_string()),
    };

    // Build client with all custom configurations
    let config = ClientConfig::builder()
        .base_url("http://localhost:8080")?
        .api_key("your-api-key-here")
        .timeout(Duration::from_secs(60))
        .retry_config(retry_config)
        .pool_config(pool_config)
        .rate_limit_config(rate_limit_config)
        .telemetry_config(telemetry_config)
        .add_header("X-Custom-Header", "custom-value")
        .add_header("X-Request-Source", "advanced-example")
        .user_agent("advanced-sdk-example/1.0")
        .build()?;

    let client = CostOpsClient::new(config)?;

    println!("Client initialized with custom configuration");
    println!("- Retry attempts: 5");
    println!("- Rate limit: 100 req/s");
    println!("- Telemetry: enabled");
    println!("- Custom headers: 2\n");

    // Example 5: Batch operations with error handling
    let org_id = Uuid::new_v4();
    let mut success_count = 0;
    let mut error_count = 0;

    println!("Submitting batch of usage records...");

    for i in 1..=10 {
        let usage = UsageRequest {
            organization_id: org_id,
            provider: "openai".to_string(),
            model: "gpt-4".to_string(),
            prompt_tokens: 100 * i,
            completion_tokens: 50 * i,
            total_tokens: 150 * i,
            timestamp: chrono::Utc::now(),
            request_id: Some(format!("batch-req-{}", i)),
            metadata: std::collections::HashMap::new(),
        };

        match client.submit_usage(usage).await {
            Ok(response) => {
                success_count += 1;
                println!("  [{}] Success - Record ID: {}", i, response.id);
            }
            Err(e) => {
                error_count += 1;
                eprintln!("  [{}] Error: {}", i, e);

                // Handle specific error types
                if e.is_retryable() {
                    eprintln!("    -> This error is retryable");
                }
                if let Some(status) = e.status_code() {
                    eprintln!("    -> HTTP status: {}", status);
                }
            }
        }

        // Small delay to demonstrate rate limiting
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    println!("\nBatch processing complete:");
    println!("  Success: {}", success_count);
    println!("  Errors: {}", error_count);

    // Example 6: Access telemetry data
    let telemetry = client.telemetry();
    println!("\nTelemetry information:");
    println!("  Metrics collection: enabled");
    println!("  Active connections tracked: yes");

    // Example 7: Graceful error handling with match
    let usage = UsageRequest {
        organization_id: org_id,
        provider: "anthropic".to_string(),
        model: "claude-3-opus".to_string(),
        prompt_tokens: 1000,
        completion_tokens: 500,
        total_tokens: 1500,
        timestamp: chrono::Utc::now(),
        request_id: Some("final-req".to_string()),
        metadata: std::collections::HashMap::new(),
    };

    match client.submit_usage(usage).await {
        Ok(response) => {
            println!("\nFinal submission successful!");
            if let Some(cost) = response.cost {
                println!("  Cost breakdown:");
                println!("    Prompt: {} {}", cost.prompt_cost, cost.currency);
                println!("    Completion: {} {}", cost.completion_cost, cost.currency);
                println!("    Total: {} {}", cost.total_cost, cost.currency);
            }
        }
        Err(e) => {
            eprintln!("\nFinal submission failed: {}", e);

            // Demonstrate error pattern matching
            use llm_cost_ops::sdk::SdkError;
            match e {
                SdkError::RateLimitExceeded { retry_after } => {
                    eprintln!("  Rate limited. Retry after: {:?}", retry_after);
                }
                SdkError::Authentication(msg) => {
                    eprintln!("  Authentication failed: {}", msg);
                }
                SdkError::Validation(msg) => {
                    eprintln!("  Validation error: {}", msg);
                }
                SdkError::Api {
                    status,
                    message,
                    details,
                } => {
                    eprintln!("  API error [{}]: {}", status, message);
                    if let Some(details) = details {
                        eprintln!("  Details: {}", details);
                    }
                }
                SdkError::RetryExhausted { attempts, .. } => {
                    eprintln!("  Retry exhausted after {} attempts", attempts);
                }
                _ => {
                    eprintln!("  Other error: {}", e);
                }
            }
        }
    }

    println!("\nAdvanced example complete!");

    Ok(())
}
