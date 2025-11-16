---
sidebar_position: 2
title: Quick Start
---

# Quick Start Guide

Get up and running with LLM-CostOps in just a few minutes. This guide will walk you through the basics of tracking and analyzing LLM costs.

## Prerequisites

- LLM-CostOps server running (see [Installation](/docs/getting-started/installation))
- API key (see [Authentication](/docs/getting-started/authentication))
- Your preferred SDK installed

## 5-Minute Quickstart

### Step 1: Initialize the Client

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

<Tabs>
<TabItem value="python" label="Python">

```python
from llm_cost_ops import CostOpsClient

client = CostOpsClient(
    api_key="your-api-key",
    base_url="https://api.llm-cost-ops.example.com"
)
```

</TabItem>
<TabItem value="typescript" label="TypeScript">

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  baseUrl: 'https://api.llm-cost-ops.example.com',
  apiKey: 'your-api-key',
});
```

</TabItem>
<TabItem value="go" label="Go">

```go
import llmcostops "github.com/llm-devops/llm-cost-ops/sdk/go"

client, err := llmcostops.NewClient(
    llmcostops.WithAPIKey("your-api-key"),
    llmcostops.WithBaseURL("https://api.llm-cost-ops.example.com"),
)
if err != nil {
    log.Fatal(err)
}
defer client.Close()
```

</TabItem>
<TabItem value="rust" label="Rust">

```rust
use llm_cost_ops::{CostOpsClient, ClientConfig};

let config = ClientConfig::builder()
    .base_url("https://api.llm-cost-ops.example.com")?
    .api_key("your-api-key")
    .build()?;

let client = CostOpsClient::new(config)?;
```

</TabItem>
</Tabs>

### Step 2: Submit Usage Data

Track LLM usage from your applications:

<Tabs>
<TabItem value="python" label="Python">

```python
# Submit usage data
usage = client.usage.submit(
    organization_id="org-123",
    provider="openai",
    model_id="gpt-4",
    input_tokens=1000,
    output_tokens=500,
    total_tokens=1500,
    metadata={
        "user_id": "user-456",
        "feature": "chat",
        "environment": "production"
    }
)

print(f"Usage ID: {usage.usage_id}")
print(f"Estimated cost: ${usage.estimated_cost}")
```

</TabItem>
<TabItem value="typescript" label="TypeScript">

```typescript
// Submit usage data
const usage = await client.submitUsage({
  organizationId: 'org-123',
  provider: 'openai',
  modelId: 'gpt-4',
  inputTokens: 1000,
  outputTokens: 500,
  totalTokens: 1500,
  metadata: {
    userId: 'user-456',
    feature: 'chat',
    environment: 'production',
  },
});

console.log('Usage ID:', usage.usageId);
console.log('Estimated cost:', usage.estimatedCost);
```

</TabItem>
<TabItem value="go" label="Go">

```go
// Submit usage data
usage, err := client.Usage.Submit(ctx, &llmcostops.UsageSubmitParams{
    OrganizationID: "org-123",
    Provider:       "openai",
    ModelID:        "gpt-4",
    InputTokens:    1000,
    OutputTokens:   500,
    TotalTokens:    1500,
    Metadata: map[string]interface{}{
        "user_id":     "user-456",
        "feature":     "chat",
        "environment": "production",
    },
})
if err != nil {
    log.Fatal(err)
}

fmt.Printf("Usage ID: %s\n", usage.UsageID)
fmt.Printf("Estimated cost: $%.4f\n", usage.EstimatedCost)
```

</TabItem>
<TabItem value="rust" label="Rust">

```rust
// Submit usage data
let usage = client.submit_usage(UsageRequest {
    organization_id: "org-123".to_string(),
    provider: "openai".to_string(),
    model_id: "gpt-4".to_string(),
    input_tokens: 1000,
    output_tokens: 500,
    total_tokens: 1500,
    metadata: Some(serde_json::json!({
        "user_id": "user-456",
        "feature": "chat",
        "environment": "production"
    })),
    ..Default::default()
}).await?;

