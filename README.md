# LLM-CostOps

Enterprise-grade cost operations platform for LLM infrastructure. Track, analyze, and optimize costs across multiple LLM providers with production-ready accuracy.

## Features

### Core Capabilities
- **Multi-Provider Support**: OpenAI, Anthropic, Google Vertex AI, Azure OpenAI, AWS Bedrock, Cohere, Mistral
- **Flexible Pricing Models**: Per-token, per-request, tiered volume pricing
- **High-Precision Calculations**: 10-decimal precision using rust_decimal for accurate financial calculations
- **Token Accounting**: Comprehensive tracking of prompt, completion, cached, and reasoning tokens
- **Cost Analytics**: Aggregate and analyze costs by provider, model, project, or organization

### Export & Reporting
- **Multiple Export Formats**: CSV, JSON, Excel (XLSX), JSON Lines
- **Scheduled Reports**: Cron-based scheduling with timezone support
- **Email Delivery**: SMTP with TLS/STARTTLS and HTML templates
- **Report Types**: Cost, Usage, Forecast, Audit, Budget, Summary
- **Multi-Channel Delivery**: Email, local storage, S3, webhooks

### Observability & Monitoring
- **Prometheus Metrics**: 40+ metrics for HTTP, costs, database, auth, forecasting
- **Distributed Tracing**: OpenTelemetry Protocol (OTLP) with correlation IDs
- **Structured Logging**: JSON format with trace context integration
- **Health Checks**: Liveness, readiness, and startup probes for Kubernetes
- **ServiceMonitor**: Automatic Prometheus scraping configuration

### Forecasting & Analytics
- **Time-Series Forecasting**: Linear trend, moving average, exponential smoothing
- **Anomaly Detection**: Z-score and IQR methods for cost anomalies
- **Budget Forecasting**: Predictive budget alerts with confidence intervals
- **Trend Analysis**: Automatic trend direction and seasonality detection

### Security & Authentication
- **API Key Management**: Secure API key generation with SHA-256 hashing
- **JWT Authentication**: Token-based auth with refresh tokens
- **RBAC**: Role-based access control with fine-grained permissions
- **Audit Logging**: Comprehensive audit trail for all operations
- **Multi-tenancy**: Organization and project-level isolation

### Infrastructure & Deployment
- **Kubernetes-Ready**: Complete K8s manifests with Helm charts
- **High Availability**: HPA, PDB, rolling updates with zero downtime
- **Security Hardening**: Non-root containers, read-only filesystem, network policies
- **Database Flexibility**: SQLite for development, PostgreSQL for production
- **Production-Ready**: Comprehensive error handling, validation, and type safety

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/llm-cost-ops.git
cd llm-cost-ops

# Build the project
cargo build --release

# Initialize the database
./target/release/cost-ops init --database-url sqlite:cost-ops.db
```

### Basic Usage

**1. Add Pricing Information**

```bash
# Add OpenAI GPT-4 pricing
cost-ops pricing add \
  --provider openai \
  --model gpt-4 \
  --input-price 10.0 \
  --output-price 30.0

# Add Anthropic Claude pricing
cost-ops pricing add \
  --provider anthropic \
  --model claude-3-sonnet-20240229 \
  --input-price 3.0 \
  --output-price 15.0
```

**2. Ingest Usage Data**

```bash
# Ingest from JSON file
cost-ops ingest --file examples/usage.json

# Example usage.json format:
# [
#   {
#     "id": "550e8400-e29b-41d4-a716-446655440001",
#     "timestamp": "2025-01-15T10:00:00Z",
#     "provider": "openai",
#     "model": {"name": "gpt-4", "context_window": 8192},
#     "organization_id": "org-example",
#     "prompt_tokens": 1000,
#     "completion_tokens": 500,
#     "total_tokens": 1500
#   }
# ]
```

**3. Query Costs**

```bash
# Query last 24 hours (table format)
cost-ops query --range last-24-hours --output table

# Query with filters
cost-ops query \
  --range last-7-days \
  --organization org-example \
  --output json

# Available formats: json, table, csv
# Available ranges: last-hour, last-24-hours, last-7-days, last-30-days
```

**4. Generate Summary Reports**

```bash
# Generate cost summary for last 30 days
cost-ops summary --period last-30-days --organization org-example

# Output:
# === Cost Summary ===
# Period: 2024-12-15 to 2025-01-15
# Organization: org-example
#
# Total Cost: $45.123456
# Total Requests: 1,250
# Avg Cost/Request: $0.036099
#
# --- By Provider ---
# openai: $30.500000
# anthropic: $14.623456
#
# --- By Model ---
# gpt-4: $30.500000
# claude-3-sonnet: $14.623456
```

**5. Export Data**

```bash
# Export to JSON
cost-ops export --output costs.json --format json --period last-30-days

