# API Reference

**Version:** 1.0.0
**Last Updated:** 2025-11-16
**Base URL:** `https://api.llm-cost-ops.example.com`

This comprehensive API reference documents all REST API endpoints, authentication methods, request/response formats, error codes, and integration patterns for the LLM Cost Ops platform.

---

## Table of Contents

- [Authentication](#authentication)
- [Rate Limiting](#rate-limiting)
- [Pagination](#pagination)
- [Filtering and Sorting](#filtering-and-sorting)
- [Error Handling](#error-handling)
- [API Endpoints](#api-endpoints)
  - [Health and Monitoring](#health-and-monitoring)
  - [Usage Management](#usage-management)
  - [Cost Analytics](#cost-analytics)
  - [Pricing Management](#pricing-management)
  - [Analytics and Reporting](#analytics-and-reporting)
  - [Export and Reports](#export-and-reports)
  - [Forecasting](#forecasting)
  - [Audit Logs](#audit-logs)
- [Webhooks](#webhooks)
- [WebSocket API](#websocket-api)
- [SDK Mappings](#sdk-mappings)

---

## Authentication

LLM Cost Ops supports multiple authentication methods for different use cases.

### API Key Authentication

API keys provide simple, secure authentication for service-to-service communication.

**Header Format:**
```http
Authorization: Bearer <api-key>
```

**Example:**
```bash
curl -H "Authorization: Bearer sk_live_a1b2c3d4e5f6" \
  https://api.llm-cost-ops.example.com/api/v1/usage
```

**API Key Types:**
- `sk_live_*` - Production API keys
- `sk_test_*` - Development/testing API keys
- `sk_admin_*` - Administrative API keys (elevated permissions)

**Generating API Keys:**

```bash
POST /api/v1/auth/api-keys
```

**Request:**
```json
{
  "name": "Production Service Key",
  "scopes": ["usage:write", "costs:read"],
  "organization_id": "org-123",
  "expires_at": "2025-12-31T23:59:59Z"
}
```

**Response:**
```json
{
  "data": {
    "id": "key_abc123",
    "key": "sk_live_a1b2c3d4e5f6g7h8i9j0",
    "name": "Production Service Key",
    "scopes": ["usage:write", "costs:read"],
    "organization_id": "org-123",
    "created_at": "2025-11-16T10:00:00Z",
    "expires_at": "2025-12-31T23:59:59Z"
  }
}
```

**Important:** Store the API key securely. It will only be displayed once.

### JWT Authentication

JWT tokens provide user-level authentication with refresh token support.

**Login:**
```bash
POST /api/v1/auth/login
```

**Request:**
```json
{
  "email": "user@example.com",
  "password": "secure-password"
}
```

**Response:**
```json
{
  "data": {
    "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "user": {
      "id": "user-123",
      "email": "user@example.com",
      "organization_id": "org-123",
      "roles": ["admin"]
    }
  }
}
```

**Using JWT:**
```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Refreshing Tokens:**
```bash
POST /api/v1/auth/refresh
```

**Request:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### OAuth 2.0 / SAML SSO

Enterprise customers can use OAuth 2.0 or SAML for single sign-on.

**OAuth 2.0 Authorization Code Flow:**

1. **Authorization Request:**
```
GET /oauth/authorize?
  response_type=code&
  client_id=your_client_id&
  redirect_uri=https://yourapp.com/callback&
  scope=costs:read+usage:write&
  state=random_state_string
```

2. **Token Exchange:**
```bash
POST /oauth/token
```

**Request:**
```json
{
  "grant_type": "authorization_code",
  "code": "auth_code_here",
  "client_id": "your_client_id",
  "client_secret": "your_client_secret",
  "redirect_uri": "https://yourapp.com/callback"
}
```

### Permissions and Scopes

**Available Scopes:**

| Scope | Description |
|-------|-------------|
| `usage:read` | Read usage records |
| `usage:write` | Submit usage data |
| `costs:read` | View cost data |
| `pricing:read` | View pricing tables |
| `pricing:write` | Manage pricing |
| `analytics:read` | Access analytics |
| `export:read` | Export data |
| `forecast:read` | View forecasts |
| `audit:read` | View audit logs |
| `admin:*` | Full administrative access |

**RBAC Roles:**

| Role | Permissions |
|------|-------------|
| `viewer` | Read-only access to costs and usage |
| `analyst` | Viewer + analytics and export |
| `engineer` | Analyst + usage:write |
| `admin` | All permissions |
| `billing_admin` | Pricing and cost management |

---

## Rate Limiting

Rate limits protect the API from abuse and ensure fair usage.

### Rate Limit Headers

Every API response includes rate limit headers:

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1700000000
X-RateLimit-Window: 3600
```

**Header Descriptions:**

- `X-RateLimit-Limit` - Maximum requests per window
- `X-RateLimit-Remaining` - Requests remaining in current window
- `X-RateLimit-Reset` - Unix timestamp when the limit resets
- `X-RateLimit-Window` - Window size in seconds

### Rate Limit Tiers

| Tier | Requests/Hour | Concurrent Requests |
|------|--------------|---------------------|
| Free | 100 | 5 |
| Developer | 1,000 | 10 |
| Professional | 10,000 | 50 |
| Enterprise | 100,000 | 200 |
| Unlimited | No limit | 500 |

### Rate Limit Exceeded

**Status Code:** `429 Too Many Requests`

**Response:**
```json
{
  "error": {
    "code": "TOO_MANY_REQUESTS",
    "message": "Rate limit exceeded. Try again in 3600 seconds.",
    "details": {
      "limit": 1000,
      "remaining": 0,
      "reset_at": "2025-11-16T11:00:00Z"
    }
  }
}
```

**Retry-After Header:**
```http
Retry-After: 3600
```

### Burst Rate Limiting

Short burst limits prevent spike abuse:

- **Burst Window:** 10 seconds
- **Burst Limit:** 10 requests (Free), 50 requests (Professional), 200 requests (Enterprise)

---

## Pagination

Large result sets are paginated for performance and usability.

### Cursor-Based Pagination

**Query Parameters:**

- `limit` - Number of items per page (default: 20, max: 100)
- `cursor` - Pagination cursor from previous response
- `order` - Sort order: `asc` or `desc` (default: `desc`)

**Request:**
```bash
GET /api/v1/usage/history?limit=50&cursor=eyJpZCI6IjEyMyJ9&order=desc
```

**Response:**
```json
{
  "data": [
    { "id": "usage-123", "...": "..." },
    { "id": "usage-124", "...": "..." }
  ],
  "pagination": {
    "has_more": true,
    "next_cursor": "eyJpZCI6IjE3NCJ9",
    "total_count": 1000,
    "limit": 50
  }
}
```

### Offset-Based Pagination

Alternative pagination using page numbers:

**Query Parameters:**

- `page` - Page number (default: 1)
- `per_page` - Items per page (default: 20, max: 100)

**Request:**
```bash
GET /api/v1/costs?page=2&per_page=50
```

**Response:**
```json
{
  "data": [...],
  "pagination": {
    "page": 2,
    "per_page": 50,
    "total_pages": 20,
    "total_count": 1000,
    "has_previous": true,
    "has_next": true
  }
}
```

---

## Filtering and Sorting

### Filtering

Filter results using query parameters:

**Date Range Filters:**
```bash
GET /api/v1/costs?start_date=2025-11-01T00:00:00Z&end_date=2025-11-16T23:59:59Z
```

**Provider Filter:**
```bash
GET /api/v1/usage?provider=openai
```

**Model Filter:**
```bash
GET /api/v1/costs?model_id=gpt-4-turbo
```

**Organization Filter:**
```bash
GET /api/v1/usage?organization_id=org-123
```

**Multiple Filters:**
```bash
GET /api/v1/costs?provider=openai&model_id=gpt-4&start_date=2025-11-01
```

**Advanced Filtering (JSON):**

```bash
POST /api/v1/costs/query
```

**Request:**
```json
{
  "filters": {
    "provider": {"in": ["openai", "anthropic"]},
    "cost": {"gte": 10.00, "lte": 100.00},
    "timestamp": {
      "gte": "2025-11-01T00:00:00Z",
      "lt": "2025-12-01T00:00:00Z"
    },
    "tags": {"contains": "production"}
  }
}
```

### Sorting

Sort results using the `sort_by` and `order` parameters:

```bash
GET /api/v1/costs?sort_by=total_cost&order=desc
```

**Multiple Sort Fields:**
```bash
GET /api/v1/costs?sort_by=provider,timestamp&order=asc,desc
```

**Supported Sort Fields:**

- `timestamp` - Record timestamp
- `total_cost` - Total cost amount
- `total_tokens` - Total tokens used
- `provider` - Provider name
- `model_id` - Model identifier
- `created_at` - Creation timestamp

---

## Error Handling

### Error Response Format

All errors follow a consistent format:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "field": "Additional context"
    }
  },
  "request_id": "req_abc123"
}
```

### HTTP Status Codes

| Code | Name | Description |
|------|------|-------------|
| 200 | OK | Request succeeded |
| 201 | Created | Resource created successfully |
| 204 | No Content | Request succeeded, no content returned |
| 400 | Bad Request | Invalid request format or parameters |
| 401 | Unauthorized | Missing or invalid authentication |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Resource already exists or conflict |
| 422 | Unprocessable Entity | Validation errors |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server error |
| 503 | Service Unavailable | Service temporarily unavailable |

### Error Codes

#### Authentication Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_API_KEY` | 401 | API key is invalid or revoked |
| `INVALID_TOKEN` | 401 | JWT token is invalid or malformed |
| `TOKEN_EXPIRED` | 401 | JWT token has expired |
| `MISSING_CREDENTIALS` | 401 | No authentication provided |
| `INSUFFICIENT_PERMISSIONS` | 403 | User lacks required permissions |

#### Validation Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `VALIDATION_ERROR` | 422 | Request validation failed |
| `INVALID_DATE_RANGE` | 422 | Start date is after end date |
| `INVALID_PROVIDER` | 422 | Unknown provider |
| `INVALID_MODEL` | 422 | Unknown model |
| `MISSING_REQUIRED_FIELD` | 422 | Required field is missing |

#### Resource Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `NOT_FOUND` | 404 | Resource not found |
| `ALREADY_EXISTS` | 409 | Resource already exists |
| `CONFLICT` | 409 | Resource conflict |

#### Rate Limiting Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `TOO_MANY_REQUESTS` | 429 | Rate limit exceeded |
| `BURST_LIMIT_EXCEEDED` | 429 | Burst rate limit exceeded |

#### Server Errors

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INTERNAL_SERVER_ERROR` | 500 | Unexpected server error |
| `DATABASE_ERROR` | 500 | Database operation failed |
| `SERVICE_UNAVAILABLE` | 503 | Service temporarily unavailable |

### Validation Error Example

**Status Code:** `422 Unprocessable Entity`

**Response:**
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Request validation failed",
    "details": [
      {
        "field": "organization_id",
        "message": "must not be empty",
        "code": "LENGTH_MIN"
      },
      {
        "field": "input_tokens",
        "message": "must be greater than or equal to 0",
        "code": "RANGE_MIN"
      }
    ]
  },
  "request_id": "req_xyz789"
}
```

---

## API Endpoints

### Health and Monitoring

#### Health Check

Check service health status.

**Endpoint:** `GET /health`

**Authentication:** None required

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 86400,
  "components": [
    {
      "name": "database",
      "status": "healthy",
      "message": null
    },
    {
      "name": "redis",
      "status": "healthy",
      "message": null
    }
  ]
}
```

**Status Values:**
- `healthy` - All systems operational
- `degraded` - Some components degraded
- `unhealthy` - Service unavailable

#### Readiness Check

Check if service is ready to accept requests (Kubernetes readiness probe).

**Endpoint:** `GET /ready`

**Response:** Same as health check

#### Liveness Check

Minimal health check (Kubernetes liveness probe).

**Endpoint:** `GET /live`

**Response:**
```json
{
  "status": "healthy"
}
```

#### Metrics

Prometheus metrics endpoint.

**Endpoint:** `GET /metrics`

**Response:** Prometheus text format

```
# HELP http_requests_total Total HTTP requests
# TYPE http_requests_total counter
http_requests_total{method="GET",status="200"} 1234

# HELP cost_calculations_total Total cost calculations
# TYPE cost_calculations_total counter
cost_calculations_total{provider="openai"} 5678
```

---

### Usage Management

#### Submit Usage

Submit usage data for cost calculation.

**Endpoint:** `POST /api/v1/usage`

**Authentication:** Required (API Key or JWT)

**Permissions:** `usage:write`

**Request:**
```json
{
  "organization_id": "org-123",
  "provider": "openai",
  "model_id": "gpt-4-turbo",
  "input_tokens": 1500,
  "output_tokens": 800,
  "total_tokens": 2300,
  "cached_tokens": 500,
  "timestamp": "2025-11-16T10:00:00Z",
  "metadata": {
    "request_id": "req-abc123",
    "user_id": "user-456",
    "endpoint": "/v1/chat/completions"
  }
}
```

**Required Fields:**
- `organization_id` - Organization identifier
- `provider` - LLM provider (openai, anthropic, google, etc.)
- `model_id` - Model identifier
- `input_tokens` - Number of input tokens
- `output_tokens` - Number of output tokens
- `total_tokens` - Total tokens (input + output)

**Optional Fields:**
- `cached_tokens` - Cached input tokens (for providers that support it)
- `reasoning_tokens` - Reasoning tokens (for models like o1)
- `timestamp` - Usage timestamp (defaults to current time)
- `metadata` - Additional metadata (JSON object)
- `tags` - Tags for categorization (array of strings)

**Response:**
```json
{
  "data": {
    "usage_id": "usage-abc123",
    "organization_id": "org-123",
    "estimated_cost": 0.023,
    "currency": "USD",
    "processed_at": "2025-11-16T10:00:01Z"
  }
}
```

**Status Codes:**
- `201 Created` - Usage submitted successfully
- `400 Bad Request` - Invalid request format
- `401 Unauthorized` - Authentication required
- `422 Unprocessable Entity` - Validation failed

#### Batch Submit Usage

Submit multiple usage records in a single request.

**Endpoint:** `POST /api/v1/usage/batch`

**Request:**
```json
{
  "records": [
    {
      "organization_id": "org-123",
      "provider": "openai",
      "model_id": "gpt-4-turbo",
      "input_tokens": 1500,
      "output_tokens": 800,
      "total_tokens": 2300
    },
    {
      "organization_id": "org-123",
      "provider": "anthropic",
      "model_id": "claude-3-sonnet-20240229",
      "input_tokens": 2000,
      "output_tokens": 1000,
      "total_tokens": 3000
    }
  ]
}
```

**Limits:**
- Maximum 1,000 records per batch
- Total request size: 10 MB

**Response:**
```json
{
  "data": {
    "submitted": 2,
    "failed": 0,
    "results": [
      {
        "index": 0,
        "usage_id": "usage-abc123",
        "status": "success"
      },
      {
        "index": 1,
        "usage_id": "usage-abc124",
        "status": "success"
      }
    ]
  }
}
```

#### Get Usage History

Retrieve usage records with filtering and pagination.

**Endpoint:** `GET /api/v1/usage/history`

**Query Parameters:**
- `start_date` - Start date (ISO 8601)
- `end_date` - End date (ISO 8601)
- `provider` - Filter by provider
- `model_id` - Filter by model
- `organization_id` - Filter by organization
- `limit` - Records per page (default: 20, max: 100)
- `cursor` - Pagination cursor

**Request:**
```bash
GET /api/v1/usage/history?start_date=2025-11-01&provider=openai&limit=50
```

**Response:**
```json
{
  "data": [
    {
      "id": "usage-abc123",
      "timestamp": "2025-11-16T10:00:00Z",
      "organization_id": "org-123",
      "provider": "openai",
      "model_id": "gpt-4-turbo",
      "input_tokens": 1500,
      "output_tokens": 800,
      "total_tokens": 2300,
      "cached_tokens": 500,
      "estimated_cost": 0.023,
      "currency": "USD",
      "metadata": {
        "request_id": "req-abc123"
      }
    }
  ],
  "pagination": {
    "has_more": true,
    "next_cursor": "eyJpZCI6IjEyMyJ9",
    "total_count": 1000,
    "limit": 50
  }
}
```

#### Get Usage by ID

Retrieve a specific usage record.

**Endpoint:** `GET /api/v1/usage/{usage_id}`

**Response:**
```json
{
  "data": {
    "id": "usage-abc123",
    "timestamp": "2025-11-16T10:00:00Z",
    "organization_id": "org-123",
    "provider": "openai",
    "model_id": "gpt-4-turbo",
    "input_tokens": 1500,
    "output_tokens": 800,
    "total_tokens": 2300,
    "estimated_cost": 0.023,
    "currency": "USD"
  }
}
```

---

### Cost Analytics

#### Get Costs

Retrieve cost data with filtering and aggregation.

**Endpoint:** `GET /api/v1/costs`

**Query Parameters:**
- `organization_id` - Filter by organization
- `provider` - Filter by provider
- `model_id` - Filter by model
- `start_date` - Start date (ISO 8601)
- `end_date` - End date (ISO 8601)
- `group_by` - Group results: `none`, `provider`, `model`, `day`, `week`, `month`

**Request:**
```bash
GET /api/v1/costs?organization_id=org-123&start_date=2025-11-01&group_by=provider
```

**Response:**
```json
{
  "data": {
    "total_cost": 1234.56,
    "currency": "USD",
    "total_tokens": 5000000,
    "total_requests": 10000,
    "period_start": "2025-11-01T00:00:00Z",
    "period_end": "2025-11-16T23:59:59Z",
    "breakdown": [
      {
        "dimension": "provider",
        "value": "openai",
        "cost": 789.12,
        "tokens": 3000000,
        "requests": 6000
      },
      {
        "dimension": "provider",
        "value": "anthropic",
        "cost": 445.44,
        "tokens": 2000000,
        "requests": 4000
      }
    ]
  }
}
```

#### Cost Summary

Get aggregated cost summary.

**Endpoint:** `GET /api/v1/costs/summary`

**Query Parameters:**
- `period` - Time period: `today`, `yesterday`, `last-7-days`, `last-30-days`, `this-month`, `last-month`
- `organization_id` - Filter by organization

**Response:**
```json
{
  "data": {
    "period": "last-30-days",
    "total_cost": 5432.10,
    "currency": "USD",
    "total_tokens": 15000000,
    "total_requests": 25000,
    "average_cost_per_request": 0.217284,
    "average_tokens_per_request": 600,
    "top_models": [
      {
        "model_id": "gpt-4-turbo",
        "provider": "openai",
        "cost": 3210.50,
        "requests": 10000
      },
      {
        "model_id": "claude-3-opus-20240229",
        "provider": "anthropic",
        "cost": 2221.60,
        "requests": 15000
      }
    ],
    "daily_trend": [
      {
        "date": "2025-11-01",
        "cost": 180.50,
        "requests": 850
      }
    ]
  }
}
```

#### Cost Breakdown

Get detailed cost breakdown by multiple dimensions.

**Endpoint:** `POST /api/v1/costs/breakdown`

**Request:**
```json
{
  "start_date": "2025-11-01T00:00:00Z",
  "end_date": "2025-11-16T23:59:59Z",
  "organization_id": "org-123",
  "dimensions": ["provider", "model", "project"],
  "metrics": ["total_cost", "total_tokens", "request_count"]
}
```

**Response:**
```json
{
  "data": {
    "breakdowns": [
      {
        "provider": "openai",
        "model": "gpt-4-turbo",
        "project": "prod-api",
        "total_cost": 450.25,
        "total_tokens": 1500000,
        "request_count": 3000
      }
    ],
    "totals": {
      "total_cost": 1234.56,
      "total_tokens": 5000000,
      "request_count": 10000
    }
  }
}
```

---

### Pricing Management

#### List Pricing

List all pricing tables.

**Endpoint:** `GET /api/v1/pricing`

**Query Parameters:**
- `provider` - Filter by provider
- `model_id` - Filter by model
- `active_only` - Show only active pricing (default: true)
- `limit` - Records per page

**Response:**
```json
{
  "data": [
    {
      "id": "price-abc123",
      "provider": "openai",
      "model_id": "gpt-4-turbo",
      "input_price_per_1k": 0.01,
      "output_price_per_1k": 0.03,
      "currency": "USD",
      "effective_date": "2025-11-01T00:00:00Z",
      "end_date": null,
      "created_at": "2025-10-15T10:00:00Z",
      "updated_at": "2025-10-15T10:00:00Z"
    }
  ],
  "pagination": {
    "has_more": false,
    "total_count": 15
  }
}
```

#### Create Pricing

Create a new pricing table.

**Endpoint:** `POST /api/v1/pricing`

**Permissions:** `pricing:write`

**Request:**
```json
{
  "provider": "openai",
  "model_id": "gpt-4-turbo",
  "input_price_per_1k": 0.01,
  "output_price_per_1k": 0.03,
  "currency": "USD",
  "effective_date": "2025-12-01T00:00:00Z"
}
```

**Response:**
```json
{
  "data": {
    "id": "price-abc123",
    "provider": "openai",
    "model_id": "gpt-4-turbo",
    "input_price_per_1k": 0.01,
    "output_price_per_1k": 0.03,
    "currency": "USD",
    "effective_date": "2025-12-01T00:00:00Z",
    "created_at": "2025-11-16T10:00:00Z"
  }
}
```

#### Get Pricing by ID

Retrieve specific pricing table.

**Endpoint:** `GET /api/v1/pricing/{pricing_id}`

**Response:**
```json
{
  "data": {
    "id": "price-abc123",
    "provider": "openai",
    "model_id": "gpt-4-turbo",
    "input_price_per_1k": 0.01,
    "output_price_per_1k": 0.03,
    "currency": "USD",
    "effective_date": "2025-11-01T00:00:00Z",
    "pricing_structure": {
      "type": "per_token",
      "tiers": null
    }
  }
}
```

#### Update Pricing

Update an existing pricing table (creates new version).

**Endpoint:** `PUT /api/v1/pricing/{pricing_id}`

**Request:**
```json
{
  "input_price_per_1k": 0.009,
  "output_price_per_1k": 0.027,
  "effective_date": "2025-12-01T00:00:00Z"
}
```

#### Delete Pricing

Set end date for pricing table (soft delete).

**Endpoint:** `DELETE /api/v1/pricing/{pricing_id}`

**Query Parameters:**
- `end_date` - End date for pricing (default: now)

---

### Analytics and Reporting

#### Get Analytics

Retrieve analytics data with time-series aggregation.

**Endpoint:** `GET /api/v1/analytics`

**Query Parameters:**
- `organization_id` - Filter by organization
- `start_date` - Start date (required)
- `end_date` - End date (required)
- `metrics` - Metrics to retrieve (comma-separated)
- `group_by` - Grouping dimensions (comma-separated)
- `interval` - Time interval: `hour`, `day`, `week`, `month`

**Request:**
```bash
GET /api/v1/analytics?start_date=2025-11-01&end_date=2025-11-16&interval=day&metrics=total_cost,total_tokens&group_by=provider
```

**Response:**
```json
{
  "data": {
    "time_series": [
      {
        "timestamp": "2025-11-01T00:00:00Z",
        "metrics": {
          "total_cost": 45.67,
          "total_tokens": 150000,
          "provider_breakdown": {
            "openai": 30.50,
            "anthropic": 15.17
          }
        }
      },
      {
        "timestamp": "2025-11-02T00:00:00Z",
        "metrics": {
          "total_cost": 52.34,
          "total_tokens": 175000
        }
      }
    ],
    "summary": {
      "total_cost": 1234.56,
      "total_tokens": 5000000,
      "total_requests": 10000,
      "average_cost_per_request": 0.123456,
      "average_tokens_per_request": 500
    }
  }
}
```

**Available Metrics:**
- `total_cost` - Total cost
- `total_tokens` - Total tokens
- `total_requests` - Total requests
- `average_cost_per_request` - Average cost per request
- `average_tokens_per_request` - Average tokens per request
- `input_tokens` - Input tokens only
- `output_tokens` - Output tokens only
- `cached_tokens` - Cached tokens

**Available Dimensions:**
- `provider` - Group by provider
- `model` - Group by model
- `organization` - Group by organization
- `project` - Group by project
- `tag` - Group by tag

---

### Export and Reports

#### Export Data

Export cost and usage data in various formats.

**Endpoint:** `POST /api/v1/export`

**Request:**
```json
{
  "format": "csv",
  "type": "costs",
  "start_date": "2025-11-01T00:00:00Z",
  "end_date": "2025-11-16T23:59:59Z",
  "organization_id": "org-123",
  "filters": {
    "provider": "openai"
  },
  "columns": ["timestamp", "model_id", "total_cost", "total_tokens"],
  "compression": "gzip"
}
```

**Supported Formats:**
- `csv` - Comma-separated values
- `json` - JSON format
- `jsonl` - JSON Lines (newline-delimited)
- `xlsx` - Excel spreadsheet
- `parquet` - Apache Parquet (columnar)

**Export Types:**
- `costs` - Cost records
- `usage` - Usage records
- `analytics` - Aggregated analytics
- `audit` - Audit logs

**Response:**
```json
{
  "data": {
    "export_id": "export-abc123",
    "status": "processing",
    "format": "csv",
    "estimated_size": 52428800,
    "created_at": "2025-11-16T10:00:00Z",
    "download_url": null
  }
}
```

#### Get Export Status

Check export job status.

**Endpoint:** `GET /api/v1/export/{export_id}`

**Response:**
```json
{
  "data": {
    "export_id": "export-abc123",
    "status": "completed",
    "format": "csv",
    "file_size": 51234567,
    "created_at": "2025-11-16T10:00:00Z",
    "completed_at": "2025-11-16T10:02:30Z",
    "download_url": "https://downloads.llm-cost-ops.example.com/exports/abc123.csv.gz",
    "expires_at": "2025-11-23T10:02:30Z"
  }
}
```

**Status Values:**
- `pending` - Export queued
- `processing` - Export in progress
- `completed` - Export ready for download
- `failed` - Export failed
- `expired` - Download link expired

#### Download Export

Download completed export file.

**Endpoint:** `GET /api/v1/export/{export_id}/download`

**Response:** Binary file download

**Headers:**
```http
Content-Type: application/gzip
Content-Disposition: attachment; filename="cost-export-2025-11-16.csv.gz"
Content-Length: 51234567
```

#### Generate Report

Generate a formatted report.

**Endpoint:** `POST /api/v1/reports/generate`

**Request:**
```json
{
  "type": "cost_summary",
  "period": "last-30-days",
  "organization_id": "org-123",
  "format": "pdf",
  "delivery": {
    "method": "email",
    "recipients": ["finance@example.com"]
  },
  "template": "executive_summary"
}
```

**Report Types:**
- `cost_summary` - Cost summary report
- `usage_analysis` - Usage analysis report
- `forecast` - Forecast report
- `budget` - Budget vs actual report
- `audit` - Audit trail report
- `custom` - Custom report from template

**Response:**
```json
{
  "data": {
    "report_id": "report-abc123",
    "status": "generating",
    "type": "cost_summary",
    "format": "pdf",
    "created_at": "2025-11-16T10:00:00Z"
  }
}
```

#### Schedule Report

Schedule recurring reports.

**Endpoint:** `POST /api/v1/reports/schedules`

**Request:**
```json
{
  "name": "Monthly Cost Report",
  "type": "cost_summary",
  "format": "pdf",
  "schedule": "0 9 1 * *",
  "timezone": "America/New_York",
  "organization_id": "org-123",
  "delivery": {
    "method": "email",
    "recipients": ["finance@example.com"]
  },
  "template": "executive_summary"
}
```

**Schedule Format:** Cron expression

**Response:**
```json
{
  "data": {
    "schedule_id": "sched-abc123",
    "name": "Monthly Cost Report",
    "schedule": "0 9 1 * *",
    "timezone": "America/New_York",
    "next_run": "2025-12-01T09:00:00Z",
    "enabled": true,
    "created_at": "2025-11-16T10:00:00Z"
  }
}
```

---

### Forecasting

#### Get Forecast

Generate cost forecast.

**Endpoint:** `GET /api/v1/forecasts`

**Query Parameters:**
- `organization_id` - Organization ID
- `horizon_days` - Forecast horizon (default: 30, max: 90)
- `model` - Forecast model: `linear`, `moving_average`, `exponential_smoothing`
- `confidence` - Confidence level: 0.80, 0.90, 0.95 (default: 0.95)

**Request:**
```bash
GET /api/v1/forecasts?organization_id=org-123&horizon_days=30&model=exponential_smoothing
```

**Response:**
```json
{
  "data": {
    "forecast_id": "forecast-abc123",
    "organization_id": "org-123",
    "model": "exponential_smoothing",
    "confidence_level": 0.95,
    "generated_at": "2025-11-16T10:00:00Z",
    "historical_period": {
      "start": "2025-10-17T00:00:00Z",
      "end": "2025-11-16T00:00:00Z"
    },
    "forecast_period": {
      "start": "2025-11-17T00:00:00Z",
      "end": "2025-12-16T23:59:59Z"
    },
    "predictions": [
      {
        "date": "2025-11-17",
        "predicted_cost": 45.67,
        "lower_bound": 38.20,
        "upper_bound": 53.14,
        "confidence": 0.95
      },
      {
        "date": "2025-11-18",
        "predicted_cost": 46.12,
        "lower_bound": 38.50,
        "upper_bound": 53.74,
        "confidence": 0.95
      }
    ],
    "summary": {
      "total_predicted_cost": 1380.50,
      "average_daily_cost": 46.02,
      "trend": "increasing",
      "seasonality_detected": true
    },
    "metrics": {
      "mape": 5.2,
      "rmse": 2.34,
      "mae": 1.87
    }
  }
}
```

#### Detect Anomalies

Detect cost anomalies.

**Endpoint:** `GET /api/v1/forecasts/anomalies`

**Query Parameters:**
- `organization_id` - Organization ID
- `start_date` - Start date
- `end_date` - End date
- `method` - Detection method: `zscore`, `iqr`, `prophet`
- `sensitivity` - Sensitivity: `low`, `medium`, `high`

**Response:**
```json
{
  "data": {
    "anomalies": [
      {
        "date": "2025-11-10",
        "actual_cost": 125.50,
        "expected_cost": 45.67,
        "deviation": 174.8,
        "severity": "high",
        "zscore": 3.5,
        "description": "Cost spike detected"
      }
    ],
    "summary": {
      "total_anomalies": 3,
      "high_severity": 1,
      "medium_severity": 2,
      "low_severity": 0
    }
  }
}
```

#### Budget Forecast

Generate budget forecast and alerts.

**Endpoint:** `POST /api/v1/forecasts/budget`

**Request:**
```json
{
  "organization_id": "org-123",
  "budget_amount": 5000.00,
  "budget_period": "monthly",
  "alert_thresholds": [0.50, 0.75, 0.90],
  "forecast_horizon_days": 30
}
```

**Response:**
```json
{
  "data": {
    "budget_id": "budget-abc123",
    "budget_amount": 5000.00,
    "currency": "USD",
    "current_spend": 3250.50,
    "spend_percentage": 65.01,
    "projected_spend": 4850.75,
    "projected_percentage": 97.02,
    "days_remaining": 14,
    "daily_rate": 232.18,
    "alerts": [
      {
        "threshold": 0.50,
        "triggered": true,
        "triggered_at": "2025-11-05T00:00:00Z"
      },
      {
        "threshold": 0.75,
        "triggered": false,
        "estimated_trigger_date": "2025-11-20"
      }
    ],
    "status": "on_track",
    "risk_level": "medium"
  }
}
```

---

### Audit Logs

#### Query Audit Logs

Retrieve audit trail of all operations.

**Endpoint:** `GET /api/v1/audit`

**Permissions:** `audit:read`

**Query Parameters:**
- `start_date` - Start date
- `end_date` - End date
- `user_id` - Filter by user
- `organization_id` - Filter by organization
- `event_type` - Filter by event type
- `resource_type` - Filter by resource type
- `severity` - Filter by severity
- `limit` - Records per page

**Request:**
```bash
GET /api/v1/audit?start_date=2025-11-01&event_type=pricing.created&limit=100
```

**Response:**
```json
{
  "data": [
    {
      "id": "audit-abc123",
      "timestamp": "2025-11-16T10:00:00Z",
      "event_type": "pricing.created",
      "severity": "info",
      "user_id": "user-123",
      "user_email": "admin@example.com",
      "organization_id": "org-123",
      "resource_type": "pricing",
      "resource_id": "price-abc123",
      "action": "create",
      "status": "success",
      "ip_address": "203.0.113.1",
      "user_agent": "Mozilla/5.0...",
      "changes": {
        "before": null,
        "after": {
          "provider": "openai",
          "model_id": "gpt-4-turbo",
          "input_price_per_1k": 0.01
        }
      },
      "metadata": {
        "request_id": "req-xyz789"
      }
    }
  ],
  "pagination": {
    "has_more": true,
    "next_cursor": "eyJpZCI6IjEyMyJ9"
  }
}
```

**Event Types:**
- `auth.login` - User login
- `auth.logout` - User logout
- `auth.api_key.created` - API key created
- `auth.api_key.revoked` - API key revoked
- `usage.submitted` - Usage submitted
- `pricing.created` - Pricing created
- `pricing.updated` - Pricing updated
- `pricing.deleted` - Pricing deleted
- `export.created` - Export created
- `report.generated` - Report generated

**Severity Levels:**
- `debug` - Debug information
- `info` - Informational
- `warning` - Warning
- `error` - Error
- `critical` - Critical event

---

## Webhooks

Webhooks provide real-time notifications for events in your account.

### Webhook Configuration

#### Create Webhook

**Endpoint:** `POST /api/v1/webhooks`

**Request:**
```json
{
  "url": "https://yourapp.com/webhooks/llm-cost-ops",
  "events": [
    "usage.submitted",
    "forecast.anomaly_detected",
    "budget.threshold_exceeded"
  ],
  "secret": "whsec_abc123xyz",
  "description": "Production webhook",
  "enabled": true
}
```

**Response:**
```json
{
  "data": {
    "webhook_id": "webhook-abc123",
    "url": "https://yourapp.com/webhooks/llm-cost-ops",
    "events": ["usage.submitted", "forecast.anomaly_detected"],
    "secret": "whsec_abc123xyz",
    "enabled": true,
    "created_at": "2025-11-16T10:00:00Z"
  }
}
```

### Webhook Delivery

**HTTP Method:** `POST`

**Headers:**
```http
Content-Type: application/json
X-LLM-CostOps-Signature: sha256=abc123...
X-LLM-CostOps-Event: usage.submitted
X-LLM-CostOps-Delivery-ID: delivery-xyz789
X-LLM-CostOps-Timestamp: 1700000000
```

**Payload:**
```json
{
  "event": "usage.submitted",
  "timestamp": "2025-11-16T10:00:00Z",
  "delivery_id": "delivery-xyz789",
  "data": {
    "usage_id": "usage-abc123",
    "organization_id": "org-123",
    "provider": "openai",
    "model_id": "gpt-4-turbo",
    "total_cost": 0.023,
    "total_tokens": 2300
  }
}
```

### Signature Verification

Verify webhook signature using HMAC SHA-256:

```python
import hmac
import hashlib

def verify_webhook(payload, signature, secret):
    expected = hmac.new(
        secret.encode(),
        payload.encode(),
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(f"sha256={expected}", signature)
```

### Webhook Events

| Event | Description |
|-------|-------------|
| `usage.submitted` | Usage data submitted |
| `cost.calculated` | Cost calculation completed |
| `forecast.generated` | Forecast generated |
| `forecast.anomaly_detected` | Cost anomaly detected |
| `budget.threshold_exceeded` | Budget threshold exceeded |
| `export.completed` | Export job completed |
| `report.generated` - Report generated |
| `pricing.updated` | Pricing updated |

### Retry Policy

Failed webhook deliveries are retried with exponential backoff:

- Retry 1: Immediate
- Retry 2: 5 seconds
- Retry 3: 30 seconds
- Retry 4: 5 minutes
- Retry 5: 30 minutes
- Retry 6: 2 hours
- Retry 7: 6 hours

After 7 failed attempts, the webhook is disabled.

---

## WebSocket API

Real-time streaming API for live cost monitoring.

### Connection

**Endpoint:** `wss://api.llm-cost-ops.example.com/v1/stream`

**Authentication:** Query parameter or header

```javascript
const ws = new WebSocket('wss://api.llm-cost-ops.example.com/v1/stream?token=YOUR_JWT_TOKEN');
```

### Subscribe to Events

**Message Format:**
```json
{
  "action": "subscribe",
  "channels": [
    "costs:org-123",
    "usage:org-123:openai"
  ]
}
```

**Channel Patterns:**
- `costs:{organization_id}` - All cost events
- `usage:{organization_id}` - All usage events
- `usage:{organization_id}:{provider}` - Provider-specific usage
- `alerts:{organization_id}` - Budget and anomaly alerts
- `forecasts:{organization_id}` - Forecast updates

### Receive Events

**Event Format:**
```json
{
  "channel": "costs:org-123",
  "event": "cost.calculated",
  "timestamp": "2025-11-16T10:00:00Z",
  "data": {
    "usage_id": "usage-abc123",
    "total_cost": 0.023,
    "provider": "openai",
    "model_id": "gpt-4-turbo"
  }
}
```

### Unsubscribe

```json
{
  "action": "unsubscribe",
  "channels": ["costs:org-123"]
}
```

### Heartbeat

Client should send ping every 30 seconds:

```json
{
  "action": "ping"
}
```

Server responds:
```json
{
  "action": "pong",
  "timestamp": "2025-11-16T10:00:00Z"
}
```

---

## SDK Mappings

### Python SDK

```python
from llm_cost_ops import CostOpsClient

client = CostOpsClient(api_key="sk_live_...")

# Submit usage
usage = client.usage.submit(
    organization_id="org-123",
    provider="openai",
    model_id="gpt-4-turbo",
    input_tokens=1500,
    output_tokens=800
)

# Get costs
costs = client.costs.get(
    organization_id="org-123",
    start_date="2025-11-01",
    group_by="provider"
)

# Export data
export = client.exports.create(
    format="csv",
    type="costs",
    start_date="2025-11-01"
)
```

### Node.js SDK

```javascript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  apiKey: 'sk_live_...'
});

// Submit usage
const usage = await client.usage.submit({
  organizationId: 'org-123',
  provider: 'openai',
  modelId: 'gpt-4-turbo',
  inputTokens: 1500,
  outputTokens: 800
});

// Get costs
const costs = await client.costs.get({
  organizationId: 'org-123',
  startDate: '2025-11-01',
  groupBy: 'provider'
});
```

### Go SDK

```go
import "github.com/llm-cost-ops/go-sdk"

client := costops.NewClient("sk_live_...")

// Submit usage
usage, err := client.Usage.Submit(&costops.UsageSubmission{
    OrganizationID: "org-123",
    Provider:       "openai",
    ModelID:        "gpt-4-turbo",
    InputTokens:    1500,
    OutputTokens:   800,
})

// Get costs
costs, err := client.Costs.Get(&costops.CostsQuery{
    OrganizationID: "org-123",
    StartDate:      time.Parse(...),
    GroupBy:        "provider",
})
```

---

## Appendix

### Supported Providers

- `openai` - OpenAI
- `anthropic` - Anthropic
- `google` - Google Vertex AI
- `azure` - Azure OpenAI
- `aws` - AWS Bedrock
- `cohere` - Cohere
- `mistral` - Mistral AI

### Supported Currencies

- `USD` - US Dollar
- `EUR` - Euro
- `GBP` - British Pound
- `JPY` - Japanese Yen
- `AUD` - Australian Dollar
- `CAD` - Canadian Dollar

### Rate Limit Best Practices

1. **Implement exponential backoff** for retries
2. **Cache responses** when appropriate
3. **Batch requests** when possible
4. **Monitor rate limit headers** proactively
5. **Request rate limit increase** for production workloads

### Versioning

The API uses semantic versioning. The current version is **v1**.

**Version Headers:**
```http
X-API-Version: v1
```

Breaking changes will result in a new API version (v2, v3, etc.).

---

**Need Help?**

- Documentation: https://docs.llm-cost-ops.example.com
- Support: support@llm-cost-ops.example.com
- Status Page: https://status.llm-cost-ops.example.com
