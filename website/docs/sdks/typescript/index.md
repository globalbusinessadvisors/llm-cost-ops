---
sidebar_position: 2
title: TypeScript SDK
---

# TypeScript SDK

Enterprise-grade TypeScript SDK for the LLM Cost Operations Platform. Works in both Node.js and browser environments with full type safety.

## Features

- **Full TypeScript Support** - Strict typing with comprehensive type definitions
- **Universal Compatibility** - Works in Node.js and browser environments
- **Tree-Shakeable** - Optimized bundle size with ES modules
- **Enterprise-Grade Quality** - Production-ready with extensive testing
- **Retry Logic** - Built-in exponential backoff for failed requests
- **Interceptors** - Request/response middleware system
- **Event Emitters** - Monitor SDK operations in real-time
- **Custom Error Classes** - Detailed error handling and type guards
- **Zero Dependencies** - Minimal footprint (only EventEmitter3)

## Quick Example

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

// Initialize client
const client = new CostOpsClient({
  baseUrl: 'https://api.llm-cost-ops.example.com',
  apiKey: 'your-api-key',
});

// Submit usage data
const usage = await client.submitUsage({
  organizationId: 'org-123',
  provider: 'openai',
  modelId: 'gpt-4',
  inputTokens: 1000,
  outputTokens: 500,
  totalTokens: 1500,
});

console.log('Usage tracked:', usage.usageId);
console.log('Estimated cost:', usage.estimatedCost);

// Get cost metrics
const costs = await client.getCosts({
  organizationId: 'org-123',
  startDate: '2025-01-01T00:00:00Z',
  endDate: '2025-01-31T23:59:59Z',
});

console.log('Total cost:', costs.totalCost);
console.log('Total requests:', costs.totalRequests);
```

## Installation

```bash
# Using npm
npm install @llm-cost-ops/sdk

# Using yarn
yarn add @llm-cost-ops/sdk

# Using pnpm
pnpm add @llm-cost-ops/sdk
```

## Requirements

- Node.js 16.0 or higher (for Node.js environments)
- TypeScript 4.5 or higher (for TypeScript projects)

## Package Structure

```
@llm-cost-ops/sdk/
├── dist/
│   ├── index.js           # CommonJS bundle
│   ├── index.mjs          # ES module bundle
│   └── index.d.ts         # TypeScript definitions
├── src/
│   ├── client.ts          # Main client
│   ├── config.ts          # Configuration
│   ├── errors.ts          # Error classes
│   ├── types.ts           # Type definitions
│   ├── middleware.ts      # Interceptor system
│   └── resources/         # API resources
│       ├── usage.ts
│       ├── costs.ts
│       ├── pricing.ts
│       └── analytics.ts
```

## Core Concepts

### Client Configuration

```typescript
import { CostOpsClient, ClientConfig } from '@llm-cost-ops/sdk';

const config: ClientConfig = {
  baseUrl: 'https://api.llm-cost-ops.example.com',
  apiKey: 'your-api-key',
  timeout: 30000,              // Request timeout in ms
  maxRetries: 3,               // Max retry attempts
  retryDelay: 1000,            // Initial retry delay in ms
  exponentialBackoff: true,    // Enable exponential backoff
  debug: false,                // Enable debug logging
  headers: {                   // Custom headers
    'X-Custom-Header': 'value',
  },
};

const client = new CostOpsClient(config);
```

### Type Safety

Full TypeScript support with intellisense:

```typescript
import type {
  CostMetric,
  UsageRecord,
  MetricsQuery,
} from '@llm-cost-ops/sdk';

const query: MetricsQuery = {
  organizationId: 'org-123',
  startDate: '2025-01-01T00:00:00Z',
  endDate: '2025-01-31T23:59:59Z',
  limit: 100,
};

const metrics: CostMetric[] = await client.getMetrics(query);
```

### Error Handling

```typescript
import {
  CostOpsClient,
  AuthenticationError,
  ValidationError,
  RateLimitError,
  NotFoundError,
  isCostOpsError,
} from '@llm-cost-ops/sdk';

try {
  await client.getMetric('id');
} catch (error) {
  if (error instanceof NotFoundError) {
    console.log('Resource not found');
  } else if (error instanceof AuthenticationError) {
    console.error('Invalid API key');
  } else if (error instanceof RateLimitError) {
    console.log('Retry after:', error.retryAfter);
  } else if (isCostOpsError(error)) {
    console.error('SDK error:', error.toJSON());
  }
}
```

## Browser Support

The SDK works in modern browsers:

```html
<script type="module">
  import { CostOpsClient } from '@llm-cost-ops/sdk';

  const client = new CostOpsClient({
    baseUrl: 'https://api.llm-cost-ops.example.com',
    apiKey: 'your-api-key',
  });

  const health = await client.health();
  console.log(health);
</script>
```

## Next Steps

- [Installation Guide](/docs/sdks/typescript/installation)
- [Quick Start](/docs/sdks/typescript/quick-start)
- [API Reference](/docs/sdks/typescript/api-reference)
- [Examples](/docs/sdks/typescript/examples)
- [Troubleshooting](/docs/sdks/typescript/troubleshooting)

## Resources

- [NPM Package](https://www.npmjs.com/package/@llm-cost-ops/sdk)
- [Source Code](https://github.com/llm-devops/llm-cost-ops/tree/main/sdk)
- [Examples](https://github.com/llm-devops/llm-cost-ops/tree/main/sdk/examples)
- [Changelog](https://github.com/llm-devops/llm-cost-ops/blob/main/sdk/CHANGELOG.md)

## Support

- [GitHub Issues](https://github.com/llm-devops/llm-cost-ops/issues)
- [Discord Community](https://discord.gg/llm-cost-ops)
- Email: support@llm-cost-ops.dev
