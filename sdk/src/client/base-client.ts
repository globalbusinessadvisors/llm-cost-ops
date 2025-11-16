/**
 * Base HTTP client with retry, interceptors, and error handling
 */

import {
  ApiError,
  AuthenticationError,
  AuthorizationError,
  ConflictError,
  NotFoundError,
  RateLimitError,
  ServerError,
} from '../errors/index.js';
import {
  createAuthInterceptor,
  createLoggingInterceptor,
  MiddlewareManager,
  type ErrorContext,
  type RequestContext,
  type ResponseContext,
} from '../middleware/index.js';
import type { ApiResponse, ClientConfig, RequestOptions } from '../types/index.js';
import { buildUrl, extractHeaders, fetchWithTimeout, getUserAgent, parseJsonResponse } from '../utils/http.js';
import { withRetry } from '../utils/retry.js';
import { validateConfig } from '../utils/validation.js';

/**
 * Base client class with HTTP functionality
 */
export class BaseClient {
  protected readonly config: Required<ClientConfig>;
  protected readonly middleware: MiddlewareManager;

  constructor(config: ClientConfig) {
    // Validate configuration
    validateConfig(config);

    // Set defaults
    this.config = {
      baseUrl: config.baseUrl,
      apiKey: config.apiKey ?? '',
      timeout: config.timeout ?? 30000,
      maxRetries: config.maxRetries ?? 3,
      retryDelay: config.retryDelay ?? 1000,
      exponentialBackoff: config.exponentialBackoff ?? true,
      headers: config.headers ?? {},
      debug: config.debug ?? false,
    };

    // Initialize middleware
    this.middleware = new MiddlewareManager();

    // Add built-in interceptors
    this.setupBuiltInInterceptors();
  }

  /**
   * Setup built-in interceptors
   */
  private setupBuiltInInterceptors(): void {
    // Add authentication interceptor if API key is provided
    if (this.config.apiKey !== '') {
      this.middleware.addRequestInterceptor(createAuthInterceptor(this.config.apiKey));
    }

    // Add logging interceptor if debug is enabled
    if (this.config.debug) {
      const loggingInterceptor = createLoggingInterceptor(this.config.debug);
      this.middleware.addRequestInterceptor(loggingInterceptor.request);
      this.middleware.addResponseInterceptor(loggingInterceptor.response);
      this.middleware.addErrorInterceptor(loggingInterceptor.error);
    }
  }

  /**
   * Make an HTTP request
   */
  protected async request<T>(options: RequestOptions): Promise<ApiResponse<T>> {
    const startTime = Date.now();

    // Create request context
    let requestContext: RequestContext = {
      request: options,
      metadata: {},
      timestamp: startTime,
    };

    try {
      // Process through request interceptors
      requestContext = await this.middleware.processRequest(requestContext);

      // Execute request with retry logic
      const { result, attempts } = await withRetry(
        () => this.executeRequest<T>(requestContext.request),
        {
          maxRetries: options.skipRetry === true ? 0 : this.config.maxRetries,
          initialDelay: this.config.retryDelay,
          exponentialBackoff: this.config.exponentialBackoff,
          onRetry: (attempt, error) => {
            this.middleware.emit('retry:attempt', attempt, error as Error);
          },
        }
      );

      // Create response context
      const responseContext: ResponseContext<T> = {
        response: {
          ...result,
          metadata: {
            ...result.metadata,
            duration: Date.now() - startTime,
            retries: attempts,
          },
        },
        request: requestContext,
        timestamp: Date.now(),
      };

      // Process through response interceptors
      const processedContext = await this.middleware.processResponse(responseContext);
      return processedContext.response;
    } catch (error) {
      // Create error context
      const errorContext: ErrorContext = {
        error: error as Error,
        request: requestContext,
        timestamp: Date.now(),
      };

      // Process through error interceptors
      await this.middleware.processError(errorContext);

      // Re-throw the error
      throw error;
    }
  }

