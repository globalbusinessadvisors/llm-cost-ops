/**
 * Main Cost Operations client
 */

import type {
  Alert,
  Budget,
  ClientConfig,
  CostMetric,
  ExportOptions,
  Forecast,
  HealthCheck,
  MetricsQuery,
  PaginatedResponse,
  UsageStats,
  Webhook,
} from '../types/index.js';
import { validateEnum, validateRequired } from '../utils/validation.js';

import { BaseClient } from './base-client.js';

/**
 * LLM Cost Operations Client
 */
export class CostOpsClient extends BaseClient {
  constructor(config: ClientConfig) {
    super(config);
  }

  /**
   * Health check endpoint
   */
  public async health(): Promise<HealthCheck> {
    const response = await this.get<HealthCheck>('/health');
    return response.data;
  }

  /**
   * Get cost metrics
   */
  public async getMetrics(query?: MetricsQuery): Promise<PaginatedResponse<CostMetric>> {
    const response = await this.get<PaginatedResponse<CostMetric>>('/api/v1/metrics', query as Record<string, string | number | boolean | undefined>);
    return response.data;
  }

  /**
   * Create a new cost metric
   */
  public async createMetric(metric: Omit<CostMetric, 'id'>): Promise<CostMetric> {
    validateRequired(metric.service, 'service');
    validateRequired(metric.cost, 'cost');
    validateRequired(metric.currency, 'currency');

    const response = await this.post<CostMetric>('/api/v1/metrics', metric);
    return response.data;
  }

  /**
   * Get a specific metric by ID
   */
  public async getMetric(id: string): Promise<CostMetric> {
    validateRequired(id, 'id');

    const response = await this.get<CostMetric>(`/api/v1/metrics/${id}`);
    return response.data;
  }

  /**
   * Delete a metric
   */
  public async deleteMetric(id: string): Promise<void> {
    validateRequired(id, 'id');

    await this.delete<void>(`/api/v1/metrics/${id}`);
  }

  /**
   * Get usage statistics
   */
  public async getUsageStats(
    startDate: string,
    endDate: string,
    services?: string[]
  ): Promise<UsageStats> {
    validateRequired(startDate, 'startDate');
    validateRequired(endDate, 'endDate');

    const response = await this.get<UsageStats>('/api/v1/usage/stats', {
      startDate,
      endDate,
      services: services?.join(','),
    });
    return response.data;
  }

  /**
   * Get all budgets
   */
  public async getBudgets(): Promise<Budget[]> {
    const response = await this.get<Budget[]>('/api/v1/budgets');
    return response.data;
  }

  /**
   * Get a specific budget by ID
   */
  public async getBudget(id: string): Promise<Budget> {
    validateRequired(id, 'id');

    const response = await this.get<Budget>(`/api/v1/budgets/${id}`);
    return response.data;
  }

  /**
   * Create a new budget
   */
  public async createBudget(
    budget: Omit<Budget, 'id' | 'status' | 'currentSpending' | 'remaining'>
  ): Promise<Budget> {
    validateRequired(budget.name, 'name');
    validateRequired(budget.amount, 'amount');
    validateRequired(budget.currency, 'currency');
    validateRequired(budget.period, 'period');
    validateEnum(budget.period, 'period', ['daily', 'weekly', 'monthly', 'yearly'] as const);

    const response = await this.post<Budget>('/api/v1/budgets', budget);
    return response.data;
  }

  /**
   * Update a budget
   */
  public async updateBudget(
    id: string,
    updates: Partial<Omit<Budget, 'id' | 'status' | 'currentSpending' | 'remaining'>>
  ): Promise<Budget> {
    validateRequired(id, 'id');

    const response = await this.patch<Budget>(`/api/v1/budgets/${id}`, updates);
    return response.data;
  }

  /**
   * Delete a budget
   */
  public async deleteBudget(id: string): Promise<void> {
    validateRequired(id, 'id');

    await this.delete<void>(`/api/v1/budgets/${id}`);
  }

  /**
   * Get all alerts
   */
  public async getAlerts(acknowledged?: boolean): Promise<Alert[]> {
    const response = await this.get<Alert[]>('/api/v1/alerts', {
      acknowledged,
    });
    return response.data;
  }

  /**
   * Get a specific alert by ID
   */
  public async getAlert(id: string): Promise<Alert> {
    validateRequired(id, 'id');

    const response = await this.get<Alert>(`/api/v1/alerts/${id}`);
    return response.data;
  }

  /**
   * Acknowledge an alert
   */
  public async acknowledgeAlert(id: string): Promise<Alert> {
    validateRequired(id, 'id');

    const response = await this.post<Alert>(`/api/v1/alerts/${id}/acknowledge`);
    return response.data;
  }

  /**
   * Delete an alert
   */
  public async deleteAlert(id: string): Promise<void> {
    validateRequired(id, 'id');

    await this.delete<void>(`/api/v1/alerts/${id}`);
  }

  /**
   * Export data
   */
  public async exportData(options: ExportOptions): Promise<{ downloadUrl: string }> {
    validateRequired(options.format, 'format');
    validateRequired(options.dateRange, 'dateRange');
    validateEnum(options.format, 'format', ['json', 'csv', 'xlsx', 'pdf'] as const);

    const response = await this.post<{ downloadUrl: string }>('/api/v1/export', options);
    return response.data;
  }

  /**
   * Get cost forecast
   */
  public async getForecast(
    period: 'week' | 'month' | 'quarter' | 'year',
    services?: string[]
  ): Promise<Forecast> {
    validateRequired(period, 'period');
    validateEnum(period, 'period', ['week', 'month', 'quarter', 'year'] as const);

    const response = await this.get<Forecast>('/api/v1/forecast', {
      period,
      services: services?.join(','),
    });
    return response.data;
  }

  /**
   * Get all webhooks
   */
  public async getWebhooks(): Promise<Webhook[]> {
    const response = await this.get<Webhook[]>('/api/v1/webhooks');
    return response.data;
  }

  /**
   * Get a specific webhook by ID
   */
  public async getWebhook(id: string): Promise<Webhook> {
    validateRequired(id, 'id');

    const response = await this.get<Webhook>(`/api/v1/webhooks/${id}`);
    return response.data;
  }

  /**
   * Create a new webhook
   */
  public async createWebhook(
    webhook: Omit<Webhook, 'id'>
  ): Promise<Webhook> {
    validateRequired(webhook.url, 'url');
    validateRequired(webhook.events, 'events');

    const response = await this.post<Webhook>('/api/v1/webhooks', webhook);
    return response.data;
  }

  /**
   * Update a webhook
   */
  public async updateWebhook(
    id: string,
    updates: Partial<Omit<Webhook, 'id'>>
  ): Promise<Webhook> {
    validateRequired(id, 'id');

    const response = await this.patch<Webhook>(`/api/v1/webhooks/${id}`, updates);
    return response.data;
  }

  /**
   * Delete a webhook
   */
  public async deleteWebhook(id: string): Promise<void> {
    validateRequired(id, 'id');

    await this.delete<void>(`/api/v1/webhooks/${id}`);
  }

  /**
   * Test a webhook
   */
  public async testWebhook(id: string): Promise<{ success: boolean; message: string }> {
    validateRequired(id, 'id');

    const response = await this.post<{ success: boolean; message: string }>(
      `/api/v1/webhooks/${id}/test`
    );
    return response.data;
  }
}
