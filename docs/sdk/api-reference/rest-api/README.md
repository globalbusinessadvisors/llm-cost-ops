# REST API Reference

Complete reference for the LLM-CostOps REST API. All endpoints are accessible at `https://api.llm-cost-ops.dev` (cloud) or your self-hosted URL.

## Base URL

**Cloud:** `https://api.llm-cost-ops.dev`
**Self-Hosted:** `https://your-domain.com` or `http://localhost:8080`

## API Version

Current version: **v1**

All endpoints are prefixed with `/api/v1/`

## Authentication

All API requests require authentication. See [Authentication Guide](../../getting-started/authentication.md) for details.

```bash
Authorization: Bearer YOUR_API_KEY
```

## Content Type

All requests and responses use JSON:

```bash
Content-Type: application/json
```

## Rate Limiting

Default rate limits per API key:
- **100 requests/minute** for standard keys
- **1000 requests/minute** for premium keys

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1705320060
```

## Pagination

Endpoints that return lists support pagination:

**Request:**
```bash
GET /api/v1/usage/history?page=1&page_size=50
```

**Response:**
```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "page_size": 50,
    "total_items": 1250,
    "total_pages": 25,
    "has_next": true,
    "has_previous": false
  }
}
```

**Parameters:**
- `page` - Page number (default: 1)
- `page_size` - Items per page (default: 20, max: 100)

## Error Handling

### Error Response Format

```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "organization_id is required",
    "details": {
      "field": "organization_id",
      "reason": "missing_required_field"
    }
  }
}
```

### HTTP Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 400 | Bad Request - Invalid parameters |
| 401 | Unauthorized - Invalid or missing API key |
| 403 | Forbidden - Insufficient permissions |
| 404 | Not Found - Resource doesn't exist |
| 429 | Too Many Requests - Rate limit exceeded |
| 500 | Internal Server Error |
| 503 | Service Unavailable |

### Error Codes

| Code | Description |
|------|-------------|
| `INVALID_REQUEST` | Request validation failed |
| `UNAUTHORIZED` | Authentication failed |
| `FORBIDDEN` | Insufficient permissions |
| `NOT_FOUND` | Resource not found |
| `RATE_LIMIT_EXCEEDED` | Too many requests |
| `INTERNAL_ERROR` | Server error |
| `SERVICE_UNAVAILABLE` | Service temporarily unavailable |
| `INVALID_PRICING` | No pricing found for model |
| `DUPLICATE_RESOURCE` | Resource already exists |

## Endpoints

### Health & Status

- [GET /health](#get-health) - Health check
- [GET /ready](#get-ready) - Readiness check

### Usage

- [POST /api/v1/usage](#post-usage) - Submit usage record
- [GET /api/v1/usage/history](#get-usage-history) - Get usage history

### Costs

- [GET /api/v1/costs](#get-costs) - Get cost summary

### Pricing

- [GET /api/v1/pricing](#get-pricing) - List pricing tables
- [POST /api/v1/pricing](#post-pricing) - Create pricing table
- [GET /api/v1/pricing/{id}](#get-pricing-id) - Get pricing by ID
- [PUT /api/v1/pricing/{id}](#put-pricing-id) - Update pricing
- [DELETE /api/v1/pricing/{id}](#delete-pricing-id) - Delete pricing

### Analytics

- [GET /api/v1/analytics](#get-analytics) - Get analytics data

### Forecasting (Coming Soon)

- [POST /api/v1/forecasts](#post-forecasts) - Generate forecast
- [GET /api/v1/forecasts/{id}](#get-forecast-id) - Get forecast results

### Budgets (Coming Soon)

- [POST /api/v1/budgets](#post-budgets) - Create budget
- [GET /api/v1/budgets](#get-budgets) - List budgets
- [GET /api/v1/budgets/{id}](#get-budget-id) - Get budget details
- [GET /api/v1/budgets/{id}/alerts](#get-budget-alerts) - Get budget alerts

---

## Health & Status Endpoints

### GET /health

Health check endpoint. No authentication required.

**Request:**
```bash
curl -X GET https://api.llm-cost-ops.dev/health
```

**Response:** `200 OK`
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
    },
    {
      "name": "redis",
      "status": "healthy"
    }
  ]
}
```

**Status Values:**
- `healthy` - All systems operational
- `degraded` - Some non-critical issues
- `unhealthy` - Critical issues

### GET /ready

Readiness check for Kubernetes. No authentication required.

**Request:**
```bash
curl -X GET https://api.llm-cost-ops.dev/ready
```

