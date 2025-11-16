/**
 * Validation utility functions
 */

import { ConfigurationError, ValidationError } from '../errors/index.js';
import type { ClientConfig } from '../types/index.js';

/**
 * Validate client configuration
 */
export function validateConfig(config: Partial<ClientConfig>): void {
  if (config.baseUrl === undefined || config.baseUrl === '') {
    throw new ConfigurationError('baseUrl is required');
  }

  try {
    new URL(config.baseUrl);
  } catch {
    throw new ConfigurationError('baseUrl must be a valid URL');
  }

  if (config.timeout !== undefined && config.timeout <= 0) {
    throw new ConfigurationError('timeout must be a positive number');
  }

  if (config.maxRetries !== undefined && config.maxRetries < 0) {
    throw new ConfigurationError('maxRetries must be non-negative');
  }

  if (config.retryDelay !== undefined && config.retryDelay <= 0) {
    throw new ConfigurationError('retryDelay must be a positive number');
  }
}

/**
 * Validate required string field
 */
export function validateRequired(value: unknown, fieldName: string): void {
  if (value === undefined || value === null || value === '') {
    throw new ValidationError(`${fieldName} is required`, fieldName);
  }
}

/**
 * Validate string length
 */
export function validateStringLength(
  value: string,
  fieldName: string,
  min?: number,
  max?: number
): void {
  if (min !== undefined && value.length < min) {
    throw new ValidationError(
      `${fieldName} must be at least ${min} characters`,
      fieldName,
      { minLength: String(min) }
    );
  }

  if (max !== undefined && value.length > max) {
    throw new ValidationError(
      `${fieldName} must be at most ${max} characters`,
      fieldName,
      { maxLength: String(max) }
    );
  }
}

/**
 * Validate number range
 */
export function validateNumberRange(
  value: number,
  fieldName: string,
  min?: number,
  max?: number
): void {
  if (min !== undefined && value < min) {
    throw new ValidationError(
      `${fieldName} must be at least ${min}`,
      fieldName,
      { min: String(min) }
    );
  }

  if (max !== undefined && value > max) {
    throw new ValidationError(
      `${fieldName} must be at most ${max}`,
      fieldName,
      { max: String(max) }
    );
  }
}

/**
 * Validate ISO 8601 date string
 */
export function validateISODate(value: string, fieldName: string): void {
  const date = new Date(value);
  if (isNaN(date.getTime())) {
    throw new ValidationError(
      `${fieldName} must be a valid ISO 8601 date string`,
      fieldName
    );
  }
}

/**
 * Validate array has items
 */
export function validateNonEmptyArray<T>(
  value: T[],
  fieldName: string
): void {
  if (!Array.isArray(value) || value.length === 0) {
    throw new ValidationError(`${fieldName} must be a non-empty array`, fieldName);
  }
}

/**
 * Validate enum value
 */
export function validateEnum<T extends string>(
  value: string,
  fieldName: string,
  validValues: readonly T[]
): void {
  if (!validValues.includes(value as T)) {
    throw new ValidationError(
      `${fieldName} must be one of: ${validValues.join(', ')}`,
      fieldName,
      { enum: validValues.join(', ') }
    );
  }
}
