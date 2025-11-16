# Cost Analysis Guide

Learn how to analyze your LLM costs across providers, models, projects, and time periods using LLM-CostOps.

## Overview

Cost analysis helps you:
- Understand spending patterns across providers and models
- Identify cost optimization opportunities
- Track costs by project, team, or customer
- Compare costs across time periods
- Generate reports for stakeholders

## Basic Cost Queries

### Get Total Costs

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Response:**
```json
{
  "data": {
    "total_cost": 465.75,
    "currency": "USD",
    "total_tokens": 7500000,
    "total_requests": 3600,
    "period_start": "2025-01-01T00:00:00Z",
    "period_end": "2025-01-31T23:59:59Z"
  }
}
```

### Group by Provider

Compare costs across different LLM providers:

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&group_by=provider" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Response:**
```json
{
  "data": {
    "total_cost": 465.75,
    "currency": "USD",
    "total_tokens": 7500000,
    "total_requests": 3600,
    "period_start": "2025-01-01T00:00:00Z",
    "period_end": "2025-01-31T23:59:59Z",
    "breakdown": [
      {
        "dimension": "provider",
        "value": "openai",
        "cost": 320.50,
        "tokens": 5200000,
        "requests": 2400,
        "percentage": 68.8
      },
      {
        "dimension": "provider",
        "value": "anthropic",
        "cost": 145.25,
        "tokens": 2300000,
        "requests": 1200,
        "percentage": 31.2
      }
    ]
  }
}
```

**Insights:**
- OpenAI accounts for 69% of total costs
- Anthropic is more cost-effective per token in this case
- Consider shifting some workloads to Anthropic

### Group by Model

Identify which models are most expensive:

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&group_by=model" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Response:**
```json
{
  "data": {
    "breakdown": [
      {
        "dimension": "model",
        "value": "gpt-4",
        "cost": 280.00,
        "tokens": 2800000,
        "requests": 1500,
        "average_cost_per_request": 0.187
      },
      {
        "dimension": "model",
        "value": "claude-3-opus",
        "cost": 120.50,
        "tokens": 1600000,
        "requests": 800,
        "average_cost_per_request": 0.151
      },
      {
        "dimension": "model",
        "value": "gpt-3.5-turbo",
        "cost": 40.50,
        "tokens": 2700000,
        "requests": 900,
        "average_cost_per_request": 0.045
      }
    ]
  }
}
```

**Insights:**
- GPT-4 is the most expensive model at $280
- GPT-3.5-Turbo is 4x cheaper per request
- Consider using GPT-3.5-Turbo for simpler tasks

### Time-Based Analysis

Analyze costs by day, week, or month:

```bash
# Daily breakdown
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&group_by=day" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Response:**
```json
{
  "data": {
    "breakdown": [
      {
        "dimension": "day",
        "value": "2025-01-15",
        "cost": 15.50,
        "tokens": 250000,
        "requests": 120
      },
      {
        "dimension": "day",
        "value": "2025-01-16",
        "cost": 18.75,
        "tokens": 310000,
        "requests": 145
      }
    ]
  }
}
```

## Advanced Analytics

### Time-Series Analysis

Get detailed time-series data for trend analysis:

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/analytics?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&interval=day&metrics=total_cost,total_tokens,total_requests" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Response:**
```json
{
  "data": {
    "time_series": [
      {
        "timestamp": "2025-01-15T00:00:00Z",
        "metrics": {
          "total_cost": 15.50,
          "total_tokens": 250000,
          "total_requests": 120,
          "average_cost_per_request": 0.129,
          "average_tokens_per_request": 2083.33
        }
      }
    ],
    "summary": {
      "total_cost": 465.75,
      "total_tokens": 7500000,
      "total_requests": 3600,
      "average_cost_per_request": 0.129,
      "average_tokens_per_request": 2083.33
    }
  }
}
```

### Multi-Dimensional Analysis

Combine multiple dimensions:

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/analytics?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&group_by=provider,model&interval=day" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

## Cost Optimization Strategies

### 1. Identify High-Cost Endpoints

Find which API endpoints are most expensive:

```bash
# Add metadata.endpoint to usage submissions
curl -X POST https://api.llm-cost-ops.dev/api/v1/usage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "organization_id": "org-123",
    "provider": "openai",
    "model_id": "gpt-4",
    "input_tokens": 1000,
    "output_tokens": 500,
    "total_tokens": 1500,
    "metadata": {
      "endpoint": "/api/chat/completion",
      "feature": "customer-support"
    }
  }'