**Response:** `200 OK`
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 86400,
  "components": []
}
```

---

## Usage Endpoints

### POST /api/v1/usage

Submit a usage record for cost tracking.

**Request:**
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

**Request Body:**
```typescript
{
  organization_id: string;       // Required: Organization ID
  provider: Provider;            // Required: LLM provider
  model_id: string;              // Required: Model identifier
  input_tokens: number;          // Required: Input token count
  output_tokens: number;         // Required: Output token count
  total_tokens: number;          // Required: Total tokens (input + output)
  timestamp?: string;            // Optional: ISO 8601 timestamp (defaults to now)
  metadata?: object;             // Optional: Custom metadata
}
```

**Provider Values:**
- `openai` - OpenAI
- `anthropic` - Anthropic
- `google` - Google Vertex AI
- `azure` - Azure OpenAI
- `aws` - AWS Bedrock
- `cohere` - Cohere
- `mistral` - Mistral

**Response:** `201 Created`
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

**Error Responses:**
- `400 Bad Request` - Invalid request (missing required fields, invalid values)
- `401 Unauthorized` - Invalid API key
- `403 Forbidden` - No permission to submit usage
- `404 Not Found` - No pricing found for the specified model

### GET /api/v1/usage/history

Get paginated usage history.

**Request:**
```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/usage/history?page=1&page_size=50&organization_id=org-123&provider=openai&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Query Parameters:**
```typescript
{
  page?: number;              // Page number (default: 1)
  page_size?: number;         // Items per page (default: 20, max: 100)
  organization_id?: string;   // Filter by organization
  provider?: Provider;        // Filter by provider
  model_id?: string;          // Filter by model
  start_date?: string;        // Start date (ISO 8601)
  end_date?: string;          // End date (ISO 8601)
  sort?: string;              // Sort field (default: timestamp)
  order?: 'asc' | 'desc';     // Sort order (default: desc)
}
```

**Response:** `200 OK`
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
      "currency": "USD",
      "timestamp": "2025-01-15T10:00:00Z",
      "metadata": {
        "request_id": "req-abc123"
      }
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 50,
    "total_items": 1250,
    "total_pages": 25,
    "has_next": true,
    "has_previous": false
  }
}
```

---

## Cost Endpoints

### GET /api/v1/costs

Get cost summary and breakdown.

**Request:**
```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/costs?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&group_by=provider" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Query Parameters:**
```typescript
{
  organization_id?: string;   // Filter by organization
  provider?: Provider;        // Filter by provider
  model_id?: string;          // Filter by model
  start_date?: string;        // Start date (ISO 8601)
  end_date?: string;          // End date (ISO 8601)
  group_by?: GroupBy;         // Group by dimension
}
```

**GroupBy Values:**
- `none` - No grouping (default)
- `provider` - Group by provider
- `model` - Group by model
- `day` - Group by day
- `week` - Group by week
- `month` - Group by month

**Response:** `200 OK`
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
        "requests": 2400
      },
      {
        "dimension": "provider",
        "value": "anthropic",
        "cost": 145.25,
        "tokens": 2300000,
        "requests": 1200
      }
    ]
  }
}
```

---

## Pricing Endpoints

### GET /api/v1/pricing

List pricing tables.

**Request:**
```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/pricing?provider=openai&model_id=gpt-4&page=1&page_size=20" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Query Parameters:**
```typescript
{
  provider?: Provider;      // Filter by provider
  model_id?: string;        // Filter by model
  effective_date?: string;  // Filter by effective date
  page?: number;            // Page number
  page_size?: number;       // Items per page
}
```

**Response:** `200 OK`
```json
{
  "data": [
    {
      "id": "pricing-123",
      "provider": "openai",
      "model_id": "gpt-4",
      "input_price_per_1k": 0.01,
      "output_price_per_1k": 0.03,
      "currency": "USD",
      "effective_date": "2025-01-01T00:00:00Z",
      "created_at": "2024-12-15T10:00:00Z",
      "updated_at": "2024-12-15T10:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 20,
    "total_items": 45,
    "total_pages": 3
  }
}
```

### POST /api/v1/pricing

Create a new pricing table.

**Request:**
```bash
curl -X POST https://api.llm-cost-ops.dev/api/v1/pricing \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "provider": "openai",
    "model_id": "gpt-4",
    "input_price_per_1k": 0.01,
    "output_price_per_1k": 0.03,
    "currency": "USD",
    "effective_date": "2025-01-01T00:00:00Z"
  }'
```

