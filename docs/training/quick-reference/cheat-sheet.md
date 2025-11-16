# LLM Cost Ops Cheat Sheet

**Version:** 1.0.0
**Last Updated:** 2025-11-16

Comprehensive cheat sheet for LLM Cost Ops covering SDK quick starts, common operations, error codes, configuration patterns, best practices, security, and performance tips.

---

## Table of Contents

- [SDK Quick Start](#sdk-quick-start)
- [Common Operations](#common-operations)
- [Error Codes and Solutions](#error-codes-and-solutions)
- [Configuration Patterns](#configuration-patterns)
- [Best Practices Summary](#best-practices-summary)
- [Security Checklist](#security-checklist)
- [Performance Tips](#performance-tips)
- [Links to Detailed Docs](#links-to-detailed-docs)

---

## SDK Quick Start

### Python SDK

**Installation:**
```bash
pip install llm-cost-ops
```

**Basic Setup:**
```python
from llm_cost_ops import CostOpsClient

# Initialize client
client = CostOpsClient(
    api_key="sk_live_abc123xyz",
    base_url="https://api.llm-cost-ops.example.com"
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
    end_date="2025-01-31"
)

# Calculate total
total = sum(cost.total_cost for cost in costs.data)
print(f"Total: ${total:.2f}")
```

**Async Support:**
```python
from llm_cost_ops import AsyncCostOpsClient

async def main():
    async with AsyncCostOpsClient(api_key="sk_live_abc123") as client:
        usage = await client.usage.create(
            provider="openai",
            model="gpt-4-turbo",
            prompt_tokens=1000,
            completion_tokens=500
        )
        return usage
```

**Error Handling:**
```python
from llm_cost_ops.exceptions import APIError, ValidationError

try:
    usage = client.usage.create(...)
except ValidationError as e:
    print(f"Validation error: {e.message}")
    print(f"Field errors: {e.field_errors}")
except APIError as e:
    print(f"API error: {e.status_code} - {e.message}")
```

**Retry Configuration:**
```python
client = CostOpsClient(
    api_key="sk_live_abc123",
    max_retries=3,
    retry_backoff=2.0,
    timeout=30.0
)
```

**Pagination:**
```python
# Iterate through all pages
for cost in client.costs.list_paginated(organization_id="org-abc"):
    print(f"Cost: ${cost.total_cost}")

# Manual pagination
page = 1
while True:
    response = client.costs.list(
        organization_id="org-abc",
        page=page,
        per_page=100
    )
    for cost in response.data:
        print(cost)
    if not response.has_more:
        break
    page += 1
```

---

### TypeScript SDK

**Installation:**
```bash
npm install @llm-cost-ops/sdk
# or
yarn add @llm-cost-ops/sdk
```

**Basic Setup:**
```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

// Initialize client
const client = new CostOpsClient({
  apiKey: 'sk_live_abc123xyz',
  baseURL: 'https://api.llm-cost-ops.example.com'
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
  endDate: '2025-01-31'
});

// Calculate total
const total = costs.data.reduce((sum, cost) => sum + cost.totalCost, 0);
console.log(`Total: $${total.toFixed(2)}`);
```

**Type Safety:**
```typescript
import { UsageRecord, CostRecord, Provider } from '@llm-cost-ops/sdk';

interface CreateUsageParams {
  provider: Provider;
  model: string;
  organizationId: string;
  promptTokens: number;
  completionTokens: number;
}

const params: CreateUsageParams = {
  provider: 'openai',  // Type-safe provider enum
  model: 'gpt-4-turbo',
  organizationId: 'org-abc',
  promptTokens: 1000,
  completionTokens: 500
};
```

**Error Handling:**
```typescript
import { APIError, ValidationError } from '@llm-cost-ops/sdk';

try {
  const usage = await client.usage.create({...});
} catch (error) {
  if (error instanceof ValidationError) {
    console.error('Validation error:', error.message);
    console.error('Field errors:', error.fieldErrors);
  } else if (error instanceof APIError) {
    console.error(`API error: ${error.statusCode} - ${error.message}`);
  } else {
    console.error('Unknown error:', error);
  }
}
```

**Retry Configuration:**
```typescript
const client = new CostOpsClient({
  apiKey: 'sk_live_abc123',
  retry: {
    maxRetries: 3,
    backoffFactor: 2.0
  },
  timeout: 30000
});
```

**Streaming:**
```typescript
// Stream costs
const stream = await client.costs.listStream({
  organizationId: 'org-abc'
});

for await (const cost of stream) {
  console.log(`Cost: $${cost.totalCost}`);
}
```

---

### Go SDK

**Installation:**
```bash
go get github.com/llm-cost-ops/sdk-go
```

**Basic Setup:**
```go
package main

import (
    "context"
    "fmt"
    "log"

    costops "github.com/llm-cost-ops/sdk-go"
)

func main() {
    // Initialize client
    client := costops.NewClient(
        costops.WithAPIKey("sk_live_abc123xyz"),
        costops.WithBaseURL("https://api.llm-cost-ops.example.com"),
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
    })
    if err != nil {
        log.Fatal(err)
    }

    // Calculate total
    var total float64
    for _, cost := range costs.Data {
        total += cost.TotalCost
    }
    fmt.Printf("Total: $%.2f\n", total)
}
```

**Context and Timeouts:**
```go
import "time"

// With timeout
ctx, cancel := context.WithTimeout(context.Background(), 30*time.Second)
defer cancel()

usage, err := client.Usage.Create(ctx, params)
```

**Error Handling:**
```go
import "errors"

usage, err := client.Usage.Create(ctx, params)
if err != nil {
    var validationErr *costops.ValidationError
    var apiErr *costops.APIError

    switch {
    case errors.As(err, &validationErr):
        fmt.Printf("Validation error: %v\n", validationErr)
        for field, msg := range validationErr.FieldErrors {
            fmt.Printf("  %s: %s\n", field, msg)
        }
    case errors.As(err, &apiErr):
        fmt.Printf("API error: %d - %s\n", apiErr.StatusCode, apiErr.Message)
    default:
        fmt.Printf("Unknown error: %v\n", err)
    }
}
```

**Retry Configuration:**
```go
client := costops.NewClient(
    costops.WithAPIKey("sk_live_abc123"),
    costops.WithRetry(3, 2.0),
    costops.WithTimeout(30*time.Second),
)
```

**Pagination:**
```go
// Iterate all pages
page := 1
for {
    costs, err := client.Costs.List(ctx, &costops.CostListParams{
        OrganizationID: "org-abc",
        Page:          page,
        PerPage:       100,
    })
    if err != nil {
        log.Fatal(err)
    }

    for _, cost := range costs.Data {
        fmt.Printf("Cost: $%.2f\n", cost.TotalCost)
    }

    if !costs.HasMore {
        break
    }
    page++
}
```

---

### Rust SDK

**Installation:**
```toml
# Cargo.toml
[dependencies]
llm-cost-ops = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

**Basic Setup:**
```rust
use llm_cost_ops::{CostOpsClient, UsageCreateParams, CostListParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let client = CostOpsClient::new(
        "sk_live_abc123xyz",
        Some("https://api.llm-cost-ops.example.com"),
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
        ..Default::default()
    }).await?;

    // Calculate total
    let total: f64 = costs.data.iter()
        .map(|cost| cost.total_cost)
        .sum();
    println!("Total: ${:.2}", total);

    Ok(())
}
```

**Builder Pattern:**
```rust
let client = CostOpsClient::builder()
    .api_key("sk_live_abc123")
    .base_url("https://api.example.com")
    .max_retries(3)
    .backoff_factor(2.0)
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
```

**Error Handling:**
```rust
use llm_cost_ops::error::{Error, ValidationError, APIError};

match client.usage().create(params).await {
    Ok(usage) => println!("Success: {:?}", usage),
    Err(Error::Validation(e)) => {
        eprintln!("Validation error: {}", e.message);
        for (field, error) in e.field_errors {
            eprintln!("  {}: {}", field, error);
        }
    }
    Err(Error::API(e)) => {
        eprintln!("API error: {} - {}", e.status_code, e.message);
    }
    Err(e) => eprintln!("Unknown error: {}", e),
}
```

**Type Safety with Enums:**
```rust
use llm_cost_ops::{Provider, ForecastModel};

let usage = client.usage().create(UsageCreateParams {
    provider: Provider::OpenAI,  // Type-safe enum
    model: "gpt-4-turbo".to_string(),
    // ... other fields
}).await?;

let forecast = client.forecast().generate(ForecastParams {
    forecast_model: ForecastModel::Linear,
    horizon_days: 30,
    // ... other fields
}).await?;
```

**Stream Processing:**
```rust
use futures::StreamExt;

let mut stream = client.costs().list_stream(params).await?;

while let Some(result) = stream.next().await {
    match result {
        Ok(cost) => println!("Cost: ${:.2}", cost.total_cost),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

---

## Common Operations

### 1. Track LLM Usage

**Basic Usage Tracking:**
```python
# Python
usage = client.usage.create(
    provider="openai",
    model="gpt-4-turbo",
    organization_id="org-abc",
    prompt_tokens=1000,
    completion_tokens=500,
    total_tokens=1500
)
```

```typescript
// TypeScript
const usage = await client.usage.create({
  provider: 'openai',
  model: 'gpt-4-turbo',
  organizationId: 'org-abc',
  promptTokens: 1000,
  completionTokens: 500,
  totalTokens: 1500
});
```

**With Optional Fields:**
```python
# Python - Full usage record
usage = client.usage.create(
    provider="anthropic",
    model="claude-3-5-sonnet-20241022",
    organization_id="org-abc",
    project_id="proj-xyz",
    user_id="user-123",
    prompt_tokens=2000,
    completion_tokens=800,
    total_tokens=2800,
    cached_tokens=500,  # Cached prompt tokens
    reasoning_tokens=100,  # For reasoning models
    latency_ms=1500,
    tags=["production", "api"],
    metadata={
        "request_id": "req-abc",
        "endpoint": "/chat/completions",
        "customer_id": "cust-123"
    }
)
```

**Batch Usage Tracking:**
```python
# Python - Batch create
usages = client.usage.batch_create([
    {
        "provider": "openai",
        "model": "gpt-4-turbo",
        "organization_id": "org-abc",
        "prompt_tokens": 1000,
        "completion_tokens": 500
    },
    {
        "provider": "anthropic",
        "model": "claude-3-5-sonnet",
        "organization_id": "org-abc",
        "prompt_tokens": 1500,
        "completion_tokens": 600
    }
])
```

### 2. Query Costs

**Simple Cost Query:**
```python
# Python
costs = client.costs.list(
    organization_id="org-abc",
    start_date="2025-01-01",
    end_date="2025-01-31"
)
```

**With Filters:**
```python
# Python - Filtered query
costs = client.costs.list(
    organization_id="org-abc",
    start_date="2025-01-01",
    end_date="2025-01-31",
    provider="openai",
    model="gpt-4-turbo",
    project_id="proj-xyz",
    tags=["production"]
)
```

**Aggregated Costs:**
```python
# Python - Get aggregated costs
aggregated = client.costs.aggregate(
    organization_id="org-abc",
    start_date="2025-01-01",
    end_date="2025-01-31",
    group_by=["provider", "model"]
)

for group in aggregated.data:
    print(f"{group.provider}/{group.model}: ${group.total_cost:.2f}")
```

**Cost Summary:**
```python
# Python - Get summary
summary = client.costs.summary(
    organization_id="org-abc",
    start_date="2025-01-01",
    end_date="2025-01-31"
)

print(f"Total Cost: ${summary.total_cost}")
print(f"Total Requests: {summary.total_requests}")
print(f"Avg Cost/Request: ${summary.avg_cost_per_request}")
```

### 3. Manage Pricing

**Add Pricing:**
```python
# Python - Add per-token pricing
pricing = client.pricing.create(
    provider="openai",
    model="gpt-4-turbo",
    pricing_structure={
        "type": "per_token",
        "input_price_per_million": 10.0,
        "output_price_per_million": 30.0,
        "cached_input_discount": 0.5  # 50% discount
    },
    currency="USD",
    effective_date="2025-01-01"
)
```

**Update Pricing:**
```python
# Python - Update pricing
pricing = client.pricing.update(
    pricing_id="price_abc123",
    input_price_per_million=9.0,
    output_price_per_million=27.0,
    effective_date="2025-02-01"
)
```

**List Active Pricing:**
```python
# Python - Get active pricing
pricing = client.pricing.list(
    provider="openai",
    active=True
)
```

### 4. Generate Forecasts

**Linear Forecast:**
```python
# Python - Generate 30-day forecast
forecast = client.forecast.generate(
    organization_id="org-abc",
    horizon_days=30,
    model="linear",
    confidence_level=0.95
)

print(f"Predicted cost in 30 days: ${forecast.predictions[-1].value:.2f}")
print(f"Confidence interval: ${forecast.predictions[-1].lower_bound:.2f} - ${forecast.predictions[-1].upper_bound:.2f}")
```

**Multiple Models:**
```python
# Python - Compare forecast models
models = ["linear", "moving_average", "exponential_smoothing"]
forecasts = {}

for model in models:
    forecasts[model] = client.forecast.generate(
        organization_id="org-abc",
        horizon_days=30,
        model=model
    )

for model, forecast in forecasts.items():
    print(f"{model}: ${forecast.predictions[-1].value:.2f}")
```

### 5. Detect Anomalies

**Z-Score Method:**
```python
# Python - Detect cost anomalies
anomalies = client.forecast.detect_anomalies(
    organization_id="org-abc",
    method="zscore",
    threshold=3.0,
    start_date="2025-01-01",
    end_date="2025-01-31"
)

for anomaly in anomalies.data:
    print(f"Anomaly on {anomaly.date}: ${anomaly.actual_cost:.2f} (expected: ${anomaly.expected_cost:.2f})")
```

**IQR Method:**
```python
# Python - IQR anomaly detection
anomalies = client.forecast.detect_anomalies(
    organization_id="org-abc",
    method="iqr",
    start_date="2025-01-01",
    end_date="2025-01-31"
)
```

### 6. Export Data

**Export to CSV:**
```python
# Python - Export to CSV
export = client.export.create(
    organization_id="org-abc",
    format="csv",
    start_date="2025-01-01",
    end_date="2025-01-31",
    include_fields=["provider", "model", "cost", "tokens"]
)

# Download export
with open("costs.csv", "wb") as f:
    f.write(export.download())
```

**Scheduled Export:**
```python
# Python - Schedule daily export
schedule = client.export.schedule(
    organization_id="org-abc",
    format="excel",
    frequency="daily",
    time="09:00",
    timezone="America/New_York",
    delivery={
        "type": "email",
        "recipients": ["team@example.com"]
    }
)
```

### 7. Generate Reports

**Cost Summary Report:**
```python
# Python - Generate cost report
report = client.reports.generate(
    type="cost_summary",
    organization_id="org-abc",
    start_date="2025-01-01",
    end_date="2025-01-31",
    format="pdf",
    include_charts=True
)

# Download report
with open("cost_report.pdf", "wb") as f:
    f.write(report.download())
```

**Usage Analysis Report:**
```python
# Python - Generate usage report
report = client.reports.generate(
    type="usage_analysis",
    organization_id="org-abc",
    start_date="2025-01-01",
    end_date="2025-01-31",
    group_by=["provider", "model"],
    format="excel"
)
```

### 8. Manage Organizations

**Create Organization:**
```python
# Python - Create organization
org = client.organizations.create(
    name="Acme Corp",
    slug="acme-corp",
    settings={
        "budget_limit": 10000.0,
        "budget_period": "monthly",
        "alert_threshold": 0.8
    }
)
```

**Manage Users:**
```python
# Python - Add user to organization
client.organizations.add_user(
    organization_id="org-abc",
    user_email="user@example.com",
    role="developer"
)

# Remove user
client.organizations.remove_user(
    organization_id="org-abc",
    user_id="user-123"
)
```

### 9. Audit Logs

**Query Audit Logs:**
```python
# Python - Get audit logs
logs = client.audit.list(
    organization_id="org-abc",
    start_date="2025-01-01",
    end_date="2025-01-31",
    action="usage.create",
    user_id="user-123"
)

for log in logs.data:
    print(f"{log.timestamp}: {log.user_email} performed {log.action}")
```

---

## Error Codes and Solutions

### HTTP Status Codes

**400 Bad Request**
```
Cause: Invalid request format or parameters
Solution: Check request body and parameters
Example: Missing required fields, invalid JSON
```

**401 Unauthorized**
```
Cause: Missing or invalid API key
Solution: Verify API key in Authorization header
Example: curl -H "Authorization: Bearer sk_live_abc123" ...
```

**403 Forbidden**
```
Cause: Insufficient permissions
Solution: Check user role and organization access
Example: User lacks "usage:write" permission
```

**404 Not Found**
```
Cause: Resource doesn't exist
Solution: Verify resource ID
Example: Usage record ID not found
```

**422 Unprocessable Entity**
```
Cause: Validation failed
Solution: Fix validation errors in request
Example: Negative token counts, invalid date format
```

**429 Too Many Requests**
```
Cause: Rate limit exceeded
Solution: Reduce request rate or increase limits
Example: More than 1000 requests per minute
```

**500 Internal Server Error**
```
Cause: Server-side error
Solution: Check logs, retry request
Example: Database connection failure
```

**503 Service Unavailable**
```
Cause: System temporarily unavailable
Solution: Retry with exponential backoff
Example: Database maintenance in progress
```

### Application Error Codes

**USAGE_001 - Invalid Usage Record**
```python
# Error
{
  "error": {
    "code": "USAGE_001",
    "message": "Invalid usage record format"
  }
}

# Solution
usage = client.usage.create(
    provider="openai",  # Required
    model="gpt-4-turbo",  # Required
    organization_id="org-abc",  # Required
    prompt_tokens=1000,  # Required, must be > 0
    completion_tokens=500  # Required, must be > 0
)
```

**COST_001 - Pricing Not Found**
```python
# Error
{
  "error": {
    "code": "COST_001",
    "message": "Pricing table not found for openai/gpt-4-turbo on 2025-01-15"
  }
}

# Solution - Add pricing first
pricing = client.pricing.create(
    provider="openai",
    model="gpt-4-turbo",
    pricing_structure={...},
    effective_date="2025-01-01"
)
```

**AUTH_001 - Invalid API Key**
```bash
# Error
{
  "error": {
    "code": "AUTH_001",
    "message": "Invalid API key"
  }
}

# Solution - Create new API key
cost-ops auth create-api-key --name "New Key"
```

**FORECAST_001 - Insufficient Data**
```python
# Error
{
  "error": {
    "code": "FORECAST_001",
    "message": "Insufficient historical data for forecast (minimum 7 days required)"
  }
}

# Solution - Wait for more data or reduce forecast horizon
forecast = client.forecast.generate(
    organization_id="org-abc",
    horizon_days=7,  # Reduce horizon
    model="linear"
)
```

### Validation Errors

**Invalid Token Count:**
```python
# Error
{
  "error": {
    "code": "VALIDATION_ERROR",
    "field_errors": {
      "prompt_tokens": "must be greater than 0"
    }
  }
}

# Solution
usage = client.usage.create(
    prompt_tokens=1000,  # Must be > 0
    completion_tokens=500
)
```

**Invalid Date Format:**
```python
# Error
{
  "error": {
    "code": "VALIDATION_ERROR",
    "field_errors": {
      "start_date": "must be in ISO 8601 format (YYYY-MM-DD)"
    }
  }
}

# Solution
costs = client.costs.list(
    start_date="2025-01-01",  # ISO format
    end_date="2025-01-31"
)
```

**Invalid Provider:**
```python
# Error
{
  "error": {
    "code": "VALIDATION_ERROR",
    "field_errors": {
      "provider": "must be one of: openai, anthropic, google, azure, aws, cohere, mistral"
    }
  }
}

# Solution
usage = client.usage.create(
    provider="openai",  # Valid provider
    model="gpt-4-turbo"
)
```

---

## Configuration Patterns

### Environment-Based Configuration

**Development:**
```bash
export DATABASE_URL="sqlite:cost-ops.db"
export LOG_LEVEL="debug"
export METRICS_ENABLED="true"
export API_KEY_PREFIX="sk_test_"
```

**Staging:**
```bash
export DATABASE_URL="postgresql://user:pass@staging-db:5432/costops"
export LOG_LEVEL="info"
export METRICS_ENABLED="true"
export API_KEY_PREFIX="sk_staging_"
export RATE_LIMIT_REQUESTS=500
```

**Production:**
```bash
export DATABASE_URL="postgresql://user:pass@prod-db:5432/costops"
export LOG_LEVEL="warn"
export METRICS_ENABLED="true"
export API_KEY_PREFIX="sk_live_"
export RATE_LIMIT_REQUESTS=1000
export TLS_CERT_PATH="/etc/ssl/certs/server.crt"
export TLS_KEY_PATH="/etc/ssl/private/server.key"
```

### Configuration File Patterns

**Basic Configuration:**
```toml
[database]
url = "postgresql://localhost/costops"
pool_size = 20

[server]
host = "0.0.0.0"
port = 8080

[auth]
jwt_secret = "${JWT_SECRET}"
api_key_prefix = "sk_live_"
```

**Multi-Environment:**
```toml
# config.toml
[database]
url = "${DATABASE_URL}"
pool_size = "${DATABASE_POOL_SIZE:20}"

[server]
host = "${SERVER_HOST:0.0.0.0}"
port = "${SERVER_PORT:8080}"

[observability]
log_level = "${LOG_LEVEL:info}"
```

### Docker Configuration

**Docker Compose:**
```yaml
version: '3.8'
services:
  cost-ops:
    image: llm-cost-ops:latest
    environment:
      - DATABASE_URL=postgresql://postgres:5432/costops
      - JWT_SECRET=${JWT_SECRET}
      - LOG_LEVEL=info
    ports:
      - "8080:8080"
      - "9090:9090"
    volumes:
      - ./config.toml:/etc/cost-ops/config.toml
```

### Kubernetes Configuration

**ConfigMap:**
```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cost-ops-config
data:
  config.toml: |
    [database]
    url = "postgresql://postgres:5432/costops"
    pool_size = 20

    [server]
    host = "0.0.0.0"
    port = 8080
```

**Secret:**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: cost-ops-secret
type: Opaque
stringData:
  jwt-secret: "your-secret-key"
  database-password: "db-password"
  api-key: "sk_live_abc123"
```

---

## Best Practices Summary

### 1. Authentication

**Use API Keys for Services:**
```python
# Good - Service-to-service
client = CostOpsClient(api_key="sk_live_abc123")

# Good - User sessions
client = CostOpsClient(jwt_token="eyJhbGc...")

# Bad - Hardcoded credentials
client = CostOpsClient(api_key="sk_live_hardcoded_key")
```

**Rotate API Keys Regularly:**
```bash
# Create new key
new_key=$(cost-ops auth create-api-key --name "Rotation Key")

# Update application
export API_KEY=$new_key

# Revoke old key after verification
cost-ops auth revoke-api-key --id old_key_id
```

### 2. Error Handling

**Always Handle Errors:**
```python
# Good
try:
    usage = client.usage.create(...)
except ValidationError as e:
    logger.error(f"Validation failed: {e.field_errors}")
    # Handle gracefully
except APIError as e:
    logger.error(f"API error: {e.status_code}")
    # Retry or alert
```

**Use Retry Logic:**
```python
# Good - With retry
client = CostOpsClient(
    api_key="sk_live_abc123",
    max_retries=3,
    retry_backoff=2.0
)

# Even better - Custom retry logic
from tenacity import retry, stop_after_attempt, wait_exponential

@retry(
    stop=stop_after_attempt(3),
    wait=wait_exponential(multiplier=1, min=2, max=10)
)
def track_usage(data):
    return client.usage.create(data)
```

### 3. Data Validation

**Validate Before Sending:**
```python
# Good - Validate first
def validate_usage(data):
    assert data["prompt_tokens"] > 0, "Prompt tokens must be positive"
    assert data["completion_tokens"] > 0, "Completion tokens must be positive"
    assert data["provider"] in ["openai", "anthropic", ...], "Invalid provider"
    return True

if validate_usage(usage_data):
    client.usage.create(**usage_data)
```

**Use Type Hints:**
```python
# Good - Type hints
from typing import Dict, Optional

def create_usage(
    provider: str,
    model: str,
    prompt_tokens: int,
    completion_tokens: int,
    metadata: Optional[Dict] = None
) -> UsageRecord:
    return client.usage.create(
        provider=provider,
        model=model,
        prompt_tokens=prompt_tokens,
        completion_tokens=completion_tokens,
        metadata=metadata
    )
```

### 4. Performance

**Use Batch Operations:**
```python
# Bad - Individual requests
for usage in usages:
    client.usage.create(usage)

# Good - Batch request
client.usage.batch_create(usages)
```

**Cache Pricing Data:**
```python
# Good - Cache pricing
from functools import lru_cache

@lru_cache(maxsize=100)
def get_pricing(provider: str, model: str):
    return client.pricing.get(provider=provider, model=model)
```

**Use Pagination:**
```python
# Good - Paginated queries
page = 1
while True:
    response = client.costs.list(page=page, per_page=100)
    process_costs(response.data)
    if not response.has_more:
        break
    page += 1
```

### 5. Monitoring

**Track Metrics:**
```python
# Good - Track important metrics
import time

start = time.time()
usage = client.usage.create(...)
duration = time.time() - start

metrics.histogram("cost_ops.usage.create.duration", duration)
metrics.increment("cost_ops.usage.create.success")
```

**Log Important Events:**
```python
# Good - Structured logging
logger.info(
    "Usage tracked",
    extra={
        "provider": usage.provider,
        "model": usage.model,
        "tokens": usage.total_tokens,
        "cost": usage.calculated_cost,
        "organization_id": usage.organization_id
    }
)
```

### 6. Security

**Never Log Sensitive Data:**
```python
# Bad
logger.info(f"API key: {api_key}")

# Good
logger.info(f"API key: {api_key[:10]}...")
```

**Use Environment Variables:**
```python
# Good
import os

client = CostOpsClient(
    api_key=os.environ["COST_OPS_API_KEY"]
)
```

**Validate Input:**
```python
# Good - Sanitize user input
def sanitize_organization_id(org_id: str) -> str:
    if not org_id.startswith("org-"):
        raise ValueError("Invalid organization ID format")
    if len(org_id) > 50:
        raise ValueError("Organization ID too long")
    return org_id
```

---

## Security Checklist

### API Security

- [ ] Use HTTPS for all API calls
- [ ] Store API keys in environment variables or secrets manager
- [ ] Rotate API keys regularly (every 90 days minimum)
- [ ] Use least-privilege API keys (minimal scopes)
- [ ] Never commit API keys to version control
- [ ] Revoke compromised keys immediately
- [ ] Use different keys for dev/staging/production
- [ ] Monitor API key usage for anomalies

### Authentication

- [ ] Enable 2FA for user accounts
- [ ] Use strong passwords (min 12 characters)
- [ ] Implement session timeout (max 24 hours)
- [ ] Use JWT tokens with short expiration
- [ ] Implement refresh token rotation
- [ ] Log all authentication attempts
- [ ] Rate limit authentication endpoints
- [ ] Use RBAC for authorization

### Data Security

- [ ] Encrypt sensitive data at rest
- [ ] Use TLS 1.2+ for data in transit
- [ ] Sanitize all user input
- [ ] Validate all request parameters
- [ ] Implement proper error handling (don't leak sensitive info)
- [ ] Use parameterized queries (prevent SQL injection)
- [ ] Implement rate limiting
- [ ] Log security events to audit log

### Infrastructure

- [ ] Use private networks for database
- [ ] Enable firewall rules
- [ ] Keep dependencies up to date
- [ ] Scan for vulnerabilities regularly
- [ ] Use least-privilege service accounts
- [ ] Enable database encryption
- [ ] Implement backup encryption
- [ ] Use secrets management (Vault, AWS Secrets Manager)

### Compliance

- [ ] Implement audit logging
- [ ] Enable data retention policies
- [ ] Support data deletion requests (GDPR)
- [ ] Document data processing
- [ ] Implement access controls
- [ ] Regular security audits
- [ ] Incident response plan
- [ ] Security training for team

---

## Performance Tips

### 1. Database Optimization

**Use Connection Pooling:**
```toml
[database]
pool_size = 20
max_lifetime_seconds = 3600
idle_timeout_seconds = 600
```

**Add Indexes:**
```sql
CREATE INDEX idx_usage_org_date ON usage_records(organization_id, timestamp);
CREATE INDEX idx_cost_provider ON cost_records(provider, timestamp);
CREATE INDEX idx_usage_model ON usage_records(model_name, timestamp);
```

**Optimize Queries:**
```python
# Bad - N+1 queries
for usage in usages:
    cost = client.costs.get(usage_id=usage.id)

# Good - Batch query
usage_ids = [u.id for u in usages]
costs = client.costs.list(usage_ids=usage_ids)
```

### 2. API Optimization

**Use Batch Endpoints:**
```python
# Bad - 100 requests
for usage in usages:
    client.usage.create(usage)

# Good - 1 request
client.usage.batch_create(usages)
```

**Implement Caching:**
```python
from cachetools import TTLCache, cached

# Cache pricing for 1 hour
pricing_cache = TTLCache(maxsize=100, ttl=3600)

@cached(pricing_cache)
def get_pricing(provider: str, model: str):
    return client.pricing.get(provider=provider, model=model)
```

**Use Pagination:**
```python
# Good - Process in chunks
def process_all_costs():
    page = 1
    while True:
        costs = client.costs.list(page=page, per_page=100)
        for cost in costs.data:
            process(cost)
        if not costs.has_more:
            break
        page += 1
```

### 3. Rate Limiting

**Implement Backoff:**
```python
from time import sleep

def create_usage_with_backoff(data, max_retries=3):
    for attempt in range(max_retries):
        try:
            return client.usage.create(data)
        except RateLimitError:
            if attempt < max_retries - 1:
                sleep(2 ** attempt)  # Exponential backoff
            else:
                raise
```

**Use Request Throttling:**
```python
from ratelimit import limits, sleep_and_retry

# Max 100 requests per minute
@sleep_and_retry
@limits(calls=100, period=60)
def track_usage(data):
    return client.usage.create(data)
```

### 4. Memory Optimization

**Stream Large Results:**
```python
# Bad - Load all in memory
costs = client.costs.list(limit=1000000)

# Good - Stream results
for cost in client.costs.list_stream():
    process(cost)
```

**Use Generators:**
```python
# Good - Generator for large datasets
def process_costs(start_date, end_date):
    page = 1
    while True:
        costs = client.costs.list(
            start_date=start_date,
            end_date=end_date,
            page=page,
            per_page=100
        )
        for cost in costs.data:
            yield cost
        if not costs.has_more:
            break
        page += 1

# Process one at a time
for cost in process_costs("2025-01-01", "2025-01-31"):
    handle(cost)
```

### 5. Async Operations

**Use Async Clients:**
```python
# Good - Async for concurrent operations
async def track_multiple_usages(usages):
    async with AsyncCostOpsClient(api_key="sk_live_abc123") as client:
        tasks = [client.usage.create(usage) for usage in usages]
        results = await asyncio.gather(*tasks)
        return results
```

**Parallel Processing:**
```python
# Good - Process in parallel
from concurrent.futures import ThreadPoolExecutor

def track_usage(usage):
    return client.usage.create(usage)

with ThreadPoolExecutor(max_workers=10) as executor:
    results = list(executor.map(track_usage, usages))
```

### 6. Monitoring

**Track Performance Metrics:**
```python
import time
from prometheus_client import Histogram

request_duration = Histogram(
    'cost_ops_request_duration_seconds',
    'Request duration',
    ['method', 'endpoint']
)

with request_duration.labels('POST', '/usage').time():
    client.usage.create(...)
```

**Set Timeouts:**
```python
# Good - Set reasonable timeouts
client = CostOpsClient(
    api_key="sk_live_abc123",
    timeout=30.0  # 30 seconds
)
```

---

## Links to Detailed Docs

### Getting Started
- [Quick Start Guide](/docs/training/user-guides/getting-started.md)
- [Installation Guide](/docs/README.md)
- [Architecture Overview](/docs/SPECIFICATION.md)

### User Guides
- [Developer Guide](/docs/training/user-guides/developer-guide.md)
- [Administrator Guide](/docs/training/user-guides/administrator-guide.md)
- [Analyst Guide](/docs/training/user-guides/analyst-guide.md)

### SDK Documentation
- [Python SDK Tutorial](/docs/training/sdk-tutorials/python-sdk-tutorial.md)
- [TypeScript SDK Tutorial](/docs/training/sdk-tutorials/typescript-sdk-tutorial.md)
- [Go SDK Tutorial](/docs/training/sdk-tutorials/go-sdk-tutorial.md)
- [Rust SDK Tutorial](/docs/training/sdk-tutorials/rust-sdk-tutorial.md)

### API Reference
- [REST API Reference](/docs/training/reference/api-reference.md)
- [CLI Reference](/docs/training/reference/cli-reference.md)
- [Configuration Reference](/docs/training/reference/configuration.md)

### Best Practices
- [Cost Optimization](/docs/training/best-practices/cost-optimization.md)
- [Security Best Practices](/docs/training/best-practices/security.md)
- [Performance Optimization](/docs/training/best-practices/performance.md)
- [Architecture Patterns](/docs/training/best-practices/architecture-patterns.md)

### Labs & Tutorials
- [Lab 1: Basic Tracking](/docs/training/labs/lab-01-basic-tracking.md)
- [Lab 2: Analytics](/docs/training/labs/lab-02-analytics.md)
- [Lab 3: Budgets](/docs/training/labs/lab-03-budgets.md)
- [Lab 4: Optimization](/docs/training/labs/lab-04-optimization.md)
- [Lab 5: Enterprise](/docs/training/labs/lab-05-enterprise.md)

### Reference Materials
- [FAQ](/docs/training/reference/faq.md)
- [Troubleshooting](/docs/training/reference/troubleshooting.md)
- [Glossary](/docs/training/quick-reference/glossary.md)

### Compliance & Security
- [Compliance Overview](/docs/compliance/COMPLIANCE_OVERVIEW.md)
- [GDPR Compliance](/docs/compliance/GDPR_COMPLIANCE.md)
- [SOC2 Controls](/docs/compliance/SOC2_CONTROLS.md)
- [Audit Logging](/docs/compliance/AUDIT_LOGGING.md)

### Operations
- [Deployment Guide](/k8s/DEPLOYMENT.md)
- [CI/CD Guide](/docs/ci-cd/CI-CD-ARCHITECTURE.md)
- [DevOps Automation](/docs/ci-cd/DEVOPS_AUTOMATION_GUIDE.md)

---

**Last Updated:** 2025-11-16
**Version:** 1.0.0

For questions or issues, contact support@llm-cost-ops.com or visit https://docs.llm-cost-ops.com
