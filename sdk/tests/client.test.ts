/**
 * Client tests
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { CostOpsClient } from '../src/client/cost-ops-client.js';
import { ConfigurationError, ValidationError } from '../src/errors/index.js';

// Mock fetch globally
global.fetch = vi.fn();

describe('CostOpsClient', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Constructor', () => {
    it('should create client with valid config', () => {
      const client = new CostOpsClient({
        baseUrl: 'https://api.example.com',
        apiKey: 'test-key',
      });

      expect(client).toBeInstanceOf(CostOpsClient);
    });

    it('should throw on invalid config', () => {
      expect(() => {
        new CostOpsClient({} as any);
      }).toThrow(ConfigurationError);
    });

    it('should use default values', () => {
      const client = new CostOpsClient({
        baseUrl: 'https://api.example.com',
      });

      const config = client.getConfig();
      expect(config.timeout).toBe(30000);
      expect(config.maxRetries).toBe(3);
      expect(config.exponentialBackoff).toBe(true);
    });
  });

  describe('API Methods', () => {
    let client: CostOpsClient;

    beforeEach(() => {
      client = new CostOpsClient({
        baseUrl: 'https://api.example.com',
        apiKey: 'test-key',
      });
    });

    it('should get health check', async () => {
      const mockResponse = {
        status: 'healthy',
        version: '1.0.0',
        components: {
          database: { status: 'up' },
          cache: { status: 'up' },
          queue: { status: 'up' },
        },
        timestamp: new Date().toISOString(),
      };

      (global.fetch as any).mockResolvedValueOnce({
        ok: true,
        status: 200,
        headers: new Headers(),
        json: async () => mockResponse,
        text: async () => JSON.stringify(mockResponse),
      });

      const health = await client.health();
      expect(health.status).toBe('healthy');
    });

    it('should validate required fields in createMetric', async () => {
      await expect(
        client.createMetric({
          service: '',
          cost: 10,
          currency: 'USD',
          timestamp: new Date().toISOString(),
        } as any)
      ).rejects.toThrow(ValidationError);
    });

    it('should validate enum values', async () => {
      await expect(
        client.getForecast('invalid' as any)
      ).rejects.toThrow(ValidationError);
    });
  });

  describe('Middleware', () => {
    it('should allow adding custom interceptors', () => {
      const client = new CostOpsClient({
        baseUrl: 'https://api.example.com',
      });

      const middleware = client.getMiddleware();
      const interceptor = vi.fn((ctx) => ctx);

      middleware.addRequestInterceptor(interceptor);
      expect(middleware.getInterceptorCount().request).toBeGreaterThan(0);
    });
  });
});
