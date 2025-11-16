---
sidebar_position: 1
title: API Overview
---

# API Reference

LLM Cost Ops provides a comprehensive REST API for all operations.

## Base URL

```
https://api.llm-cost-ops.dev/v1
```

## Authentication

All API requests require authentication using API keys or JWT tokens.

```bash
curl -H "Authorization: Bearer YOUR_API_KEY" \
  https://api.llm-cost-ops.dev/v1/costs
```

## Rate Limiting

- 1000 requests per hour for authenticated users
- 100 requests per hour for unauthenticated users

## Endpoints

### Cost Tracking
- `POST /costs/calculate` - Calculate costs for usage records
- `GET /costs` - Query cost records
- `GET /costs/:id` - Get specific cost record

### Pricing
- `GET /pricing` - List pricing tables
- `POST /pricing` - Create pricing table
- `PUT /pricing/:id` - Update pricing table

### Analytics
- `GET /analytics/summary` - Get cost summary
- `GET /analytics/forecast` - Get cost forecast
- `GET /analytics/anomalies` - Detect cost anomalies

## SDKs

Use our official SDKs for easier integration:

- [Python SDK](/docs/sdks/python)
- [TypeScript SDK](/docs/sdks/typescript)
- [Rust SDK](/docs/sdks/rust)
