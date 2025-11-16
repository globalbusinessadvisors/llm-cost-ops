# LLM Cost Ops - SDK

[![Crates.io](https://img.shields.io/crates/v/llm-cost-ops-sdk.svg)](https://crates.io/crates/llm-cost-ops-sdk)
[![Documentation](https://docs.rs/llm-cost-ops-sdk/badge.svg)](https://docs.rs/llm-cost-ops-sdk)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Rust SDK client for LLM Cost Ops platform**

A production-ready Rust client library for interacting with the LLM Cost Ops API, providing type-safe access to cost tracking, analytics, and reporting features.

## Features

- **Type-Safe API Client** - Fully typed request/response models
- **Async/Await Support** - Built on tokio for high-performance async operations
- **Authentication** - JWT and API key authentication support
- **Error Handling** - Comprehensive error types with context
- **Retry Logic** - Automatic retry with exponential backoff
- **Telemetry** - Built-in metrics and tracing
- **Pagination** - Helper methods for paginated endpoints
- **Rate Limiting** - Client-side rate limiting support

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-cost-ops-sdk = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use llm_cost_ops_sdk::{Client, ClientConfig, AuthMethod};
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with API key
    let config = ClientConfig {
        base_url: "https://api.example.com".to_string(),
        auth: AuthMethod::ApiKey {
            key: "your-api-key".to_string(),
        },
        timeout_secs: 30,
        max_retries: 3,
    };

    let client = Client::new(config)?;

    // Submit usage record
    let usage = client
        .usage()
        .create()
        .provider("openai")
        .model("gpt-4")
        .organization_id("org-123")
        .prompt_tokens(1500)
        .completion_tokens(500)
        .submit()
        .await?;

    println!("Usage recorded: {}", usage.id);

    Ok(())
}
```

### JWT Authentication

```rust
use llm_cost_ops_sdk::{Client, ClientConfig, AuthMethod};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with JWT token
    let config = ClientConfig {
        base_url: "https://api.example.com".to_string(),
        auth: AuthMethod::Jwt {
            token: "your-jwt-token".to_string(),
        },
        timeout_secs: 30,
        max_retries: 3,
    };

    let client = Client::new(config)?;

    // Refresh token automatically
    let refreshed_client = client.refresh_token().await?;

    Ok(())
}
```

### Query Costs

```rust
use llm_cost_ops_sdk::Client;
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(config)?;

    // Query costs for last 30 days
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(30);

    let costs = client
        .costs()
        .query()
        .organization_id("org-123")
        .date_range(start_date, end_date)
        .group_by("provider")
        .execute()
        .await?;

    for cost in costs.data {
        println!("{}: ${}", cost.provider, cost.total_cost);
    }

    Ok(())
}
```

### Generate Reports

```rust
use llm_cost_ops_sdk::{Client, ReportType, ExportFormat};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(config)?;

    // Generate cost report
    let report = client
        .reports()
        .generate()
        .report_type(ReportType::Cost)
        .organization_id("org-123")
        .format(ExportFormat::Csv)
        .email_to("finance@example.com")
        .submit()
        .await?;

    println!("Report generated: {}", report.id);

    // Download report
    let data = client
        .reports()
        .download(&report.id)
        .await?;

    std::fs::write("report.csv", data)?;

    Ok(())
}
```

### Usage Analytics

```rust
use llm_cost_ops_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(config)?;

    // Get usage analytics
    let analytics = client
        .analytics()
        .usage()
        .organization_id("org-123")
        .group_by("model")
        .top(10)
        .execute()
        .await?;

    for item in analytics.data {
        println!("{}: {} tokens", item.model, item.total_tokens);
    }

    Ok(())
}
```

### Forecasting

```rust
use llm_cost_ops_sdk::{Client, ForecastHorizon};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(config)?;

    // Get cost forecast
    let forecast = client
        .forecasting()
        .cost_forecast()
        .organization_id("org-123")
        .horizon(ForecastHorizon::Days30)
        .confidence_level(0.95)
        .execute()
        .await?;

    println!("Forecast for next 30 days: ${}", forecast.predicted_cost);
    println!("Confidence interval: ${} - ${}",
        forecast.lower_bound, forecast.upper_bound);

    Ok(())
}
```

### Pagination

```rust
use llm_cost_ops_sdk::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(config)?;

    // Paginate through usage records
    let mut page = 1;
    loop {
        let response = client
            .usage()
            .list()
            .organization_id("org-123")
            .page(page)
            .per_page(100)
            .execute()
            .await?;

        println!("Page {}: {} records", page, response.data.len());

        if !response.has_more {
            break;
        }
        page += 1;
    }

    Ok(())
}
```

### Error Handling

```rust
use llm_cost_ops_sdk::{Client, SdkError};

#[tokio::main]
async fn main() {
    let client = Client::new(config).unwrap();

    match client.usage().list().execute().await {
        Ok(response) => {
            println!("Success: {} records", response.data.len());
        }
        Err(SdkError::Authentication(msg)) => {
            eprintln!("Authentication failed: {}", msg);
        }
        Err(SdkError::RateLimited { retry_after }) => {
            eprintln!("Rate limited, retry after {} seconds", retry_after);
        }
        Err(SdkError::Network(err)) => {
            eprintln!("Network error: {}", err);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
```

## Configuration

### Environment Variables

```bash
export COST_OPS_API_URL="https://api.example.com"
export COST_OPS_API_KEY="your-api-key"
export COST_OPS_TIMEOUT=30
export COST_OPS_MAX_RETRIES=3
```

### Configuration File

```rust
use llm_cost_ops_sdk::ClientConfig;

let config = ClientConfig::from_env()?;
let client = Client::new(config)?;
```

## Features

- **Automatic Retries** - Failed requests are automatically retried with exponential backoff
- **Connection Pooling** - Efficient HTTP connection reuse
- **Timeout Handling** - Configurable request timeouts
- **Custom Headers** - Add custom headers to all requests
- **Telemetry** - OpenTelemetry integration for distributed tracing
- **Mock Server** - Built-in mock server for testing

## Testing

```rust
use llm_cost_ops_sdk::testing::MockServer;

#[tokio::test]
async fn test_usage_creation() {
    let mock = MockServer::start().await;

    mock.expect_usage_create()
        .with_provider("openai")
        .respond_with_id("usage-123")
        .await;

    let client = Client::new(mock.client_config())?;
    let usage = client.usage().create().provider("openai").submit().await?;

    assert_eq!(usage.id, "usage-123");
}
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Links

- **Documentation**: [https://docs.rs/llm-cost-ops-sdk](https://docs.rs/llm-cost-ops-sdk)
- **Core Library**: [https://crates.io/crates/llm-cost-ops](https://crates.io/crates/llm-cost-ops)
- **API Documentation**: [https://github.com/globalbusinessadvisors/llm-cost-ops/blob/main/docs/api.md](https://github.com/globalbusinessadvisors/llm-cost-ops/blob/main/docs/api.md)
- **Repository**: [https://github.com/globalbusinessadvisors/llm-cost-ops](https://github.com/globalbusinessadvisors/llm-cost-ops)