println!("Usage ID: {}", usage.usage_id);
println!("Estimated cost: ${}", usage.estimated_cost);
```

</TabItem>
</Tabs>

### Step 3: Query Costs

Retrieve cost data for analysis:

<Tabs>
<TabItem value="python" label="Python">

```python
from datetime import datetime, timedelta

# Get costs for the last 7 days
end_date = datetime.now()
start_date = end_date - timedelta(days=7)

costs = client.costs.get(
    organization_id="org-123",
    start_date=start_date.isoformat(),
    end_date=end_date.isoformat()
)

print(f"Total cost: ${costs.total_cost}")
print(f"Total requests: {costs.total_requests}")
print(f"Average cost per request: ${costs.average_cost_per_request}")

# Breakdown by provider
for provider, cost in costs.by_provider.items():
    print(f"{provider}: ${cost}")
```

</TabItem>
<TabItem value="typescript" label="TypeScript">

```typescript
// Get costs for the last 7 days
const endDate = new Date();
const startDate = new Date(endDate.getTime() - 7 * 24 * 60 * 60 * 1000);

const costs = await client.getCosts({
  organizationId: 'org-123',
  startDate: startDate.toISOString(),
  endDate: endDate.toISOString(),
});

console.log('Total cost:', costs.totalCost);
console.log('Total requests:', costs.totalRequests);
console.log('Average cost per request:', costs.averageCostPerRequest);

// Breakdown by provider
Object.entries(costs.byProvider).forEach(([provider, cost]) => {
  console.log(`${provider}: $${cost}`);
});
```

</TabItem>
<TabItem value="go" label="Go">

```go
import "time"

// Get costs for the last 7 days
endDate := time.Now()
startDate := endDate.AddDate(0, 0, -7)

costs, err := client.Costs.Get(ctx, &llmcostops.CostGetParams{
    OrganizationID: "org-123",
    StartDate:      startDate,
    EndDate:        endDate,
})
if err != nil {
    log.Fatal(err)
}

fmt.Printf("Total cost: $%.4f\n", costs.TotalCost)
fmt.Printf("Total requests: %d\n", costs.TotalRequests)
fmt.Printf("Average cost per request: $%.4f\n", costs.AverageCostPerRequest)

// Breakdown by provider
for provider, cost := range costs.ByProvider {
    fmt.Printf("%s: $%.4f\n", provider, cost)
}
```

</TabItem>
<TabItem value="rust" label="Rust">

```rust
use chrono::{Duration, Utc};

// Get costs for the last 7 days
let end_date = Utc::now();
let start_date = end_date - Duration::days(7);

let costs = client.get_costs(CostRequest {
    organization_id: "org-123".to_string(),
    start_date: start_date.to_rfc3339(),
    end_date: end_date.to_rfc3339(),
    ..Default::default()
}).await?;

println!("Total cost: ${}", costs.total_cost);
println!("Total requests: {}", costs.total_requests);
println!("Average cost per request: ${}", costs.average_cost_per_request);

// Breakdown by provider
for (provider, cost) in &costs.by_provider {
    println!("{}: ${}", provider, cost);
}
```

</TabItem>
</Tabs>

### Step 4: Get Forecasts

Predict future costs based on historical data:

<Tabs>
<TabItem value="python" label="Python">

```python
# Get 30-day forecast
forecast = client.forecasts.get(
    organization_id="org-123",
    forecast_days=30,
    model_type="linear_trend"
)

print(f"Predicted cost (30 days): ${forecast.predicted_cost}")
print(f"Confidence interval: ${forecast.lower_bound} - ${forecast.upper_bound}")
print(f"Trend: {forecast.trend_direction}")
```

</TabItem>
<TabItem value="typescript" label="TypeScript">

```typescript
// Get 30-day forecast
const forecast = await client.getForecast({
  organizationId: 'org-123',
  forecastDays: 30,
  modelType: 'linear_trend',
});

