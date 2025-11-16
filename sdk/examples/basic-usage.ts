/**
 * Basic usage example
 */

import { CostOpsClient } from '../src/index.js';

async function main(): Promise<void> {
  // Initialize the client
  const client = new CostOpsClient({
    baseUrl: 'https://api.llm-cost-ops.example.com',
    apiKey: process.env['API_KEY'] ?? 'your-api-key',
    timeout: 30000,
    maxRetries: 3,
    debug: true,
  });

  try {
    // Check service health
    const health = await client.health();
    console.log('Service Status:', health.status);

    // Get cost metrics
    const metrics = await client.getMetrics({
      startDate: '2024-01-01T00:00:00Z',
      endDate: '2024-01-31T23:59:59Z',
      limit: 10,
    });
    console.log('Metrics:', metrics.items);

    // Create a new metric
    const newMetric = await client.createMetric({
      service: 'gpt-4',
      cost: 0.05,
      currency: 'USD',
      tokens: 1000,
      requests: 1,
      timestamp: new Date().toISOString(),
    });
    console.log('Created Metric:', newMetric);

    // Get usage statistics
    const stats = await client.getUsageStats(
      '2024-01-01T00:00:00Z',
      '2024-01-31T23:59:59Z'
    );
    console.log('Total Cost:', stats.totalCost);
    console.log('Total Tokens:', stats.totalTokens);

    // Get budgets
    const budgets = await client.getBudgets();
    console.log('Budgets:', budgets);

    // Create a budget
    const budget = await client.createBudget({
      name: 'Monthly AI Budget',
      amount: 1000,
      currency: 'USD',
      period: 'monthly',
      alertThreshold: 80,
    });
    console.log('Created Budget:', budget);

    // Get forecast
    const forecast = await client.getForecast('month');
    console.log('Forecast:', forecast.predictedCost);
  } catch (error) {
    console.error('Error:', error);
  }
}

main().catch(console.error);
