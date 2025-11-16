# LLM Cost Operations SDK

Enterprise-grade TypeScript SDK for the LLM Cost Operations Platform.

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

## Installation

```bash
npm install @llm-cost-ops/sdk
```

## Quick Start

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  baseUrl: 'https://api.llm-cost-ops.example.com',
  apiKey: 'your-api-key',
});

// Check service health
const health = await client.health();

// Get cost metrics
const metrics = await client.getMetrics({
  startDate: '2024-01-01T00:00:00Z',
  endDate: '2024-01-31T23:59:59Z',
});

// Get usage statistics
const stats = await client.getUsageStats(
  '2024-01-01T00:00:00Z',
  '2024-01-31T23:59:59Z'
);
```

## Configuration

### Client Options

```typescript
interface ClientConfig {
  baseUrl: string;              // API base URL (required)
  apiKey?: string;              // API key for authentication
  timeout?: number;             // Request timeout (default: 30000ms)
  maxRetries?: number;          // Max retry attempts (default: 3)
  retryDelay?: number;          // Initial retry delay (default: 1000ms)
  exponentialBackoff?: boolean; // Enable exponential backoff (default: true)
  headers?: Record<string, string>; // Custom headers
  debug?: boolean;              // Enable debug logging (default: false)
}
```

### Example Configuration

```typescript
const client = new CostOpsClient({
  baseUrl: 'https://api.llm-cost-ops.example.com',
  apiKey: process.env.API_KEY,
  timeout: 60000,
  maxRetries: 5,
  exponentialBackoff: true,
  debug: process.env.NODE_ENV === 'development',
  headers: {
    'X-Custom-Header': 'value',
  },
});
```

## API Methods

### Metrics

```typescript
// Get metrics with filtering
const metrics = await client.getMetrics({
  startDate: '2024-01-01T00:00:00Z',
  endDate: '2024-01-31T23:59:59Z',
  services: ['gpt-4', 'claude-3'],
  limit: 100,
  sortBy: 'cost',
  sortOrder: 'desc',
});

// Create a metric
const metric = await client.createMetric({
  service: 'gpt-4',
  cost: 0.05,
  currency: 'USD',
  tokens: 1000,
  timestamp: new Date().toISOString(),
});

// Get specific metric
const metric = await client.getMetric('metric-id');

// Delete metric
await client.deleteMetric('metric-id');
```

### Usage Statistics

```typescript
const stats = await client.getUsageStats(
  '2024-01-01T00:00:00Z',
  '2024-01-31T23:59:59Z',
  ['gpt-4', 'claude-3'] // optional service filter
);

console.log(stats.totalCost);
console.log(stats.totalTokens);
console.log(stats.byService['gpt-4'].cost);
```

### Budgets

```typescript
// Get all budgets
const budgets = await client.getBudgets();

// Create budget
const budget = await client.createBudget({
  name: 'Monthly AI Budget',
  amount: 1000,
  currency: 'USD',
  period: 'monthly',
  alertThreshold: 80,
});

// Update budget
await client.updateBudget('budget-id', {
  amount: 1500,
  alertThreshold: 90,
});

// Delete budget
await client.deleteBudget('budget-id');
```

### Alerts

```typescript
// Get alerts
const alerts = await client.getAlerts();
const unacknowledged = await client.getAlerts(false);

// Acknowledge alert
await client.acknowledgeAlert('alert-id');
```

### Forecasting

```typescript
const forecast = await client.getForecast('month', ['gpt-4']);

console.log(forecast.predictedCost);
console.log(forecast.confidenceInterval);
```

### Export

```typescript
const result = await client.exportData({
  format: 'xlsx',
  dateRange: {
    start: '2024-01-01T00:00:00Z',
    end: '2024-01-31T23:59:59Z',
  },
  services: ['gpt-4'],
  detailed: true,
});

console.log(result.downloadUrl);
```

### Webhooks

```typescript
// Create webhook
const webhook = await client.createWebhook({
  url: 'https://example.com/webhook',
  events: ['budget.exceeded', 'cost.spike'],
  active: true,
});

// Test webhook
const testResult = await client.testWebhook('webhook-id');
```

## Error Handling

The SDK provides comprehensive error handling with custom error classes:

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

### Error Types

- `CostOpsError` - Base error class
- `ConfigurationError` - Invalid client configuration
- `ValidationError` - Invalid request parameters
- `AuthenticationError` - Authentication failed
- `AuthorizationError` - Insufficient permissions
- `ApiError` - API error response
- `NetworkError` - Network request failed
- `TimeoutError` - Request timeout
- `RateLimitError` - Rate limit exceeded
- `NotFoundError` - Resource not found
- `ConflictError` - Resource conflict
- `ServerError` - Server error (5xx)
- `RetryExhaustedError` - All retries failed

## Middleware & Interceptors

Add custom request/response processing:

```typescript
const middleware = client.getMiddleware();

// Request interceptor
middleware.addRequestInterceptor((context) => {
  console.log('Request:', context.request.path);
  context.request.headers = {
    ...context.request.headers,
    'X-Custom': 'value',
  };
  return context;
});

// Response interceptor
middleware.addResponseInterceptor((context) => {
  console.log('Duration:', context.response.metadata.duration);
  return context;
});

// Error interceptor
middleware.addErrorInterceptor((context) => {
  console.error('Error:', context.error);
  return context;
});
```

### Built-in Interceptors

```typescript
import {
  createLoggingInterceptor,
  createAuthInterceptor,
  createMetricsInterceptor,
} from '@llm-cost-ops/sdk';

// Metrics tracking
const metricsInterceptor = createMetricsInterceptor();
middleware.addRequestInterceptor(metricsInterceptor.request);
middleware.addResponseInterceptor(metricsInterceptor.response);

const metrics = metricsInterceptor.getMetrics();
console.log('Total requests:', metrics.totalRequests);
console.log('Average duration:', metrics.averageDuration);
```

## Events

Monitor SDK operations:

```typescript
const middleware = client.getMiddleware();

middleware.on('request:start', (context) => {
  console.log('Starting request:', context.request.path);
});

middleware.on('request:end', (context) => {
  console.log('Request completed:', context.response.status);
});

middleware.on('request:error', (context) => {
  console.error('Request failed:', context.error);
});

middleware.on('retry:attempt', (attempt, error) => {
  console.log(`Retry ${attempt}:`, error.message);
});
```

## TypeScript Support

Full TypeScript support with strict typing:

```typescript
import type {
  CostMetric,
  Budget,
  UsageStats,
  MetricsQuery,
} from '@llm-cost-ops/sdk';

const query: MetricsQuery = {
  startDate: '2024-01-01T00:00:00Z',
  endDate: '2024-01-31T23:59:59Z',
  limit: 10,
};

const metrics = await client.getMetrics(query);
// metrics is typed as PaginatedResponse<CostMetric>
```

## Browser Support

The SDK works in both Node.js and browser environments:

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

## Testing

The SDK includes comprehensive tests:

```bash
# Run tests
npm test

# Watch mode
npm run test:watch

# Coverage
npm run test:coverage
```

## Development

```bash
# Install dependencies
npm install

# Build
npm run build

# Type check
npm run typecheck

# Lint
npm run lint

# Format
npm run format

# Verify (typecheck + lint + test)
npm run verify
```

## License

Apache-2.0

## Support

For issues and questions, please file an issue on GitHub or contact support.