  /**
   * Execute the actual HTTP request
   */
  private async executeRequest<T>(options: RequestOptions): Promise<ApiResponse<T>> {
    const { method, path, query, body, headers = {}, timeout = this.config.timeout } = options;

    // Build URL
    const url = buildUrl(this.config.baseUrl, path, query);

    // Prepare headers
    const requestHeaders: Record<string, string> = {
      'Content-Type': 'application/json',
      'User-Agent': getUserAgent(),
      ...this.config.headers,
      ...headers,
    };

    // Prepare request options
    const requestOptions: RequestInit = {
      method,
      headers: requestHeaders,
    };

    // Add body for non-GET requests
    if (body !== undefined && method !== 'GET') {
      requestOptions.body = JSON.stringify(body);
    }

    // Make the request
    const response = await fetchWithTimeout(url, requestOptions, timeout);

    // Extract response headers
    const responseHeaders = extractHeaders(response.headers);

    // Handle error responses
    if (!response.ok) {
      await this.handleErrorResponse(response, responseHeaders);
    }

    // Parse response body
    const data = await parseJsonResponse<T>(response);

    return {
      data,
      status: response.status,
      headers: responseHeaders,
      metadata: {
        requestId: responseHeaders['x-request-id'],
        duration: 0, // Will be set by caller
        retries: 0, // Will be set by caller
      },
    };
  }

  /**
   * Handle error responses
   */
  private async handleErrorResponse(
    response: Response,
    headers: Record<string, string>
  ): Promise<never> {
    const status = response.status;
    const requestId = headers['x-request-id'];

    let errorData: unknown;
    try {
      errorData = await parseJsonResponse(response);
    } catch {
      errorData = await response.text();
    }

    const errorMessage = this.extractErrorMessage(errorData);

    switch (status) {
      case 401:
        throw new AuthenticationError(errorMessage ?? 'Authentication failed');

      case 403:
        throw new AuthorizationError(errorMessage ?? 'Insufficient permissions');

      case 404:
        throw new NotFoundError(errorMessage ?? 'Resource not found');

      case 409:
        throw new ConflictError(errorMessage ?? 'Resource conflict');

      case 429: {
        const retryAfter = headers['retry-after'] !== undefined
          ? parseInt(headers['retry-after'], 10)
          : undefined;
        const rateLimit = headers['x-ratelimit-limit'] !== undefined
          ? parseInt(headers['x-ratelimit-limit'], 10)
          : undefined;
        throw new RateLimitError(errorMessage ?? 'Rate limit exceeded', retryAfter, rateLimit);
      }

      case 500:
      case 502:
      case 503:
      case 504:
        throw new ServerError(errorMessage ?? 'Internal server error', status);

      default:
        throw new ApiError(errorMessage ?? 'API request failed', status, errorData, requestId);
    }
  }

  /**
   * Extract error message from error response
   */
  private extractErrorMessage(errorData: unknown): string | undefined {
    if (typeof errorData === 'string') {
      return errorData;
    }

    if (typeof errorData === 'object' && errorData !== null) {
      const data = errorData as Record<string, unknown>;
      return (
        (data['message'] as string) ??
        (data['error'] as string) ??
        (data['detail'] as string) ??
        undefined
      );
    }

    return undefined;
  }

  /**
   * Make a GET request
   */
  protected async get<T>(
    path: string,
    query?: Record<string, string | number | boolean | undefined>
  ): Promise<ApiResponse<T>> {
    return this.request<T>({
      method: 'GET',
      path,
      query,
    });
  }

  /**
   * Make a POST request
   */
  protected async post<T>(path: string, body?: unknown): Promise<ApiResponse<T>> {
    return this.request<T>({
      method: 'POST',
      path,
      body,
    });
  }

  /**
   * Make a PUT request
   */
  protected async put<T>(path: string, body?: unknown): Promise<ApiResponse<T>> {
    return this.request<T>({
      method: 'PUT',
      path,
      body,
    });
  }

  /**
   * Make a PATCH request
   */
  protected async patch<T>(path: string, body?: unknown): Promise<ApiResponse<T>> {
    return this.request<T>({
      method: 'PATCH',
      path,
      body,
    });
  }

  /**
   * Make a DELETE request
   */
  protected async delete<T>(path: string): Promise<ApiResponse<T>> {
    return this.request<T>({
      method: 'DELETE',
      path,
    });
  }

  /**
   * Get middleware manager for adding custom interceptors
   */
  public getMiddleware(): MiddlewareManager {
    return this.middleware;
  }

  /**
   * Get current configuration (read-only)
   */
  public getConfig(): Readonly<Required<ClientConfig>> {
    return { ...this.config };
  }
}
