# Cost Attribution Agent - Quick Reference

## File Structure

```
/workspaces/cost-ops/agents/cost-attribution/
├── src/
│   ├── handler/
│   │   ├── index.ts          # Main HTTP handler
│   │   ├── middleware.ts     # Validation, rate limiting, logging
│   │   └── response.ts       # Response formatters
│   ├── index.ts              # Entry point & exports
│   ├── types.ts              # TypeScript type definitions
│   ├── schemas.ts            # Zod validation schemas
│   ├── calculator.ts         # Cost calculation engine
│   └── attributor.ts         # Cost attribution engine
├── tests/
│   ├── calculator.test.ts    # Calculator unit tests
│   └── attributor.test.ts    # Attributor unit tests
├── package.json              # Dependencies & scripts
├── tsconfig.json             # TypeScript configuration
├── jest.config.js            # Test configuration
├── README.md                 # Full documentation
├── IMPLEMENTATION_SUMMARY.md # Implementation details
├── example-request.json      # Sample request
├── .env.example              # Environment template
├── .gitignore                # Git ignore rules
├── .gcloudignore             # Deployment ignore rules
└── verify-structure.sh       # Structure verification script
```

## Commands

```bash
# Install dependencies
npm install

# Run tests
npm test
npm run test:coverage

# Build TypeScript
npm run build
npm run watch

# Code quality
npm run lint
npm run format

# Deploy to Google Cloud
npm run deploy          # Development
npm run deploy:prod     # Production

# Verify structure
./verify-structure.sh
```

## Core Components

### 1. Handler (`src/handler/index.ts`)
- **Function**: `costAttributionHandler(req, res)`
- **Method**: POST only
- **Input**: CostAttributionInput (Zod validated)
- **Output**: CostAttributionOutput (JSON)
- **Emissions**: DecisionEvent → RuVector, Telemetry → Observatory

### 2. Middleware (`src/handler/middleware.ts`)
- `validateInput()` - Zod schema validation
- `checkRateLimit()` - 100 req/min per IP
- `logRequest()` - Structured JSON logging
- `handleError()` - Error handling & logging
- `validateMethod()` - POST-only enforcement
- `setCorsHeaders()` / `handleOptions()` - CORS support

### 3. Response Formatters (`src/handler/response.ts`)
- `sendSuccess(res, output)` - 200 response
- `sendValidationError(res, msg, details)` - 400 response
- `sendRateLimitError(res, msg)` - 429 response
- `sendInternalError(res, msg)` - 500 response

### 4. Cost Calculator (`src/calculator.ts`)
- **Method**: `calculate(usage, pricingContext)`
- **Returns**: CostBreakdown
- **Pricing**: 15+ models across 3 providers
- **Precision**: 6 decimal places

### 5. Cost Attributor (`src/attributor.ts`)
- **Method**: `attribute(dimensions)`
- **Returns**: AttributionResult
- **Priority**: org > project > user > environment
- **Confidence**: 0-1 score based on dimensions

## Request Format

```json
{
  "requestId": "<uuid>",
  "timestamp": "<ISO 8601>",
  "usage": {
    "provider": "anthropic|openai|google",
    "model": "<model-name>",
    "inputTokens": <int>,
    "outputTokens": <int>,
    "cachedTokens": <int> (optional),
    "latencyMs": <number> (optional)
  },
  "pricingContext": {
    "tier": "<string>" (optional),
    "currency": "USD" (optional),
    "customPricing": { ... } (optional)
  },
  "dimensions": {
    "userId": "<string>" (optional),
    "projectId": "<string>" (optional),
    "organizationId": "<string>" (optional),
    "environment": "<string>" (optional),
    "tags": { ... } (optional)
  }
}
```

## Response Format

```json
{
  "requestId": "<uuid>",
  "analysisTimestamp": "<ISO 8601>",
  "costs": {
    "totalCost": <number>,
    "inputCost": <number>,
    "outputCost": <number>,
    "cachedCost": <number> (optional),
    "currency": "USD",
    "costPer1kTokens": <number>
  },
  "attribution": {
    "primary": "<dimension>:<value>",
    "dimensions": { ... },
    "tags": { ... },
    "confidence": <0-1>
  },
  "decisionEvent": { ... },
  "telemetry": { ... }
}
```

## Environment Variables

```bash
# Required for production
RUVECTOR_SERVICE_URL=https://ruvector.example.com
OBSERVATORY_URL=https://observatory.example.com

# Optional
AGENT_ID=cost-attribution-agent
AGENT_VERSION=1.0.0
```

## Supported Models

### OpenAI (5 models)
- gpt-4, gpt-4-turbo, gpt-4o, gpt-4o-mini, gpt-3.5-turbo

### Anthropic (6 models + cache)
- claude-opus-4, claude-sonnet-4, claude-haiku-4
- claude-3-opus, claude-3-sonnet, claude-3-haiku

### Google (3 models)
- gemini-pro, gemini-ultra, gemini-flash

## Testing

```bash
# Run all tests
npm test

# Watch mode
npm run test:watch

# Coverage (requires >80%)
npm run test:coverage

# Test a specific file
npx jest calculator.test.ts
```

## Error Codes

- `VALIDATION_ERROR` (400) - Invalid input
- `METHOD_NOT_ALLOWED` (405) - Not POST
- `RATE_LIMIT_EXCEEDED` (429) - >100 req/min
- `INTERNAL_ERROR` (500) - Unexpected error
- `SERVICE_UNAVAILABLE` (503) - Service down

## Rate Limiting

- **Limit**: 100 requests per minute per IP
- **Window**: 60 seconds rolling
- **Storage**: In-memory (resets on cold start)
- **Response**: 429 with error message

## Deployment Checklist

- [ ] Set environment variables
- [ ] Configure RUVECTOR_SERVICE_URL
- [ ] Configure OBSERVATORY_URL
- [ ] Run tests (`npm test`)
- [ ] Check coverage (`npm run test:coverage`)
- [ ] Build (`npm run build`)
- [ ] Deploy (`npm run deploy` or `npm run deploy:prod`)
- [ ] Test deployed endpoint
- [ ] Monitor logs in Google Cloud Console

## Development Workflow

1. Edit TypeScript files in `src/`
2. Run tests: `npm test`
3. Build: `npm run build`
4. Check output in `dist/`
5. Deploy: `npm run deploy`

## Debugging

```bash
# View function logs
gcloud functions logs read cost-attribution-handler --limit 50

# Test locally with Functions Framework
npm install @google-cloud/functions-framework
npx functions-framework --target=costAttributionHandler --port=8080

# Test with curl
curl -X POST http://localhost:8080 \
  -H "Content-Type: application/json" \
  -d @example-request.json
```

## Key Design Principles

✅ Stateless - No runtime state
✅ Deterministic - Same input → same output
✅ Type-safe - TypeScript + Zod validation
✅ Testable - Comprehensive unit tests
✅ Observable - Structured logging + telemetry
✅ Resilient - Error handling + timeout protection
✅ Scalable - Edge function, auto-scaling

## Performance Targets

- Cold start: ~500ms
- Warm invocation: <50ms (calc + attribution)
- External calls: <5s each (timeout protected)
- Total latency: <100ms (excluding emissions)

## Links

- Main Docs: `README.md`
- Implementation: `IMPLEMENTATION_SUMMARY.md`
- Example: `example-request.json`
- Tests: `tests/`
- Source: `src/`