console.log('Predicted cost (30 days):', forecast.predictedCost);
console.log('Confidence interval:', `${forecast.lowerBound} - ${forecast.upperBound}`);
console.log('Trend:', forecast.trendDirection);
```

</TabItem>
<TabItem value="go" label="Go">

```go
// Get 30-day forecast
forecast, err := client.Forecasts.Get(ctx, &llmcostops.ForecastGetParams{
    OrganizationID: "org-123",
    ForecastDays:   30,
    ModelType:      "linear_trend",
})
if err != nil {
    log.Fatal(err)
}

fmt.Printf("Predicted cost (30 days): $%.4f\n", forecast.PredictedCost)
fmt.Printf("Confidence interval: $%.4f - $%.4f\n", forecast.LowerBound, forecast.UpperBound)
fmt.Printf("Trend: %s\n", forecast.TrendDirection)
```

</TabItem>
<TabItem value="rust" label="Rust">

```rust
// Get 30-day forecast
let forecast = client.get_forecast(ForecastRequest {
    organization_id: "org-123".to_string(),
    forecast_days: 30,
    model_type: Some("linear_trend".to_string()),
    ..Default::default()
}).await?;

println!("Predicted cost (30 days): ${}", forecast.predicted_cost);
println!("Confidence interval: ${} - ${}", forecast.lower_bound, forecast.upper_bound);
println!("Trend: {}", forecast.trend_direction);
```

</TabItem>
</Tabs>

## Common Workflows

### Tracking OpenAI Usage

<Tabs>
<TabItem value="python" label="Python">

```python
import openai
from llm_cost_ops import CostOpsClient

# Initialize clients
openai_client = openai.OpenAI(api_key="your-openai-key")
cost_client = CostOpsClient(api_key="your-cost-ops-key")

# Make OpenAI request
response = openai_client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello!"}]
)

# Track usage
cost_client.usage.submit(
    organization_id="org-123",
    provider="openai",
    model_id=response.model,
    input_tokens=response.usage.prompt_tokens,
    output_tokens=response.usage.completion_tokens,
    total_tokens=response.usage.total_tokens,
    metadata={"request_id": response.id}
)
```

</TabItem>
<TabItem value="typescript" label="TypeScript">

```typescript
import OpenAI from 'openai';
import { CostOpsClient } from '@llm-cost-ops/sdk';

// Initialize clients
const openaiClient = new OpenAI({ apiKey: 'your-openai-key' });
const costClient = new CostOpsClient({ apiKey: 'your-cost-ops-key' });

// Make OpenAI request
const response = await openaiClient.chat.completions.create({
  model: 'gpt-4',
  messages: [{ role: 'user', content: 'Hello!' }],
});

// Track usage
await costClient.submitUsage({
  organizationId: 'org-123',
  provider: 'openai',
  modelId: response.model,
  inputTokens: response.usage!.prompt_tokens,
  outputTokens: response.usage!.completion_tokens,
  totalTokens: response.usage!.total_tokens,
  metadata: { requestId: response.id },
});
```

</TabItem>
</Tabs>

### Setting Up Budgets

<Tabs>
<TabItem value="python" label="Python">

```python
# Create a monthly budget
budget = client.budgets.create(
    organization_id="org-123",
    name="Monthly AI Budget",
    amount=1000.00,
    period="monthly",
    alert_threshold=0.8,  # Alert at 80%
    notification_emails=["finance@example.com"]
)

# Check budget status
status = client.budgets.get_status(budget.budget_id)
print(f"Spent: ${status.spent} / ${status.limit}")
print(f"Remaining: ${status.remaining}")
print(f"Usage: {status.usage_percentage}%")
```

</TabItem>
<TabItem value="typescript" label="TypeScript">

```typescript
// Create a monthly budget
const budget = await client.createBudget({
  organizationId: 'org-123',
  name: 'Monthly AI Budget',
  amount: 1000.0,
  period: 'monthly',
  alertThreshold: 0.8, // Alert at 80%
  notificationEmails: ['finance@example.com'],
});

