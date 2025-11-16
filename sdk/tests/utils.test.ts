/**
 * Utility functions tests
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  buildUrl,
  calculateRetryDelay,
  sleep,
  validateConfig,
  validateRequired,
  validateEnum,
} from '../src/utils/index.js';
import { ConfigurationError, ValidationError } from '../src/errors/index.js';

describe('Utils', () => {
  describe('buildUrl', () => {
    it('should build URL without query parameters', () => {
      const url = buildUrl('https://api.example.com', '/metrics');

      expect(url).toBe('https://api.example.com/metrics');
    });

    it('should build URL with query parameters', () => {
      const url = buildUrl('https://api.example.com', '/metrics', {
        limit: 10,
        offset: 0,
        active: true,
      });

      expect(url).toContain('limit=10');
      expect(url).toContain('offset=0');
      expect(url).toContain('active=true');
    });

    it('should skip undefined query parameters', () => {
      const url = buildUrl('https://api.example.com', '/metrics', {
        limit: 10,
        offset: undefined,
      });

      expect(url).toContain('limit=10');
      expect(url).not.toContain('offset');
    });
  });

  describe('calculateRetryDelay', () => {
    it('should return initial delay without exponential backoff', () => {
      const delay = calculateRetryDelay(0, 1000, false);

      expect(delay).toBe(1000);
    });

    it('should calculate exponential backoff', () => {
      const delay1 = calculateRetryDelay(0, 1000, true);
      const delay2 = calculateRetryDelay(1, 1000, true);

      expect(delay1).toBeGreaterThanOrEqual(1000);
      expect(delay2).toBeGreaterThan(delay1);
    });

    it('should respect max delay', () => {
      const delay = calculateRetryDelay(10, 1000, true, 5000);

      expect(delay).toBeLessThanOrEqual(5000);
    });
  });

  describe('sleep', () => {
    beforeEach(() => {
      vi.useFakeTimers();
    });

    afterEach(() => {
      vi.useRealTimers();
    });

    it('should sleep for specified duration', async () => {
      const promise = sleep(1000);
      vi.advanceTimersByTime(1000);
      await promise;

      expect(true).toBe(true); // Sleep completed
    });
  });

  describe('validateConfig', () => {
    it('should validate valid configuration', () => {
      expect(() => {
        validateConfig({
          baseUrl: 'https://api.example.com',
          timeout: 5000,
          maxRetries: 3,
        });
      }).not.toThrow();
    });

    it('should throw on missing baseUrl', () => {
      expect(() => {
        validateConfig({} as any);
      }).toThrow(ConfigurationError);
    });

    it('should throw on invalid URL', () => {
      expect(() => {
        validateConfig({
          baseUrl: 'not-a-url',
        });
      }).toThrow(ConfigurationError);
    });

    it('should throw on invalid timeout', () => {
      expect(() => {
        validateConfig({
          baseUrl: 'https://api.example.com',
          timeout: -1,
        });
      }).toThrow(ConfigurationError);
    });
  });

  describe('validateRequired', () => {
    it('should pass for valid value', () => {
      expect(() => {
        validateRequired('test', 'field');
      }).not.toThrow();
    });

    it('should throw for undefined', () => {
      expect(() => {
        validateRequired(undefined, 'field');
      }).toThrow(ValidationError);
    });

    it('should throw for empty string', () => {
      expect(() => {
        validateRequired('', 'field');
      }).toThrow(ValidationError);
    });
  });

  describe('validateEnum', () => {
    it('should pass for valid enum value', () => {
      expect(() => {
        validateEnum('active', 'status', ['active', 'inactive'] as const);
      }).not.toThrow();
    });

    it('should throw for invalid enum value', () => {
      expect(() => {
        validateEnum('invalid', 'status', ['active', 'inactive'] as const);
      }).toThrow(ValidationError);
    });
  });
});
