/**
 * Error handling example
 */

import {
  CostOpsClient,
  AuthenticationError,
  ValidationError,
  RateLimitError,
  NotFoundError,
  isCostOpsError,
} from '../src/index.js';

async function main(): Promise<void> {
  const client = new CostOpsClient({
    baseUrl: 'https://api.llm-cost-ops.example.com',
    apiKey: process.env['API_KEY'] ?? 'your-api-key',
  });

  // Handle specific error types
  try {
    await client.getMetric('non-existent-id');
  } catch (error) {
    if (error instanceof NotFoundError) {
      console.log('Metric not found:', error.message);
      console.log('Resource ID:', error.resourceId);
    } else if (error instanceof AuthenticationError) {
      console.error('Authentication failed - check your API key');
    } else if (error instanceof ValidationError) {
      console.error('Validation error:', {
        field: error.field,
        message: error.message,
        constraints: error.constraints,
      });
    } else if (error instanceof RateLimitError) {
      console.error('Rate limited - retry after:', error.retryAfter, 'seconds');
    } else if (isCostOpsError(error)) {
      console.error('SDK Error:', {
        name: error.name,
        message: error.message,
        code: error.code,
        timestamp: error.timestamp,
      });
    } else {
      console.error('Unknown error:', error);
    }
  }

  // Handle validation errors
  try {
    await client.createMetric({
      service: '', // Invalid: empty string
      cost: -10, // Invalid: negative cost
      currency: 'USD',
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    if (error instanceof ValidationError) {
      console.error('Validation failed:', error.field);
    }
  }

  // Handle rate limiting
  try {
    // Make many requests in quick succession
    const promises = Array.from({ length: 100 }, (_, i) =>
      client.getMetrics({ limit: 1, offset: i })
    );
    await Promise.all(promises);
  } catch (error) {
    if (error instanceof RateLimitError) {
      console.log('Rate limited!');
      console.log('Retry after:', error.retryAfter, 'seconds');
      console.log('Rate limit:', error.limit, 'requests');
    }
  }

  // Graceful error handling with fallbacks
  async function getMetricWithFallback(id: string): Promise<any | null> {
    try {
      return await client.getMetric(id);
    } catch (error) {
      if (error instanceof NotFoundError) {
        console.warn(`Metric ${id} not found, returning null`);
        return null;
      }
      throw error; // Re-throw other errors
    }
  }

  const metric = await getMetricWithFallback('test-id');
  console.log('Metric:', metric);
}

main().catch(console.error);