```

Then query by metadata:

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/usage/history?organization_id=org-123&metadata.endpoint=/api/chat/completion" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

### 2. Compare Provider Costs

**Analysis Query:**
```sql
-- Example SQL query on exported data
SELECT
  provider,
  model,
  SUM(total_cost) as total_cost,
  SUM(total_tokens) as total_tokens,
  SUM(total_cost) / SUM(total_tokens) * 1000 as cost_per_1k_tokens,
  COUNT(*) as request_count
FROM usage_records
GROUP BY provider, model
ORDER BY total_cost DESC;
```

**Results:**
| Provider | Model | Total Cost | Cost/1K Tokens | Requests |
|----------|-------|------------|----------------|----------|
| OpenAI | gpt-4 | $280.00 | $0.100 | 1500 |
| Anthropic | claude-3-opus | $120.50 | $0.075 | 800 |
| OpenAI | gpt-3.5-turbo | $40.50 | $0.015 | 900 |

**Optimization:**
- Move non-critical workloads from GPT-4 to GPT-3.5-Turbo
- Consider Claude-3-Sonnet for medium complexity tasks
- Use GPT-3.5-Turbo for simple queries

### 3. Optimize by Use Case

Segment costs by use case:

```bash
# Tag usage by use case
curl -X POST https://api.llm-cost-ops.dev/api/v1/usage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "organization_id": "org-123",
    "provider": "openai",
    "model_id": "gpt-4",
    "input_tokens": 1000,
    "output_tokens": 500,
    "total_tokens": 1500,
    "metadata": {
      "use_case": "code-review",
      "priority": "high"
    }
  }'
```

**Use Case Cost Breakdown:**
| Use Case | Model | Monthly Cost | Optimization |
|----------|-------|--------------|--------------|
| Code Review | GPT-4 | $120.00 | Keep on GPT-4 (high value) |
| Documentation | GPT-4 | $80.00 | **Switch to GPT-3.5** (-75%) |
| Chat Support | Claude-3 | $60.00 | Optimized |
| Email Drafting | GPT-3.5 | $15.00 | Optimized |

**Potential Savings:** $60/month by switching documentation to GPT-3.5

## Cost Reports

### Daily Cost Report

Generate a daily cost summary:

```bash
#!/bin/bash
# daily-cost-report.sh

YESTERDAY=$(date -d "yesterday" +%Y-%m-%d)
TODAY=$(date +%Y-%m-%d)

curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123&start_date=${YESTERDAY}T00:00:00Z&end_date=${YESTERDAY}T23:59:59Z&group_by=provider" \
  -H "Authorization: Bearer $LLM_COST_OPS_API_KEY" \
  | jq '.data'
```

**Output:**
```json
{
  "total_cost": 15.50,
  "currency": "USD",
  "total_tokens": 250000,
  "total_requests": 120,
  "breakdown": [
    {
      "dimension": "provider",
      "value": "openai",
      "cost": 10.75
    },
    {
      "dimension": "provider",
      "value": "anthropic",
      "cost": 4.75
    }
  ]
}
```

### Monthly Executive Summary

```bash
#!/bin/bash
# monthly-executive-summary.sh

MONTH_START=$(date -d "first day of this month" +%Y-%m-%d)
TODAY=$(date +%Y-%m-%d)

echo "LLM Cost Summary - $(date +%B %Y)"
echo "=================================="

# Total costs
TOTAL=$(curl -s -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123&start_date=${MONTH_START}T00:00:00Z&end_date=${TODAY}T23:59:59Z" \
  -H "Authorization: Bearer $LLM_COST_OPS_API_KEY" \
  | jq -r '.data.total_cost')

echo "Total Spend: \$$TOTAL"

# Provider breakdown
echo ""
echo "By Provider:"
curl -s -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123&start_date=${MONTH_START}T00:00:00Z&end_date=${TODAY}T23:59:59Z&group_by=provider" \
  -H "Authorization: Bearer $LLM_COST_OPS_API_KEY" \
  | jq -r '.data.breakdown[] | "  \(.value): $\(.cost)"'
```

**Output:**
```
LLM Cost Summary - January 2025
==================================
Total Spend: $465.75

By Provider:
  openai: $320.50
  anthropic: $145.25
```

### Export for BI Tools

Export cost data to CSV for analysis in Excel, Tableau, or Power BI:

```bash
# Export to CSV
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs/export?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&format=csv" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -o costs-january-2025.csv

# Export to Excel
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs/export?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&format=xlsx" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -o costs-january-2025.xlsx
```

## Real-Time Monitoring

### Set Up Cost Alerts

Monitor costs in real-time and get alerts:

```bash
# Create a budget with alerts
curl -X POST https://api.llm-cost-ops.dev/api/v1/budgets \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "organization_id": "org-123",
    "name": "Monthly LLM Budget",
    "amount": 500.00,
    "currency": "USD",
    "period": "monthly",
    "alerts": [
      {
        "threshold_percentage": 50,
        "channels": ["email", "slack"]
      },
      {
        "threshold_percentage": 80,
        "channels": ["email", "slack", "pagerduty"]
      },
      {
        "threshold_percentage": 100,
        "channels": ["email", "slack", "pagerduty"]
      }
    ]
  }'
