# Quickstart Guide

Get started with LLM-CostOps in under 5 minutes. This guide will walk you through tracking your first LLM usage and querying costs.

## Prerequisites

- An LLM-CostOps account (or self-hosted instance)
- An API key (see [Authentication Guide](authentication.md))
- cURL or your preferred HTTP client

## Step 1: Get Your API Key

### Cloud (Hosted Service)

1. Sign up at https://app.llm-cost-ops.dev
2. Navigate to Settings → API Keys
3. Click "Create API Key"
4. Copy and securely store your API key

### Self-Hosted

1. Generate an API key:
   ```bash
   cost-ops auth create-key --organization org-123 --name "My API Key"
   ```

2. Save the generated key securely

## Step 2: Test Your Connection

```bash
curl -X GET https://api.llm-cost-ops.dev/health \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Expected Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 86400,
  "components": [
    {
      "name": "api",
      "status": "healthy"
    },
    {
      "name": "database",
      "status": "healthy"
    }
  ]
}
```

## Step 3: Submit Your First Usage Record

Track usage from an OpenAI GPT-4 call:

```bash
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
    "timestamp": "2025-01-15T10:00:00Z",
    "metadata": {
      "request_id": "req-abc123",
      "user_id": "user-456"
    }
  }'
```

**Expected Response:**
```json
{
  "data": {
    "usage_id": "550e8400-e29b-41d4-a716-446655440001",
    "organization_id": "org-123",
    "estimated_cost": 0.025,
    "currency": "USD",
    "processed_at": "2025-01-15T10:00:01Z"
  }
}
```

## Step 4: Query Your Costs

Get a cost summary for the last 24 hours:

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123&start_date=2025-01-14T00:00:00Z&end_date=2025-01-15T23:59:59Z" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Expected Response:**
```json
{
  "data": {
    "total_cost": 0.025,
    "currency": "USD",
    "total_tokens": 1500,
    "total_requests": 1,
    "period_start": "2025-01-14T00:00:00Z",
    "period_end": "2025-01-15T23:59:59Z",
    "breakdown": [
      {
        "dimension": "provider",
        "value": "openai",
        "cost": 0.025,
        "tokens": 1500,
        "requests": 1
      }
    ]
  }
}
```

## Step 5: View Usage History

Get a paginated list of usage records:

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/usage/history?page=1&page_size=10" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Expected Response:**
```json
{
  "data": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440001",
      "organization_id": "org-123",
      "provider": "openai",
      "model_id": "gpt-4",
      "input_tokens": 1000,
      "output_tokens": 500,
      "total_tokens": 1500,
      "estimated_cost": 0.025,
      "timestamp": "2025-01-15T10:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total_items": 1,
    "total_pages": 1
  }
}
```

## Common Use Cases

### Track Anthropic Claude Usage

```bash
curl -X POST https://api.llm-cost-ops.dev/api/v1/usage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "organization_id": "org-123",
    "provider": "anthropic",
    "model_id": "claude-3-sonnet-20240229",
    "input_tokens": 2000,
    "output_tokens": 800,
    "total_tokens": 2800
  }'
```

### Track Usage with Cached Tokens

OpenAI's prompt caching reduces costs. Track it accurately:

```bash
curl -X POST https://api.llm-cost-ops.dev/api/v1/usage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "organization_id": "org-123",
    "provider": "openai",
    "model_id": "gpt-4-turbo",
    "input_tokens": 3000,
    "output_tokens": 1000,
    "total_tokens": 4000,
    "metadata": {
      "cached_tokens": 2000
    }
  }'
```

### Get Costs Grouped by Model

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123&group_by=model" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

### Get Analytics with Time Series

```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/analytics?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&interval=day" \
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
          "total_requests": 120
        }
      }
    ],
    "summary": {
      "total_cost": 465.00,
      "total_tokens": 7500000,
      "total_requests": 3600,
      "average_cost_per_request": 0.129,
      "average_tokens_per_request": 2083.33
    }
  }
}
```

## Environment Variables

For easier development, set these environment variables:

```bash
export LLM_COST_OPS_API_KEY="your-api-key"
export LLM_COST_OPS_BASE_URL="https://api.llm-cost-ops.dev"
export LLM_COST_OPS_ORG_ID="org-123"
```

Then use them in your requests:

```bash
curl -X POST $LLM_COST_OPS_BASE_URL/api/v1/usage \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $LLM_COST_OPS_API_KEY" \
  -d '{
    "organization_id": "'$LLM_COST_OPS_ORG_ID'",
    "provider": "openai",
    "model_id": "gpt-4",
    "input_tokens": 1000,
    "output_tokens": 500,
    "total_tokens": 1500
  }'
```

## Self-Hosted Setup

If you're running LLM-CostOps on-premises:

### 1. Deploy with Docker

```bash
docker run -d \
  -p 8080:8080 \
  -e DATABASE_URL=postgresql://user:pass@localhost/costops \
  -e JWT_SECRET=your-secret \
  llm-cost-ops/api:latest
```

### 2. Deploy with Kubernetes

```bash
# Using Helm
helm install llm-cost-ops ./k8s/helm/llm-cost-ops \
  --namespace llm-cost-ops \
  --create-namespace \
  --values ./values.yaml

# Verify deployment
kubectl get pods -n llm-cost-ops
```

### 3. Initialize Database

```bash
# Run migrations
cost-ops init --database-url postgresql://user:pass@localhost/costops

# Load default pricing
cost-ops pricing load-defaults
```

See [Self-Hosted Deployment Guide](../guides/self-hosted.md) for detailed instructions.

## Troubleshooting

### Error: "Unauthorized"

- Check that your API key is valid
- Ensure the `Authorization` header is formatted correctly: `Bearer YOUR_API_KEY`
- Verify the API key has not expired

### Error: "No pricing found for model"

Add pricing for your model:

```bash
curl -X POST https://api.llm-cost-ops.dev/api/v1/pricing \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "provider": "openai",
    "model_id": "gpt-4",
    "input_price_per_1k": 0.01,
    "output_price_per_1k": 0.03,
    "currency": "USD"
  }'
```

### Error: "Rate limit exceeded"

- Default rate limits: 100 requests/minute per API key
- Contact support to increase limits for your organization
- Implement exponential backoff in your client

## Next Steps

1. [Set Up Authentication](authentication.md) - Learn about API keys and JWT
2. [Submit Your First Usage](first-usage.md) - Detailed usage submission guide
3. [Query Costs](query-costs.md) - Advanced querying and filtering
4. [Cost Analysis Guide](../guides/cost-analysis.md) - In-depth cost analysis
5. [Budget Management](../guides/budget-management.md) - Set up budget alerts
6. [Forecasting Guide](../guides/forecasting.md) - Predict future costs

## SDK Examples

Once SDKs are available, you can use them instead of cURL:

- [Python Examples](../examples/python/)
- [TypeScript Examples](../examples/typescript/)
- [Go Examples](../examples/go/)
- [Java Examples](../examples/java/)

## Support

- **Documentation**: https://docs.llm-cost-ops.dev
- **GitHub Issues**: https://github.com/llm-devops/llm-cost-ops/issues
- **Discord**: https://discord.gg/llm-cost-ops
- **Email**: support@llm-cost-ops.dev
