/**
 * Middleware tests
 */

import { describe, it, expect, vi } from 'vitest';
import {
  MiddlewareManager,
  createLoggingInterceptor,
  createAuthInterceptor,
  createMetricsInterceptor,
  type RequestContext,
  type ResponseContext,
  type ErrorContext,
} from '../src/middleware/index.js';

describe('Middleware', () => {
  describe('MiddlewareManager', () => {
    it('should add and remove request interceptors', () => {
      const manager = new MiddlewareManager();
      const interceptor = (ctx: RequestContext) => ctx;

      const remove = manager.addRequestInterceptor(interceptor);
      expect(manager.getInterceptorCount().request).toBe(1);

      remove();
      expect(manager.getInterceptorCount().request).toBe(0);
    });

    it('should process request through interceptors', async () => {
      const manager = new MiddlewareManager();
      const spy = vi.fn((ctx: RequestContext) => ctx);

      manager.addRequestInterceptor(spy);

      const context: RequestContext = {
        request: {
          method: 'GET',
          path: '/test',
        },
        metadata: {},
        timestamp: Date.now(),
      };

      await manager.processRequest(context);
      expect(spy).toHaveBeenCalledWith(context);
    });

    it('should emit events', async () => {
      const manager = new MiddlewareManager();
      const spy = vi.fn();

      manager.on('request:start', spy);

      const context: RequestContext = {
        request: {
          method: 'GET',
          path: '/test',
        },
        metadata: {},
        timestamp: Date.now(),
      };

      await manager.processRequest(context);
      expect(spy).toHaveBeenCalled();
    });

    it('should clear all interceptors', () => {
      const manager = new MiddlewareManager();

      manager.addRequestInterceptor((ctx) => ctx);
      manager.addResponseInterceptor((ctx) => ctx);
      manager.addErrorInterceptor((ctx) => ctx);

      manager.clearInterceptors();

      const counts = manager.getInterceptorCount();
      expect(counts.request).toBe(0);
      expect(counts.response).toBe(0);
      expect(counts.error).toBe(0);
    });
  });

  describe('createAuthInterceptor', () => {
    it('should add authorization header', () => {
      const interceptor = createAuthInterceptor('test-api-key');

      const context: RequestContext = {
        request: {
          method: 'GET',
          path: '/test',
        },
        metadata: {},
        timestamp: Date.now(),
      };

      const result = interceptor(context);
      expect(result.request.headers).toHaveProperty('Authorization', 'Bearer test-api-key');
    });
  });

  describe('createMetricsInterceptor', () => {
    it('should track request metrics', async () => {
      const { request, response, getMetrics } = createMetricsInterceptor();

      const requestContext: RequestContext = {
        request: {
          method: 'GET',
          path: '/test',
        },
        metadata: {},
        timestamp: Date.now(),
      };

      request(requestContext);

      const responseContext: ResponseContext = {
        response: {
          data: {},
          status: 200,
          headers: {},
          metadata: {
            duration: 100,
            retries: 0,
          },
        },
        request: requestContext,
        timestamp: Date.now(),
      };

      response(responseContext);

      const metrics = getMetrics();
      expect(metrics.totalRequests).toBe(1);
      expect(metrics.successfulRequests).toBe(1);
      expect(metrics.failedRequests).toBe(0);
    });
  });
});