```

### Dashboard Metrics

Key metrics to track on your dashboard:

1. **Current Month Spend**: Total cost so far this month
2. **Daily Average**: Average cost per day
3. **Projected Month-End**: Forecast based on current trends
4. **Budget Utilization**: Percentage of budget used
5. **Top Cost Drivers**: Top 5 models or providers by cost
6. **Cost per Request**: Average cost per API request
7. **Cost per User**: Total cost divided by active users

## Best Practices

### 1. Tag Everything

Add meaningful metadata to all usage records:

```json
{
  "organization_id": "org-123",
  "provider": "openai",
  "model_id": "gpt-4",
  "input_tokens": 1000,
  "output_tokens": 500,
  "total_tokens": 1500,
  "metadata": {
    "feature": "customer-support",
    "endpoint": "/api/chat",
    "user_id": "user-456",
    "session_id": "sess-789",
    "environment": "production",
    "team": "support",
    "priority": "high"
  }
}
```

### 2. Regular Reviews

Schedule regular cost reviews:
- **Daily**: Quick check of yesterday's costs
- **Weekly**: Detailed analysis of cost trends
- **Monthly**: Executive summary and optimization planning

### 3. Set Budgets

Always set budgets with alerts:
- **Overall budget**: Total monthly LLM spend limit
- **Per-team budgets**: Limits for each team or project
- **Per-feature budgets**: Limits for specific features

### 4. Compare Periods

Always compare current costs to previous periods:

```bash
# This month vs last month
curl -X GET "https://api.llm-cost-ops.dev/api/v1/analytics/compare?organization_id=org-123&current_start=2025-01-01&current_end=2025-01-31&previous_start=2024-12-01&previous_end=2024-12-31" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

### 5. Optimize Continuously

- **A/B Test Models**: Compare costs and quality for different models
- **Implement Caching**: Reduce costs by caching common responses
- **Use Cheaper Models**: Use GPT-3.5 instead of GPT-4 where possible
- **Optimize Prompts**: Shorter prompts = lower costs
- **Batch Requests**: Combine multiple requests when possible

## Code Examples

### Python - Daily Cost Alert

```python
from llm_cost_ops import CostOpsClient
from datetime import datetime, timedelta
import os

client = CostOpsClient(api_key=os.getenv("LLM_COST_OPS_API_KEY"))

# Get yesterday's costs
yesterday = datetime.now() - timedelta(days=1)
costs = client.costs.get(
    organization_id="org-123",
    start_date=yesterday.replace(hour=0, minute=0, second=0),
    end_date=yesterday.replace(hour=23, minute=59, second=59),
    group_by="provider"
)

# Alert if costs exceed threshold
DAILY_THRESHOLD = 20.00
if costs.total_cost > DAILY_THRESHOLD:
    print(f"⚠️  Daily cost alert: ${costs.total_cost} exceeds ${DAILY_THRESHOLD}")
    # Send alert to Slack, email, etc.
else:
    print(f"✅ Daily cost: ${costs.total_cost} (under threshold)")

# Print breakdown
for item in costs.breakdown:
    print(f"  {item.value}: ${item.cost}")
```

### TypeScript - Real-Time Cost Tracking

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  apiKey: process.env.LLM_COST_OPS_API_KEY
});

// Track costs in real-time
async function trackLLMUsage(
  provider: string,
  modelId: string,
  inputTokens: number,
  outputTokens: number
) {
  const usage = await client.usage.submit({
    organizationId: 'org-123',
    provider,
    modelId,
    inputTokens,
    outputTokens,
    totalTokens: inputTokens + outputTokens,
    metadata: {
      feature: 'chat',
      timestamp: new Date().toISOString()
    }
  });

  console.log(`Cost: $${usage.estimatedCost} (${modelId})`);

  // Check if approaching daily limit
  const today = new Date();
  today.setHours(0, 0, 0, 0);

  const todayCosts = await client.costs.get({
    organizationId: 'org-123',
    startDate: today,
    endDate: new Date()
  });

  const DAILY_LIMIT = 50.00;
  if (todayCosts.totalCost > DAILY_LIMIT * 0.9) {
    console.warn(`⚠️  Approaching daily limit: $${todayCosts.totalCost}/$${DAILY_LIMIT}`);
  }
}
```

## Next Steps

- [Budget Management Guide](budget-management.md)
- [Forecasting Guide](forecasting.md)
- [Export & Reporting Guide](export-reports.md)
- [Anomaly Detection](anomaly-detection.md)
