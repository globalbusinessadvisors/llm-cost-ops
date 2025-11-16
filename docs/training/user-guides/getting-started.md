# Getting Started with LLM Cost Ops

Welcome to LLM Cost Ops! This guide will help you get up and running in minutes.

## Table of Contents

1. [What is LLM Cost Ops?](#what-is-llm-cost-ops)
2. [Prerequisites](#prerequisites)
3. [Installation](#installation)
4. [Quick Start](#quick-start)
5. [Your First Cost Tracking](#your-first-cost-tracking)
6. [Next Steps](#next-steps)

## What is LLM Cost Ops?

LLM Cost Ops is an enterprise-grade platform for tracking, analyzing, and optimizing costs across your Large Language Model infrastructure. Whether you're using OpenAI, Anthropic, Google, or self-hosted models, LLM Cost Ops provides unified cost visibility and control.

### Key Features

- **Real-time Cost Tracking**: Monitor LLM costs as they happen
- **Multi-Provider Support**: OpenAI, Anthropic, Google, Azure, AWS Bedrock, and more
- **Budget Management**: Set budgets and receive alerts before overruns
- **Cost Analytics**: Detailed insights and forecasting
- **Team Management**: Multi-tenant support with role-based access
- **Compliance**: GDPR, SOC 2, HIPAA compliant
- **SDKs**: Python, TypeScript, Go, and Rust

## Prerequisites

Before you begin, ensure you have:

- **An LLM Cost Ops Account**: Sign up at https://app.llm-cost-ops.dev
- **API Key**: Generate from your account dashboard
- **Programming Environment**: One of:
  - Python 3.9+ (for Python SDK)
  - Node.js 18+ (for TypeScript/JavaScript SDK)
  - Go 1.21+ (for Go SDK)
  - Rust 1.70+ (for Rust SDK)

## Installation

Choose your preferred SDK:

### Python

```bash
pip install llm-cost-ops
```

### TypeScript/JavaScript

```bash
npm install @llm-cost-ops/sdk
# or
yarn add @llm-cost-ops/sdk
# or
pnpm add @llm-cost-ops/sdk
```

### Go

```bash
go get github.com/your-org/llm-cost-ops-go
```

### Rust

```toml
# Add to Cargo.toml
[dependencies]
llm-cost-ops = "0.1.0"
```

## Quick Start

### Step 1: Set Up Authentication

Create a `.env` file in your project root:

```bash
COST_OPS_API_KEY=your_api_key_here
COST_OPS_BASE_URL=https://api.llm-cost-ops.dev
```

### Step 2: Initialize the Client

**Python:**
```python
from llm_cost_ops import CostOpsClient

client = CostOpsClient()
# Client automatically reads from environment variables
```

**TypeScript:**
```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  apiKey: process.env.COST_OPS_API_KEY,
  baseUrl: process.env.COST_OPS_BASE_URL,
});
```

**Go:**
```go
import "github.com/your-org/llm-cost-ops-go"

client, err := costops.NewClient(
    costops.WithAPIKey(os.Getenv("COST_OPS_API_KEY")),
    costops.WithBaseURL(os.Getenv("COST_OPS_BASE_URL")),
)
if err != nil {
    log.Fatal(err)
}
defer client.Close()
```

**Rust:**
```rust
use llm_cost_ops::Client;

let client = Client::builder()
    .api_key(std::env::var("COST_OPS_API_KEY")?)
    .base_url(std::env::var("COST_OPS_BASE_URL")?)
    .build()?;
```

### Step 3: Verify Connection

**Python:**
```python
# Check API health
health = client.health.check()
print(f"API Status: {health.status}")  # Should print "healthy"
```

**TypeScript:**
```typescript
// Check API health
const health = await client.health.check();
console.log(`API Status: ${health.status}`);  // Should print "healthy"
```

**Go:**
```go
// Check API health
health, err := client.Health.Check(context.Background())
if err != nil {
    log.Fatal(err)
}
fmt.Printf("API Status: %s\n", health.Status)
```

**Rust:**
```rust
// Check API health
let health = client.health().check().await?;
println!("API Status: {}", health.status);
```

## Your First Cost Tracking

Let's track costs from an OpenAI API call:

### Python Example

```python
from llm_cost_ops import CostOpsClient
from llm_cost_ops.types import UsageRecord
import openai

# Initialize clients
cost_ops = CostOpsClient()
openai_client = openai.Client()

# Make an LLM call
response = openai_client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello, world!"}],
)

# Track the usage
usage_record = UsageRecord(
    provider="openai",
    model="gpt-4",
    operation="chat.completions",
    input_tokens=response.usage.prompt_tokens,
    output_tokens=response.usage.completion_tokens,
    total_tokens=response.usage.total_tokens,
    metadata={
        "request_id": response.id,
        "user": "demo_user",
        "environment": "production",
    }
)

# Record to Cost Ops
result = cost_ops.usage.record(usage_record)
print(f"Cost tracked: ${result.cost:.4f}")
print(f"Usage ID: {result.id}")
```

### TypeScript Example

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';
import OpenAI from 'openai';

// Initialize clients
const costOps = new CostOpsClient();
const openai = new OpenAI();

// Make an LLM call
const response = await openai.chat.completions.create({
  model: 'gpt-4',
  messages: [{ role: 'user', content: 'Hello, world!' }],
});

// Track the usage
const usageRecord = {
  provider: 'openai',
  model: 'gpt-4',
  operation: 'chat.completions',
  inputTokens: response.usage?.prompt_tokens || 0,
  outputTokens: response.usage?.completion_tokens || 0,
  totalTokens: response.usage?.total_tokens || 0,
  metadata: {
    requestId: response.id,
    user: 'demo_user',
    environment: 'production',
  },
};

// Record to Cost Ops
const result = await costOps.usage.record(usageRecord);
console.log(`Cost tracked: $${result.cost.toFixed(4)}`);
console.log(`Usage ID: ${result.id}`);
```

### Go Example

```go
package main

import (
    "context"
    "fmt"
    "log"

    costops "github.com/your-org/llm-cost-ops-go"
    "github.com/sashabaranov/go-openai"
)

func main() {
    ctx := context.Background()

    // Initialize clients
    costOpsClient, _ := costops.NewClient()
    openaiClient := openai.NewClient(os.Getenv("OPENAI_API_KEY"))

    // Make an LLM call
    resp, err := openaiClient.CreateChatCompletion(ctx,
        openai.ChatCompletionRequest{
            Model: "gpt-4",
            Messages: []openai.ChatCompletionMessage{
                {Role: "user", Content: "Hello, world!"},
            },
        },
    )
    if err != nil {
        log.Fatal(err)
    }

    // Track the usage
    usage := &costops.UsageRecord{
        Provider:     "openai",
        Model:        "gpt-4",
        Operation:    "chat.completions",
        InputTokens:  resp.Usage.PromptTokens,
        OutputTokens: resp.Usage.CompletionTokens,
        TotalTokens:  resp.Usage.TotalTokens,
        Metadata: map[string]interface{}{
            "request_id":  resp.ID,
            "user":        "demo_user",
            "environment": "production",
        },
    }

    // Record to Cost Ops
    result, err := costOpsClient.Usage.Record(ctx, usage)
    if err != nil {
        log.Fatal(err)
    }

    fmt.Printf("Cost tracked: $%.4f\n", result.Cost)
    fmt.Printf("Usage ID: %s\n", result.ID)
}
```

### Rust Example

```rust
use llm_cost_ops::{Client, UsageRecord};
use async_openai::{Client as OpenAIClient, types::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize clients
    let cost_ops = Client::from_env()?;
    let openai = OpenAIClient::new();

    // Make an LLM call
    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4")
        .messages(vec![ChatCompletionRequestMessageArgs::default()
            .role(Role::User)
            .content("Hello, world!")
            .build()?])
        .build()?;

    let response = openai.chat().create(request).await?;

    // Track the usage
    let usage = UsageRecord {
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        operation: "chat.completions".to_string(),
        input_tokens: response.usage.prompt_tokens,
        output_tokens: response.usage.completion_tokens,
        total_tokens: response.usage.total_tokens,
        metadata: serde_json::json!({
            "request_id": response.id,
            "user": "demo_user",
            "environment": "production",
        }),
        ..Default::default()
    };

    // Record to Cost Ops
    let result = cost_ops.usage().record(&usage).await?;

    println!("Cost tracked: ${:.4}", result.cost);
    println!("Usage ID: {}", result.id);

    Ok(())
}
```

## View Your Costs

After tracking some usage, view your costs:

### Python

```python
from datetime import datetime, timedelta

# Get costs for the last 24 hours
end_time = datetime.now()
start_time = end_time - timedelta(days=1)

costs = cost_ops.costs.get(
    start_time=start_time,
    end_time=end_time,
    group_by=["provider", "model"]
)

for cost in costs:
    print(f"{cost.provider}/{cost.model}: ${cost.total_cost:.2f}")
```

### TypeScript

```typescript
// Get costs for the last 24 hours
const endTime = new Date();
const startTime = new Date(endTime.getTime() - 24 * 60 * 60 * 1000);

const costs = await costOps.costs.get({
  startTime,
  endTime,
  groupBy: ['provider', 'model'],
});

costs.forEach(cost => {
  console.log(`${cost.provider}/${cost.model}: $${cost.totalCost.toFixed(2)}`);
});
```

### Go

```go
// Get costs for the last 24 hours
endTime := time.Now()
startTime := endTime.Add(-24 * time.Hour)

costs, err := client.Costs.Get(ctx, &costops.CostQuery{
    StartTime: startTime,
    EndTime:   endTime,
    GroupBy:   []string{"provider", "model"},
})
if err != nil {
    log.Fatal(err)
}

for _, cost := range costs {
    fmt.Printf("%s/%s: $%.2f\n", cost.Provider, cost.Model, cost.TotalCost)
}
```

### Rust

```rust
use chrono::{Utc, Duration};

// Get costs for the last 24 hours
let end_time = Utc::now();
let start_time = end_time - Duration::days(1);

let costs = client.costs()
    .get()
    .start_time(start_time)
    .end_time(end_time)
    .group_by(&["provider", "model"])
    .send()
    .await?;

for cost in costs {
    println!("{}/{}: ${:.2}", cost.provider, cost.model, cost.total_cost);
}
```

## Next Steps

Congratulations! You've successfully set up LLM Cost Ops and tracked your first costs. Here's what to do next:

### Learn More

1. **Developer Guide**: Deep dive into SDK features
   - [Python SDK Tutorial](../sdk-tutorials/python-sdk-tutorial.md)
   - [TypeScript SDK Tutorial](../sdk-tutorials/typescript-sdk-tutorial.md)
   - [Go SDK Tutorial](../sdk-tutorials/go-sdk-tutorial.md)
   - [Rust SDK Tutorial](../sdk-tutorials/rust-sdk-tutorial.md)

2. **Hands-On Labs**: Practice with real scenarios
   - [Lab 1: Basic Cost Tracking](../labs/lab-01-basic-tracking.md)
   - [Lab 2: Analytics and Reporting](../labs/lab-02-analytics.md)
   - [Lab 3: Budget Management](../labs/lab-03-budgets.md)

3. **Best Practices**: Optimize your implementation
   - [Cost Optimization Strategies](../best-practices/cost-optimization.md)
   - [Performance Tuning](../best-practices/performance.md)

### Explore Advanced Features

- **Budgets and Alerts**: Set up cost controls
- **Analytics Dashboards**: Build custom reports
- **Team Management**: Add team members and set permissions
- **Compliance**: Configure GDPR and audit logging
- **Exports**: Schedule automated cost reports

### Get Help

- **Documentation**: https://docs.llm-cost-ops.dev
- **Community Forum**: https://community.llm-cost-ops.dev
- **Support Email**: support@llm-cost-ops.dev
- **GitHub Issues**: https://github.com/your-org/llm-cost-ops/issues

## Common Issues

### Authentication Errors

If you see `401 Unauthorized`:
- Verify your API key is correct
- Check that the API key hasn't expired
- Ensure environment variables are loaded correctly

### Connection Errors

If you can't connect to the API:
- Verify your internet connection
- Check the base URL is correct
- Ensure firewall allows HTTPS traffic

### Import Errors

If imports fail:
- Verify the package is installed: `pip list | grep llm-cost-ops`
- Check you're using the correct Python environment
- Try reinstalling: `pip install --force-reinstall llm-cost-ops`

For more troubleshooting, see the [Troubleshooting Guide](../reference/troubleshooting.md).
