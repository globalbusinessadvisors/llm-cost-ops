# Rust SDK Tutorial

## Table of Contents
- [Prerequisites](#prerequisites)
- [Cargo Setup](#cargo-setup)
- [Basic Usage](#basic-usage)
- [Builder Pattern](#builder-pattern)
- [Async/Await with Tokio](#asyncawait-with-tokio)
- [Result and Option Handling](#result-and-option-handling)
- [API Methods](#api-methods)
- [Error Handling](#error-handling)
- [Custom Types with Serde](#custom-types-with-serde)
- [Testing](#testing)
- [Benchmarking](#benchmarking)
- [Zero-Copy Optimizations](#zero-copy-optimizations)
- [Lifetime Management](#lifetime-management)
- [Advanced Patterns](#advanced-patterns)
- [Performance Optimization](#performance-optimization)

## Prerequisites

Before getting started with the Rust SDK, ensure you have:

- Rust 1.70 or higher
- Cargo package manager
- API key from LLM Cost Ops platform
- Basic understanding of Rust's ownership model
- Familiarity with async/await in Rust

## Cargo Setup

### Initialize Project

```bash
cargo new llm-cost-tracker
cd llm-cost-tracker
```

### Cargo.toml Configuration

```toml
[package]
name = "llm-cost-tracker"
version = "0.1.0"
edition = "2021"

[dependencies]
llm-cost-ops = "1.0.0"
tokio = { version = "1.35", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
thiserror = "1.0"
chrono = "0.4"
dotenv = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"

[dev-dependencies]
mockito = "1.2"
criterion = "0.5"

[[bench]]
name = "cost_ops_bench"
harness = false
```

### Environment Setup

Create a `.env` file:

```bash
LLM_COST_OPS_API_KEY=your_api_key_here
LLM_COST_OPS_BASE_URL=https://api.llmcostops.com
LLM_COST_OPS_TIMEOUT=30
```

### Load Environment Variables

```rust
use dotenv::dotenv;
use std::env;

fn main() {
    // Load .env file
    dotenv().ok();

    let api_key = env::var("LLM_COST_OPS_API_KEY")
        .expect("LLM_COST_OPS_API_KEY must be set");

    let base_url = env::var("LLM_COST_OPS_BASE_URL")
        .unwrap_or_else(|_| "https://api.llmcostops.com".to_string());
}
```

## Basic Usage

### Client Initialization

```rust
use llm_cost_ops::{Client, Config};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = Config {
        api_key: std::env::var("LLM_COST_OPS_API_KEY")?,
        base_url: "https://api.llmcostops.com".to_string(),
        timeout: Duration::from_secs(30),
        max_retries: 3,
    };

    // Create client
    let client = Client::new(config)?;

    // Use client
    let costs = client.costs()
        .get_costs("2025-01-01", "2025-01-31")
        .await?;

    println!("Total cost: ${:.2}", costs.total_cost);

    Ok(())
}
```

### Simple Cost Query

```rust
use llm_cost_ops::Client;
use chrono::{DateTime, Duration, Utc};

async fn get_costs(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let end_date = Utc::now();
    let start_date = end_date - Duration::days(7);

    let costs = client.costs()
        .get_costs(
            &start_date.format("%Y-%m-%d").to_string(),
            &end_date.format("%Y-%m-%d").to_string(),
        )
        .await?;

    println!("Total cost: ${:.2}", costs.total_cost);

    for item in costs.items {
        println!("{}: ${:.2}", item.date, item.amount);
    }

    Ok(())
}
```

### Track Usage

```rust
use llm_cost_ops::{Client, UsageCreate};
use chrono::Utc;
use std::collections::HashMap;

async fn track_usage(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let mut metadata = HashMap::new();
    metadata.insert("user_id".to_string(), "user_123".to_string());
    metadata.insert("session_id".to_string(), "session_456".to_string());

    let usage = UsageCreate {
        model: "gpt-4".to_string(),
        tokens_prompt: 1000,
        tokens_completion: 500,
        request_count: 1,
        timestamp: Utc::now(),
        metadata: Some(metadata),
    };

    let created = client.usage()
        .create_usage(usage)
        .await?;

    println!("Usage ID: {}", created.id);
    println!("Cost: ${:.4}", created.cost);

    Ok(())
}
```

### Get Pricing

```rust
use llm_cost_ops::Client;

async fn get_pricing(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let pricing = client.pricing()
        .get_model_pricing("gpt-4", "openai")
        .await?;

    println!("Model: {}", pricing.model);
    println!("Prompt: ${:.4} per 1K tokens", pricing.prompt_price_per_1k);
    println!("Completion: ${:.4} per 1K tokens", pricing.completion_price_per_1k);

    Ok(())
}
```

## Builder Pattern

### Client Builder

```rust
use llm_cost_ops::ClientBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .api_key(std::env::var("LLM_COST_OPS_API_KEY")?)
        .base_url("https://api.llmcostops.com")
        .timeout(Duration::from_secs(60))
        .max_retries(5)
        .user_agent("my-app/1.0.0")
        .build()?;

    Ok(())
}
```

### Request Builder

```rust
use llm_cost_ops::{Client, CostsQueryBuilder};

async fn query_costs(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let costs = client.costs()
        .query()
        .start_date("2025-01-01")
        .end_date("2025-01-31")
        .group_by(vec!["model", "provider"])
        .granularity("daily")
        .filters(|f| {
            f.model("gpt-4")
                .environment("production")
        })
        .execute()
        .await?;

    println!("Total cost: ${:.2}", costs.total_cost);

    Ok(())
}
```

### Usage Builder

```rust
use llm_cost_ops::{Client, UsageBuilder};
use chrono::Utc;

async fn create_usage(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let usage = UsageBuilder::new()
        .model("gpt-4")
        .tokens_prompt(1000)
        .tokens_completion(500)
        .request_count(1)
        .timestamp(Utc::now())
        .metadata("user_id", "user_123")
        .metadata("application", "chatbot")
        .build()?;

    let created = client.usage()
        .create(usage)
        .await?;

    println!("Created usage: {}", created.id);

    Ok(())
}
```

### Budget Builder

```rust
use llm_cost_ops::{Client, BudgetBuilder, BudgetAlert};

async fn create_budget(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let budget = BudgetBuilder::new()
        .name("Monthly Production Budget")
        .amount(1000.0)
        .period("monthly")
        .start_date("2025-01-01")
        .alert(BudgetAlert {
            threshold: 0.5,
            alert_type: "email".to_string(),
            config: None,
        })
        .alert(BudgetAlert {
            threshold: 0.8,
            alert_type: "slack".to_string(),
            config: None,
        })
        .filter("environment", "production")
        .build()?;

    let created = client.budgets()
        .create(budget)
        .await?;

    println!("Created budget: {}", created.id);

    Ok(())
}
```

## Async/Await with Tokio

### Basic Async Usage

```rust
use llm_cost_ops::Client;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    let costs = client.costs()
        .get_costs("2025-01-01", "2025-01-31")
        .await?;

    println!("Total cost: ${:.2}", costs.total_cost);

    Ok(())
}
```

### Concurrent Requests

```rust
use llm_cost_ops::Client;
use tokio;

async fn fetch_all_data(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let (costs, usage, analytics) = tokio::try_join!(
        client.costs().get_costs("2025-01-01", "2025-01-31"),
        client.usage().list_usage("2025-01-01", "2025-01-31"),
        client.analytics().get_usage_analytics("2025-01-01", "2025-01-31")
    )?;

    println!("Total cost: ${:.2}", costs.total_cost);
    println!("Total requests: {}", usage.total_count);
    println!("Analytics groups: {}", analytics.groups.len());

    Ok(())
}
```

### Async Iterator

```rust
use llm_cost_ops::Client;
use futures::stream::{self, StreamExt};

async fn process_usage_stream(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let mut page = 1;
    let page_size = 100;

    loop {
        let usage = client.usage()
            .list_usage("2025-01-01", "2025-01-31")
            .page(page)
            .page_size(page_size)
            .await?;

        for record in usage.items {
            println!("{}: ${:.4}", record.model, record.cost);
        }

        if !usage.has_next {
            break;
        }

        page += 1;
    }

    Ok(())
}
```

### Task Spawning

```rust
use llm_cost_ops::Client;
use tokio::task;
use std::sync::Arc;

async fn parallel_processing(client: Arc<Client>) -> Result<(), Box<dyn std::error::Error>> {
    let client1 = Arc::clone(&client);
    let handle1 = task::spawn(async move {
        client1.costs().get_costs("2025-01-01", "2025-01-31").await
    });

    let client2 = Arc::clone(&client);
    let handle2 = task::spawn(async move {
        client2.usage().list_usage("2025-01-01", "2025-01-31").await
    });

    let client3 = Arc::clone(&client);
    let handle3 = task::spawn(async move {
        client3.analytics().get_usage_analytics("2025-01-01", "2025-01-31").await
    });

    let (costs, usage, analytics) = tokio::try_join!(handle1, handle2, handle3)?;

    println!("Total cost: ${:.2}", costs?.total_cost);
    println!("Total requests: {}", usage?.total_count);
    println!("Analytics groups: {}", analytics?.groups.len());

    Ok(())
}
```

### Select! Macro

```rust
use llm_cost_ops::Client;
use tokio::time::{timeout, Duration};

async fn fetch_with_timeout(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    tokio::select! {
        result = client.costs().get_costs("2025-01-01", "2025-01-31") => {
            match result {
                Ok(costs) => println!("Total cost: ${:.2}", costs.total_cost),
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        _ = tokio::time::sleep(Duration::from_secs(30)) => {
            eprintln!("Request timed out");
        }
    }

    Ok(())
}
```

## Result and Option Handling

### Result Type

```rust
use llm_cost_ops::{Client, CostOpsError};

async fn handle_result(client: &Client) -> Result<f64, CostOpsError> {
    let costs = client.costs()
        .get_costs("2025-01-01", "2025-01-31")
        .await?;

    Ok(costs.total_cost)
}

// Usage
match handle_result(&client).await {
    Ok(total) => println!("Total cost: ${:.2}", total),
    Err(e) => eprintln!("Error: {}", e),
}
```

### Option Handling

```rust
use llm_cost_ops::Client;

async fn find_usage(client: &Client, id: &str) -> Option<f64> {
    client.usage()
        .get_by_id(id)
        .await
        .ok()
        .map(|usage| usage.cost)
}

// Usage
if let Some(cost) = find_usage(&client, "usage_123").await {
    println!("Cost: ${:.4}", cost);
} else {
    println!("Usage not found");
}
```

### Combinators

```rust
use llm_cost_ops::Client;

async fn get_total_cost(client: &Client) -> Result<f64, Box<dyn std::error::Error>> {
    let total = client.costs()
        .get_costs("2025-01-01", "2025-01-31")
        .await
        .map(|costs| costs.total_cost)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    Ok(total)
}
```

### Question Mark Operator

```rust
use llm_cost_ops::Client;

async fn complex_operation(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    // All of these use ? for error propagation
    let costs = client.costs().get_costs("2025-01-01", "2025-01-31").await?;
    let usage = client.usage().list_usage("2025-01-01", "2025-01-31").await?;
    let analytics = client.analytics().get_usage_analytics("2025-01-01", "2025-01-31").await?;

    println!("Total cost: ${:.2}", costs.total_cost);
    println!("Total requests: {}", usage.total_count);
    println!("Analytics groups: {}", analytics.groups.len());

    Ok(())
}
```

## API Methods

### Usage Methods

```rust
use llm_cost_ops::{Client, UsageCreate, UsageUpdate};
use chrono::Utc;

impl UsageService {
    async fn create_usage(&self, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let usage = UsageCreate {
            model: "gpt-4".to_string(),
            tokens_prompt: 1000,
            tokens_completion: 500,
            request_count: 1,
            timestamp: Utc::now(),
            metadata: None,
        };

        let created = client.usage().create_usage(usage).await?;
        println!("Created: {}", created.id);

        Ok(())
    }

    async fn get_usage(&self, client: &Client, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let usage = client.usage().get_by_id(id).await?;
        println!("Usage: {} - ${:.4}", usage.id, usage.cost);

        Ok(())
    }

    async fn list_usage(&self, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let usage_list = client.usage()
            .list_usage("2025-01-01", "2025-01-31")
            .await?;

        for usage in usage_list.items {
            println!("{}: ${:.4}", usage.model, usage.cost);
        }

        Ok(())
    }

    async fn update_usage(&self, client: &Client, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut metadata = std::collections::HashMap::new();
        metadata.insert("updated".to_string(), "true".to_string());

        let update = UsageUpdate { metadata };
        let updated = client.usage().update_usage(id, update).await?;

        println!("Updated: {}", updated.id);

        Ok(())
    }

    async fn delete_usage(&self, client: &Client, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        client.usage().delete_usage(id).await?;
        println!("Deleted: {}", id);

        Ok(())
    }

    async fn batch_create(&self, client: &Client, usages: Vec<UsageCreate>) -> Result<(), Box<dyn std::error::Error>> {
        let result = client.usage().batch_create_usage(usages).await?;

        println!("Created: {}, Failed: {}", result.created_count, result.failed_count);

        Ok(())
    }
}
```

### Costs Methods

```rust
use llm_cost_ops::{Client, CostsQuery};

impl CostsService {
    async fn get_costs(&self, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let query = CostsQuery {
            start_date: "2025-01-01".to_string(),
            end_date: "2025-01-31".to_string(),
            group_by: Some(vec!["model".to_string(), "provider".to_string()]),
            granularity: None,
            filters: None,
        };

        let costs = client.costs().get_costs_with_query(query).await?;

        println!("Total cost: ${:.2}", costs.total_cost);

        for item in costs.items {
            println!("{} ({}): ${:.2}", item.model, item.provider, item.amount);
        }

        Ok(())
    }

    async fn get_cost_breakdown(&self, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let breakdown = client.costs()
            .get_cost_breakdown("2025-01-01", "2025-01-31", "daily")
            .await?;

        for item in breakdown.items {
            println!("{}: ${:.2}", item.date, item.amount);
        }

        Ok(())
    }

    async fn get_cost_trends(&self, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let trends = client.costs()
            .get_cost_trends(30, true)
            .await?;

        println!("30-day trend: {:+.2}%", trends.trend_percentage);
        println!("Forecasted next month: ${:.2}", trends.forecast);

        Ok(())
    }
}
```

### Analytics Methods

```rust
use llm_cost_ops::{Client, AnalyticsQuery};

impl AnalyticsService {
    async fn get_usage_analytics(&self, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let query = AnalyticsQuery {
            start_date: "2025-01-01".to_string(),
            end_date: "2025-01-31".to_string(),
            group_by: Some(vec!["model".to_string()]),
            metrics: Some(vec![
                "total_tokens".to_string(),
                "total_cost".to_string(),
                "request_count".to_string(),
            ]),
            include_comparison: None,
        };

        let analytics = client.analytics().get_usage_analytics_with_query(query).await?;

        for group in analytics.groups {
            println!("\n{}:", group.key);
            println!("  Tokens: {}", group.total_tokens);
            println!("  Cost: ${:.2}", group.total_cost);
            println!("  Requests: {}", group.request_count);
        }

        Ok(())
    }

    async fn get_performance_metrics(&self, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let performance = client.analytics()
            .get_performance_metrics("2025-01-01", "2025-01-31")
            .await?;

        println!("Average latency: {:.2}ms", performance.avg_latency_ms);
        println!("Success rate: {:.2}%", performance.success_rate * 100.0);
        println!("P95 latency: {:.2}ms", performance.p95_latency_ms);

        Ok(())
    }

    async fn get_top_consumers(&self, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
        let top_users = client.analytics()
            .get_top_consumers("2025-01-01", "2025-01-31", 10, "total_cost")
            .await?;

        for user in top_users.items {
            println!("{}: ${:.2}", user.user_id, user.total_cost);
        }

        Ok(())
    }
}
```

## Error Handling

### Error Types

```rust
use llm_cost_ops::{Client, CostOpsError};

async fn handle_errors(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    match client.costs().get_costs("2025-01-01", "2025-01-31").await {
        Ok(costs) => {
            println!("Total cost: ${:.2}", costs.total_cost);
        }
        Err(e) => match e {
            CostOpsError::Authentication(msg) => {
                eprintln!("Authentication failed: {}", msg);
            }
            CostOpsError::Authorization(msg) => {
                eprintln!("Authorization failed: {}", msg);
            }
            CostOpsError::ResourceNotFound(msg) => {
                eprintln!("Resource not found: {}", msg);
            }
            CostOpsError::Validation(errors) => {
                eprintln!("Validation error:");
                for (field, msg) in errors {
                    eprintln!("  {}: {}", field, msg);
                }
            }
            CostOpsError::RateLimit { message, retry_after } => {
                eprintln!("Rate limit exceeded: {}", message);
                eprintln!("Retry after: {}s", retry_after);
            }
            CostOpsError::Server { status_code, message } => {
                eprintln!("Server error ({}): {}", status_code, message);
            }
            CostOpsError::Network(msg) => {
                eprintln!("Network error: {}", msg);
            }
            CostOpsError::Timeout(msg) => {
                eprintln!("Request timeout: {}", msg);
            }
            _ => {
                eprintln!("Unexpected error: {}", e);
            }
        },
    }

    Ok(())
}
```

### Custom Error Type

```rust
use thiserror::Error;
use llm_cost_ops::CostOpsError;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Cost ops error: {0}")]
    CostOps(#[from] CostOpsError),

    #[error("Budget exceeded: {actual} > {limit}")]
    BudgetExceeded { actual: f64, limit: f64 },

    #[error("Invalid date range: {0}")]
    InvalidDateRange(String),

    #[error("Configuration error: {0}")]
    Config(String),
}

async fn validate_budget(client: &Client, limit: f64) -> Result<(), AppError> {
    let costs = client.costs()
        .get_costs("2025-01-01", "2025-01-31")
        .await?;

    if costs.total_cost > limit {
        return Err(AppError::BudgetExceeded {
            actual: costs.total_cost,
            limit,
        });
    }

    Ok(())
}
```

### Error Context

```rust
use anyhow::{Context, Result};
use llm_cost_ops::Client;

async fn fetch_costs_with_context(client: &Client) -> Result<()> {
    let costs = client.costs()
        .get_costs("2025-01-01", "2025-01-31")
        .await
        .context("Failed to fetch costs for January 2025")?;

    println!("Total cost: ${:.2}", costs.total_cost);

    Ok(())
}
```

### Retry on Error

```rust
use llm_cost_ops::{Client, CostOpsError};
use tokio::time::{sleep, Duration};

async fn retry_on_error<F, T, E>(
    mut f: F,
    max_retries: u32,
    delay: Duration,
) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>>,
{
    let mut retries = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if retries < max_retries => {
                retries += 1;
                eprintln!("Retry {}/{}", retries, max_retries);
                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

// Usage
let costs = retry_on_error(
    || Box::pin(client.costs().get_costs("2025-01-01", "2025-01-31")),
    3,
    Duration::from_secs(2),
).await?;
```

## Custom Types with Serde

### Serialization

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageRecord {
    pub id: String,
    pub model: String,
    pub tokens_prompt: u32,
    pub tokens_completion: u32,
    pub total_tokens: u32,
    pub cost: f64,
    pub timestamp: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl UsageRecord {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
```

### Custom Deserializer

```rust
use serde::{Deserialize, Deserializer};

fn deserialize_cost<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.trim_start_matches('$')
        .parse()
        .map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
pub struct CostItem {
    pub date: String,
    #[serde(deserialize_with = "deserialize_cost")]
    pub amount: f64,
}
```

### Validation

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageCreate {
    pub model: String,
    pub tokens_prompt: u32,
    pub tokens_completion: u32,
    pub request_count: u32,
}

impl UsageCreate {
    pub fn validate(&self) -> Result<(), String> {
        if self.model.is_empty() {
            return Err("Model cannot be empty".to_string());
        }

        if self.tokens_prompt == 0 && self.tokens_completion == 0 {
            return Err("At least one of tokens_prompt or tokens_completion must be > 0".to_string());
        }

        if self.request_count == 0 {
            return Err("Request count must be > 0".to_string());
        }

        Ok(())
    }
}

// Usage
let usage = UsageCreate {
    model: "gpt-4".to_string(),
    tokens_prompt: 1000,
    tokens_completion: 500,
    request_count: 1,
};

usage.validate()?;
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llm_cost_ops::{Client, CostsResponse};

    #[tokio::test]
    async fn test_get_costs() {
        // Mock client setup would go here
        let client = create_mock_client();

        let costs = client.costs()
            .get_costs("2025-01-01", "2025-01-31")
            .await
            .unwrap();

        assert_eq!(costs.total_cost, 100.50);
        assert_eq!(costs.items.len(), 2);
    }

    #[test]
    fn test_usage_validation() {
        let usage = UsageCreate {
            model: "gpt-4".to_string(),
            tokens_prompt: 1000,
            tokens_completion: 500,
            request_count: 1,
        };

        assert!(usage.validate().is_ok());

        let invalid_usage = UsageCreate {
            model: "".to_string(),
            tokens_prompt: 0,
            tokens_completion: 0,
            request_count: 0,
        };

        assert!(invalid_usage.validate().is_err());
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use llm_cost_ops::Client;

    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored
    async fn test_create_and_get_usage() {
        let client = Client::from_env().unwrap();

        // Create usage
        let usage = UsageCreate {
            model: "gpt-4".to_string(),
            tokens_prompt: 1000,
            tokens_completion: 500,
            request_count: 1,
            timestamp: chrono::Utc::now(),
            metadata: None,
        };

        let created = client.usage()
            .create_usage(usage)
            .await
            .unwrap();

        assert!(!created.id.is_empty());
        assert_eq!(created.model, "gpt-4");

        // Get usage
        let fetched = client.usage()
            .get_by_id(&created.id)
            .await
            .unwrap();

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.model, created.model);

        // Cleanup
        client.usage()
            .delete_usage(&created.id)
            .await
            .unwrap();
    }
}
```

### Mock Testing

```rust
use mockito::{mock, Matcher};

#[cfg(test)]
mod mock_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_costs_with_mock() {
        let _m = mock("GET", "/v1/costs")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("start_date".into(), "2025-01-01".into()),
                Matcher::UrlEncoded("end_date".into(), "2025-01-31".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "total_cost": 100.50,
                "items": [
                    {"date": "2025-01-01", "amount": 50.25},
                    {"date": "2025-01-02", "amount": 50.25}
                ]
            }"#)
            .create();

        let client = Client::builder()
            .api_key("test_key")
            .base_url(&mockito::server_url())
            .build()
            .unwrap();

        let costs = client.costs()
            .get_costs("2025-01-01", "2025-01-31")
            .await
            .unwrap();

        assert_eq!(costs.total_cost, 100.50);
        assert_eq!(costs.items.len(), 2);
    }
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_cost_calculation(
        tokens_prompt in 0u32..100000,
        tokens_completion in 0u32..100000
    ) {
        let cost = calculate_cost(tokens_prompt, tokens_completion);
        assert!(cost >= 0.0);
        assert!(cost < 1000.0); // Sanity check
    }

    #[test]
    fn test_usage_create_validation(
        model in "[a-z]{1,50}",
        tokens_prompt in 0u32..100000,
        tokens_completion in 0u32..100000,
        request_count in 1u32..1000
    ) {
        let usage = UsageCreate {
            model,
            tokens_prompt,
            tokens_completion,
            request_count,
            timestamp: chrono::Utc::now(),
            metadata: None,
        };

        if usage.model.is_empty() {
            assert!(usage.validate().is_err());
        } else if usage.tokens_prompt == 0 && usage.tokens_completion == 0 {
            assert!(usage.validate().is_err());
        } else {
            assert!(usage.validate().is_ok());
        }
    }
}
```

## Benchmarking

### Basic Benchmark

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llm_cost_ops::Client;

fn benchmark_cost_calculation(c: &mut Criterion) {
    c.bench_function("calculate_cost", |b| {
        b.iter(|| {
            calculate_cost(black_box(1000), black_box(500))
        })
    });
}

fn benchmark_serialization(c: &mut Criterion) {
    let usage = UsageRecord {
        id: "usage_123".to_string(),
        model: "gpt-4".to_string(),
        tokens_prompt: 1000,
        tokens_completion: 500,
        total_tokens: 1500,
        cost: 0.045,
        timestamp: chrono::Utc::now(),
        metadata: None,
    };

    c.bench_function("serialize_usage", |b| {
        b.iter(|| {
            black_box(usage.to_json().unwrap())
        })
    });

    let json = usage.to_json().unwrap();

    c.bench_function("deserialize_usage", |b| {
        b.iter(|| {
            black_box(UsageRecord::from_json(&json).unwrap())
        })
    });
}

criterion_group!(benches, benchmark_cost_calculation, benchmark_serialization);
criterion_main!(benches);
```

### Async Benchmark

```rust
use criterion::{criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

fn benchmark_async_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let client = Client::from_env().unwrap();

    c.bench_function("async_get_costs", |b| {
        b.to_async(&rt).iter(|| async {
            client.costs()
                .get_costs("2025-01-01", "2025-01-31")
                .await
                .unwrap()
        })
    });
}

criterion_group!(async_benches, benchmark_async_operations);
criterion_main!(async_benches);
```

## Zero-Copy Optimizations

### String Slices

```rust
pub fn parse_date(date: &str) -> Result<(u16, u8, u8), String> {
    let parts: Vec<&str> = date.split('-').collect();

    if parts.len() != 3 {
        return Err("Invalid date format".to_string());
    }

    let year = parts[0].parse().map_err(|_| "Invalid year")?;
    let month = parts[1].parse().map_err(|_| "Invalid month")?;
    let day = parts[2].parse().map_err(|_| "Invalid day")?;

    Ok((year, month, day))
}
```

### Borrowed Types

```rust
use std::borrow::Cow;

pub struct CostsQuery<'a> {
    pub start_date: Cow<'a, str>,
    pub end_date: Cow<'a, str>,
    pub group_by: Option<Vec<Cow<'a, str>>>,
}

impl<'a> CostsQuery<'a> {
    pub fn new(start_date: &'a str, end_date: &'a str) -> Self {
        Self {
            start_date: Cow::Borrowed(start_date),
            end_date: Cow::Borrowed(end_date),
            group_by: None,
        }
    }

    pub fn with_group_by(mut self, group_by: Vec<&'a str>) -> Self {
        self.group_by = Some(group_by.into_iter().map(Cow::Borrowed).collect());
        self
    }
}
```

### Byte Slices

```rust
pub fn parse_response(data: &[u8]) -> Result<CostsResponse, serde_json::Error> {
    serde_json::from_slice(data)
}
```

## Lifetime Management

### Basic Lifetimes

```rust
pub struct Client {
    config: Config,
}

pub struct CostsService<'a> {
    client: &'a Client,
}

impl<'a> CostsService<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn get_costs(&self, start_date: &str, end_date: &str) -> Result<CostsResponse, CostOpsError> {
        // Implementation
        todo!()
    }
}
```

### Multiple Lifetimes

```rust
pub struct QueryBuilder<'a, 'b> {
    start_date: &'a str,
    end_date: &'b str,
    filters: Vec<Filter<'a>>,
}

impl<'a, 'b> QueryBuilder<'a, 'b> {
    pub fn new(start_date: &'a str, end_date: &'b str) -> Self {
        Self {
            start_date,
            end_date,
            filters: Vec::new(),
        }
    }

    pub fn add_filter(mut self, filter: Filter<'a>) -> Self {
        self.filters.push(filter);
        self
    }
}
```

### Static Lifetime

```rust
const DEFAULT_BASE_URL: &'static str = "https://api.llmcostops.com";

pub struct Config {
    pub api_key: String,
    pub base_url: &'static str,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: DEFAULT_BASE_URL,
        }
    }
}
```

## Advanced Patterns

### Type State Pattern

```rust
pub struct Uninitialized;
pub struct Initialized;

pub struct ClientBuilder<State = Uninitialized> {
    api_key: Option<String>,
    base_url: Option<String>,
    timeout: Option<Duration>,
    _state: std::marker::PhantomData<State>,
}

impl ClientBuilder<Uninitialized> {
    pub fn new() -> Self {
        Self {
            api_key: None,
            base_url: None,
            timeout: None,
            _state: std::marker::PhantomData,
        }
    }

    pub fn api_key(mut self, api_key: impl Into<String>) -> ClientBuilder<Initialized> {
        ClientBuilder {
            api_key: Some(api_key.into()),
            base_url: self.base_url,
            timeout: self.timeout,
            _state: std::marker::PhantomData,
        }
    }
}

impl ClientBuilder<Initialized> {
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = Some(base_url.into());
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn build(self) -> Result<Client, String> {
        Ok(Client {
            config: Config {
                api_key: self.api_key.unwrap(),
                base_url: self.base_url.unwrap_or_else(|| DEFAULT_BASE_URL.to_string()),
                timeout: self.timeout.unwrap_or(Duration::from_secs(30)),
                max_retries: 3,
            },
        })
    }
}
```

### Newtype Pattern

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageId(String);

impl UsageId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UsageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Cost(f64);

impl Cost {
    pub fn new(value: f64) -> Result<Self, String> {
        if value < 0.0 {
            return Err("Cost cannot be negative".to_string());
        }
        Ok(Self(value))
    }

    pub fn value(&self) -> f64 {
        self.0
    }
}

impl std::ops::Add for Cost {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}
```

### Trait Objects

```rust
pub trait CostCalculator {
    fn calculate(&self, tokens_prompt: u32, tokens_completion: u32) -> f64;
}

pub struct OpenAICostCalculator {
    prompt_price_per_1k: f64,
    completion_price_per_1k: f64,
}

impl CostCalculator for OpenAICostCalculator {
    fn calculate(&self, tokens_prompt: u32, tokens_completion: u32) -> f64 {
        let prompt_cost = (tokens_prompt as f64 / 1000.0) * self.prompt_price_per_1k;
        let completion_cost = (tokens_completion as f64 / 1000.0) * self.completion_price_per_1k;
        prompt_cost + completion_cost
    }
}

pub struct CostService {
    calculator: Box<dyn CostCalculator + Send + Sync>,
}

impl CostService {
    pub fn new(calculator: Box<dyn CostCalculator + Send + Sync>) -> Self {
        Self { calculator }
    }

    pub fn calculate_cost(&self, tokens_prompt: u32, tokens_completion: u32) -> f64 {
        self.calculator.calculate(tokens_prompt, tokens_completion)
    }
}
```

## Performance Optimization

### Connection Pooling

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct ClientPool {
    clients: Vec<Client>,
    semaphore: Arc<Semaphore>,
}

impl ClientPool {
    pub fn new(api_key: &str, size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let mut clients = Vec::with_capacity(size);

        for _ in 0..size {
            let client = Client::builder()
                .api_key(api_key)
                .build()?;
            clients.push(client);
        }

        Ok(Self {
            clients,
            semaphore: Arc::new(Semaphore::new(size)),
        })
    }

    pub async fn get(&self) -> ClientGuard<'_> {
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();
        ClientGuard {
            client: &self.clients[0], // Simplified, would use a queue in production
            _permit: permit,
        }
    }
}

pub struct ClientGuard<'a> {
    client: &'a Client,
    _permit: tokio::sync::OwnedSemaphorePermit,
}

impl<'a> std::ops::Deref for ClientGuard<'a> {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        self.client
    }
}
```

### Caching

```rust
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub struct Cache<K, V> {
    data: Arc<RwLock<HashMap<K, CacheEntry<V>>>>,
    ttl: Duration,
}

struct CacheEntry<V> {
    value: V,
    expires_at: Instant,
}

impl<K: Eq + std::hash::Hash + Clone, V: Clone> Cache<K, V> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub fn get(&self, key: &K) -> Option<V> {
        let data = self.data.read().unwrap();
        data.get(key).and_then(|entry| {
            if Instant::now() < entry.expires_at {
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    pub fn set(&self, key: K, value: V) {
        let mut data = self.data.write().unwrap();
        data.insert(
            key,
            CacheEntry {
                value,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }

    pub fn clear_expired(&self) {
        let mut data = self.data.write().unwrap();
        let now = Instant::now();
        data.retain(|_, entry| entry.expires_at > now);
    }
}

// Cached client
pub struct CachedClient {
    client: Client,
    cache: Cache<String, CostsResponse>,
}

impl CachedClient {
    pub fn new(client: Client, cache_ttl: Duration) -> Self {
        Self {
            client,
            cache: Cache::new(cache_ttl),
        }
    }

    pub async fn get_costs(&self, start_date: &str, end_date: &str) -> Result<CostsResponse, CostOpsError> {
        let cache_key = format!("costs:{}:{}", start_date, end_date);

        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(cached);
        }

        let costs = self.client.costs()
            .get_costs(start_date, end_date)
            .await?;

        self.cache.set(cache_key, costs.clone());

        Ok(costs)
    }
}
```

### Batch Processing

```rust
use futures::stream::{self, StreamExt};

pub async fn batch_create_usage(
    client: &Client,
    usages: Vec<UsageCreate>,
    batch_size: usize,
    concurrency: usize,
) -> Result<Vec<BatchResult>, Box<dyn std::error::Error>> {
    let chunks: Vec<_> = usages.chunks(batch_size).collect();

    let results = stream::iter(chunks)
        .map(|chunk| async move {
            client.usage()
                .batch_create_usage(chunk.to_vec())
                .await
        })
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    let mut batch_results = Vec::new();

    for result in results {
        match result {
            Ok(r) => batch_results.push(r),
            Err(e) => eprintln!("Batch failed: {}", e),
        }
    }

    Ok(batch_results)
}
```

---

## Complete Example Application

```rust
use llm_cost_ops::{Client, ClientBuilder, UsageCreate};
use chrono::Utc;
use anyhow::Result;
use std::time::Duration;

struct CostTracker {
    client: Client,
}

impl CostTracker {
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        let client = ClientBuilder::new()
            .api_key(api_key)
            .timeout(Duration::from_secs(30))
            .max_retries(3)
            .build()?;

        Ok(Self { client })
    }

    pub async fn track_usage(&self, model: &str, tokens_prompt: u32, tokens_completion: u32) -> Result<()> {
        let usage = UsageCreate {
            model: model.to_string(),
            tokens_prompt,
            tokens_completion,
            request_count: 1,
            timestamp: Utc::now(),
            metadata: None,
        };

        let created = self.client.usage()
            .create_usage(usage)
            .await?;

        println!("✓ Tracked usage: {} - ${:.4}", created.id, created.cost);

        Ok(())
    }

    pub async fn get_cost_summary(&self, days: i64) -> Result<()> {
        let end_date = Utc::now();
        let start_date = end_date - chrono::Duration::days(days);

        let costs = self.client.costs()
            .get_costs(
                &start_date.format("%Y-%m-%d").to_string(),
                &end_date.format("%Y-%m-%d").to_string(),
            )
            .await?;

        println!("\n📊 Cost Summary (Last {} days)", days);
        println!("Total Cost: ${:.2}", costs.total_cost);

        for (i, item) in costs.items.iter().enumerate() {
            if i >= 10 {
                break;
            }
            println!("  {}: ${:.2}", item.date, item.amount);
        }

        Ok(())
    }

    pub async fn check_budgets(&self) -> Result<()> {
        let budgets = self.client.budgets()
            .list_budgets(true)
            .await?;

        println!("\n💰 Active Budgets");

        for budget in budgets.items {
            let percentage = budget.percentage_used;
            let status = if percentage < 50.0 {
                "🟢"
            } else if percentage < 80.0 {
                "🟡"
            } else {
                "🔴"
            };

            println!(
                "{} {}: ${:.2} / ${:.2} ({:.1}%)",
                status, budget.name, budget.spent, budget.amount, percentage
            );
        }

        Ok(())
    }

    pub async fn run_analytics(&self, days: i64) -> Result<()> {
        let end_date = Utc::now();
        let start_date = end_date - chrono::Duration::days(days);

        let analytics = self.client.analytics()
            .get_usage_analytics(
                &start_date.format("%Y-%m-%d").to_string(),
                &end_date.format("%Y-%m-%d").to_string(),
            )
            .with_group_by(vec!["model"])
            .with_metrics(vec!["total_tokens", "total_cost", "request_count"])
            .execute()
            .await?;

        println!("\n📈 Analytics (Last {} days)", days);

        for group in analytics.groups {
            println!("\n{}:", group.key);
            println!("  Tokens: {}", group.total_tokens);
            println!("  Cost: ${:.2}", group.total_cost);
            println!("  Requests: {}", group.request_count);
            println!("  Avg Cost/Request: ${:.4}", group.total_cost / group.request_count as f64);
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment
    dotenv::dotenv().ok();

    let api_key = std::env::var("LLM_COST_OPS_API_KEY")?;

    // Create tracker
    let tracker = CostTracker::new(api_key)?;

    // Track usage
    tracker.track_usage("gpt-4", 1000, 500).await?;

    // Get cost summary
    tracker.get_cost_summary(7).await?;

    // Check budgets
    tracker.check_budgets().await?;

    // Run analytics
    tracker.run_analytics(30).await?;

    Ok(())
}
```

---

## Additional Resources

- **API Documentation**: https://docs.rs/llm-cost-ops
- **GitHub Repository**: https://github.com/llmcostops/rust-sdk
- **Examples**: https://github.com/llmcostops/rust-sdk/tree/main/examples
- **Rust Book**: https://doc.rust-lang.org/book/
- **Tokio Documentation**: https://tokio.rs/
- **Support**: support@llmcostops.com

## Next Steps

1. Set up your Rust project with Cargo
2. Install the SDK and configure dependencies
3. Initialize the client with your API key
4. Try basic usage examples
5. Implement error handling with Result types
6. Add tests and benchmarks
7. Optimize for production with async, caching, and zero-copy patterns

For more advanced use cases, check out the [Advanced Integration Guide](./advanced-integration.md).
