/**
 * Error classes tests
 */

import { describe, it, expect } from 'vitest';
import {
  CostOpsError,
  ConfigurationError,
  ValidationError,
  AuthenticationError,
  ApiError,
  RateLimitError,
  isCostOpsError,
  isApiError,
  isRetryableError,
} from '../src/errors/index.js';

describe('Error Classes', () => {
  describe('CostOpsError', () => {
    it('should create base error with message and code', () => {
      const error = new CostOpsError('Test error', 'TEST_ERROR');

      expect(error).toBeInstanceOf(Error);
      expect(error).toBeInstanceOf(CostOpsError);
      expect(error.message).toBe('Test error');
      expect(error.code).toBe('TEST_ERROR');
      expect(error.name).toBe('CostOpsError');
      expect(error.timestamp).toBeInstanceOf(Date);
    });

    it('should convert to JSON', () => {
      const error = new CostOpsError('Test error', 'TEST_ERROR');
      const json = error.toJSON();

      expect(json).toHaveProperty('name', 'CostOpsError');
      expect(json).toHaveProperty('message', 'Test error');
      expect(json).toHaveProperty('code', 'TEST_ERROR');
      expect(json).toHaveProperty('timestamp');
    });
  });

  describe('ConfigurationError', () => {
    it('should create configuration error', () => {
      const error = new ConfigurationError('Invalid config');

      expect(error).toBeInstanceOf(CostOpsError);
      expect(error.name).toBe('ConfigurationError');
      expect(error.code).toBe('CONFIGURATION_ERROR');
    });
  });

  describe('ValidationError', () => {
    it('should create validation error with field and constraints', () => {
      const error = new ValidationError('Invalid field', 'email', {
        format: 'email',
      });

      expect(error).toBeInstanceOf(CostOpsError);
      expect(error.name).toBe('ValidationError');
      expect(error.field).toBe('email');
      expect(error.constraints).toEqual({ format: 'email' });
    });
  });

  describe('AuthenticationError', () => {
    it('should create authentication error', () => {
      const error = new AuthenticationError();

      expect(error).toBeInstanceOf(CostOpsError);
      expect(error.name).toBe('AuthenticationError');
      expect(error.message).toBe('Authentication failed');
    });
  });

  describe('ApiError', () => {
    it('should create API error with status code', () => {
      const error = new ApiError('Bad request', 400, { detail: 'Invalid input' }, 'req-123');

      expect(error).toBeInstanceOf(CostOpsError);
      expect(error.name).toBe('ApiError');
      expect(error.statusCode).toBe(400);
      expect(error.response).toEqual({ detail: 'Invalid input' });
      expect(error.requestId).toBe('req-123');
    });
  });

  describe('RateLimitError', () => {
    it('should create rate limit error with retry info', () => {
      const error = new RateLimitError('Rate limited', 60, 100);

      expect(error).toBeInstanceOf(CostOpsError);
      expect(error.name).toBe('RateLimitError');
      expect(error.retryAfter).toBe(60);
      expect(error.limit).toBe(100);
    });
  });

  describe('Type Guards', () => {
    it('should identify CostOpsError', () => {
      const error = new CostOpsError('Test');
      const regularError = new Error('Test');

      expect(isCostOpsError(error)).toBe(true);
      expect(isCostOpsError(regularError)).toBe(false);
    });

    it('should identify ApiError', () => {
      const apiError = new ApiError('Test', 400);
      const otherError = new ConfigurationError('Test');

      expect(isApiError(apiError)).toBe(true);
      expect(isApiError(otherError)).toBe(false);
    });

    it('should identify retryable errors', () => {
      const rateLimitError = new RateLimitError();
      const serverError = new ApiError('Server error', 500);
      const notFoundError = new ApiError('Not found', 404);

      expect(isRetryableError(rateLimitError)).toBe(true);
      expect(isRetryableError(serverError)).toBe(true);
      expect(isRetryableError(notFoundError)).toBe(false);
    });
  });
});
