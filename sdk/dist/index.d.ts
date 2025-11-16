import EventEmitter from 'eventemitter3';

/**
 * Core type definitions for LLM Cost Operations SDK
 */
/**
 * Configuration options for the SDK client
 */
interface ClientConfig {
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
type HttpMethod = 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
/**
 * Request options for API calls
 */
interface RequestOptions {
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
interface ApiResponse<T = unknown> {
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
interface CostMetric {
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
interface UsageStats {
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
interface ServiceUsage {
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
interface MetricsQuery {
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
interface Budget {
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
interface Alert {
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
interface ExportOptions {
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
interface Forecast {
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
interface Webhook {
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
type WebhookEvent = 'budget.exceeded' | 'budget.warning' | 'cost.spike' | 'usage.anomaly' | 'export.completed' | 'metric.created';
/**
 * Pagination metadata
 */
interface PaginationMeta {
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
interface PaginatedResponse<T> {
    /** Array of items */
    items: T[];
    /** Pagination metadata */
    pagination: PaginationMeta;
}
/**
 * Health check response
 */
interface HealthCheck {
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
interface ComponentHealth {
    /** Component status */
    status: 'up' | 'down' | 'degraded';
    /** Response time in milliseconds */
    responseTime?: number;
    /** Error message if any */
    error?: string;
}

/**
 * Middleware and interceptor system for request/response processing
 */

/**
 * Request context passed to interceptors
 */
interface RequestContext {
    /** Request options */
    request: RequestOptions;
    /** Additional metadata */
    metadata: Record<string, unknown>;
    /** Request timestamp */
    timestamp: number;
}
/**
 * Response context passed to interceptors
 */
interface ResponseContext<T = unknown> {
    /** API response */
    response: ApiResponse<T>;
    /** Request context */
    request: RequestContext;
    /** Response timestamp */
    timestamp: number;
}
/**
 * Error context passed to error interceptors
 */
interface ErrorContext {
    /** Error that occurred */
    error: Error;
    /** Request context */
    request: RequestContext;
    /** Error timestamp */
    timestamp: number;
}
/**
 * Request interceptor function
 */
type RequestInterceptor = (context: RequestContext) => RequestContext | Promise<RequestContext>;
/**
 * Response interceptor function
 */
type ResponseInterceptor = <T>(context: ResponseContext<T>) => ResponseContext<T> | Promise<ResponseContext<T>>;
/**
 * Error interceptor function
 */
type ErrorInterceptor = (context: ErrorContext) => ErrorContext | Promise<ErrorContext>;
/**
 * Middleware events
 */
interface MiddlewareEvents {
    'request:start': (context: RequestContext) => void;
    'request:end': <T>(context: ResponseContext<T>) => void;
    'request:error': (context: ErrorContext) => void;
    'retry:attempt': (attempt: number, error: Error) => void;
}
/**
 * Middleware manager for handling interceptors and events
 */
declare class MiddlewareManager extends EventEmitter<MiddlewareEvents> {
    private requestInterceptors;
    private responseInterceptors;
    private errorInterceptors;
    /**
     * Add a request interceptor
     */
    addRequestInterceptor(interceptor: RequestInterceptor): () => void;
    /**
     * Add a response interceptor
     */
    addResponseInterceptor(interceptor: ResponseInterceptor): () => void;
    /**
     * Add an error interceptor
     */
    addErrorInterceptor(interceptor: ErrorInterceptor): () => void;
    /**
     * Process request through all request interceptors
     */
    processRequest(context: RequestContext): Promise<RequestContext>;
    /**
     * Process response through all response interceptors
     */
    processResponse<T>(context: ResponseContext<T>): Promise<ResponseContext<T>>;
    /**
     * Process error through all error interceptors
     */
    processError(context: ErrorContext): Promise<ErrorContext>;
    /**
     * Clear all interceptors
     */
    clearInterceptors(): void;
    /**
     * Get count of registered interceptors
     */
    getInterceptorCount(): {
        request: number;
        response: number;
        error: number;
    };
}
/**
 * Built-in request interceptor for logging
 */
declare function createLoggingInterceptor(debug: boolean): {
    request: RequestInterceptor;
    response: ResponseInterceptor;
    error: ErrorInterceptor;
};
/**
 * Built-in request interceptor for adding authentication headers
 */
declare function createAuthInterceptor(apiKey: string): RequestInterceptor;
/**
 * Built-in request interceptor for tracking request metrics
 */
declare function createMetricsInterceptor(): {
    request: RequestInterceptor;
    response: ResponseInterceptor;
    error: ErrorInterceptor;
    getMetrics: () => RequestMetrics;
};
/**
 * Request metrics tracked by metrics interceptor
 */
interface RequestMetrics {
    totalRequests: number;
    successfulRequests: number;
    failedRequests: number;
    totalDuration: number;
    averageDuration: number;
}

/**
 * Base HTTP client with retry, interceptors, and error handling
 */

/**
 * Base client class with HTTP functionality
 */
declare class BaseClient {
    protected readonly config: Required<ClientConfig>;
    protected readonly middleware: MiddlewareManager;
    constructor(config: ClientConfig);
    /**
     * Setup built-in interceptors
     */
    private setupBuiltInInterceptors;
    /**
     * Make an HTTP request
     */
    protected request<T>(options: RequestOptions): Promise<ApiResponse<T>>;
    /**
     * Execute the actual HTTP request
     */
    private executeRequest;
    /**
     * Handle error responses
     */
    private handleErrorResponse;
    /**
     * Extract error message from error response
     */
    private extractErrorMessage;
    /**
     * Make a GET request
     */
    protected get<T>(path: string, query?: Record<string, string | number | boolean | undefined>): Promise<ApiResponse<T>>;
    /**
     * Make a POST request
     */
    protected post<T>(path: string, body?: unknown): Promise<ApiResponse<T>>;
    /**
     * Make a PUT request
     */
    protected put<T>(path: string, body?: unknown): Promise<ApiResponse<T>>;
    /**
     * Make a PATCH request
     */
    protected patch<T>(path: string, body?: unknown): Promise<ApiResponse<T>>;
    /**
     * Make a DELETE request
     */
    protected delete<T>(path: string): Promise<ApiResponse<T>>;
    /**
     * Get middleware manager for adding custom interceptors
     */
    getMiddleware(): MiddlewareManager;
    /**
     * Get current configuration (read-only)
     */
    getConfig(): Readonly<Required<ClientConfig>>;
}

/**
 * Main Cost Operations client
 */

/**
 * LLM Cost Operations Client
 */
declare class CostOpsClient extends BaseClient {
    constructor(config: ClientConfig);
    /**
     * Health check endpoint
     */
    health(): Promise<HealthCheck>;
    /**
     * Get cost metrics
     */
    getMetrics(query?: MetricsQuery): Promise<PaginatedResponse<CostMetric>>;
    /**
     * Create a new cost metric
     */
    createMetric(metric: Omit<CostMetric, 'id'>): Promise<CostMetric>;
    /**
     * Get a specific metric by ID
     */
    getMetric(id: string): Promise<CostMetric>;
    /**
     * Delete a metric
     */
    deleteMetric(id: string): Promise<void>;
    /**
     * Get usage statistics
     */
    getUsageStats(startDate: string, endDate: string, services?: string[]): Promise<UsageStats>;
    /**
     * Get all budgets
     */
    getBudgets(): Promise<Budget[]>;
    /**
     * Get a specific budget by ID
     */
    getBudget(id: string): Promise<Budget>;
    /**
     * Create a new budget
     */
    createBudget(budget: Omit<Budget, 'id' | 'status' | 'currentSpending' | 'remaining'>): Promise<Budget>;
    /**
     * Update a budget
     */
    updateBudget(id: string, updates: Partial<Omit<Budget, 'id' | 'status' | 'currentSpending' | 'remaining'>>): Promise<Budget>;
    /**
     * Delete a budget
     */
    deleteBudget(id: string): Promise<void>;
    /**
     * Get all alerts
     */
    getAlerts(acknowledged?: boolean): Promise<Alert[]>;
    /**
     * Get a specific alert by ID
     */
    getAlert(id: string): Promise<Alert>;
    /**
     * Acknowledge an alert
     */
    acknowledgeAlert(id: string): Promise<Alert>;
    /**
     * Delete an alert
     */
    deleteAlert(id: string): Promise<void>;
    /**
     * Export data
     */
    exportData(options: ExportOptions): Promise<{
        downloadUrl: string;
    }>;
    /**
     * Get cost forecast
     */
    getForecast(period: 'week' | 'month' | 'quarter' | 'year', services?: string[]): Promise<Forecast>;
    /**
     * Get all webhooks
     */
    getWebhooks(): Promise<Webhook[]>;
    /**
     * Get a specific webhook by ID
     */
    getWebhook(id: string): Promise<Webhook>;
    /**
     * Create a new webhook
     */
    createWebhook(webhook: Omit<Webhook, 'id'>): Promise<Webhook>;
    /**
     * Update a webhook
     */
    updateWebhook(id: string, updates: Partial<Omit<Webhook, 'id'>>): Promise<Webhook>;
    /**
     * Delete a webhook
     */
    deleteWebhook(id: string): Promise<void>;
    /**
     * Test a webhook
     */
    testWebhook(id: string): Promise<{
        success: boolean;
        message: string;
    }>;
}

/**
 * Custom error classes for LLM Cost Operations SDK
 */
/**
 * Base error class for all SDK errors
 */
declare class CostOpsError extends Error {
    readonly name: string;
    readonly timestamp: Date;
    readonly code?: string;
    constructor(message: string, code?: string);
    /**
     * Convert error to JSON representation
     */
    toJSON(): Record<string, unknown>;
}
/**
 * Configuration error - invalid client configuration
 */
declare class ConfigurationError extends CostOpsError {
    constructor(message: string);
}
/**
 * Validation error - invalid request parameters
 */
declare class ValidationError extends CostOpsError {
    readonly field?: string;
    readonly constraints?: Record<string, string>;
    constructor(message: string, field?: string, constraints?: Record<string, string>);
    toJSON(): Record<string, unknown>;
}
/**
 * Authentication error - missing or invalid API key
 */
declare class AuthenticationError extends CostOpsError {
    constructor(message?: string);
}
/**
 * Authorization error - insufficient permissions
 */
declare class AuthorizationError extends CostOpsError {
    readonly requiredPermission?: string;
    constructor(message?: string, requiredPermission?: string);
    toJSON(): Record<string, unknown>;
}
/**
 * API error - error response from the API
 */
declare class ApiError extends CostOpsError {
    readonly statusCode: number;
    readonly response?: unknown;
    readonly requestId?: string;
    constructor(message: string, statusCode: number, response?: unknown, requestId?: string);
    toJSON(): Record<string, unknown>;
}
/**
 * Network error - connection or network-related issues
 */
declare class NetworkError extends CostOpsError {
    readonly cause?: Error;
    constructor(message?: string, cause?: Error);
    toJSON(): Record<string, unknown>;
}
/**
 * Timeout error - request exceeded timeout limit
 */
declare class TimeoutError extends CostOpsError {
    readonly timeout: number;
    constructor(message: string, timeout: number);
    toJSON(): Record<string, unknown>;
}
/**
 * Rate limit error - too many requests
 */
declare class RateLimitError extends CostOpsError {
    readonly retryAfter?: number;
    readonly limit?: number;
    constructor(message?: string, retryAfter?: number, limit?: number);
    toJSON(): Record<string, unknown>;
}
/**
 * Not found error - resource not found
 */
declare class NotFoundError extends CostOpsError {
    readonly resourceType?: string;
    readonly resourceId?: string;
    constructor(message?: string, resourceType?: string, resourceId?: string);
    toJSON(): Record<string, unknown>;
}
/**
 * Conflict error - resource conflict (e.g., duplicate)
 */
declare class ConflictError extends CostOpsError {
    readonly conflictingField?: string;
    constructor(message?: string, conflictingField?: string);
    toJSON(): Record<string, unknown>;
}
/**
 * Server error - internal server error
 */
declare class ServerError extends CostOpsError {
    readonly statusCode: number;
    constructor(message?: string, statusCode?: number);
    toJSON(): Record<string, unknown>;
}
/**
 * Retry exhausted error - all retry attempts failed
 */
declare class RetryExhaustedError extends CostOpsError {
    readonly attempts: number;
    readonly lastError?: Error;
    constructor(message: string, attempts: number, lastError?: Error);
    toJSON(): Record<string, unknown>;
}
/**
 * Type guard to check if an error is a CostOpsError
 */
declare function isCostOpsError(error: unknown): error is CostOpsError;
/**
 * Type guard to check if an error is an ApiError
 */
declare function isApiError(error: unknown): error is ApiError;
/**
 * Type guard to check if an error is retryable
 */
declare function isRetryableError(error: unknown): boolean;

/**
 * HTTP utility functions for making requests
 */
/**
 * Fetch with timeout support
 */
declare function fetchWithTimeout(url: string, options?: RequestInit, timeout?: number): Promise<Response>;
/**
 * Build URL with query parameters
 */
declare function buildUrl(baseUrl: string, path: string, query?: Record<string, string | number | boolean | undefined>): string;
/**
 * Safely parse JSON response
 */
declare function parseJsonResponse<T>(response: Response): Promise<T>;
/**
 * Extract headers as plain object
 */
declare function extractHeaders(headers: Headers): Record<string, string>;
/**
 * Check if running in browser environment
 */
declare function isBrowser(): boolean;
/**
 * Check if running in Node.js environment
 */
declare function isNode(): boolean;
/**
 * Get user agent string based on environment
 */
declare function getUserAgent(): string;

/**
 * Retry utility functions with exponential backoff
 */
interface RetryOptions {
    /** Maximum number of retry attempts */
    maxRetries: number;
    /** Initial delay in milliseconds */
    initialDelay: number;
    /** Enable exponential backoff */
    exponentialBackoff: boolean;
    /** Maximum delay in milliseconds */
    maxDelay?: number;
    /** Function to determine if error should be retried */
    shouldRetry?: (error: unknown) => boolean;
    /** Callback on retry attempt */
    onRetry?: (attempt: number, error: unknown) => void;
}
/**
 * Calculate delay for retry attempt with exponential backoff
 */
declare function calculateRetryDelay(attempt: number, initialDelay: number, exponentialBackoff: boolean, maxDelay?: number): number;
/**
 * Sleep for specified milliseconds
 */
declare function sleep(ms: number): Promise<void>;
/**
 * Execute function with retry logic
 */
declare function withRetry<T>(fn: () => Promise<T>, options: RetryOptions): Promise<{
    result: T;
    attempts: number;
}>;

/**
 * Validation utility functions
 */

/**
 * Validate client configuration
 */
declare function validateConfig(config: Partial<ClientConfig>): void;
/**
 * Validate required string field
 */
declare function validateRequired(value: unknown, fieldName: string): void;
/**
 * Validate string length
 */
declare function validateStringLength(value: string, fieldName: string, min?: number, max?: number): void;
/**
 * Validate number range
 */
declare function validateNumberRange(value: number, fieldName: string, min?: number, max?: number): void;
/**
 * Validate ISO 8601 date string
 */
declare function validateISODate(value: string, fieldName: string): void;
/**
 * Validate array has items
 */
declare function validateNonEmptyArray<T>(value: T[], fieldName: string): void;
/**
 * Validate enum value
 */
declare function validateEnum<T extends string>(value: string, fieldName: string, validValues: readonly T[]): void;

export { type Alert, ApiError, type ApiResponse, AuthenticationError, AuthorizationError, BaseClient, type Budget, type ClientConfig, type ComponentHealth, ConfigurationError, ConflictError, type CostMetric, CostOpsClient, CostOpsError, type ErrorContext, type ErrorInterceptor, type ExportOptions, type Forecast, type HealthCheck, type HttpMethod, type MetricsQuery, type MiddlewareEvents, MiddlewareManager, NetworkError, NotFoundError, type PaginatedResponse, type PaginationMeta, RateLimitError, type RequestContext, type RequestInterceptor, type RequestMetrics, type RequestOptions, type ResponseContext, type ResponseInterceptor, RetryExhaustedError, type RetryOptions, ServerError, type ServiceUsage, TimeoutError, type UsageStats, ValidationError, type Webhook, type WebhookEvent, buildUrl, calculateRetryDelay, createAuthInterceptor, createLoggingInterceptor, createMetricsInterceptor, CostOpsClient as default, extractHeaders, fetchWithTimeout, getUserAgent, isApiError, isBrowser, isCostOpsError, isNode, isRetryableError, parseJsonResponse, sleep, validateConfig, validateEnum, validateISODate, validateNonEmptyArray, validateNumberRange, validateRequired, validateStringLength, withRetry };
