---
sidebar_position: 4
title: Rust SDK
---

# Rust SDK

Enterprise-grade Rust SDK for the LLM Cost Operations platform. Maximum performance and type safety for systems programming.

## Features

- **Type-Safe**: Compile-time guarantees with Rust's type system
- **High-Performance**: Zero-cost abstractions and minimal allocations
- **Async/Await**: Built on Tokio for efficient concurrency
- **Error Handling**: Result types and comprehensive error patterns
- **Builder Pattern**: Ergonomic client configuration
- **Tracing**: Structured logging with tracing crate
- **Telemetry**: Built-in metrics collection
- **Rate Limiting**: Efficient semaphore-based rate limiting
- **Retry Logic**: Configurable retry policies with backoff

## Quick Example

```rust
use llm_cost_ops::{CostOpsClient, ClientConfig, UsageRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let config = ClientConfig::builder()
        .base_url("https://api.llm-cost-ops.example.com")?
        .api_key("your-api-key")
        .build()?;

    let client = CostOpsClient::new(config)?;

    // Submit usage
    let usage = client.submit_usage(UsageRequest {
        organization_id: "org-123".to_string(),
        provider: "openai".to_string(),
        model_id: "gpt-4".to_string(),
        input_tokens: 1000,
        output_tokens: 500,
        total_tokens: 1500,
        ..Default::default()
    }).await?;

    println!("Usage ID: {}", usage.usage_id);
    println!("Estimated cost: ${}", usage.estimated_cost);

    // Get costs
    let costs = client.get_costs(CostRequest {
        organization_id: "org-123".to_string(),
        start_date: "2025-01-01T00:00:00Z".to_string(),
        end_date: "2025-01-31T23:59:59Z".to_string(),
        ..Default::default()
    }).await?;

    println!("Total cost: ${}", costs.total_cost);

    Ok(())
}
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-cost-ops = "0.1.0"
```

Or use cargo:

```bash
cargo add llm-cost-ops
```

## Requirements

- Rust 1.70 or higher
- Tokio runtime for async operations

## Crate Structure

```
llm-cost-ops/
├── src/
│   ├── lib.rs         # Public API
│   ├── client.rs      # Main client
│   ├── config.rs      # Configuration
│   ├── error.rs       # Error types
│   ├── types.rs       # Data models
│   ├── retry.rs       # Retry logic
│   └── telemetry.rs   # Telemetry
```

## Core Concepts

### Builder Pattern

```rust
use llm_cost_ops::{CostOpsClient, ClientConfig, RetryConfig};
use std::time::Duration;

let config = ClientConfig::builder()
    .base_url("https://api.llm-cost-ops.example.com")?
    .api_key("your-api-key")
    .timeout(Duration::from_secs(30))
    .retry_config(RetryConfig {
        max_retries: 3,
        initial_delay: Duration::from_secs(1),
        max_delay: Duration::from_secs(30),
        backoff_factor: 2.0,
    })
    .add_header("X-Custom", "value")
    .build()?;

let client = CostOpsClient::new(config)?;
```

### Error Handling

```rust
use llm_cost_ops::{CostOpsClient, SdkError};

match client.submit_usage(request).await {
    Ok(usage) => println!("Success: {}", usage.usage_id),
    Err(SdkError::Authentication(msg)) => eprintln!("Auth failed: {}", msg),
    Err(SdkError::RateLimit { retry_after }) => {
        eprintln!("Rate limited, retry after {} seconds", retry_after);
    }
    Err(SdkError::Validation(msg)) => eprintln!("Invalid data: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Async Operations

All SDK operations are async:

```rust
use tokio::time::{timeout, Duration};

// With timeout
let result = timeout(
    Duration::from_secs(30),
    client.get_costs(request)
).await??;

// Concurrent operations
let (usage, costs) = tokio::join!(
    client.submit_usage(usage_request),
    client.get_costs(cost_request)
);
```

## Performance

- **Zero-Copy**: Efficient data handling with references
- **Connection Pooling**: Reusable HTTP connections
- **Async I/O**: Non-blocking operations with Tokio
- **Type Safety**: Compile-time error detection
- **Memory Efficient**: No garbage collection overhead

## Next Steps

- [Installation Guide](/docs/sdks/rust/installation)
- [Quick Start](/docs/sdks/rust/quick-start)
- [API Reference](/docs/sdks/rust/api-reference)
- [Examples](/docs/sdks/rust/examples)
- [Troubleshooting](/docs/sdks/rust/troubleshooting)

## Resources

- [docs.rs](https://docs.rs/llm-cost-ops)
- [crates.io](https://crates.io/crates/llm-cost-ops)
- [Source Code](https://github.com/llm-devops/llm-cost-ops/tree/main/src/sdk)
- [Examples](https://github.com/llm-devops/llm-cost-ops/tree/main/examples)

## Support

- [GitHub Issues](https://github.com/llm-devops/llm-cost-ops/issues)
- [Discord Community](https://discord.gg/llm-cost-ops)
- Email: support@llm-cost-ops.dev
