# LLM Cost Ops

[![Crates.io](https://img.shields.io/crates/v/llm-cost-ops.svg)](https://crates.io/crates/llm-cost-ops)
[![Documentation](https://docs.rs/llm-cost-ops/badge.svg)](https://docs.rs/llm-cost-ops)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-554%2B%20passing-success)](https://github.com/globalbusinessadvisors/llm-cost-ops)
[![Coverage](https://img.shields.io/badge/coverage-90%25%2B-brightgreen)](https://github.com/globalbusinessadvisors/llm-cost-ops)

**Enterprise-grade cost operations platform for LLM deployments**

A comprehensive, production-ready Rust library for tracking, analyzing, and optimizing costs across multiple Large Language Model (LLM) providers including OpenAI, Anthropic, Google Vertex AI, Azure OpenAI, AWS Bedrock, Cohere, and Mistral.

## Features

### Core Cost Management
- **Multi-Provider Support** - OpenAI, Anthropic, Google Vertex AI, Azure, AWS Bedrock, Cohere, Mistral
- **Flexible Pricing Models** - Per-token, per-request, tiered pricing with cache discounts
- **Real-time Cost Calculation** - Sub-millisecond cost computations with decimal precision
- **Multi-Currency Support** - USD, EUR, GBP with automatic conversion
- **Usage Analytics** - Token consumption, cost trends, provider comparisons

### Data Ingestion
- **Webhook Server** - High-performance ingestion with validation
- **Stream Processing** - NATS and Redis Streams support
- **Rate Limiting** - Per-organization limits with burst allowance
- **Batch Processing** - Bulk ingestion with partial success handling
- **Dead Letter Queue** - Failed ingestion handling with retry policies

### Compliance & Governance
- **GDPR Compliance** - Data Subject Access Requests, Right to Erasure, Consent Management
- **Breach Notifications** - Automated 72-hour breach notification workflow
- **Audit Logging** - Comprehensive audit trail with tamper detection
- **Policy Management** - Retention policies, access policies, data classification
- **SOC2 Compliance** - Automated compliance checks and evidence collection

### Authentication & Security
- **JWT Authentication** - Secure token-based authentication
- **Role-Based Access Control (RBAC)** - Granular permission system
- **API Key Management** - Secure key rotation and revocation
- **Audit Middleware** - Automatic request/response logging
- **Rate Limiting** - Protect APIs from abuse

### Observability
- **Prometheus Metrics** - Request counts, latencies, error rates
- **Distributed Tracing** - OpenTelemetry integration
- **Structured Logging** - JSON logs with correlation IDs
- **Health Checks** - Database, cache, external service monitoring
- **Performance Monitoring** - Query optimization, slow query detection

### Export & Reporting
- **Multiple Formats** - CSV, Excel, JSON, PDF
- **Scheduled Reports** - Automated daily/weekly/monthly reports
- **Email Delivery** - Automated report distribution
- **Custom Templates** - Handlebars-based report templates
- **Data Aggregation** - Flexible grouping by organization, project, provider, model

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-cost-ops = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Basic Usage

```rust
use llm_cost_ops::{
    CostCalculator, PricingTable, PricingStructure,
    UsageRecord, Provider, ModelIdentifier, Currency
};
use rust_decimal_macros::dec;
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a pricing table for GPT-4
    let pricing = PricingTable::new(
        Provider::OpenAI,
        "gpt-4".to_string(),
        PricingStructure::simple_per_token(
            dec!(30.0),  // $30 per million input tokens
            dec!(60.0),  // $60 per million output tokens
        ),
    );

    // Create a usage record
    let usage = UsageRecord {
        id: uuid::Uuid::new_v4(),
        timestamp: Utc::now(),
        provider: Provider::OpenAI,
        model: ModelIdentifier::new("gpt-4".to_string(), 8192),
        organization_id: "org-123".to_string(),
        project_id: Some("project-456".to_string()),
        user_id: Some("user-789".to_string()),
        prompt_tokens: 1500,
        completion_tokens: 500,
        total_tokens: 2000,
        cached_tokens: None,
        reasoning_tokens: None,
        latency_ms: Some(1200),
        time_to_first_token_ms: Some(150),
        tags: vec!["production".to_string()],
        metadata: serde_json::json!({"endpoint": "/v1/chat"}),
        ingested_at: Utc::now(),
        source: llm_cost_ops::IngestionSource::Api {
            endpoint: "/v1/usage".to_string(),
        },
    };

    // Calculate cost
    let calculator = CostCalculator::new();
    let cost_record = calculator.calculate(&usage, &pricing)?;

    println!("Input cost: ${}", cost_record.input_cost);
    println!("Output cost: ${}", cost_record.output_cost);
    println!("Total cost: ${}", cost_record.total_cost);

    Ok(())
}
```

### Using the Database Repository

```rust
use llm_cost_ops::{
    DatabaseConfig, UsageRepository, CostRepository,
    UsageRecord, CostRecord
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure database
    let config = DatabaseConfig::sqlite("cost-ops.db".into());
    let db = config.connect().await?;

    // Run migrations
    llm_cost_ops::run_migrations(&db).await?;

    // Create repositories
    let usage_repo = UsageRepository::new(db.clone());
    let cost_repo = CostRepository::new(db);

    // Insert usage record
    usage_repo.insert(&usage).await?;

    // Insert cost record
    cost_repo.insert(&cost_record).await?;

    // Query usage by organization
    let org_usage = usage_repo
        .find_by_organization("org-123", None, None)
        .await?;

    println!("Found {} usage records", org_usage.len());

    Ok(())
}
```

### Starting the Web Server

```rust
use llm_cost_ops::{start_server, ServerConfig, DatabaseConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure database
    let db_config = DatabaseConfig::sqlite("cost-ops.db".into());
    let db = db_config.connect().await?;

    // Configure server
    let config = ServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        database: db_config,
        jwt_secret: "your-secret-key".to_string(),
        enable_metrics: true,
        metrics_port: 9090,
        log_level: "info".to_string(),
    };

    // Start server
    start_server(config).await?;

    Ok(())
}
```

## CLI Usage

Install the CLI tool:

```bash
cargo install llm-cost-ops
```

Initialize the database:

```bash
cost-ops init --database-url sqlite:cost-ops.db
```

Import usage data:

```bash
cost-ops import --file usage-data.json --format openai
```

Generate reports:

```bash
cost-ops report --organization org-123 --format csv --output report.csv
```

Query costs:

```bash
cost-ops query \
  --organization org-123 \
  --start-date 2024-01-01 \
  --end-date 2024-01-31
```

## Docker Deployment

```bash
# Build Docker image
docker build -f deployment/Dockerfile -t llm-cost-ops:latest .

# Run with Docker Compose
docker-compose -f deployment/docker-compose.yml up -d
```

## Supported Providers

| Provider | Per-Token Pricing | Tiered Pricing | Cache Discounts | Reasoning Tokens |
|----------|------------------|----------------|-----------------|------------------|
| OpenAI | ✅ | ✅ | ✅ | ✅ |
| Anthropic | ✅ | ✅ | ✅ | ✅ |
| Google Vertex AI | ✅ | ✅ | ✅ | ❌ |
| Azure OpenAI | ✅ | ✅ | ✅ | ✅ |
| AWS Bedrock | ✅ | ✅ | ❌ | ❌ |
| Cohere | ✅ | ❌ | ❌ | ❌ |
| Mistral | ✅ | ✅ | ❌ | ❌ |

## Performance

- **Cost Calculation**: <1ms per record
- **Query Performance**: <100ms p99 for complex aggregations
- **Throughput**: 1000+ requests/second
- **Test Coverage**: 90%+ with 554+ test cases
- **Memory Safety**: No unsafe code in core logic

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     LLM Cost Ops Platform                    │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Ingestion   │  │   Storage    │  │  Analytics   │     │
│  │   Engine     │→ │   Layer      │→ │   Engine     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│         ↓                 ↓                  ↓              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Validation  │  │  PostgreSQL/ │  │   Reporting  │     │
│  │  & Transform │  │   SQLite     │  │   & Export   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│         ↓                 ↓                  ↓              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │ Rate Limiting│  │  Compliance  │  │ Observability│     │
│  │  & DLQ       │  │   & GDPR     │  │  & Metrics   │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Links

- **Documentation**: [https://docs.rs/llm-cost-ops](https://docs.rs/llm-cost-ops)
- **Repository**: [https://github.com/globalbusinessadvisors/llm-cost-ops](https://github.com/globalbusinessadvisors/llm-cost-ops)
- **Full README**: [https://github.com/globalbusinessadvisors/llm-cost-ops/blob/main/README.md](https://github.com/globalbusinessadvisors/llm-cost-ops/blob/main/README.md)
- **Issues**: [https://github.com/globalbusinessadvisors/llm-cost-ops/issues](https://github.com/globalbusinessadvisors/llm-cost-ops/issues)

## Contributing

Contributions are welcome! Please see the full README on GitHub for contribution guidelines.
