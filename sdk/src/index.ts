/**
 * LLM Cost Operations SDK
 * Enterprise-grade TypeScript SDK for cost monitoring and management
 */

// Export main client
export { CostOpsClient, BaseClient } from './client/index.js';

// Export types
export type {
  ClientConfig,
  HttpMethod,
  RequestOptions,
  ApiResponse,
  CostMetric,
  UsageStats,
  ServiceUsage,
  MetricsQuery,
  Budget,
  Alert,
  ExportOptions,
  Forecast,
  Webhook,
  WebhookEvent,
  PaginationMeta,
  PaginatedResponse,
  HealthCheck,
  ComponentHealth,
} from './types/index.js';

// Export errors
export {
  CostOpsError,
  ConfigurationError,
  ValidationError,
  AuthenticationError,
  AuthorizationError,
  ApiError,
  NetworkError,
  TimeoutError,
  RateLimitError,
  NotFoundError,
  ConflictError,
  ServerError,
  RetryExhaustedError,
  isCostOpsError,
  isApiError,
  isRetryableError,
} from './errors/index.js';

// Export middleware
export {
  MiddlewareManager,
  createLoggingInterceptor,
  createAuthInterceptor,
  createMetricsInterceptor,
  type RequestContext,
  type ResponseContext,
  type ErrorContext,
  type RequestInterceptor,
  type ResponseInterceptor,
  type ErrorInterceptor,
  type MiddlewareEvents,
  type RequestMetrics,
} from './middleware/index.js';

// Export utilities
export {
  fetchWithTimeout,
  buildUrl,
  parseJsonResponse,
  extractHeaders,
  isBrowser,
  isNode,
  getUserAgent,
  withRetry,
  calculateRetryDelay,
  sleep,
  validateConfig,
  validateRequired,
  validateStringLength,
  validateNumberRange,
  validateISODate,
  validateNonEmptyArray,
  validateEnum,
  type RetryOptions,
} from './utils/index.js';

// Default export
export { CostOpsClient as default } from './client/index.js';
