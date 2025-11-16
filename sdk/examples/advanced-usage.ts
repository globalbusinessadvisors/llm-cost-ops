/**
 * Advanced usage with interceptors and event handling
 */

import { CostOpsClient, createMetricsInterceptor } from '../src/index.js';

async function main(): Promise<void> {
  // Initialize client
  const client = new CostOpsClient({
    baseUrl: 'https://api.llm-cost-ops.example.com',
    apiKey: process.env['API_KEY'] ?? 'your-api-key',
  });

  // Get middleware manager
  const middleware = client.getMiddleware();

  // Add custom request interceptor
  middleware.addRequestInterceptor((context) => {
    console.log('Making request to:', context.request.path);
    context.metadata['customHeader'] = 'custom-value';
    return context;
  });

  // Add custom response interceptor
  middleware.addResponseInterceptor((context) => {
    console.log('Response received:', {
      status: context.response.status,
      duration: context.response.metadata.duration,
    });
    return context;
  });

  // Add custom error interceptor
  middleware.addErrorInterceptor((context) => {
    console.error('Request failed:', {
      error: context.error.message,
      path: context.request.request.path,
    });
    return context;
  });

  // Add metrics interceptor
  const metricsInterceptor = createMetricsInterceptor();
  middleware.addRequestInterceptor(metricsInterceptor.request);
  middleware.addResponseInterceptor(metricsInterceptor.response);
  middleware.addErrorInterceptor(metricsInterceptor.error);

  // Listen to events
  middleware.on('request:start', (context) => {
    console.log('Request started:', context.request.path);
  });

  middleware.on('request:end', (context) => {
    console.log('Request completed:', {
      status: context.response.status,
      duration: context.response.metadata.duration,
    });
  });

  middleware.on('request:error', (context) => {
    console.error('Request error:', context.error.message);
  });

  middleware.on('retry:attempt', (attempt, error) => {
    console.log(`Retry attempt ${attempt}:`, error.message);
  });

  // Make some requests
  try {
    await client.health();
    await client.getMetrics({ limit: 5 });
    await client.getBudgets();

    // Get metrics
    const metrics = metricsInterceptor.getMetrics();
    console.log('\nRequest Metrics:');
    console.log('Total Requests:', metrics.totalRequests);
    console.log('Successful:', metrics.successfulRequests);
    console.log('Failed:', metrics.failedRequests);
    console.log('Average Duration:', metrics.averageDuration.toFixed(2), 'ms');
  } catch (error) {
    console.error('Error:', error);
  }
}

main().catch(console.error);