**Request Body:**
```typescript
{
  provider: Provider;            // Required: Provider
  model_id: string;              // Required: Model ID
  input_price_per_1k: number;    // Required: Input price per 1K tokens
  output_price_per_1k: number;   // Required: Output price per 1K tokens
  currency: Currency;            // Required: Currency (default: USD)
  effective_date?: string;       // Optional: Effective date (default: now)
}
```

**Response:** `201 Created`
```json
{
  "data": {
    "id": "pricing-456",
    "provider": "openai",
    "model_id": "gpt-4",
    "input_price_per_1k": 0.01,
    "output_price_per_1k": 0.03,
    "currency": "USD",
    "effective_date": "2025-01-01T00:00:00Z",
    "created_at": "2025-01-15T10:00:00Z",
    "updated_at": "2025-01-15T10:00:00Z"
  }
}
```

---

## Analytics Endpoints

### GET /api/v1/analytics

Get time-series analytics data.

**Request:**
```bash
curl -X GET "https://api.llm-cost-ops.dev/api/v1/analytics?organization_id=org-123&start_date=2025-01-01T00:00:00Z&end_date=2025-01-31T23:59:59Z&interval=day&metrics=total_cost,total_tokens&group_by=provider" \
  -H "Authorization: Bearer YOUR_API_KEY"
```

**Query Parameters:**
```typescript
{
  organization_id?: string;     // Filter by organization
  start_date: string;           // Required: Start date (ISO 8601)
  end_date: string;             // Required: End date (ISO 8601)
  interval?: Interval;          // Time interval (default: day)
  metrics?: Metric[];           // Metrics to include
  group_by?: Dimension[];       // Dimensions to group by
}
```

**Interval Values:**
- `hour` - Hourly data
- `day` - Daily data (default)
- `week` - Weekly data
- `month` - Monthly data

**Metric Values:**
- `total_cost` - Total cost
- `total_tokens` - Total tokens
- `total_requests` - Total requests
- `average_cost_per_request` - Average cost per request
- `average_tokens_per_request` - Average tokens per request

**Dimension Values:**
- `provider` - Group by provider
- `model` - Group by model
- `organization` - Group by organization

**Response:** `200 OK`
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
      },
      {
        "timestamp": "2025-01-16T00:00:00Z",
        "metrics": {
          "total_cost": 18.75,
          "total_tokens": 310000,
          "total_requests": 145
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

---

## SDK Code Examples

### Python

```python
from llm_cost_ops import CostOpsClient

client = CostOpsClient(api_key="YOUR_API_KEY")

# Submit usage
usage = client.usage.submit(
    organization_id="org-123",
    provider="openai",
    model_id="gpt-4",
    input_tokens=1000,
    output_tokens=500
)

# Get costs
costs = client.costs.get(
    organization_id="org-123",
    start_date="2025-01-01T00:00:00Z",
    end_date="2025-01-31T23:59:59Z",
    group_by="provider"
)
```

### TypeScript

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({ apiKey: 'YOUR_API_KEY' });

// Submit usage
const usage = await client.usage.submit({
  organizationId: 'org-123',
  provider: 'openai',
  modelId: 'gpt-4',
  inputTokens: 1000,
  outputTokens: 500
});

// Get costs
const costs = await client.costs.get({
  organizationId: 'org-123',
  startDate: '2025-01-01T00:00:00Z',
  endDate: '2025-01-31T23:59:59Z',
  groupBy: 'provider'
});
```

### Go

```go
import "github.com/llm-devops/llm-cost-ops-go"

client := costops.NewClient("YOUR_API_KEY")

// Submit usage
usage, err := client.Usage.Submit(ctx, &costops.UsageRequest{
    OrganizationID: "org-123",
    Provider:       costops.ProviderOpenAI,
    ModelID:        "gpt-4",
    InputTokens:    1000,
    OutputTokens:   500,
})

// Get costs
costs, err := client.Costs.Get(ctx, &costops.CostsQuery{
    OrganizationID: "org-123",
    StartDate:      "2025-01-01T00:00:00Z",
    EndDate:        "2025-01-31T23:59:59Z",
    GroupBy:        costops.GroupByProvider,
})
```

## Next Steps

- [Authentication Guide](../../getting-started/authentication.md)
- [Usage Examples](../../examples/curl/)
- [Python SDK Reference](../python/README.md)
- [TypeScript SDK Reference](../typescript/README.md)
- [Go SDK Reference](../go/README.md)
