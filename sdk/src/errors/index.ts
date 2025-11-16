/**
 * Custom error classes for LLM Cost Operations SDK
 */

/**
 * Base error class for all SDK errors
 */
export class CostOpsError extends Error {
  public override readonly name: string;
  public readonly timestamp: Date;
  public readonly code?: string;

  constructor(message: string, code?: string) {
    super(message);
    this.name = this.constructor.name;
    this.code = code;
    this.timestamp = new Date();

    // Maintains proper stack trace for where our error was thrown (only available on V8)
    if (Error.captureStackTrace !== undefined) {
      Error.captureStackTrace(this, this.constructor);
    }
  }

  /**
   * Convert error to JSON representation
   */
  public toJSON(): Record<string, unknown> {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      timestamp: this.timestamp.toISOString(),
      stack: this.stack,
    };
  }
}

/**
 * Configuration error - invalid client configuration
 */
export class ConfigurationError extends CostOpsError {
  constructor(message: string) {
    super(message, 'CONFIGURATION_ERROR');
    Object.defineProperty(this, 'name', { value: 'ConfigurationError' });
  }
}

/**
 * Validation error - invalid request parameters
 */
export class ValidationError extends CostOpsError {
  public readonly field?: string;
  public readonly constraints?: Record<string, string>;

  constructor(message: string, field?: string, constraints?: Record<string, string>) {
    super(message, 'VALIDATION_ERROR');
    Object.defineProperty(this, 'name', { value: 'ValidationError' });
    this.field = field;
    this.constraints = constraints;
  }

  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      field: this.field,
      constraints: this.constraints,
    };
  }
}

/**
 * Authentication error - missing or invalid API key
 */
export class AuthenticationError extends CostOpsError {
  constructor(message: string = 'Authentication failed') {
    super(message, 'AUTHENTICATION_ERROR');
    Object.defineProperty(this, 'name', { value: 'AuthenticationError' });
  }
}

/**
 * Authorization error - insufficient permissions
 */
export class AuthorizationError extends CostOpsError {
  public readonly requiredPermission?: string;

  constructor(message: string = 'Insufficient permissions', requiredPermission?: string) {
    super(message, 'AUTHORIZATION_ERROR');
    Object.defineProperty(this, 'name', { value: 'AuthorizationError' });
    this.requiredPermission = requiredPermission;
  }

  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      requiredPermission: this.requiredPermission,
    };
  }
}

/**
 * API error - error response from the API
 */
export class ApiError extends CostOpsError {
  public readonly statusCode: number;
  public readonly response?: unknown;
  public readonly requestId?: string;

  constructor(
    message: string,
    statusCode: number,
    response?: unknown,
    requestId?: string
  ) {
    super(message, 'API_ERROR');
    Object.defineProperty(this, 'name', { value: 'ApiError' });
    this.statusCode = statusCode;
    this.response = response;
    this.requestId = requestId;
  }

  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      statusCode: this.statusCode,
      response: this.response,
      requestId: this.requestId,
    };
  }
}

/**
 * Network error - connection or network-related issues
 */
export class NetworkError extends CostOpsError {
  public override readonly cause?: Error;

  constructor(message: string = 'Network request failed', cause?: Error) {
    super(message, 'NETWORK_ERROR');
    Object.defineProperty(this, 'name', { value: 'NetworkError' });
    this.cause = cause;
  }

  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      cause: this.cause?.message,
    };
  }
}

/**
 * Timeout error - request exceeded timeout limit
 */
export class TimeoutError extends CostOpsError {
  public readonly timeout: number;

  constructor(message: string, timeout: number) {
    super(message, 'TIMEOUT_ERROR');
    Object.defineProperty(this, 'name', { value: 'TimeoutError' });
    this.timeout = timeout;
  }

  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      timeout: this.timeout,
    };
  }
}

/**
 * Rate limit error - too many requests
 */
export class RateLimitError extends CostOpsError {
  public readonly retryAfter?: number;
  public readonly limit?: number;

  constructor(
    message: string = 'Rate limit exceeded',
    retryAfter?: number,
    limit?: number
  ) {
    super(message, 'RATE_LIMIT_ERROR');
    Object.defineProperty(this, 'name', { value: 'RateLimitError' });
    this.retryAfter = retryAfter;
    this.limit = limit;
  }

  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      retryAfter: this.retryAfter,
      limit: this.limit,
    };
  }
}

/**
 * Not found error - resource not found
 */
export class NotFoundError extends CostOpsError {
  public readonly resourceType?: string;
  public readonly resourceId?: string;

  constructor(
    message: string = 'Resource not found',
    resourceType?: string,
    resourceId?: string
  ) {
    super(message, 'NOT_FOUND_ERROR');
    Object.defineProperty(this, 'name', { value: 'NotFoundError' });
    this.resourceType = resourceType;
    this.resourceId = resourceId;
  }

  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      resourceType: this.resourceType,
      resourceId: this.resourceId,
    };
  }
}

/**
 * Conflict error - resource conflict (e.g., duplicate)
 */
export class ConflictError extends CostOpsError {
  public readonly conflictingField?: string;

  constructor(message: string = 'Resource conflict', conflictingField?: string) {
    super(message, 'CONFLICT_ERROR');
    Object.defineProperty(this, 'name', { value: 'ConflictError' });
    this.conflictingField = conflictingField;
  }

  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      conflictingField: this.conflictingField,
    };
  }
}

/**
 * Server error - internal server error
 */
export class ServerError extends CostOpsError {
  public readonly statusCode: number;

  constructor(message: string = 'Internal server error', statusCode: number = 500) {
    super(message, 'SERVER_ERROR');
    Object.defineProperty(this, 'name', { value: 'ServerError' });
    this.statusCode = statusCode;
  }

  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      statusCode: this.statusCode,
    };
  }
}

/**
 * Retry exhausted error - all retry attempts failed
 */
export class RetryExhaustedError extends CostOpsError {
  public readonly attempts: number;
  public readonly lastError?: Error;

  constructor(message: string, attempts: number, lastError?: Error) {
    super(message, 'RETRY_EXHAUSTED_ERROR');
    Object.defineProperty(this, 'name', { value: 'RetryExhaustedError' });
    this.attempts = attempts;
    this.lastError = lastError;
  }

  public override toJSON(): Record<string, unknown> {
    return {
      ...super.toJSON(),
      attempts: this.attempts,
      lastError: this.lastError?.message,
    };
  }
}

/**
 * Type guard to check if an error is a CostOpsError
 */
export function isCostOpsError(error: unknown): error is CostOpsError {
  return error instanceof CostOpsError;
}

/**
 * Type guard to check if an error is an ApiError
 */
export function isApiError(error: unknown): error is ApiError {
  return error instanceof ApiError;
}

/**
 * Type guard to check if an error is retryable
 */
export function isRetryableError(error: unknown): boolean {
  if (error instanceof NetworkError) {
    return true;
  }
  if (error instanceof TimeoutError) {
    return true;
  }
  if (error instanceof RateLimitError) {
    return true;
  }
  if (error instanceof ApiError) {
    // Retry on 5xx errors and 429 (rate limit)
    return error.statusCode >= 500 || error.statusCode === 429;
  }
  return false;
}
