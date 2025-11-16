# TypeScript SDK Tutorial

## Table of Contents
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [TypeScript Configuration](#typescript-configuration)
- [Basic Usage](#basic-usage)
- [Promise-Based API](#promise-based-api)
- [Resource Management](#resource-management)
- [Error Handling](#error-handling)
- [Request/Response Interceptors](#requestresponse-interceptors)
- [Middleware Usage](#middleware-usage)
- [Retry and Timeout Configuration](#retry-and-timeout-configuration)
- [Pagination](#pagination)
- [Testing](#testing)
- [Framework Integration](#framework-integration)
- [WebSocket Support](#websocket-support)
- [Advanced Patterns](#advanced-patterns)
- [Performance Optimization](#performance-optimization)

## Prerequisites

Before getting started with the TypeScript SDK, ensure you have:

- Node.js 16.x or higher
- TypeScript 4.5 or higher
- npm, yarn, or pnpm package manager
- API key from LLM Cost Ops platform
- Basic understanding of TypeScript and async/await

## Installation

### Using npm

```bash
npm install @llm-cost-ops/sdk
```

### Using yarn

```bash
yarn add @llm-cost-ops/sdk
```

### Using pnpm

```bash
pnpm add @llm-cost-ops/sdk
```

### Verify Installation

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';
console.log('SDK loaded successfully');
```

### Environment Setup

Create a `.env` file in your project root:

```bash
LLM_COST_OPS_API_KEY=your_api_key_here
LLM_COST_OPS_BASE_URL=https://api.llmcostops.com
LLM_COST_OPS_TIMEOUT=30000
```

## TypeScript Configuration

### Basic tsconfig.json

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "lib": ["ES2020"],
    "declaration": true,
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "resolveJsonModule": true,
    "moduleResolution": "node"
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

### For ESM Projects

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "node",
    "esModuleInterop": true,
    "strict": true
  }
}
```

### Type Declarations

```typescript
// types.d.ts
declare module '@llm-cost-ops/sdk' {
  export class CostOpsClient {
    constructor(config: ClientConfig);
    usage: UsageResource;
    costs: CostsResource;
    pricing: PricingResource;
    analytics: AnalyticsResource;
    budgets: BudgetsResource;
    export: ExportResource;
    health: HealthResource;
  }

  export interface ClientConfig {
    apiKey: string;
    baseUrl?: string;
    timeout?: number;
    retries?: number;
  }
}
```

## Basic Usage

### Client Initialization

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';
import dotenv from 'dotenv';

// Load environment variables
dotenv.config();

// Initialize client
const client = new CostOpsClient({
  apiKey: process.env.LLM_COST_OPS_API_KEY!,
  baseUrl: process.env.LLM_COST_OPS_BASE_URL,
  timeout: 30000,
  retries: 3
});
```

### Simple Cost Query

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  apiKey: 'your_api_key'
});

async function getCosts() {
  try {
    const endDate = new Date();
    const startDate = new Date(endDate);
    startDate.setDate(startDate.getDate() - 7);

    const costs = await client.costs.getCosts({
      startDate: startDate.toISOString(),
      endDate: endDate.toISOString()
    });

    console.log(`Total cost: $${costs.totalCost.toFixed(2)}`);

    costs.items.forEach(item => {
      console.log(`${item.date}: $${item.amount.toFixed(2)}`);
    });
  } catch (error) {
    console.error('Error fetching costs:', error);
  }
}

getCosts();
```

### Usage Tracking

```typescript
interface UsageMetadata {
  userId?: string;
  sessionId?: string;
  application?: string;
  [key: string]: any;
}

async function trackUsage() {
  const usage = await client.usage.createUsage({
    model: 'gpt-4',
    tokensPrompt: 1000,
    tokensCompletion: 500,
    requestCount: 1,
    timestamp: new Date().toISOString(),
    metadata: {
      userId: 'user_123',
      sessionId: 'session_456'
    } as UsageMetadata
  });

  console.log(`Usage ID: ${usage.id}`);
  console.log(`Cost: $${usage.cost.toFixed(4)}`);
}
```

### Get Pricing Information

```typescript
async function getPricing() {
  const pricing = await client.pricing.getModelPricing({
    model: 'gpt-4',
    provider: 'openai'
  });

  console.log(`Model: ${pricing.model}`);
  console.log(`Prompt: $${pricing.promptPricePer1k} per 1K tokens`);
  console.log(`Completion: $${pricing.completionPricePer1k} per 1K tokens`);
}
```

## Promise-Based API

### Basic Promises

```typescript
// Promise chaining
client.costs.getCosts({
  startDate: '2025-01-01',
  endDate: '2025-01-31'
})
  .then(costs => {
    console.log(`Total: $${costs.totalCost}`);
    return client.analytics.getUsageAnalytics({
      startDate: '2025-01-01',
      endDate: '2025-01-31'
    });
  })
  .then(analytics => {
    console.log(`Total requests: ${analytics.totalRequests}`);
  })
  .catch(error => {
    console.error('Error:', error);
  });
```

### Async/Await

```typescript
async function fetchData() {
  try {
    const [costs, usage, analytics] = await Promise.all([
      client.costs.getCosts({
        startDate: '2025-01-01',
        endDate: '2025-01-31'
      }),
      client.usage.getUsage({
        startDate: '2025-01-01',
        endDate: '2025-01-31'
      }),
      client.analytics.getUsageAnalytics({
        startDate: '2025-01-01',
        endDate: '2025-01-31'
      })
    ]);

    console.log(`Total cost: $${costs.totalCost}`);
    console.log(`Total requests: ${usage.totalRequests}`);
    console.log(`Analytics groups: ${analytics.groups.length}`);
  } catch (error) {
    console.error('Error fetching data:', error);
  }
}
```

### Promise.allSettled for Resilience

```typescript
async function fetchAllData() {
  const results = await Promise.allSettled([
    client.costs.getCosts({ startDate: '2025-01-01', endDate: '2025-01-31' }),
    client.usage.getUsage({ startDate: '2025-01-01', endDate: '2025-01-31' }),
    client.analytics.getUsageAnalytics({ startDate: '2025-01-01', endDate: '2025-01-31' })
  ]);

  results.forEach((result, index) => {
    if (result.status === 'fulfilled') {
      console.log(`Request ${index} succeeded`);
    } else {
      console.error(`Request ${index} failed:`, result.reason);
    }
  });
}
```

### Race Conditions

```typescript
// Fetch from primary and backup endpoints, use whichever responds first
async function fetchWithFallback() {
  const primaryRequest = client.costs.getCosts({
    startDate: '2025-01-01',
    endDate: '2025-01-31'
  });

  const backupRequest = new Promise((resolve, reject) => {
    setTimeout(() => {
      // Fallback to cached data
      resolve({ totalCost: 0, items: [] });
    }, 5000);
  });

  const result = await Promise.race([primaryRequest, backupRequest]);
  return result;
}
```

## Resource Management

### Usage Resource

```typescript
import {
  UsageCreateRequest,
  UsageResponse,
  UsageListResponse
} from '@llm-cost-ops/sdk';

class UsageManager {
  constructor(private client: CostOpsClient) {}

  async createUsage(data: UsageCreateRequest): Promise<UsageResponse> {
    return await this.client.usage.createUsage(data);
  }

  async getUsageById(id: string): Promise<UsageResponse> {
    return await this.client.usage.getUsageById(id);
  }

  async listUsage(params: {
    startDate: string;
    endDate: string;
    filters?: Record<string, any>;
  }): Promise<UsageListResponse> {
    return await this.client.usage.getUsage(params);
  }

  async updateUsage(
    id: string,
    metadata: Record<string, any>
  ): Promise<UsageResponse> {
    return await this.client.usage.updateUsage(id, { metadata });
  }

  async deleteUsage(id: string): Promise<void> {
    await this.client.usage.deleteUsage(id);
  }
}

// Usage
const usageManager = new UsageManager(client);

const usage = await usageManager.createUsage({
  model: 'gpt-4',
  tokensPrompt: 1000,
  tokensCompletion: 500,
  requestCount: 1,
  metadata: {
    userId: 'user_123',
    application: 'chatbot'
  }
});
```

### Costs Resource

```typescript
interface CostQuery {
  startDate: string;
  endDate: string;
  groupBy?: string[];
  granularity?: 'hourly' | 'daily' | 'weekly' | 'monthly';
}

class CostsManager {
  constructor(private client: CostOpsClient) {}

  async getCosts(query: CostQuery) {
    const costs = await this.client.costs.getCosts(query);

    console.log(`Total cost: $${costs.totalCost.toFixed(2)}`);

    costs.items.forEach(item => {
      console.log(`${item.model} (${item.provider}): $${item.amount.toFixed(2)}`);
    });

    return costs;
  }

  async getCostBreakdown(query: CostQuery) {
    const breakdown = await this.client.costs.getCostBreakdown(query);

    breakdown.items.forEach(item => {
      console.log(`${item.date}: $${item.amount.toFixed(2)}`);
    });

    return breakdown;
  }

  async getCostTrends(days: number = 30, includeForecast: boolean = true) {
    const trends = await this.client.costs.getCostTrends({
      days,
      includeForecast
    });

    console.log(`30-day trend: ${trends.trendPercentage > 0 ? '+' : ''}${trends.trendPercentage.toFixed(2)}%`);

    if (includeForecast) {
      console.log(`Forecasted next month: $${trends.forecast.toFixed(2)}`);
    }

    return trends;
  }
}
```

### Pricing Resource

```typescript
class PricingManager {
  constructor(private client: CostOpsClient) {}

  async listPricing(params: { provider?: string; modelType?: string }) {
    const pricingList = await this.client.pricing.listPricing(params);

    pricingList.items.forEach(pricing => {
      console.log(`${pricing.model}: $${pricing.promptPricePer1k}/1K prompt tokens`);
    });

    return pricingList;
  }

  async getModelPricing(model: string, provider: string) {
    return await this.client.pricing.getModelPricing({ model, provider });
  }

  async estimateCost(params: {
    model: string;
    tokensPrompt: number;
    tokensCompletion: number;
  }) {
    const estimate = await this.client.pricing.estimateCost(params);

    console.log(`Estimated cost: $${estimate.totalCost.toFixed(4)}`);
    console.log(`Prompt cost: $${estimate.promptCost.toFixed(4)}`);
    console.log(`Completion cost: $${estimate.completionCost.toFixed(4)}`);

    return estimate;
  }

  async updateCustomPricing(params: {
    model: string;
    promptPricePer1k: number;
    completionPricePer1k: number;
    metadata?: Record<string, any>;
  }) {
    return await this.client.pricing.updatePricing(params);
  }
}
```

### Analytics Resource

```typescript
interface AnalyticsQuery {
  startDate: string;
  endDate: string;
  groupBy?: string[];
  metrics?: string[];
}

class AnalyticsManager {
  constructor(private client: CostOpsClient) {}

  async getUsageAnalytics(query: AnalyticsQuery) {
    const analytics = await this.client.analytics.getUsageAnalytics(query);

    analytics.groups.forEach(group => {
      console.log(`${group.key}:`);
      console.log(`  Tokens: ${group.totalTokens.toLocaleString()}`);
      console.log(`  Cost: $${group.totalCost.toFixed(2)}`);
      console.log(`  Requests: ${group.requestCount.toLocaleString()}`);
    });

    return analytics;
  }

  async getCostAnalytics(query: AnalyticsQuery & { includeComparison?: boolean }) {
    return await this.client.analytics.getCostAnalytics(query);
  }

  async getPerformanceMetrics(query: { startDate: string; endDate: string }) {
    const performance = await this.client.analytics.getPerformanceMetrics(query);

    console.log(`Average latency: ${performance.avgLatencyMs.toFixed(2)}ms`);
    console.log(`Success rate: ${(performance.successRate * 100).toFixed(2)}%`);
    console.log(`P95 latency: ${performance.p95LatencyMs.toFixed(2)}ms`);

    return performance;
  }

  async getTopConsumers(params: {
    startDate: string;
    endDate: string;
    limit?: number;
    metric?: 'totalCost' | 'totalTokens' | 'requestCount';
  }) {
    const topUsers = await this.client.analytics.getTopConsumers(params);

    topUsers.items.forEach(user => {
      console.log(`${user.userId}: $${user.totalCost.toFixed(2)}`);
    });

    return topUsers;
  }
}
```

### Budget Resource

```typescript
interface BudgetAlert {
  threshold: number;
  type: 'email' | 'slack' | 'pagerduty' | 'webhook';
  config?: Record<string, any>;
}

interface BudgetCreateRequest {
  name: string;
  amount: number;
  period: 'daily' | 'weekly' | 'monthly' | 'yearly';
  startDate: string;
  alerts?: BudgetAlert[];
  filters?: Record<string, any>;
}

class BudgetManager {
  constructor(private client: CostOpsClient) {}

  async createBudget(data: BudgetCreateRequest) {
    return await this.client.budgets.createBudget(data);
  }

  async listBudgets(activeOnly: boolean = true) {
    const budgets = await this.client.budgets.listBudgets({ activeOnly });

    budgets.items.forEach(budget => {
      const percentage = budget.percentageUsed;
      const status = percentage < 50 ? '🟢' : percentage < 80 ? '🟡' : '🔴';

      console.log(
        `${status} ${budget.name}: $${budget.spent.toFixed(2)} / $${budget.amount.toFixed(2)} ` +
        `(${percentage.toFixed(1)}%)`
      );
    });

    return budgets;
  }

  async getBudget(id: string) {
    return await this.client.budgets.getBudget(id);
  }

  async updateBudget(id: string, updates: Partial<BudgetCreateRequest>) {
    return await this.client.budgets.updateBudget(id, updates);
  }

  async deleteBudget(id: string) {
    await this.client.budgets.deleteBudget(id);
  }

  async getBudgetAlerts(params: {
    budgetId: string;
    startDate: string;
    endDate: string;
  }) {
    const alerts = await this.client.budgets.getBudgetAlerts(params);

    alerts.items.forEach(alert => {
      console.log(`${alert.timestamp}: ${alert.type} - ${alert.message}`);
    });

    return alerts;
  }
}
```

### Export Resource

```typescript
type ExportFormat = 'csv' | 'json' | 'parquet';

interface ExportCreateRequest {
  format: ExportFormat;
  startDate: string;
  endDate: string;
  includeUsage?: boolean;
  includeCosts?: boolean;
  includeAnalytics?: boolean;
  filters?: Record<string, any>;
}

class ExportManager {
  constructor(private client: CostOpsClient) {}

  async createExport(data: ExportCreateRequest) {
    const exportJob = await this.client.export.createExport(data);

    console.log(`Export job ID: ${exportJob.id}`);
    console.log(`Status: ${exportJob.status}`);

    return exportJob;
  }

  async waitForExport(exportId: string, pollInterval: number = 5000): Promise<string> {
    while (true) {
      const status = await this.client.export.getExportStatus(exportId);

      console.log(`Progress: ${status.progress}%`);

      if (status.status === 'completed') {
        return await this.client.export.getExportDownloadUrl(exportId);
      }

      if (status.status === 'failed') {
        throw new Error(`Export failed: ${status.error}`);
      }

      await new Promise(resolve => setTimeout(resolve, pollInterval));
    }
  }

  async downloadExport(exportId: string, filename: string) {
    await this.client.export.downloadExport(exportId, filename);
    console.log(`Downloaded to ${filename}`);
  }

  async listExports(params?: { status?: string; limit?: number }) {
    return await this.client.export.listExports(params);
  }

  async deleteExport(id: string) {
    await this.client.export.deleteExport(id);
  }
}

// Usage
const exportManager = new ExportManager(client);

const exportJob = await exportManager.createExport({
  format: 'csv',
  startDate: '2025-01-01',
  endDate: '2025-01-31',
  includeUsage: true,
  includeCosts: true,
  filters: { model: 'gpt-4' }
});

const downloadUrl = await exportManager.waitForExport(exportJob.id);
console.log(`Download URL: ${downloadUrl}`);
```

### Health Resource

```typescript
class HealthManager {
  constructor(private client: CostOpsClient) {}

  async checkHealth() {
    const health = await this.client.health.checkHealth();

    console.log(`Status: ${health.status}`);
    console.log(`Version: ${health.version}`);
    console.log(`Uptime: ${health.uptimeSeconds}s`);

    return health;
  }

  async getHealthMetrics() {
    const metrics = await this.client.health.getHealthMetrics();

    console.log(`Request rate: ${metrics.requestsPerSecond.toFixed(2)} req/s`);
    console.log(`Error rate: ${(metrics.errorRate * 100).toFixed(2)}%`);
    console.log(`Average latency: ${metrics.avgLatencyMs.toFixed(2)}ms`);

    return metrics;
  }

  async checkServices() {
    const services = ['database', 'cache', 'queue'];
    const healthChecks = await Promise.all(
      services.map(service => this.client.health.checkService(service))
    );

    services.forEach((service, index) => {
      const health = healthChecks[index];
      const status = health.healthy ? '✓' : '✗';
      console.log(`${status} ${service}: ${health.status}`);
    });

    return healthChecks;
  }
}
```

## Error Handling

### Custom Error Classes

```typescript
import {
  CostOpsError,
  AuthenticationError,
  AuthorizationError,
  ResourceNotFoundError,
  ValidationError,
  RateLimitError,
  ServerError,
  NetworkError,
  TimeoutError
} from '@llm-cost-ops/sdk';

async function handleErrors() {
  try {
    const costs = await client.costs.getCosts({
      startDate: '2025-01-01',
      endDate: '2025-01-31'
    });
    return costs;
  } catch (error) {
    if (error instanceof AuthenticationError) {
      console.error('Authentication failed:', error.message);
      // Redirect to login
    } else if (error instanceof AuthorizationError) {
      console.error('Authorization failed:', error.message);
      // Show access denied message
    } else if (error instanceof ResourceNotFoundError) {
      console.error('Resource not found:', error.message);
      // Handle missing resource
    } else if (error instanceof ValidationError) {
      console.error('Validation error:', error.message);
      console.error('Errors:', error.errors);
      // Show validation errors to user
    } else if (error instanceof RateLimitError) {
      console.error('Rate limit exceeded:', error.message);
      console.log(`Retry after: ${error.retryAfter} seconds`);
      // Wait and retry
    } else if (error instanceof TimeoutError) {
      console.error('Request timeout:', error.message);
      // Retry or show timeout message
    } else if (error instanceof ServerError) {
      console.error('Server error:', error.message);
      console.error('Status code:', error.statusCode);
      // Show error message
    } else if (error instanceof NetworkError) {
      console.error('Network error:', error.message);
      // Check connection
    } else if (error instanceof CostOpsError) {
      console.error('General error:', error.message);
      // Generic error handling
    } else {
      console.error('Unexpected error:', error);
      throw error;
    }
  }
}
```

### Error Handler Wrapper

```typescript
type AsyncFunction<T> = (...args: any[]) => Promise<T>;

function withErrorHandling<T>(
  fn: AsyncFunction<T>,
  fallbackValue?: T
): AsyncFunction<T> {
  return async (...args: any[]): Promise<T> => {
    try {
      return await fn(...args);
    } catch (error) {
      if (error instanceof RateLimitError) {
        console.log(`Rate limited, waiting ${error.retryAfter}s`);
        await new Promise(resolve => setTimeout(resolve, error.retryAfter * 1000));
        return await fn(...args);
      }

      if (error instanceof NetworkError || error instanceof TimeoutError) {
        console.log('Network error, retrying in 1s');
        await new Promise(resolve => setTimeout(resolve, 1000));
        return await fn(...args);
      }

      if (fallbackValue !== undefined) {
        console.error('Error occurred, using fallback value:', error);
        return fallbackValue;
      }

      throw error;
    }
  };
}

// Usage
const safeFetchCosts = withErrorHandling(
  (startDate: string, endDate: string) =>
    client.costs.getCosts({ startDate, endDate }),
  { totalCost: 0, items: [] }
);

const costs = await safeFetchCosts('2025-01-01', '2025-01-31');
```

### Retry Decorator

```typescript
function retry(maxRetries: number = 3, delayMs: number = 1000) {
  return function (
    target: any,
    propertyKey: string,
    descriptor: PropertyDescriptor
  ) {
    const originalMethod = descriptor.value;

    descriptor.value = async function (...args: any[]) {
      let lastError: Error;

      for (let i = 0; i < maxRetries; i++) {
        try {
          return await originalMethod.apply(this, args);
        } catch (error) {
          lastError = error as Error;

          if (i < maxRetries - 1) {
            console.log(`Retry ${i + 1}/${maxRetries} after ${delayMs}ms`);
            await new Promise(resolve => setTimeout(resolve, delayMs));
          }
        }
      }

      throw lastError!;
    };

    return descriptor;
  };
}

class ApiService {
  constructor(private client: CostOpsClient) {}

  @retry(3, 1000)
  async fetchCosts(startDate: string, endDate: string) {
    return await this.client.costs.getCosts({ startDate, endDate });
  }
}
```

## Request/Response Interceptors

### Adding Interceptors

```typescript
import { CostOpsClient, RequestInterceptor, ResponseInterceptor } from '@llm-cost-ops/sdk';

// Request interceptor
const requestInterceptor: RequestInterceptor = (config) => {
  // Add custom headers
  config.headers = {
    ...config.headers,
    'X-Client-Version': '1.0.0',
    'X-Request-ID': generateRequestId()
  };

  // Log request
  console.log(`[REQUEST] ${config.method} ${config.url}`);

  return config;
};

// Response interceptor
const responseInterceptor: ResponseInterceptor = (response) => {
  // Log response
  console.log(`[RESPONSE] ${response.status} ${response.config.url}`);

  // Transform response
  if (response.data) {
    response.data.timestamp = new Date().toISOString();
  }

  return response;
};

// Error interceptor
const errorInterceptor = (error: any) => {
  console.error(`[ERROR] ${error.message}`);

  // Transform error
  if (error.response?.status === 401) {
    // Redirect to login
    window.location.href = '/login';
  }

  return Promise.reject(error);
};

// Create client with interceptors
const client = new CostOpsClient({
  apiKey: 'your_api_key',
  interceptors: {
    request: [requestInterceptor],
    response: [responseInterceptor],
    error: [errorInterceptor]
  }
});
```

### Authentication Interceptor

```typescript
class TokenManager {
  private token: string | null = null;
  private refreshToken: string | null = null;

  async getAccessToken(): Promise<string> {
    if (!this.token || this.isTokenExpired(this.token)) {
      await this.refreshAccessToken();
    }
    return this.token!;
  }

  private async refreshAccessToken(): Promise<void> {
    // Refresh token logic
    const response = await fetch('/auth/refresh', {
      method: 'POST',
      body: JSON.stringify({ refreshToken: this.refreshToken })
    });

    const data = await response.json();
    this.token = data.accessToken;
  }

  private isTokenExpired(token: string): boolean {
    // JWT expiration check
    const payload = JSON.parse(atob(token.split('.')[1]));
    return payload.exp * 1000 < Date.now();
  }
}

const tokenManager = new TokenManager();

const authInterceptor: RequestInterceptor = async (config) => {
  const token = await tokenManager.getAccessToken();
  config.headers = {
    ...config.headers,
    'Authorization': `Bearer ${token}`
  };
  return config;
};
```

### Logging Interceptor

```typescript
interface RequestLog {
  timestamp: string;
  method: string;
  url: string;
  duration?: number;
  status?: number;
  error?: string;
}

class RequestLogger {
  private logs: RequestLog[] = [];
  private requestMap = new Map<string, number>();

  onRequest(config: any): any {
    const requestId = this.generateId();
    this.requestMap.set(requestId, Date.now());

    this.logs.push({
      timestamp: new Date().toISOString(),
      method: config.method,
      url: config.url
    });

    config.headers['X-Request-ID'] = requestId;
    return config;
  }

  onResponse(response: any): any {
    const requestId = response.config.headers['X-Request-ID'];
    const startTime = this.requestMap.get(requestId);

    if (startTime) {
      const duration = Date.now() - startTime;
      const log = this.logs.find(l =>
        l.url === response.config.url && !l.duration
      );

      if (log) {
        log.duration = duration;
        log.status = response.status;
      }

      this.requestMap.delete(requestId);
    }

    return response;
  }

  onError(error: any): Promise<never> {
    const log = this.logs.find(l =>
      l.url === error.config?.url && !l.error
    );

    if (log) {
      log.error = error.message;
    }

    return Promise.reject(error);
  }

  private generateId(): string {
    return Math.random().toString(36).substring(7);
  }

  getLogs(): RequestLog[] {
    return this.logs;
  }
}

const logger = new RequestLogger();

const client = new CostOpsClient({
  apiKey: 'your_api_key',
  interceptors: {
    request: [(config) => logger.onRequest(config)],
    response: [(response) => logger.onResponse(response)],
    error: [(error) => logger.onError(error)]
  }
});
```

## Middleware Usage

### Middleware Pattern

```typescript
type Middleware = (
  request: Request,
  next: () => Promise<Response>
) => Promise<Response>;

class MiddlewareChain {
  private middlewares: Middleware[] = [];

  use(middleware: Middleware) {
    this.middlewares.push(middleware);
    return this;
  }

  async execute(request: Request): Promise<Response> {
    let index = 0;

    const next = async (): Promise<Response> => {
      if (index >= this.middlewares.length) {
        return await this.finalHandler(request);
      }

      const middleware = this.middlewares[index++];
      return await middleware(request, next);
    };

    return await next();
  }

  private async finalHandler(request: Request): Promise<Response> {
    // Execute actual request
    return await fetch(request);
  }
}

// Usage
const chain = new MiddlewareChain();

// Logging middleware
chain.use(async (request, next) => {
  console.log(`[${request.method}] ${request.url}`);
  const response = await next();
  console.log(`[${response.status}] ${request.url}`);
  return response;
});

// Authentication middleware
chain.use(async (request, next) => {
  request.headers.set('Authorization', `Bearer ${getToken()}`);
  return await next();
});

// Caching middleware
chain.use(async (request, next) => {
  const cacheKey = request.url;
  const cached = getFromCache(cacheKey);

  if (cached) {
    return cached;
  }

  const response = await next();
  setInCache(cacheKey, response);
  return response;
});
```

### Rate Limiting Middleware

```typescript
class RateLimiter {
  private requests: number[] = [];

  constructor(
    private maxRequests: number,
    private windowMs: number
  ) {}

  async acquire(): Promise<void> {
    const now = Date.now();

    // Remove old requests
    this.requests = this.requests.filter(
      time => time > now - this.windowMs
    );

    if (this.requests.length >= this.maxRequests) {
      const oldestRequest = this.requests[0];
      const waitTime = oldestRequest + this.windowMs - now;
      await new Promise(resolve => setTimeout(resolve, waitTime));
      return this.acquire();
    }

    this.requests.push(now);
  }
}

const rateLimiter = new RateLimiter(100, 60000); // 100 requests per minute

const rateLimitMiddleware: Middleware = async (request, next) => {
  await rateLimiter.acquire();
  return await next();
};
```

## Retry and Timeout Configuration

### Retry Configuration

```typescript
import { CostOpsClient, RetryConfig } from '@llm-cost-ops/sdk';

const retryConfig: RetryConfig = {
  maxRetries: 5,
  backoffFactor: 2,
  retryOnStatus: [429, 500, 502, 503, 504],
  retryOnTimeout: true
};

const client = new CostOpsClient({
  apiKey: 'your_api_key',
  retryConfig
});
```

### Exponential Backoff

```typescript
interface BackoffStrategy {
  calculate(attempt: number): number;
}

class ExponentialBackoff implements BackoffStrategy {
  constructor(
    private base: number = 1000,
    private multiplier: number = 2,
    private maxDelay: number = 60000
  ) {}

  calculate(attempt: number): number {
    const delay = this.base * Math.pow(this.multiplier, attempt);
    return Math.min(delay, this.maxDelay);
  }
}

class JitteredBackoff implements BackoffStrategy {
  constructor(
    private backoff: BackoffStrategy,
    private jitterFactor: number = 0.1
  ) {}

  calculate(attempt: number): number {
    const delay = this.backoff.calculate(attempt);
    const jitter = delay * this.jitterFactor * Math.random();
    return delay + jitter;
  }
}

async function retryWithBackoff<T>(
  fn: () => Promise<T>,
  maxRetries: number = 3,
  backoff: BackoffStrategy = new ExponentialBackoff()
): Promise<T> {
  let lastError: Error;

  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      return await fn();
    } catch (error) {
      lastError = error as Error;

      if (attempt < maxRetries - 1) {
        const delay = backoff.calculate(attempt);
        console.log(`Retry ${attempt + 1}/${maxRetries} after ${delay}ms`);
        await new Promise(resolve => setTimeout(resolve, delay));
      }
    }
  }

  throw lastError!;
}

// Usage
const costs = await retryWithBackoff(
  () => client.costs.getCosts({ startDate: '2025-01-01', endDate: '2025-01-31' }),
  5,
  new JitteredBackoff(new ExponentialBackoff(1000, 2, 60000), 0.1)
);
```

### Timeout Configuration

```typescript
const client = new CostOpsClient({
  apiKey: 'your_api_key',
  timeout: 30000, // 30 seconds
  timeoutConfig: {
    connect: 10000,  // 10 seconds for connection
    read: 30000,     // 30 seconds for reading response
    total: 60000     // 60 seconds total
  }
});

// Per-request timeout
const costs = await client.costs.getCosts({
  startDate: '2025-01-01',
  endDate: '2025-01-31'
}, {
  timeout: 60000  // Override default timeout
});
```

### Timeout Helper

```typescript
async function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  timeoutError?: Error
): Promise<T> {
  let timeoutHandle: NodeJS.Timeout;

  const timeoutPromise = new Promise<never>((_, reject) => {
    timeoutHandle = setTimeout(() => {
      reject(timeoutError || new TimeoutError(`Operation timed out after ${timeoutMs}ms`));
    }, timeoutMs);
  });

  try {
    return await Promise.race([promise, timeoutPromise]);
  } finally {
    clearTimeout(timeoutHandle!);
  }
}

// Usage
const costs = await withTimeout(
  client.costs.getCosts({ startDate: '2025-01-01', endDate: '2025-01-31' }),
  30000,
  new Error('Cost fetch timed out')
);
```

## Pagination

### Manual Pagination

```typescript
async function fetchAllUsage(startDate: string, endDate: string) {
  let page = 1;
  const pageSize = 100;
  const allRecords: any[] = [];

  while (true) {
    const usage = await client.usage.getUsage({
      startDate,
      endDate,
      page,
      pageSize
    });

    allRecords.push(...usage.items);

    console.log(`Page ${page}: ${usage.items.length} records`);

    if (!usage.hasNext) {
      break;
    }

    page++;
  }

  console.log(`Total records: ${allRecords.length}`);
  return allRecords;
}
```

### Async Iterator

```typescript
async function* paginateUsage(
  client: CostOpsClient,
  startDate: string,
  endDate: string,
  pageSize: number = 100
): AsyncGenerator<any, void, undefined> {
  let page = 1;

  while (true) {
    const usage = await client.usage.getUsage({
      startDate,
      endDate,
      page,
      pageSize
    });

    for (const record of usage.items) {
      yield record;
    }

    if (!usage.hasNext) {
      break;
    }

    page++;
  }
}

// Usage
for await (const record of paginateUsage(client, '2025-01-01', '2025-01-31')) {
  console.log(`${record.model}: $${record.cost.toFixed(4)}`);
}
```

### Cursor-Based Pagination

```typescript
async function fetchWithCursor(startDate: string, endDate: string) {
  let cursor: string | undefined;
  const allRecords: any[] = [];

  while (true) {
    const usage = await client.usage.getUsage({
      startDate,
      endDate,
      cursor,
      pageSize: 1000
    });

    allRecords.push(...usage.items);

    if (!usage.nextCursor) {
      break;
    }

    cursor = usage.nextCursor;
  }

  return allRecords;
}
```

### Pagination Helper Class

```typescript
class Paginator<T> {
  constructor(
    private fetchPage: (page: number, pageSize: number) => Promise<{
      items: T[];
      hasNext: boolean;
    }>,
    private pageSize: number = 100
  ) {}

  async *iterate(): AsyncGenerator<T, void, undefined> {
    let page = 1;

    while (true) {
      const result = await this.fetchPage(page, this.pageSize);

      for (const item of result.items) {
        yield item;
      }

      if (!result.hasNext) {
        break;
      }

      page++;
    }
  }

  async fetchAll(): Promise<T[]> {
    const items: T[] = [];

    for await (const item of this.iterate()) {
      items.push(item);
    }

    return items;
  }
}

// Usage
const paginator = new Paginator(
  (page, pageSize) => client.usage.getUsage({
    startDate: '2025-01-01',
    endDate: '2025-01-31',
    page,
    pageSize
  })
);

const allUsage = await paginator.fetchAll();
```

## Testing

### Unit Testing with Vitest

```typescript
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CostOpsClient } from '@llm-cost-ops/sdk';

describe('CostOpsClient', () => {
  let client: CostOpsClient;

  beforeEach(() => {
    client = new CostOpsClient({ apiKey: 'test_key' });
  });

  it('should fetch costs', async () => {
    // Mock the API call
    const mockCosts = {
      totalCost: 100.50,
      items: [
        { date: '2025-01-01', amount: 50.25 },
        { date: '2025-01-02', amount: 50.25 }
      ]
    };

    vi.spyOn(client.costs, 'getCosts').mockResolvedValue(mockCosts);

    const costs = await client.costs.getCosts({
      startDate: '2025-01-01',
      endDate: '2025-01-31'
    });

    expect(costs.totalCost).toBe(100.50);
    expect(costs.items).toHaveLength(2);
  });

  it('should handle errors', async () => {
    vi.spyOn(client.costs, 'getCosts').mockRejectedValue(
      new Error('API Error')
    );

    await expect(
      client.costs.getCosts({
        startDate: '2025-01-01',
        endDate: '2025-01-31'
      })
    ).rejects.toThrow('API Error');
  });
});
```

### Testing with Jest

```typescript
import { CostOpsClient } from '@llm-cost-ops/sdk';

describe('CostOpsClient', () => {
  let client: CostOpsClient;

  beforeEach(() => {
    client = new CostOpsClient({ apiKey: 'test_key' });
  });

  test('creates usage record', async () => {
    const mockUsage = {
      id: 'usage_123',
      model: 'gpt-4',
      cost: 0.03
    };

    jest.spyOn(client.usage, 'createUsage').mockResolvedValue(mockUsage);

    const usage = await client.usage.createUsage({
      model: 'gpt-4',
      tokensPrompt: 1000,
      tokensCompletion: 500
    });

    expect(usage.id).toBe('usage_123');
    expect(usage.cost).toBe(0.03);
  });
});
```

### Integration Testing

```typescript
import { describe, it, expect } from 'vitest';
import { CostOpsClient } from '@llm-cost-ops/sdk';

describe('Integration Tests', () => {
  const client = new CostOpsClient({
    apiKey: process.env.LLM_COST_OPS_TEST_API_KEY!
  });

  it('should create and fetch usage', async () => {
    // Create usage
    const created = await client.usage.createUsage({
      model: 'gpt-4',
      tokensPrompt: 1000,
      tokensCompletion: 500,
      requestCount: 1
    });

    expect(created.id).toBeDefined();
    expect(created.model).toBe('gpt-4');

    // Fetch usage
    const fetched = await client.usage.getUsageById(created.id);

    expect(fetched.id).toBe(created.id);
    expect(fetched.model).toBe(created.model);

    // Cleanup
    await client.usage.deleteUsage(created.id);
  }, 30000); // 30 second timeout
});
```

### Mock Service Worker

```typescript
import { rest } from 'msw';
import { setupServer } from 'msw/node';

const server = setupServer(
  rest.get('https://api.llmcostops.com/v1/costs', (req, res, ctx) => {
    return res(
      ctx.json({
        totalCost: 100.50,
        items: [
          { date: '2025-01-01', amount: 50.25 }
        ]
      })
    );
  }),

  rest.post('https://api.llmcostops.com/v1/usage', (req, res, ctx) => {
    return res(
      ctx.json({
        id: 'usage_123',
        model: 'gpt-4',
        cost: 0.03
      })
    );
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('fetches costs with MSW', async () => {
  const client = new CostOpsClient({ apiKey: 'test_key' });
  const costs = await client.costs.getCosts({
    startDate: '2025-01-01',
    endDate: '2025-01-31'
  });

  expect(costs.totalCost).toBe(100.50);
});
```

## Framework Integration

### React Integration

```typescript
import React, { useEffect, useState } from 'react';
import { CostOpsClient } from '@llm-cost-ops/sdk';

// Create client instance
const client = new CostOpsClient({
  apiKey: process.env.REACT_APP_COST_OPS_API_KEY!
});

// Custom hook
function useCosts(startDate: string, endDate: string) {
  const [costs, setCosts] = useState<any>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    let cancelled = false;

    async function fetchCosts() {
      try {
        setLoading(true);
        const data = await client.costs.getCosts({ startDate, endDate });

        if (!cancelled) {
          setCosts(data);
          setError(null);
        }
      } catch (err) {
        if (!cancelled) {
          setError(err as Error);
        }
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    fetchCosts();

    return () => {
      cancelled = true;
    };
  }, [startDate, endDate]);

  return { costs, loading, error };
}

// Component
function CostsDashboard() {
  const { costs, loading, error } = useCosts('2025-01-01', '2025-01-31');

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error.message}</div>;

  return (
    <div>
      <h1>Total Cost: ${costs?.totalCost.toFixed(2)}</h1>
      <ul>
        {costs?.items.map((item: any) => (
          <li key={item.date}>
            {item.date}: ${item.amount.toFixed(2)}
          </li>
        ))}
      </ul>
    </div>
  );
}
```

### Vue Integration

```typescript
import { ref, onMounted, watch } from 'vue';
import { CostOpsClient } from '@llm-cost-ops/sdk';

const client = new CostOpsClient({
  apiKey: import.meta.env.VITE_COST_OPS_API_KEY
});

export function useCosts(startDate: Ref<string>, endDate: Ref<string>) {
  const costs = ref<any>(null);
  const loading = ref(true);
  const error = ref<Error | null>(null);

  async function fetchCosts() {
    try {
      loading.value = true;
      costs.value = await client.costs.getCosts({
        startDate: startDate.value,
        endDate: endDate.value
      });
      error.value = null;
    } catch (err) {
      error.value = err as Error;
    } finally {
      loading.value = false;
    }
  }

  onMounted(fetchCosts);
  watch([startDate, endDate], fetchCosts);

  return { costs, loading, error, refetch: fetchCosts };
}

// Component
export default {
  setup() {
    const startDate = ref('2025-01-01');
    const endDate = ref('2025-01-31');
    const { costs, loading, error } = useCosts(startDate, endDate);

    return { costs, loading, error, startDate, endDate };
  }
};
```

### Angular Integration

```typescript
import { Injectable } from '@angular/core';
import { BehaviorSubject, Observable } from 'rxjs';
import { CostOpsClient } from '@llm-cost-ops/sdk';

@Injectable({
  providedIn: 'root'
})
export class CostOpsService {
  private client: CostOpsClient;
  private costsSubject = new BehaviorSubject<any>(null);
  public costs$: Observable<any> = this.costsSubject.asObservable();

  constructor() {
    this.client = new CostOpsClient({
      apiKey: environment.costOpsApiKey
    });
  }

  async getCosts(startDate: string, endDate: string): Promise<void> {
    try {
      const costs = await this.client.costs.getCosts({ startDate, endDate });
      this.costsSubject.next(costs);
    } catch (error) {
      console.error('Error fetching costs:', error);
      throw error;
    }
  }

  async trackUsage(data: any): Promise<void> {
    await this.client.usage.createUsage(data);
  }
}

// Component
@Component({
  selector: 'app-costs-dashboard',
  template: `
    <div *ngIf="costs$ | async as costs">
      <h1>Total Cost: \${{ costs.totalCost | number:'1.2-2' }}</h1>
      <ul>
        <li *ngFor="let item of costs.items">
          {{ item.date }}: \${{ item.amount | number:'1.2-2' }}
        </li>
      </ul>
    </div>
  `
})
export class CostsDashboardComponent implements OnInit {
  costs$: Observable<any>;

  constructor(private costOpsService: CostOpsService) {
    this.costs$ = this.costOpsService.costs$;
  }

  ngOnInit(): void {
    this.costOpsService.getCosts('2025-01-01', '2025-01-31');
  }
}
```

## WebSocket Support

### WebSocket Client

```typescript
import { EventEmitter } from 'events';

interface WebSocketConfig {
  url: string;
  apiKey: string;
  reconnect?: boolean;
  reconnectInterval?: number;
  maxReconnectAttempts?: number;
}

class CostOpsWebSocket extends EventEmitter {
  private ws: WebSocket | null = null;
  private reconnectAttempts = 0;
  private reconnectTimer: NodeJS.Timeout | null = null;

  constructor(private config: WebSocketConfig) {
    super();
    this.connect();
  }

  private connect(): void {
    const url = `${this.config.url}?apiKey=${this.config.apiKey}`;
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.reconnectAttempts = 0;
      this.emit('connected');
    };

    this.ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        this.emit('message', data);

        // Emit specific event types
        if (data.type) {
          this.emit(data.type, data.payload);
        }
      } catch (error) {
        console.error('Error parsing message:', error);
      }
    };

    this.ws.onerror = (error) => {
      console.error('WebSocket error:', error);
      this.emit('error', error);
    };

    this.ws.onclose = () => {
      console.log('WebSocket disconnected');
      this.emit('disconnected');

      if (this.config.reconnect) {
        this.scheduleReconnect();
      }
    };
  }

  private scheduleReconnect(): void {
    if (
      this.config.maxReconnectAttempts &&
      this.reconnectAttempts >= this.config.maxReconnectAttempts
    ) {
      console.error('Max reconnect attempts reached');
      return;
    }

    this.reconnectAttempts++;
    const interval = this.config.reconnectInterval || 5000;

    console.log(`Reconnecting in ${interval}ms (attempt ${this.reconnectAttempts})`);

    this.reconnectTimer = setTimeout(() => {
      this.connect();
    }, interval);
  }

  send(data: any): void {
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    } else {
      console.error('WebSocket not connected');
    }
  }

  close(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }

    if (this.ws) {
      this.ws.close();
    }
  }
}

// Usage
const ws = new CostOpsWebSocket({
  url: 'wss://api.llmcostops.com/ws',
  apiKey: 'your_api_key',
  reconnect: true,
  reconnectInterval: 5000,
  maxReconnectAttempts: 10
});

ws.on('connected', () => {
  console.log('Connected to WebSocket');

  // Subscribe to events
  ws.send({
    type: 'subscribe',
    events: ['usage.created', 'budget.alert']
  });
});

ws.on('usage.created', (data) => {
  console.log('New usage:', data);
});

ws.on('budget.alert', (data) => {
  console.log('Budget alert:', data);
});

ws.on('error', (error) => {
  console.error('WebSocket error:', error);
});

ws.on('disconnected', () => {
  console.log('Disconnected from WebSocket');
});
```

## Advanced Patterns

### Repository Pattern

```typescript
interface Repository<T> {
  findById(id: string): Promise<T | null>;
  findAll(params?: any): Promise<T[]>;
  create(data: Partial<T>): Promise<T>;
  update(id: string, data: Partial<T>): Promise<T>;
  delete(id: string): Promise<void>;
}

class UsageRepository implements Repository<any> {
  constructor(private client: CostOpsClient) {}

  async findById(id: string) {
    try {
      return await this.client.usage.getUsageById(id);
    } catch (error) {
      if (error instanceof ResourceNotFoundError) {
        return null;
      }
      throw error;
    }
  }

  async findAll(params?: any) {
    const result = await this.client.usage.getUsage(params);
    return result.items;
  }

  async create(data: any) {
    return await this.client.usage.createUsage(data);
  }

  async update(id: string, data: any) {
    return await this.client.usage.updateUsage(id, data);
  }

  async delete(id: string) {
    await this.client.usage.deleteUsage(id);
  }
}
```

### Service Layer Pattern

```typescript
class CostAnalyticsService {
  constructor(private client: CostOpsClient) {}

  async getDailyCostTrend(days: number = 30): Promise<any[]> {
    const endDate = new Date();
    const startDate = new Date(endDate);
    startDate.setDate(startDate.getDate() - days);

    const breakdown = await this.client.costs.getCostBreakdown({
      startDate: startDate.toISOString(),
      endDate: endDate.toISOString(),
      granularity: 'daily'
    });

    return breakdown.items;
  }

  async getCostByModel(startDate: string, endDate: string): Promise<Map<string, number>> {
    const costs = await this.client.costs.getCosts({
      startDate,
      endDate,
      groupBy: ['model']
    });

    const costMap = new Map<string, number>();

    costs.items.forEach(item => {
      costMap.set(item.model, item.amount);
    });

    return costMap;
  }

  async predictNextMonthCost(): Promise<number> {
    const trends = await this.client.costs.getCostTrends({
      days: 30,
      includeForecast: true
    });

    return trends.forecast;
  }

  async getAnomalies(threshold: number = 2.0): Promise<any[]> {
    const analytics = await this.client.analytics.getUsageAnalytics({
      startDate: new Date(Date.now() - 30 * 24 * 60 * 60 * 1000).toISOString(),
      endDate: new Date().toISOString(),
      groupBy: ['date']
    });

    const costs = analytics.groups.map(g => g.totalCost);
    const mean = costs.reduce((a, b) => a + b, 0) / costs.length;
    const stdDev = Math.sqrt(
      costs.reduce((sq, n) => sq + Math.pow(n - mean, 2), 0) / costs.length
    );

    return analytics.groups.filter(g =>
      Math.abs(g.totalCost - mean) > threshold * stdDev
    );
  }
}
```

### Factory Pattern

```typescript
class ClientFactory {
  private static instances = new Map<string, CostOpsClient>();

  static create(config: {
    apiKey: string;
    environment?: 'production' | 'staging' | 'development';
  }): CostOpsClient {
    const key = `${config.apiKey}-${config.environment || 'production'}`;

    if (!this.instances.has(key)) {
      const baseUrl = this.getBaseUrl(config.environment);

      const client = new CostOpsClient({
        apiKey: config.apiKey,
        baseUrl,
        timeout: 30000,
        retries: 3
      });

      this.instances.set(key, client);
    }

    return this.instances.get(key)!;
  }

  private static getBaseUrl(environment?: string): string {
    switch (environment) {
      case 'production':
        return 'https://api.llmcostops.com';
      case 'staging':
        return 'https://staging-api.llmcostops.com';
      case 'development':
        return 'http://localhost:3000';
      default:
        return 'https://api.llmcostops.com';
    }
  }

  static clear(): void {
    this.instances.clear();
  }
}

// Usage
const prodClient = ClientFactory.create({
  apiKey: 'prod_key',
  environment: 'production'
});

const stagingClient = ClientFactory.create({
  apiKey: 'staging_key',
  environment: 'staging'
});
```

## Performance Optimization

### Request Batching

```typescript
class RequestBatcher<T> {
  private queue: Array<{
    data: any;
    resolve: (value: T) => void;
    reject: (error: any) => void;
  }> = [];
  private timer: NodeJS.Timeout | null = null;

  constructor(
    private batchFn: (items: any[]) => Promise<T[]>,
    private batchSize: number = 100,
    private batchDelay: number = 100
  ) {}

  async add(data: any): Promise<T> {
    return new Promise((resolve, reject) => {
      this.queue.push({ data, resolve, reject });

      if (this.queue.length >= this.batchSize) {
        this.flush();
      } else if (!this.timer) {
        this.timer = setTimeout(() => this.flush(), this.batchDelay);
      }
    });
  }

  private async flush(): Promise<void> {
    if (this.timer) {
      clearTimeout(this.timer);
      this.timer = null;
    }

    if (this.queue.length === 0) {
      return;
    }

    const batch = this.queue.splice(0, this.batchSize);
    const items = batch.map(item => item.data);

    try {
      const results = await this.batchFn(items);

      batch.forEach((item, index) => {
        item.resolve(results[index]);
      });
    } catch (error) {
      batch.forEach(item => {
        item.reject(error);
      });
    }
  }
}

// Usage
const usageBatcher = new RequestBatcher(
  async (items) => {
    const result = await client.usage.batchCreateUsage(items);
    return result.items;
  },
  100,
  100
);

// These will be batched together
const usage1 = await usageBatcher.add({ model: 'gpt-4', tokensPrompt: 1000 });
const usage2 = await usageBatcher.add({ model: 'gpt-4', tokensPrompt: 2000 });
const usage3 = await usageBatcher.add({ model: 'gpt-4', tokensPrompt: 3000 });
```

### Response Caching

```typescript
interface CacheOptions {
  ttl: number; // Time to live in milliseconds
  maxSize: number; // Maximum cache size
}

class ResponseCache {
  private cache = new Map<string, { data: any; expires: number }>();
  private accessCount = new Map<string, number>();

  constructor(private options: CacheOptions) {}

  get(key: string): any | null {
    const entry = this.cache.get(key);

    if (!entry) {
      return null;
    }

    if (Date.now() > entry.expires) {
      this.cache.delete(key);
      this.accessCount.delete(key);
      return null;
    }

    // Track access for LRU eviction
    this.accessCount.set(key, (this.accessCount.get(key) || 0) + 1);

    return entry.data;
  }

  set(key: string, data: any): void {
    // Evict if cache is full
    if (this.cache.size >= this.options.maxSize) {
      this.evictLRU();
    }

    this.cache.set(key, {
      data,
      expires: Date.now() + this.options.ttl
    });
    this.accessCount.set(key, 1);
  }

  private evictLRU(): void {
    let minCount = Infinity;
    let lruKey: string | null = null;

    for (const [key, count] of this.accessCount.entries()) {
      if (count < minCount) {
        minCount = count;
        lruKey = key;
      }
    }

    if (lruKey) {
      this.cache.delete(lruKey);
      this.accessCount.delete(lruKey);
    }
  }

  clear(): void {
    this.cache.clear();
    this.accessCount.clear();
  }
}

// Cached client wrapper
class CachedCostOpsClient {
  private cache: ResponseCache;

  constructor(
    private client: CostOpsClient,
    cacheOptions: CacheOptions = { ttl: 60000, maxSize: 100 }
  ) {
    this.cache = new ResponseCache(cacheOptions);
  }

  async getCosts(params: any): Promise<any> {
    const cacheKey = `costs:${JSON.stringify(params)}`;
    const cached = this.cache.get(cacheKey);

    if (cached) {
      return cached;
    }

    const costs = await this.client.costs.getCosts(params);
    this.cache.set(cacheKey, costs);

    return costs;
  }
}
```

---

## Complete Example Application

```typescript
#!/usr/bin/env ts-node
import { CostOpsClient } from '@llm-cost-ops/sdk';
import dotenv from 'dotenv';

dotenv.config();

class CostTrackingApp {
  private client: CostOpsClient;

  constructor(apiKey: string) {
    this.client = new CostOpsClient({
      apiKey,
      timeout: 30000,
      retries: 3
    });
  }

  async trackUsage(model: string, tokensPrompt: number, tokensCompletion: number): Promise<void> {
    try {
      const usage = await this.client.usage.createUsage({
        model,
        tokensPrompt,
        tokensCompletion,
        requestCount: 1,
        timestamp: new Date().toISOString()
      });

      console.log(`✓ Tracked usage: ${usage.id} - $${usage.cost.toFixed(4)}`);
    } catch (error) {
      console.error('✗ Error tracking usage:', error);
    }
  }

  async getCostSummary(days: number = 7): Promise<void> {
    const endDate = new Date();
    const startDate = new Date(endDate);
    startDate.setDate(startDate.getDate() - days);

    try {
      const costs = await this.client.costs.getCosts({
        startDate: startDate.toISOString(),
        endDate: endDate.toISOString()
      });

      console.log(`\n📊 Cost Summary (Last ${days} days)`);
      console.log(`Total Cost: $${costs.totalCost.toFixed(2)}`);

      costs.items.slice(0, 10).forEach(item => {
        console.log(`  ${item.date}: $${item.amount.toFixed(2)}`);
      });
    } catch (error) {
      console.error('✗ Error fetching costs:', error);
    }
  }

  async checkBudgets(): Promise<void> {
    try {
      const budgets = await this.client.budgets.listBudgets({ activeOnly: true });

      console.log(`\n💰 Active Budgets`);

      budgets.items.forEach(budget => {
        const percentage = budget.percentageUsed;
        const status = percentage < 50 ? '🟢' : percentage < 80 ? '🟡' : '🔴';

        console.log(
          `${status} ${budget.name}: $${budget.spent.toFixed(2)} / $${budget.amount.toFixed(2)} ` +
          `(${percentage.toFixed(1)}%)`
        );
      });
    } catch (error) {
      console.error('✗ Error fetching budgets:', error);
    }
  }

  async runAnalytics(days: number = 30): Promise<void> {
    const endDate = new Date();
    const startDate = new Date(endDate);
    startDate.setDate(startDate.getDate() - days);

    try {
      const analytics = await this.client.analytics.getUsageAnalytics({
        startDate: startDate.toISOString(),
        endDate: endDate.toISOString(),
        groupBy: ['model'],
        metrics: ['total_tokens', 'total_cost', 'request_count']
      });

      console.log(`\n📈 Analytics (Last ${days} days)`);

      analytics.groups.forEach(group => {
        console.log(`\n${group.key}:`);
        console.log(`  Tokens: ${group.totalTokens.toLocaleString()}`);
        console.log(`  Cost: $${group.totalCost.toFixed(2)}`);
        console.log(`  Requests: ${group.requestCount.toLocaleString()}`);
        console.log(`  Avg Cost/Request: $${(group.totalCost / group.requestCount).toFixed(4)}`);
      });
    } catch (error) {
      console.error('✗ Error running analytics:', error);
    }
  }

  async run(): Promise<void> {
    await this.trackUsage('gpt-4', 1000, 500);
    await this.getCostSummary(7);
    await this.checkBudgets();
    await this.runAnalytics(30);
  }
}

// Main
async function main() {
  const apiKey = process.env.LLM_COST_OPS_API_KEY;

  if (!apiKey) {
    console.error('Error: LLM_COST_OPS_API_KEY not set');
    process.exit(1);
  }

  const app = new CostTrackingApp(apiKey);
  await app.run();
}

main().catch(console.error);
```

---

## Additional Resources

- **API Reference**: https://docs.llmcostops.com/api
- **GitHub Repository**: https://github.com/llmcostops/typescript-sdk
- **Examples**: https://github.com/llmcostops/typescript-sdk/tree/main/examples
- **Support**: support@llmcostops.com

## Next Steps

1. Install the SDK and configure TypeScript
2. Initialize the client with your API key
3. Try basic usage examples
4. Implement error handling
5. Add testing to your application
6. Integrate with your framework (React, Vue, Angular)
7. Optimize for production with caching and batching

For more advanced use cases, check out the [Advanced Integration Guide](./advanced-integration.md).
