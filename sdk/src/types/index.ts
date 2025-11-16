/**
 * Core type definitions for LLM Cost Operations SDK
 */

/**
 * Configuration options for the SDK client
 */
export interface ClientConfig {
  /** API base URL */
  baseUrl: string;
  /** API key for authentication */
  apiKey?: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Maximum number of retry attempts */
  maxRetries?: number;
  /** Initial retry delay in milliseconds */
  retryDelay?: number;
  /** Enable exponential backoff for retries */
  exponentialBackoff?: boolean;
  /** Custom headers to include in all requests */
  headers?: Record<string, string>;
  /** Enable debug logging */
  debug?: boolean;
}

/**
 * HTTP methods supported by the SDK
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';

/**
 * Request options for API calls
 */
export interface RequestOptions {
  /** HTTP method */
  method: HttpMethod;
  /** Request path */
  path: string;
  /** Query parameters */
  query?: Record<string, string | number | boolean | undefined>;
  /** Request body */
  body?: unknown;
  /** Custom headers for this request */
  headers?: Record<string, string>;
  /** Override timeout for this request */
  timeout?: number;
  /** Skip retry logic for this request */
  skipRetry?: boolean;
}

/**
 * Response from API calls
 */
export interface ApiResponse<T = unknown> {
  /** Response data */
  data: T;
  /** HTTP status code */
  status: number;
  /** Response headers */
  headers: Record<string, string>;
  /** Request metadata */
  metadata: {
    /** Request ID for tracking */
    requestId?: string;
    /** Time taken for the request in milliseconds */
    duration: number;
    /** Number of retry attempts made */
    retries: number;
  };
}

/**
 * Cost metric data point
 */
export interface CostMetric {
  /** Unique identifier */
  id: string;
  /** Timestamp of the metric */
  timestamp: string;
  /** Service or model name */
  service: string;
  /** Cost amount */
  cost: number;
  /** Currency code (e.g., 'USD') */
  currency: string;
  /** Number of tokens used */
  tokens?: number;
  /** Number of requests */
  requests?: number;
  /** Additional metadata */
  metadata?: Record<string, unknown>;
}

/**
 * Usage statistics
 */
export interface UsageStats {
  /** Total cost */
  totalCost: number;
  /** Total tokens used */
  totalTokens: number;
  /** Total requests made */
  totalRequests: number;
  /** Time period start */
  periodStart: string;
  /** Time period end */
  periodEnd: string;
  /** Breakdown by service */
  byService: Record<string, ServiceUsage>;
}

/**
 * Service-specific usage
 */
export interface ServiceUsage {
  /** Service name */
  service: string;
  /** Total cost for this service */
  cost: number;
  /** Total tokens for this service */
  tokens: number;
  /** Total requests for this service */
  requests: number;
}

/**
 * Query parameters for fetching metrics
 */
export interface MetricsQuery {
  /** Start date (ISO 8601) */
  startDate?: string;
  /** End date (ISO 8601) */
  endDate?: string;
  /** Filter by service names */
  services?: string[];
  /** Limit number of results */
  limit?: number;
  /** Offset for pagination */
  offset?: number;
  /** Sort field */
  sortBy?: 'timestamp' | 'cost' | 'tokens';
  /** Sort order */
  sortOrder?: 'asc' | 'desc';
}

/**
 * Budget configuration
 */
export interface Budget {
  /** Budget ID */
  id: string;
  /** Budget name */
  name: string;
  /** Budget amount */
  amount: number;
  /** Currency code */
  currency: string;
  /** Period type */
  period: 'daily' | 'weekly' | 'monthly' | 'yearly';
  /** Alert threshold percentage (0-100) */
  alertThreshold?: number;
  /** Services covered by this budget */
  services?: string[];
  /** Budget status */
  status: 'active' | 'exceeded' | 'inactive';
  /** Current spending */
  currentSpending: number;
  /** Remaining budget */
  remaining: number;
}

/**
 * Alert configuration
 */
export interface Alert {
  /** Alert ID */
  id: string;
  /** Alert type */
  type: 'budget' | 'cost_spike' | 'usage_anomaly';
  /** Severity level */
  severity: 'info' | 'warning' | 'critical';
  /** Alert message */
  message: string;
  /** Timestamp when alert was triggered */
  timestamp: string;
  /** Whether alert has been acknowledged */
  acknowledged: boolean;
  /** Related resource ID */
  resourceId?: string;
}

/**
 * Export options for reports
 */
export interface ExportOptions {
  /** Export format */
  format: 'json' | 'csv' | 'xlsx' | 'pdf';
  /** Date range for export */
  dateRange: {
    start: string;
    end: string;
  };
  /** Services to include */
  services?: string[];
  /** Include detailed breakdown */
  detailed?: boolean;
}

/**
 * Forecast result
 */
export interface Forecast {
  /** Forecast ID */
  id: string;
  /** Forecast period */
  period: 'week' | 'month' | 'quarter' | 'year';
  /** Predicted cost */
  predictedCost: number;
  /** Confidence interval (min, max) */
  confidenceInterval: {
    min: number;
    max: number;
  };
  /** Confidence level (0-1) */
  confidence: number;
  /** Historical data used */
  basedOnDays: number;
  /** Forecast timestamp */
  generatedAt: string;
}

/**
 * Webhook configuration
 */
export interface Webhook {
  /** Webhook ID */
  id: string;
  /** Webhook URL */
  url: string;
  /** Events to trigger on */
  events: WebhookEvent[];
  /** Secret for signature verification */
  secret?: string;
  /** Whether webhook is active */
  active: boolean;
  /** Custom headers */
  headers?: Record<string, string>;
}

/**
 * Webhook event types
 */
export type WebhookEvent =
  | 'budget.exceeded'
  | 'budget.warning'
  | 'cost.spike'
  | 'usage.anomaly'
  | 'export.completed'
  | 'metric.created';

/**
 * Pagination metadata
 */
export interface PaginationMeta {
  /** Current page number */
  page: number;
  /** Items per page */
  pageSize: number;
  /** Total number of items */
  total: number;
  /** Total number of pages */
  totalPages: number;
  /** Whether there's a next page */
  hasNext: boolean;
  /** Whether there's a previous page */
  hasPrevious: boolean;
}

/**
 * Paginated response
 */
export interface PaginatedResponse<T> {
  /** Array of items */
  items: T[];
  /** Pagination metadata */
  pagination: PaginationMeta;
}

/**
 * Health check response
 */
export interface HealthCheck {
  /** Service status */
  status: 'healthy' | 'degraded' | 'unhealthy';
  /** Service version */
  version: string;
  /** Component health status */
  components: {
    database: ComponentHealth;
    cache: ComponentHealth;
    queue: ComponentHealth;
  };
  /** Timestamp of health check */
  timestamp: string;
}

/**
 * Component health status
 */
export interface ComponentHealth {
  /** Component status */
  status: 'up' | 'down' | 'degraded';
  /** Response time in milliseconds */
  responseTime?: number;
  /** Error message if any */
  error?: string;
}
