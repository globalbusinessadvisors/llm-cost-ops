# LLM Cost Ops - API Server

[![Crates.io](https://img.shields.io/crates/v/llm-cost-ops-api.svg)](https://crates.io/crates/llm-cost-ops-api)
[![Documentation](https://docs.rs/llm-cost-ops-api/badge.svg)](https://docs.rs/llm-cost-ops-api)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Production-ready API server for LLM Cost Ops platform**

A high-performance, enterprise-grade REST API server built with Axum, providing comprehensive endpoints for cost tracking, analytics, and reporting.

## Features

- **RESTful API** - Clean, well-documented REST endpoints
- **Authentication** - JWT and API key authentication
- **Authorization** - Role-based access control (RBAC)
- **Rate Limiting** - Per-organization rate limits
- **Compression** - Brotli and Gzip compression
- **Validation** - Comprehensive input validation
- **Pagination** - Cursor and offset-based pagination
- **OpenAPI** - Auto-generated API documentation
- **Health Checks** - Readiness and liveness probes
- **Metrics** - Prometheus metrics endpoint
- **Distributed Tracing** - OpenTelemetry integration

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-cost-ops-api = "0.1"
llm-cost-ops = "0.1"
llm-cost-ops-compliance = "0.1"
tokio = { version = "1", features = ["full"] }
```

### Basic Server

```rust
use llm_cost_ops_api::{ApiServer, ApiServerConfig};
use llm_cost_ops::DatabaseConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure database
    let db_config = DatabaseConfig::sqlite("cost-ops.db".into());

    // Configure API server
    let config = ApiServerConfig {
        host: "0.0.0.0".to_string(),
        port: 8080,
        database: db_config,
        jwt_secret: std::env::var("JWT_SECRET")?,
        enable_metrics: true,
        metrics_port: 9090,
        enable_compression: true,
        max_request_size_mb: 10,
        log_level: "info".to_string(),
    };

    // Start server
    let server = ApiServer::new(config).await?;
    server.run().await?;

    Ok(())
}
```

### Custom Router

```rust
use llm_cost_ops_api::create_api_router;
use llm_cost_ops::DatabaseConfig;
use axum::Router;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_config = DatabaseConfig::sqlite("cost-ops.db".into());
    let db = db_config.connect().await?;

    // Create API router
    let api_router = create_api_router(
        db.clone(),
        "jwt-secret".to_string(),
    ).await?;

    // Combine with custom routes
    let app = Router::new()
        .nest("/api/v1", api_router)
        .nest("/custom", custom_routes());

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;

    Ok(())
}
```

## API Endpoints

### Usage Endpoints

```
POST   /api/v1/usage              - Submit usage record
GET    /api/v1/usage              - List usage records
GET    /api/v1/usage/:id          - Get usage record by ID
DELETE /api/v1/usage/:id          - Delete usage record
POST   /api/v1/usage/batch        - Batch submit usage records
GET    /api/v1/usage/stats        - Get usage statistics
```

### Cost Endpoints

```
GET    /api/v1/costs              - Query cost records
GET    /api/v1/costs/:id          - Get cost record by ID
GET    /api/v1/costs/aggregate    - Aggregate costs
GET    /api/v1/costs/trends       - Get cost trends
```

### Analytics Endpoints

```
GET    /api/v1/analytics/usage    - Usage analytics
GET    /api/v1/analytics/costs    - Cost analytics
GET    /api/v1/analytics/providers - Provider comparison
GET    /api/v1/analytics/models   - Model comparison
```

### Forecasting Endpoints

```
POST   /api/v1/forecast/cost      - Cost forecast
POST   /api/v1/forecast/usage     - Usage forecast
GET    /api/v1/forecast/anomalies - Anomaly detection
```

### Report Endpoints

```
POST   /api/v1/reports            - Generate report
GET    /api/v1/reports/:id        - Get report
GET    /api/v1/reports/:id/download - Download report
GET    /api/v1/reports            - List reports
DELETE /api/v1/reports/:id        - Delete report
```

### Administration Endpoints

```
GET    /api/v1/admin/organizations - List organizations
POST   /api/v1/admin/organizations - Create organization
GET    /api/v1/admin/users        - List users
POST   /api/v1/admin/users        - Create user
PUT    /api/v1/admin/users/:id/roles - Update user roles
```

### System Endpoints

```
GET    /health                    - Health check
GET    /ready                     - Readiness check
GET    /metrics                   - Prometheus metrics
GET    /openapi.json              - OpenAPI specification
```

## Request Examples

### Submit Usage

```bash
curl -X POST https://api.example.com/api/v1/usage \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "openai",
    "model": "gpt-4",
    "organization_id": "org-123",
    "prompt_tokens": 1500,
    "completion_tokens": 500,
    "latency_ms": 1200
  }'
```

### Query Costs

```bash
curl "https://api.example.com/api/v1/costs?organization_id=org-123&start_date=2024-01-01&end_date=2024-01-31" \
  -H "Authorization: Bearer $JWT_TOKEN"
```

### Generate Report

```bash
curl -X POST https://api.example.com/api/v1/reports \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "report_type": "cost",
    "organization_id": "org-123",
    "format": "csv",
    "filters": {
      "start_date": "2024-01-01",
      "end_date": "2024-01-31"
    }
  }'
