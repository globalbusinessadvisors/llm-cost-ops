# Cost Attribution Agent

Google Cloud Edge Function for LLM cost analysis and attribution.

## Overview

The Cost Attribution Agent is a stateless edge function that:

1. **Validates input** using Zod schemas
2. **Calculates costs** based on LLM provider pricing models
3. **Attributes costs** to dimensions (user, project, organization, environment)
4. **Emits DecisionEvent** to ruvector-service for decision intelligence
5. **Emits telemetry** to LLM-Observatory for observability
6. **Returns structured output** with cost breakdown and attribution metadata

## Architecture

```
┌─────────────────────────────────────────────────────┐
│          Cost Attribution Agent (Edge)              │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────┐    ┌──────────────┐             │
│  │   Request    │───▶│  Validation  │             │
│  │  (HTTP POST) │    │   (Zod)      │             │
│  └──────────────┘    └──────┬───────┘             │
│                             │                       │
│                             ▼                       │
│                      ┌──────────────┐              │
│                      │    Cost      │              │
│                      │  Calculator  │              │
│                      └──────┬───────┘              │
│                             │                       │
│                             ▼                       │
│                      ┌──────────────┐              │
│                      │    Cost      │              │
│                      │  Attributor  │              │
│                      └──────┬───────┘              │
│                             │                       │
│         ┌───────────────────┼───────────────────┐  │
│         │                   │                   │  │
│         ▼                   ▼                   ▼  │
│  ┌─────────────┐    ┌─────────────┐    ┌──────────┴──┐
│  │  Decision   │    │  Telemetry  │    │   Response   │
│  │    Event    │    │             │    │   (JSON)     │
│  └──────┬──────┘    └──────┬──────┘    └──────────────┘
│         │                   │                       │
└─────────┼───────────────────┼───────────────────────┘
          │                   │
          ▼                   ▼
   ┌──────────────┐    ┌──────────────┐
   │  RuVector    │    │     LLM      │
   │  Service     │    │ Observatory  │
   └──────────────┘    └──────────────┘
```

## Key Features

### Stateless Design
- No runtime state management
- No workflow interception
- Pure analysis and emission
- Deterministic output

### Input Validation
- Zod schema validation
- Type-safe inputs
- Comprehensive error messages

### Cost Calculation
- Support for major LLM providers (OpenAI, Anthropic, Google)
- Per-token pricing
- Input/output/cached token differentiation
- Custom pricing overrides

### Cost Attribution
- Multi-dimensional attribution (user, project, org, environment)
- Confidence scoring based on available dimensions
- Flexible tagging system

### Event Emission
- DecisionEvent to ruvector-service
- Telemetry to LLM-Observatory
- Fire-and-forget pattern
- Timeout protection

## API

### Request

```typescript
POST /costAttributionHandler

{
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2025-01-20T18:00:00.000Z",
  "usage": {
    "provider": "anthropic",
    "model": "claude-sonnet-4",
    "inputTokens": 1000,
    "outputTokens": 500,
    "cachedTokens": 200,
    "latencyMs": 1234
  },
  "pricingContext": {
    "tier": "enterprise",
    "currency": "USD"
  },
  "dimensions": {
    "userId": "user-123",
    "projectId": "proj-456",
    "organizationId": "org-789",
    "environment": "production",
    "tags": {
      "team": "ml-platform",
      "feature": "chatbot"
    }
  }
}
```

### Response

```typescript
{
  "requestId": "550e8400-e29b-41d4-a716-446655440000",
  "analysisTimestamp": "2025-01-20T18:00:00.123Z",
  "costs": {
    "totalCost": 0.00625,
    "inputCost": 0.003,
    "outputCost": 0.0025,
    "cachedCost": 0.00025,
    "currency": "USD",
    "costPer1kTokens": 0.003676
  },
  "attribution": {
    "primary": "organization:org-789",
    "dimensions": {
      "userId": "user-123",
      "projectId": "proj-456",
      "organizationId": "org-789",
      "environment": "production"
    },
    "tags": {
      "team": "ml-platform",
      "feature": "chatbot"
    },
    "confidence": 1.0
  },
  "decisionEvent": {
    "eventId": "...",
    "eventType": "cost_attribution",
    "timestamp": "2025-01-20T18:00:00.123Z",
    "agentId": "cost-attribution-agent",
    "decision": {
      "action": "attribute_cost",
      "result": "success",
      "confidence": 1.0
    },
    "context": {
      "provider": "anthropic",
      "model": "claude-sonnet-4",
      "totalTokens": 1700,
      "totalCost": 0.00625
    }
  },
  "telemetry": {
    "telemetryId": "...",
    "agentId": "cost-attribution-agent",
    "timestamp": "2025-01-20T18:00:00.123Z",
    "metrics": {
      "processingDurationMs": 23,
      "dimensionCount": 4
    }
  }
}
```

## Environment Variables

- `AGENT_ID`: Agent identifier (default: "cost-attribution-agent")
- `RUVECTOR_SERVICE_URL`: RuVector service endpoint for DecisionEvents
- `OBSERVATORY_URL`: LLM-Observatory endpoint for telemetry
- `AGENT_VERSION`: Agent version for telemetry (default: "1.0.0")

## Development

### Build
```bash
npm install
npm run build
```

### Test
```bash
npm test
npm run test:coverage
```

### Local Development
```bash
npm run watch
```

### Deploy to Google Cloud

```bash
# Development
npm run deploy

# Production
npm run deploy:prod
```

## Supported Providers

### OpenAI
- gpt-4
- gpt-4-turbo
- gpt-4o
- gpt-4o-mini
- gpt-3.5-turbo

### Anthropic
- claude-opus-4
- claude-sonnet-4
- claude-haiku-4
- claude-3-opus
- claude-3-sonnet
- claude-3-haiku

### Google
- gemini-pro
- gemini-ultra
- gemini-flash

## Rate Limiting

- 100 requests per minute per IP address
- In-memory rate limiting (resets on function cold start)
- Returns 429 status code when exceeded

## Error Handling

All errors return structured JSON with:
- `error.code`: Error code
- `error.message`: Human-readable message
- `error.details`: Validation errors (if applicable)
- `requestId`: Original request ID (if available)
- `timestamp`: Error timestamp

### Error Codes
- `VALIDATION_ERROR`: Invalid input data
- `RATE_LIMIT_EXCEEDED`: Rate limit exceeded
- `METHOD_NOT_ALLOWED`: Invalid HTTP method
- `INTERNAL_ERROR`: Unexpected error
- `SERVICE_UNAVAILABLE`: Service temporarily unavailable

## License

MIT
