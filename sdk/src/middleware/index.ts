/**
 * Middleware and interceptor system for request/response processing
 */

import EventEmitter from 'eventemitter3';

import type { ApiResponse, RequestOptions } from '../types/index.js';

/**
 * Request context passed to interceptors
 */
export interface RequestContext {
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
export interface ResponseContext<T = unknown> {
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
export interface ErrorContext {
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
export type RequestInterceptor = (
  context: RequestContext
) => RequestContext | Promise<RequestContext>;

/**
 * Response interceptor function
 */
export type ResponseInterceptor = <T>(
  context: ResponseContext<T>
) => ResponseContext<T> | Promise<ResponseContext<T>>;

/**
 * Error interceptor function
 */
export type ErrorInterceptor = (
  context: ErrorContext
) => ErrorContext | Promise<ErrorContext>;

/**
 * Middleware events
 */
export interface MiddlewareEvents {
  'request:start': (context: RequestContext) => void;
  'request:end': <T>(context: ResponseContext<T>) => void;
  'request:error': (context: ErrorContext) => void;
  'retry:attempt': (attempt: number, error: Error) => void;
}

/**
 * Middleware manager for handling interceptors and events
 */
export class MiddlewareManager extends EventEmitter<MiddlewareEvents> {
  private requestInterceptors: RequestInterceptor[] = [];
  private responseInterceptors: ResponseInterceptor[] = [];
  private errorInterceptors: ErrorInterceptor[] = [];

  /**
   * Add a request interceptor
   */
  public addRequestInterceptor(interceptor: RequestInterceptor): () => void {
    this.requestInterceptors.push(interceptor);

    // Return function to remove the interceptor
    return () => {
      const index = this.requestInterceptors.indexOf(interceptor);
      if (index > -1) {
        this.requestInterceptors.splice(index, 1);
      }
    };
  }

  /**
   * Add a response interceptor
   */
  public addResponseInterceptor(interceptor: ResponseInterceptor): () => void {
    this.responseInterceptors.push(interceptor);

    // Return function to remove the interceptor
    return () => {
      const index = this.responseInterceptors.indexOf(interceptor);
      if (index > -1) {
        this.responseInterceptors.splice(index, 1);
      }
    };
  }

  /**
   * Add an error interceptor
   */
  public addErrorInterceptor(interceptor: ErrorInterceptor): () => void {
    this.errorInterceptors.push(interceptor);

    // Return function to remove the interceptor
    return () => {
      const index = this.errorInterceptors.indexOf(interceptor);
      if (index > -1) {
        this.errorInterceptors.splice(index, 1);
      }
    };
  }

  /**
   * Process request through all request interceptors
   */
  public async processRequest(context: RequestContext): Promise<RequestContext> {
    let processedContext = context;

    for (const interceptor of this.requestInterceptors) {
      processedContext = await interceptor(processedContext);
    }

    this.emit('request:start', processedContext);
    return processedContext;
  }

  /**
   * Process response through all response interceptors
   */
  public async processResponse<T>(context: ResponseContext<T>): Promise<ResponseContext<T>> {
    let processedContext = context;

    for (const interceptor of this.responseInterceptors) {
      processedContext = await interceptor(processedContext);
    }

    this.emit('request:end', processedContext);
    return processedContext;
  }

  /**
   * Process error through all error interceptors
   */
  public async processError(context: ErrorContext): Promise<ErrorContext> {
    let processedContext = context;

    for (const interceptor of this.errorInterceptors) {
      processedContext = await interceptor(processedContext);
    }

    this.emit('request:error', processedContext);
    return processedContext;
  }

  /**
   * Clear all interceptors
   */
  public clearInterceptors(): void {
    this.requestInterceptors = [];
    this.responseInterceptors = [];
    this.errorInterceptors = [];
  }

  /**
   * Get count of registered interceptors
   */
  public getInterceptorCount(): {
    request: number;
    response: number;
    error: number;
  } {
    return {
      request: this.requestInterceptors.length,
      response: this.responseInterceptors.length,
      error: this.errorInterceptors.length,
    };
  }
}

/**
 * Built-in request interceptor for logging
 */
export function createLoggingInterceptor(debug: boolean): {
  request: RequestInterceptor;
  response: ResponseInterceptor;
  error: ErrorInterceptor;
} {
  return {
    request: (context: RequestContext): RequestContext => {
      if (debug) {
        console.log('[SDK Request]', {
          method: context.request.method,
          path: context.request.path,
          timestamp: new Date(context.timestamp).toISOString(),
        });
      }
      return context;
    },
    response: <T>(context: ResponseContext<T>): ResponseContext<T> => {
      if (debug) {
        console.log('[SDK Response]', {
          status: context.response.status,
          duration: context.response.metadata.duration,
          timestamp: new Date(context.timestamp).toISOString(),
        });
      }
      return context;
    },
    error: (context: ErrorContext): ErrorContext => {
      if (debug) {
        console.error('[SDK Error]', {
          error: context.error.message,
          timestamp: new Date(context.timestamp).toISOString(),
        });
      }
      return context;
    },
  };
}

/**
 * Built-in request interceptor for adding authentication headers
 */
export function createAuthInterceptor(apiKey: string): RequestInterceptor {
  return (context: RequestContext) => {
    if (!context.request.headers) {
      context.request.headers = {};
    }

    context.request.headers['Authorization'] = `Bearer ${apiKey}`;
    return context;
  };
}

/**
 * Built-in request interceptor for tracking request metrics
 */
export function createMetricsInterceptor(): {
  request: RequestInterceptor;
  response: ResponseInterceptor;
  error: ErrorInterceptor;
  getMetrics: () => RequestMetrics;
} {
  const metrics: RequestMetrics = {
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    totalDuration: 0,
    averageDuration: 0,
  };

  return {
    request: (context: RequestContext): RequestContext => {
      metrics.totalRequests++;
      return context;
    },
    response: <T>(context: ResponseContext<T>): ResponseContext<T> => {
      metrics.successfulRequests++;
      metrics.totalDuration += context.response.metadata.duration;
      metrics.averageDuration = metrics.totalDuration / metrics.successfulRequests;
      return context;
    },
    error: (context: ErrorContext): ErrorContext => {
      metrics.failedRequests++;
      return context;
    },
    getMetrics: (): RequestMetrics => ({ ...metrics }),
  };
}

/**
 * Request metrics tracked by metrics interceptor
 */
export interface RequestMetrics {
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  totalDuration: number;
  averageDuration: number;
}