// Check budget status
const status = await client.getBudgetStatus(budget.budgetId);
console.log(`Spent: $${status.spent} / $${status.limit}`);
console.log(`Remaining: $${status.remaining}`);
console.log(`Usage: ${status.usagePercentage}%`);
```

</TabItem>
</Tabs>

### Generating Reports

<Tabs>
<TabItem value="python" label="Python">

```python
# Generate monthly cost report
report = client.reports.generate(
    organization_id="org-123",
    report_type="cost_summary",
    format="excel",
    start_date="2024-01-01",
    end_date="2024-01-31",
    group_by=["provider", "model", "project"]
)

# Download report
with open("cost_report.xlsx", "wb") as f:
    f.write(report.data)

print(f"Report generated: {report.file_name}")
```

</TabItem>
<TabItem value="typescript" label="TypeScript">

```typescript
import fs from 'fs';

// Generate monthly cost report
const report = await client.generateReport({
  organizationId: 'org-123',
  reportType: 'cost_summary',
  format: 'excel',
  startDate: '2024-01-01',
  endDate: '2024-01-31',
  groupBy: ['provider', 'model', 'project'],
});

// Download report
fs.writeFileSync('cost_report.xlsx', report.data);
console.log('Report generated:', report.fileName);
```

</TabItem>
</Tabs>

## Best Practices

### 1. Tag Your Usage

Always add meaningful tags to track costs by feature, environment, or team:

```python
client.usage.submit(
    organization_id="org-123",
    provider="openai",
    model_id="gpt-4",
    input_tokens=1000,
    output_tokens=500,
    total_tokens=1500,
    tags=["production", "chat", "customer-support"],
    metadata={
        "team": "support",
        "feature": "ai-chat",
        "version": "v2"
    }
)
```

### 2. Use Async for High-Throughput

For applications with high request volumes, use async SDKs:

```python
from llm_cost_ops import AsyncCostOpsClient
import asyncio

async def track_usage():
    async with AsyncCostOpsClient(api_key="your-key") as client:
        await client.usage.submit(...)
```

### 3. Batch Operations

When tracking multiple usage records, use batch operations:

```python
# Submit multiple records at once
client.usage.submit_batch([
    {
        "organization_id": "org-123",
        "provider": "openai",
        "model_id": "gpt-4",
        "input_tokens": 1000,
        "output_tokens": 500,
    },
    {
        "organization_id": "org-123",
        "provider": "anthropic",
        "model_id": "claude-3-sonnet",
        "input_tokens": 2000,
        "output_tokens": 800,
    }
])
```

### 4. Set Up Alerts

Configure alerts to stay informed about cost anomalies:

```python
# Create anomaly alert
client.alerts.create(
    organization_id="org-123",
    alert_type="cost_anomaly",
    threshold=2.0,  # 2x normal spending
    notification_channels=["email", "slack"]
)
```

## Next Steps

Now that you've got the basics down:

1. [Explore SDK-specific guides](/docs/sdks/)
   - [Python SDK](/docs/sdks/python/)
   - [TypeScript SDK](/docs/sdks/typescript/)
   - [Go SDK](/docs/sdks/go/)
   - [Rust SDK](/docs/sdks/rust/)

2. [Learn about cost tracking](/docs/guides/cost-tracking)

3. [Set up forecasting](/docs/guides/forecasting)

4. [Configure analytics](/docs/guides/analytics)

5. [Deploy to production](/docs/deployment/)

## Getting Help

- Check the [FAQ](/docs/faq)
- Read the [API Reference](/docs/api/)
- Join our [Discord community](https://discord.gg/llm-cost-ops)
- [Open an issue](https://github.com/llm-devops/llm-cost-ops/issues)