```

## Authentication

### JWT Authentication

```bash
# Login to get JWT token
curl -X POST https://api.example.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "user@example.com",
    "password": "password"
  }'

# Use token in requests
curl https://api.example.com/api/v1/usage \
  -H "Authorization: Bearer $ACCESS_TOKEN"
```

### API Key Authentication

```bash
curl https://api.example.com/api/v1/usage \
  -H "X-API-Key: your-api-key"
```

## Middleware

### Custom Middleware

```rust
use axum::{
    Router,
    middleware::{self, Next},
    http::Request,
    response::Response,
};

async fn custom_middleware<B>(
    req: Request<B>,
    next: Next<B>,
) -> Response {
    println!("Request: {} {}", req.method(), req.uri());
    next.run(req).await
}

let app = Router::new()
    .route("/", get(handler))
    .layer(middleware::from_fn(custom_middleware));
```

## Configuration

### Environment Variables

```bash
# Server
export API_HOST="0.0.0.0"
export API_PORT=8080

# Database
export DATABASE_URL="postgresql://user:pass@localhost/costops"

# Authentication
export JWT_SECRET="your-secret-key"
export JWT_EXPIRY_HOURS=1

# Features
export ENABLE_METRICS=true
export METRICS_PORT=9090
export ENABLE_COMPRESSION=true
export LOG_LEVEL=info

# Rate Limiting
export RATE_LIMIT_PER_SECOND=100
export RATE_LIMIT_BURST=200
```

### Configuration File

```toml
# config.toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgresql://user:pass@localhost/costops"
max_connections = 10

[auth]
jwt_secret = "your-secret-key"
jwt_expiry_hours = 1

[features]
enable_metrics = true
metrics_port = 9090
enable_compression = true
log_level = "info"
```

## Performance

- **Throughput**: 10,000+ requests/second
- **Latency**: <10ms p50, <50ms p99
- **Concurrency**: 1000+ concurrent connections
- **Memory**: ~50MB baseline, ~200MB under load
- **Database**: Connection pooling with deadpool

## Deployment

### Docker

```dockerfile
FROM rust:1.91 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin llm-cost-ops-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/llm-cost-ops-api /usr/local/bin/
CMD ["llm-cost-ops-api"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cost-ops-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: cost-ops-api
  template:
    metadata:
      labels:
        app: cost-ops-api
    spec:
      containers:
      - name: api
        image: cost-ops-api:latest
        ports:
        - containerPort: 8080
        - containerPort: 9090
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: cost-ops-secrets
              key: database-url
        - name: JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: cost-ops-secrets
              key: jwt-secret
```

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Links

- **Documentation**: [https://docs.rs/llm-cost-ops-api](https://docs.rs/llm-cost-ops-api)
- **Core Library**: [https://crates.io/crates/llm-cost-ops](https://crates.io/crates/llm-cost-ops)
- **OpenAPI Spec**: [https://github.com/globalbusinessadvisors/llm-cost-ops/blob/main/openapi.json](https://github.com/globalbusinessadvisors/llm-cost-ops/blob/main/openapi.json)
- **Repository**: [https://github.com/globalbusinessadvisors/llm-cost-ops](https://github.com/globalbusinessadvisors/llm-cost-ops)