# Export to CSV
cost-ops export --output costs.csv --format csv --period last-7-days
```

## Architecture

### Core Components

```
llm-cost-ops/
├── src/
│   ├── domain/          # Domain models and business logic
│   │   ├── provider.rs  # Multi-provider abstraction
│   │   ├── usage.rs     # Usage record definitions
│   │   ├── pricing.rs   # Pricing models and structures
│   │   ├── cost.rs      # Cost calculation models
│   │   └── error.rs     # Error types
│   ├── engine/          # Cost calculation and analytics
│   │   ├── calculator.rs   # Cost calculation engine
│   │   ├── normalizer.rs   # Token normalization
│   │   └── aggregator.rs   # Cost aggregation
│   ├── storage/         # Data persistence layer
│   │   ├── repository.rs   # Repository implementations
│   │   └── models.rs       # Database models
│   ├── auth/            # Authentication and authorization
│   │   ├── api_key.rs      # API key management
│   │   ├── jwt.rs          # JWT token handling
│   │   ├── rbac.rs         # Role-based access control
│   │   └── audit.rs        # Audit logging
│   ├── export/          # Export and reporting
│   │   ├── formats.rs      # CSV, JSON, Excel exporters
│   │   ├── reports.rs      # Report generation
│   │   ├── delivery.rs     # Email, storage, webhook delivery
│   │   ├── scheduler.rs    # Cron-based scheduling
│   │   └── config.rs       # Export configuration
│   ├── observability/   # Observability stack
│   │   ├── metrics.rs      # Prometheus metrics
│   │   ├── tracing.rs      # Distributed tracing
│   │   ├── logging.rs      # Structured logging
│   │   └── health.rs       # Health checks
│   ├── forecasting/     # Forecasting and analytics
│   │   ├── models.rs       # Forecast models
│   │   ├── engine.rs       # Forecast engine
│   │   ├── anomaly.rs      # Anomaly detection
│   │   └── budget.rs       # Budget forecasting
│   ├── api/             # REST API server
│   │   ├── routes/         # API route handlers
│   │   └── middleware/     # Auth, CORS, rate limiting
│   ├── ingestion/       # Data ingestion
│   │   ├── webhook.rs      # Webhook server
│   │   └── stream.rs       # Stream processing
│   ├── cli/             # Command-line interface
│   └── bin/             # Binary entry point
├── k8s/                 # Kubernetes deployment
│   ├── base/            # Base manifests
│   ├── overlays/        # Environment overlays
│   └── helm/            # Helm charts
├── migrations/          # Database schema migrations
└── examples/           # Sample data and usage examples
```

### Data Flow

1. **Ingestion**: Usage records are validated and stored in the database
2. **Pricing Lookup**: Active pricing table is retrieved for the provider/model/date
3. **Cost Calculation**: Engine calculates costs using appropriate pricing structure
4. **Storage**: Calculated costs are persisted with full audit trail
5. **Analytics**: Aggregation and summarization for reporting

## Configuration

Create a `config.toml` file:

```toml
[database]
url = "sqlite:cost-ops.db"
# For production:
# url = "postgresql://user:pass@localhost/costops"

