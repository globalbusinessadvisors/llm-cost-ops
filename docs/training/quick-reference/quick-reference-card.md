# Quick Reference Card

**Version:** 1.0.0
**Last Updated:** 2025-11-16
**Format:** Designed for printing and desktop reference

Complete quick reference for daily LLM Cost Ops operations. Keep this handy for common commands, code snippets, API calls, and troubleshooting.

---

## Table of Contents

- [Essential Commands](#essential-commands)
- [API Endpoints Quick Reference](#api-endpoints-quick-reference)
- [Code Snippets by Language](#code-snippets-by-language)
- [Configuration Quick Reference](#configuration-quick-reference)
- [Troubleshooting Quick Fixes](#troubleshooting-quick-fixes)
- [Error Codes Reference](#error-codes-reference)
- [Support Contacts](#support-contacts)
- [Keyboard Shortcuts](#keyboard-shortcuts)

---

## Essential Commands

### CLI Installation

```bash
# Linux/macOS
curl -L https://github.com/llm-cost-ops/releases/latest/download/cost-ops-$(uname -s)-$(uname -m) -o cost-ops
chmod +x cost-ops
sudo mv cost-ops /usr/local/bin/

# Verify installation
cost-ops --version
```

### Database Initialization

```bash
# SQLite (development)
cost-ops init --database-url sqlite:cost-ops.db

# PostgreSQL (production)
cost-ops init --database-url postgresql://user:password@localhost:5432/costops

# Run migrations
cost-ops migrate --database-url sqlite:cost-ops.db
```

### Pricing Management

```bash
# Add pricing for OpenAI GPT-4
cost-ops pricing add \
  --provider openai \
  --model gpt-4-turbo \
  --input-price 10.0 \
  --output-price 30.0 \
  --currency USD

# Add pricing for Anthropic Claude
cost-ops pricing add \
  --provider anthropic \
  --model claude-3-5-sonnet-20241022 \
  --input-price 3.0 \
  --output-price 15.0

# Add pricing with cache discount
cost-ops pricing add \
  --provider anthropic \
  --model claude-3-5-sonnet-20241022 \
  --input-price 3.0 \
  --output-price 15.0 \
  --cached-discount 0.9  # 90% discount

# List all pricing
cost-ops pricing list --provider openai

# Update pricing
cost-ops pricing update \
  --id price_abc123 \
  --input-price 9.0 \
  --output-price 27.0
```

### Data Ingestion

```bash
# Ingest from JSON file
cost-ops ingest --file usage.json

# Ingest from stdin
cat usage.json | cost-ops ingest --stdin

# Ingest with validation only
cost-ops ingest --file usage.json --validate-only

# Batch ingest
cost-ops ingest --file large-usage.jsonl --format jsonl --batch-size 1000
```

### Querying Costs

```bash
# Query last 24 hours
cost-ops query --range last-24-hours --output table

# Query last 7 days with filters
cost-ops query \
  --range last-7-days \
  --organization org-abc \
  --provider openai \
  --output json

# Query specific date range
cost-ops query \
  --start 2025-01-01 \
  --end 2025-01-31 \
  --output csv > january-costs.csv

# Query with aggregation
cost-ops query \
  --range last-30-days \
  --group-by provider,model \
  --output table

# Available ranges:
# last-hour, last-24-hours, last-7-days, last-30-days, last-90-days, last-year
```

### Summary Reports

```bash
# Generate cost summary
cost-ops summary --period last-30-days

# Summary by organization
cost-ops summary \
  --period last-30-days \
  --organization org-abc

# Summary by provider
cost-ops summary \
  --period last-7-days \
  --group-by provider

# Summary with forecast
cost-ops summary \
  --period last-30-days \
  --include-forecast \
  --forecast-days 30
```

### Export Data

```bash
# Export to JSON
cost-ops export \
  --output costs.json \
  --format json \
  --period last-30-days

# Export to CSV
cost-ops export \
  --output costs.csv \
  --format csv \
  --period last-7-days \
  --organization org-abc

# Export to Excel
cost-ops export \
  --output costs.xlsx \
  --format excel \
  --period last-30-days

# Export with filters
cost-ops export \
  --output openai-costs.json \
  --format json \
  --provider openai \
  --start 2025-01-01 \
  --end 2025-01-31
```

### Forecasting

```bash
# Generate forecast (linear model)
cost-ops forecast \
  --horizon 30 \
  --model linear \
  --organization org-abc

# Generate forecast (moving average)
cost-ops forecast \
  --horizon 14 \
  --model moving-average \
  --window 7

# Forecast with confidence intervals
cost-ops forecast \
  --horizon 30 \
  --model exponential-smoothing \
  --confidence 0.95

# Detect anomalies
cost-ops forecast \
  --detect-anomalies \
  --method zscore \
  --threshold 3.0
```

### Authentication

```bash
# Create API key
cost-ops auth create-api-key \
  --name "Production API Key" \
  --scopes "usage:write,costs:read" \
  --organization org-abc

# List API keys
cost-ops auth list-api-keys --organization org-abc

# Revoke API key
cost-ops auth revoke-api-key --id key_abc123

# Create user
cost-ops auth create-user \
  --email user@example.com \
  --role developer \
  --organization org-abc

# Login (get JWT)
cost-ops auth login \
  --email user@example.com \
  --password "secure-password"
```

### Server Operations

```bash
# Start API server
cost-ops server start \
  --host 0.0.0.0 \
  --port 8080 \
  --database-url postgresql://localhost/costops

# Start with custom config
cost-ops server start --config /etc/cost-ops/config.toml

# Health check
curl http://localhost:8080/health

# Metrics endpoint
curl http://localhost:8080/metrics
```

### Organization Management

```bash
# Create organization
cost-ops org create \
  --name "Acme Corp" \
  --slug acme-corp

# List organizations
cost-ops org list

# Add user to organization
cost-ops org add-user \
  --org acme-corp \
  --user user@example.com \
  --role admin

# Remove user from organization
cost-ops org remove-user \
  --org acme-corp \
  --user user@example.com
```

---

## API Endpoints Quick Reference

### Base URL

```
Production: https://api.llm-cost-ops.example.com
Development: http://localhost:8080
API Version: /api/v1
```

### Authentication

```bash
# API Key (preferred for services)
curl -H "Authorization: Bearer sk_live_abc123xyz" \
  https://api.llm-cost-ops.example.com/api/v1/usage

# JWT Token (for user sessions)
curl -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  https://api.llm-cost-ops.example.com/api/v1/costs
```

### Health & Monitoring

```bash
# Health check
GET /health
curl http://localhost:8080/health

# Readiness check
GET /health/ready
curl http://localhost:8080/health/ready

# Liveness check
GET /health/live
curl http://localhost:8080/health/live

# Metrics (Prometheus format)
GET /metrics
curl http://localhost:8080/metrics
```

### Usage Management

```bash
# Create usage record
POST /api/v1/usage
curl -X POST https://api.example.com/api/v1/usage \
  -H "Authorization: Bearer sk_live_abc123" \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "openai",
    "model": {"name": "gpt-4-turbo"},
    "organization_id": "org-abc",
    "prompt_tokens": 1000,
    "completion_tokens": 500,
    "total_tokens": 1500
  }'

# Batch create usage records
POST /api/v1/usage/batch
curl -X POST https://api.example.com/api/v1/usage/batch \
  -H "Authorization: Bearer sk_live_abc123" \
  -H "Content-Type: application/json" \
  -d '{
    "records": [
      {"provider": "openai", "model": {"name": "gpt-4"}, ...},
      {"provider": "anthropic", "model": {"name": "claude-3-5-sonnet"}, ...}
    ]
  }'

# Get usage record
GET /api/v1/usage/{id}
curl https://api.example.com/api/v1/usage/550e8400-e29b-41d4-a716-446655440000 \
  -H "Authorization: Bearer sk_live_abc123"

# List usage records
GET /api/v1/usage?organization_id=org-abc&limit=100
curl "https://api.example.com/api/v1/usage?organization_id=org-abc&limit=100" \
  -H "Authorization: Bearer sk_live_abc123"
```

### Cost Analytics

```bash
# Get costs
GET /api/v1/costs?start=2025-01-01&end=2025-01-31
curl "https://api.example.com/api/v1/costs?start=2025-01-01&end=2025-01-31" \
  -H "Authorization: Bearer sk_live_abc123"

# Get cost by ID
GET /api/v1/costs/{id}
curl https://api.example.com/api/v1/costs/cost_abc123 \
  -H "Authorization: Bearer sk_live_abc123"

# Get aggregated costs
GET /api/v1/costs/aggregate?group_by=provider,model&start=2025-01-01
curl "https://api.example.com/api/v1/costs/aggregate?group_by=provider,model&start=2025-01-01" \
  -H "Authorization: Bearer sk_live_abc123"

# Cost summary
GET /api/v1/costs/summary?organization_id=org-abc
curl "https://api.example.com/api/v1/costs/summary?organization_id=org-abc" \
  -H "Authorization: Bearer sk_live_abc123"
```

### Pricing Management

```bash
# Create pricing table
POST /api/v1/pricing
curl -X POST https://api.example.com/api/v1/pricing \
  -H "Authorization: Bearer sk_live_abc123" \
  -H "Content-Type: application/json" \
  -d '{
    "provider": "openai",
    "model": "gpt-4-turbo",
    "pricing_structure": {
      "type": "per_token",
      "input_price_per_million": 10.0,
      "output_price_per_million": 30.0
    },
    "currency": "USD"
  }'

# List pricing tables
GET /api/v1/pricing?provider=openai&active=true
curl "https://api.example.com/api/v1/pricing?provider=openai&active=true" \
  -H "Authorization: Bearer sk_live_abc123"

# Update pricing
PUT /api/v1/pricing/{id}
curl -X PUT https://api.example.com/api/v1/pricing/price_abc123 \
  -H "Authorization: Bearer sk_live_abc123" \
  -H "Content-Type: application/json" \
  -d '{"input_price_per_million": 9.0}'
```

### Forecasting

```bash
# Generate forecast
POST /api/v1/forecast
curl -X POST https://api.example.com/api/v1/forecast \
  -H "Authorization: Bearer sk_live_abc123" \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "org-abc",
    "horizon_days": 30,
    "model": "linear",
    "confidence_level": 0.95
  }'

# Detect anomalies
POST /api/v1/forecast/anomalies
curl -X POST https://api.example.com/api/v1/forecast/anomalies \
  -H "Authorization: Bearer sk_live_abc123" \
  -H "Content-Type: application/json" \
  -d '{
    "organization_id": "org-abc",
    "method": "zscore",
    "threshold": 3.0
  }'
```

### Export & Reports

```bash
# Generate report
POST /api/v1/reports
curl -X POST https://api.example.com/api/v1/reports \
  -H "Authorization: Bearer sk_live_abc123" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "cost_summary",
    "format": "pdf",
    "start_date": "2025-01-01",
    "end_date": "2025-01-31",
    "organization_id": "org-abc"
  }'

# Schedule report
POST /api/v1/reports/schedule
curl -X POST https://api.example.com/api/v1/reports/schedule \
  -H "Authorization: Bearer sk_live_abc123" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "cost_summary",
    "frequency": "daily",
    "time": "09:00",
    "timezone": "America/New_York",
    "delivery": {
      "type": "email",
      "recipients": ["team@example.com"]
    }
  }'
```

---

## Code Snippets by Language

### Python SDK

```python
# Installation
pip install llm-cost-ops

# Basic usage
from llm_cost_ops import CostOpsClient

client = CostOpsClient(
    api_key="sk_live_abc123",
    base_url="https://api.example.com"
)

# Track usage
usage = client.usage.create(
    provider="openai",
    model="gpt-4-turbo",
    organization_id="org-abc",
    prompt_tokens=1000,
    completion_tokens=500,
    total_tokens=1500
)

# Get costs
costs = client.costs.list(
    organization_id="org-abc",
    start_date="2025-01-01",
    end_date="2025-01-31",
    provider="openai"
)

# Calculate total cost
total = sum(cost.total_cost for cost in costs.data)
print(f"Total cost: ${total:.2f}")

# Get cost summary
summary = client.costs.summary(
    organization_id="org-abc",
    start_date="2025-01-01",
    end_date="2025-01-31"
)

# Generate forecast
forecast = client.forecast.generate(
    organization_id="org-abc",
    horizon_days=30,
    model="linear"
)

# Error handling
from llm_cost_ops.exceptions import APIError, ValidationError

try:
    usage = client.usage.create(...)
except ValidationError as e:
    print(f"Invalid data: {e}")
except APIError as e:
    print(f"API error: {e.status_code} - {e.message}")

# Async client
from llm_cost_ops import AsyncCostOpsClient

async def track_usage():
    async with AsyncCostOpsClient(api_key="sk_live_abc123") as client:
        usage = await client.usage.create(
            provider="openai",
            model="gpt-4-turbo",
            prompt_tokens=1000,
            completion_tokens=500
        )
        return usage

# Context manager for automatic retry
with client.with_retry(max_retries=3, backoff_factor=2.0):
    usage = client.usage.create(...)
```

### TypeScript SDK

```typescript
// Installation
npm install @llm-cost-ops/sdk

// Basic usage
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  apiKey: 'sk_live_abc123',
  baseURL: 'https://api.example.com'
});

// Track usage
const usage = await client.usage.create({
  provider: 'openai',
  model: 'gpt-4-turbo',
  organizationId: 'org-abc',
  promptTokens: 1000,
  completionTokens: 500,
  totalTokens: 1500
});

// Get costs
const costs = await client.costs.list({
  organizationId: 'org-abc',
  startDate: '2025-01-01',
  endDate: '2025-01-31',
  provider: 'openai'
});

// Calculate total cost
const total = costs.data.reduce((sum, cost) => sum + cost.totalCost, 0);
console.log(`Total cost: $${total.toFixed(2)}`);

// Get cost summary
const summary = await client.costs.summary({
  organizationId: 'org-abc',
  startDate: '2025-01-01',
  endDate: '2025-01-31'
});

// Generate forecast
const forecast = await client.forecast.generate({
  organizationId: 'org-abc',
  horizonDays: 30,
  model: 'linear'
});

// Error handling
import { APIError, ValidationError } from '@llm-cost-ops/sdk';

try {
  const usage = await client.usage.create({...});
} catch (error) {
  if (error instanceof ValidationError) {
    console.error('Invalid data:', error.message);
  } else if (error instanceof APIError) {
    console.error(`API error: ${error.statusCode} - ${error.message}`);
  }
}

// With retry configuration
const clientWithRetry = new CostOpsClient({
  apiKey: 'sk_live_abc123',
  retry: {
    maxRetries: 3,
    backoffFactor: 2.0
  }
});

// Streaming responses
const stream = await client.costs.listStream({
  organizationId: 'org-abc'
});

for await (const cost of stream) {
  console.log(cost);
}
```

### Go SDK

```go
// Installation
go get github.com/llm-cost-ops/sdk-go

// Basic usage
package main

import (
    "context"
    "fmt"
    "log"

    costops "github.com/llm-cost-ops/sdk-go"
)

func main() {
    client := costops.NewClient(
        costops.WithAPIKey("sk_live_abc123"),
        costops.WithBaseURL("https://api.example.com"),
    )

    ctx := context.Background()

    // Track usage
    usage, err := client.Usage.Create(ctx, &costops.UsageCreateParams{
        Provider:         "openai",
        Model:            "gpt-4-turbo",
        OrganizationID:   "org-abc",
        PromptTokens:     1000,
        CompletionTokens: 500,
        TotalTokens:      1500,
    })
    if err != nil {
        log.Fatal(err)
    }

    // Get costs
    costs, err := client.Costs.List(ctx, &costops.CostListParams{
        OrganizationID: "org-abc",
        StartDate:      "2025-01-01",
        EndDate:        "2025-01-31",
        Provider:       "openai",
    })
    if err != nil {
        log.Fatal(err)
    }

    // Calculate total cost
    var total float64
    for _, cost := range costs.Data {
        total += cost.TotalCost
    }
    fmt.Printf("Total cost: $%.2f\n", total)

    // Get cost summary
    summary, err := client.Costs.Summary(ctx, &costops.CostSummaryParams{
        OrganizationID: "org-abc",
        StartDate:      "2025-01-01",
        EndDate:        "2025-01-31",
    })

    // Generate forecast
    forecast, err := client.Forecast.Generate(ctx, &costops.ForecastParams{
        OrganizationID: "org-abc",
        HorizonDays:    30,
        Model:          "linear",
    })

    // Error handling
    if err != nil {
        switch e := err.(type) {
        case *costops.ValidationError:
            fmt.Printf("Invalid data: %v\n", e)
        case *costops.APIError:
            fmt.Printf("API error: %d - %s\n", e.StatusCode, e.Message)
        default:
            fmt.Printf("Unknown error: %v\n", e)
        }
    }
}

// With retry configuration
client := costops.NewClient(
    costops.WithAPIKey("sk_live_abc123"),
    costops.WithRetry(3, 2.0),
)
```

### Rust SDK

```rust
// Installation
// Add to Cargo.toml:
// llm-cost-ops = "1.0"

// Basic usage
use llm_cost_ops::{CostOpsClient, UsageCreateParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CostOpsClient::new(
        "sk_live_abc123",
        Some("https://api.example.com"),
    );

    // Track usage
    let usage = client.usage().create(UsageCreateParams {
        provider: "openai".to_string(),
        model: "gpt-4-turbo".to_string(),
        organization_id: "org-abc".to_string(),
        prompt_tokens: 1000,
        completion_tokens: 500,
        total_tokens: 1500,
        ..Default::default()
    }).await?;

    // Get costs
    let costs = client.costs().list(CostListParams {
        organization_id: Some("org-abc".to_string()),
        start_date: Some("2025-01-01".to_string()),
        end_date: Some("2025-01-31".to_string()),
        provider: Some("openai".to_string()),
        ..Default::default()
    }).await?;

    // Calculate total cost
    let total: f64 = costs.data.iter()
        .map(|cost| cost.total_cost)
        .sum();
    println!("Total cost: ${:.2}", total);

    // Get cost summary
    let summary = client.costs().summary(CostSummaryParams {
        organization_id: "org-abc".to_string(),
        start_date: "2025-01-01".to_string(),
        end_date: "2025-01-31".to_string(),
        ..Default::default()
    }).await?;

    // Generate forecast
    let forecast = client.forecast().generate(ForecastParams {
        organization_id: "org-abc".to_string(),
        horizon_days: 30,
        model: ForecastModel::Linear,
        ..Default::default()
    }).await?;

    Ok(())
}

// Error handling
use llm_cost_ops::error::{Error, ValidationError, APIError};

match client.usage().create(params).await {
    Ok(usage) => println!("Success: {:?}", usage),
    Err(Error::Validation(e)) => eprintln!("Invalid data: {}", e),
    Err(Error::API(e)) => eprintln!("API error: {} - {}", e.status_code, e.message),
    Err(e) => eprintln!("Unknown error: {}", e),
}

// With retry configuration
let client = CostOpsClient::builder()
    .api_key("sk_live_abc123")
    .base_url("https://api.example.com")
    .max_retries(3)
    .backoff_factor(2.0)
    .build()?;
```

---

## Configuration Quick Reference

### Environment Variables

```bash
# Database
export DATABASE_URL="postgresql://user:password@localhost:5432/costops"
export DATABASE_POOL_SIZE=20

# Server
export SERVER_HOST="0.0.0.0"
export SERVER_PORT=8080
export SERVER_WORKERS=4

# Authentication
export JWT_SECRET="your-secret-key"
export JWT_EXPIRATION=3600
export API_KEY_PREFIX="sk_live_"

# Observability
export RUST_LOG="info,cost_ops=debug"
export OTLP_ENDPOINT="http://localhost:4317"
export METRICS_PORT=9090

# Rate Limiting
export RATE_LIMIT_REQUESTS=1000
export RATE_LIMIT_WINDOW=60

# Export
export EXPORT_S3_BUCKET="cost-ops-exports"
export EXPORT_S3_REGION="us-east-1"

# Email
export SMTP_HOST="smtp.gmail.com"
export SMTP_PORT=587
export SMTP_USERNAME="noreply@example.com"
export SMTP_PASSWORD="app-password"
```

### Configuration File (config.toml)

```toml
[database]
url = "postgresql://localhost/costops"
pool_size = 20
max_lifetime_seconds = 3600

[server]
host = "0.0.0.0"
port = 8080
workers = 4
request_timeout_seconds = 30

[auth]
jwt_secret = "your-secret-key"
jwt_expiration_seconds = 3600
api_key_prefix = "sk_live_"
require_api_key = true

[observability]
log_level = "info"
log_format = "json"
metrics_enabled = true
metrics_port = 9090
tracing_enabled = true
otlp_endpoint = "http://localhost:4317"

[rate_limit]
enabled = true
requests_per_window = 1000
window_seconds = 60

[export]
default_format = "json"
max_export_size_mb = 100

[export.s3]
bucket = "cost-ops-exports"
region = "us-east-1"
enabled = true

[export.email]
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = "noreply@example.com"
smtp_use_tls = true
from_address = "noreply@example.com"

[forecasting]
default_horizon_days = 30
default_model = "linear"
min_historical_days = 7
```

### Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cost-ops-config
  namespace: llm-cost-ops
data:
  config.toml: |
    [database]
    url = "postgresql://postgres:5432/costops"
    pool_size = 20

    [server]
    host = "0.0.0.0"
    port = 8080

    [observability]
    log_level = "info"
    metrics_enabled = true
```

---

## Troubleshooting Quick Fixes

### Common Issues and Solutions

**Database Connection Failed**
```bash
# Check database is running
pg_isready -h localhost -p 5432

# Test connection
psql postgresql://user:password@localhost:5432/costops

# Reset database
cost-ops migrate reset --database-url postgresql://localhost/costops
cost-ops migrate run --database-url postgresql://localhost/costops
```

**API Key Authentication Failed**
```bash
# Verify API key format
echo $API_KEY | grep -E '^sk_(live|test|admin)_[a-zA-Z0-9]{20,}$'

# Create new API key
cost-ops auth create-api-key --name "New Key"

# Test API key
curl -H "Authorization: Bearer $API_KEY" http://localhost:8080/health
```

**High Memory Usage**
```bash
# Check current usage
ps aux | grep cost-ops

# Adjust pool size
export DATABASE_POOL_SIZE=10

# Monitor metrics
curl http://localhost:9090/metrics | grep memory
```

**Slow Queries**
```bash
# Enable query logging
export RUST_LOG="sqlx=debug"

# Check database indexes
psql $DATABASE_URL -c "\di"

# Analyze slow queries
psql $DATABASE_URL -c "SELECT * FROM pg_stat_statements ORDER BY total_exec_time DESC LIMIT 10;"
```

**Rate Limit Errors**
```bash
# Check rate limit configuration
curl http://localhost:8080/api/v1/health | jq '.rate_limit'

# Adjust rate limits
export RATE_LIMIT_REQUESTS=2000
export RATE_LIMIT_WINDOW=60

# Bypass rate limiting (development only)
export RATE_LIMIT_ENABLED=false
```

**Import/Export Failures**
```bash
# Validate JSON format
cat usage.json | jq .

# Check file permissions
ls -la usage.json

# Test with smaller batch
head -n 100 large-usage.jsonl > test.jsonl
cost-ops ingest --file test.jsonl

# Check disk space
df -h /var/lib/cost-ops
```

**Metrics Not Showing**
```bash
# Verify metrics endpoint
curl http://localhost:9090/metrics

# Check Prometheus configuration
kubectl get servicemonitor -n llm-cost-ops

# Restart metrics server
kubectl rollout restart deployment/cost-ops -n llm-cost-ops
```

**SSL/TLS Certificate Errors**
```bash
# Skip verification (development only)
export SSL_VERIFY=false

# Update CA certificates
sudo update-ca-certificates

# Check certificate expiration
openssl s_client -connect api.example.com:443 -servername api.example.com </dev/null 2>/dev/null | openssl x509 -noout -dates
```

---

## Error Codes Reference

### HTTP Status Codes

| Code | Meaning | Common Cause | Solution |
|------|---------|--------------|----------|
| 400 | Bad Request | Invalid input data | Check request format |
| 401 | Unauthorized | Missing/invalid API key | Verify authentication |
| 403 | Forbidden | Insufficient permissions | Check user role |
| 404 | Not Found | Resource doesn't exist | Verify ID is correct |
| 409 | Conflict | Duplicate resource | Check for existing record |
| 422 | Unprocessable Entity | Validation failed | Fix validation errors |
| 429 | Too Many Requests | Rate limit exceeded | Reduce request rate |
| 500 | Internal Server Error | Server error | Check logs |
| 503 | Service Unavailable | System overloaded | Retry later |

### Application Error Codes

```
USAGE_001 - Invalid usage record format
USAGE_002 - Missing required fields
USAGE_003 - Invalid token count (negative or zero)
USAGE_004 - Provider not supported
USAGE_005 - Model not found

COST_001 - Pricing table not found
COST_002 - Cost calculation failed
COST_003 - Invalid date range
COST_004 - Currency mismatch
COST_005 - Negative cost calculated

AUTH_001 - Invalid API key
AUTH_002 - API key expired
AUTH_003 - Invalid JWT token
AUTH_004 - JWT token expired
AUTH_005 - Insufficient permissions
AUTH_006 - Invalid credentials
AUTH_007 - User not found
AUTH_008 - Organization access denied

PRICING_001 - Invalid pricing structure
PRICING_002 - Negative price value
PRICING_003 - Overlapping effective dates
PRICING_004 - Missing pricing for date

FORECAST_001 - Insufficient historical data
FORECAST_002 - Invalid forecast model
FORECAST_003 - Invalid horizon
FORECAST_004 - Forecast generation failed

EXPORT_001 - Invalid export format
EXPORT_002 - Export size exceeded
EXPORT_003 - Export generation failed
EXPORT_004 - Delivery failed

DB_001 - Database connection failed
DB_002 - Query execution failed
DB_003 - Transaction failed
DB_004 - Constraint violation
DB_005 - Deadlock detected
```

### Validation Error Messages

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Validation failed",
    "details": [
      {
        "field": "prompt_tokens",
        "error": "must be greater than 0",
        "provided": -100
      },
      {
        "field": "provider",
        "error": "must be one of: openai, anthropic, google, azure, aws, cohere, mistral",
        "provided": "invalid"
      }
    ]
  }
}
```

---

## Support Contacts

### Technical Support

**Email:** support@llm-cost-ops.com
**Response Time:** < 24 hours

**Slack Community:** https://llm-cost-ops.slack.com
**Discord:** https://discord.gg/llm-cost-ops

### Emergency Contacts

**Production Incidents:** incidents@llm-cost-ops.com
**Security Issues:** security@llm-cost-ops.com
**On-Call:** +1-555-COST-OPS

### Documentation and Resources

**Documentation:** https://docs.llm-cost-ops.com
**API Reference:** https://api-docs.llm-cost-ops.com
**Status Page:** https://status.llm-cost-ops.com
**GitHub:** https://github.com/llm-cost-ops/llm-cost-ops

### Office Hours

**Weekly Office Hours:**
- Tuesdays: 2:00 PM - 3:00 PM EST
- Thursdays: 10:00 AM - 11:00 AM EST
- Zoom Link: https://zoom.us/j/llmcostops

---

## Keyboard Shortcuts

### CLI Shortcuts

```bash
# Bash aliases (add to .bashrc or .zshrc)
alias co="cost-ops"
alias coq="cost-ops query"
alias cos="cost-ops summary"
alias coi="cost-ops ingest"
alias coe="cost-ops export"

# Functions for common operations
function co-today() {
    cost-ops query --range last-24-hours --output table
}

function co-week() {
    cost-ops query --range last-7-days --output table
}

function co-month() {
    cost-ops query --range last-30-days --output table
}

function co-org() {
    cost-ops query --organization "$1" --range last-30-days --output table
}
```

### Shell Completion

```bash
# Bash
cost-ops completion bash > /etc/bash_completion.d/cost-ops
source /etc/bash_completion.d/cost-ops

# Zsh
cost-ops completion zsh > "${fpath[1]}/_cost-ops"

# Fish
cost-ops completion fish > ~/.config/fish/completions/cost-ops.fish
```

---

## Quick Command Reference Card

### Most Common Commands

```bash
# Daily Operations
cost-ops query --range last-24-hours          # Check today's costs
cost-ops summary --period last-7-days         # Weekly summary
cost-ops forecast --horizon 30                # 30-day forecast

# Data Management
cost-ops ingest --file usage.json             # Import usage data
cost-ops export --output costs.csv            # Export to CSV
cost-ops pricing list --provider openai       # List pricing

# Administration
cost-ops auth create-api-key --name "Key"     # Create API key
cost-ops org create --name "Org"              # Create organization
cost-ops server start                         # Start API server

# Monitoring
curl http://localhost:8080/health             # Health check
curl http://localhost:9090/metrics            # Prometheus metrics
cost-ops query --range last-hour --output json | jq '.total_cost'  # Current cost
```

---

**Print Instructions:**
- Print double-sided for compact reference
- Laminate for durability
- Keep near workstation
- Share with team members

**Last Updated:** 2025-11-16
**Version:** 1.0.0