[observability]
log_level = "info"
```

Use with CLI:

```bash
cost-ops --config config.toml query --range last-24-hours
```

## Pricing Models

### Per-Token Pricing (Most Common)

Used by OpenAI, Anthropic, Google:

```rust
PricingStructure::PerToken {
    input_price_per_million: 10.0,
    output_price_per_million: 30.0,
    cached_input_discount: Some(0.5), // 50% discount for cached tokens
}
```

### Per-Request Pricing

Fixed cost per request with included tokens:

```rust
PricingStructure::PerRequest {
    price_per_request: 0.01,
    included_tokens: 1000,
    overage_price_per_million: 5.0,
}
```

### Tiered Volume Pricing

Volume-based discounts:

```rust
PricingStructure::Tiered {
    tiers: vec![
        PricingTier {
            threshold: 0,
            input_price_per_million: 10.0,
            output_price_per_million: 30.0,
        },
        PricingTier {
            threshold: 1_000_000,
            input_price_per_million: 8.0,
            output_price_per_million: 24.0,
        },
    ],
}
```

## Database Schema

### Usage Records

```sql
CREATE TABLE usage_records (
    id TEXT PRIMARY KEY,
    timestamp TEXT NOT NULL,
    provider TEXT NOT NULL,
    model_name TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    prompt_tokens INTEGER NOT NULL,
    completion_tokens INTEGER NOT NULL,
    total_tokens INTEGER NOT NULL,
    cached_tokens INTEGER,
    reasoning_tokens INTEGER,
    latency_ms INTEGER,
    tags TEXT,  -- JSON array
    metadata TEXT,  -- JSON object
    ingested_at TEXT NOT NULL
);
```

### Cost Records

```sql
CREATE TABLE cost_records (
    id TEXT PRIMARY KEY,
    usage_id TEXT NOT NULL,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    input_cost TEXT NOT NULL,  -- Decimal as string
    output_cost TEXT NOT NULL,
    total_cost TEXT NOT NULL,
    currency TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    FOREIGN KEY (usage_id) REFERENCES usage_records(id)
);
```

### Pricing Tables

```sql
CREATE TABLE pricing_tables (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    pricing_structure TEXT NOT NULL,  -- JSON
    currency TEXT NOT NULL,
    effective_date TEXT NOT NULL,
    end_date TEXT
);
```

## API Integration

### Usage Data Format

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440001",
  "timestamp": "2025-01-15T10:00:00Z",
  "provider": "openai",
  "model": {
    "name": "gpt-4-turbo",
    "version": "gpt-4-turbo-2024-04-09",
    "context_window": 128000
  },
  "organization_id": "org-123",
  "project_id": "proj-456",
  "user_id": "user-789",
  "prompt_tokens": 1500,
  "completion_tokens": 800,
  "total_tokens": 2300,
  "cached_tokens": 500,
  "latency_ms": 3200,
  "tags": ["production", "api"],
  "metadata": {
    "request_id": "req-abc",
    "endpoint": "/v1/chat/completions"
  },
  "ingested_at": "2025-01-15T10:00:01Z",
  "source": {
    "type": "api",
    "endpoint": "https://api.example.com"
  }
}
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test module
cargo test domain::usage
```

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# With specific features
cargo build --features postgres
```

### Database Migrations

```bash
# Create new migration
sqlx migrate add <name>

# Run migrations
sqlx migrate run --database-url sqlite:cost-ops.db

# Revert last migration
sqlx migrate revert --database-url sqlite:cost-ops.db
```

## Production Deployment

### PostgreSQL Setup

```bash
# Update Cargo.toml features
cargo build --release --features postgres --no-default-features

# Run with PostgreSQL
cost-ops init --database-url postgresql://user:pass@localhost/costops
```

### Environment Variables

```bash
export DATABASE_URL=postgresql://user:pass@localhost/costops
export RUST_LOG=info
export CONFIG_PATH=/etc/cost-ops/config.toml
```

### Docker Deployment

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --features postgres --no-default-features

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/cost-ops /usr/local/bin/
ENTRYPOINT ["cost-ops"]
```

## Kubernetes Deployment

### Quick Deploy

```bash
# Deploy to development
kubectl apply -k k8s/overlays/dev/

# Deploy to production with Helm
helm install llm-cost-ops ./k8s/helm/llm-cost-ops \
  --namespace llm-cost-ops-prod \
  --create-namespace \
  --values ./k8s/helm/llm-cost-ops/values-prod.yaml
```

### Features

- **High Availability**: 3+ replicas with HPA (3-20 pods)
- **Security**: Non-root containers, read-only filesystem, network policies
- **Zero Downtime**: Rolling updates with PDB
- **Observability**: Prometheus ServiceMonitor, health endpoints
- **Auto-scaling**: CPU, memory, and custom metrics

See [k8s/DEPLOYMENT.md](k8s/DEPLOYMENT.md) for detailed documentation.

## Roadmap

### Completed ✅
- [x] PostgreSQL repository implementation
- [x] REST API server with Axum
- [x] Time-series forecasting
- [x] Budget enforcement and alerts
- [x] Real-time ingestion from message queues
- [x] Kubernetes deployment with Helm
- [x] Prometheus metrics and distributed tracing
- [x] RBAC and audit logging
- [x] Export and reporting system
- [x] Anomaly detection

### In Progress
- [ ] Advanced ML-based cost forecasting
- [ ] Custom dashboards and visualizations
- [ ] Integration with cloud billing APIs

### Future
- [ ] ROI correlation engine
- [ ] Advanced multi-tenant isolation
- [ ] Real-time streaming analytics
- [ ] Cost optimization recommendations

## Contributing

Contributions are welcome! Please see CONTRIBUTING.md for guidelines.

## License

Apache 2.0 / MIT dual-licensed. See LICENSE-APACHE and LICENSE-MIT for details.

## Support

- Documentation: https://docs.example.com/llm-cost-ops
- Issues: https://github.com/yourusername/llm-cost-ops/issues
- Discord: https://discord.gg/example
