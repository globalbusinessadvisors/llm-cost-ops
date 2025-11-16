/**
 * Retry utility functions with exponential backoff
 */

import { isRetryableError } from '../errors/index.js';
import { RetryExhaustedError } from '../errors/index.js';

export interface RetryOptions {
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
export function calculateRetryDelay(
  attempt: number,
  initialDelay: number,
  exponentialBackoff: boolean,
  maxDelay: number = 60000
): number {
  if (!exponentialBackoff) {
    return initialDelay;
  }

  // Exponential backoff: initialDelay * 2^attempt with jitter
  const exponentialDelay = initialDelay * Math.pow(2, attempt);
  const jitter = Math.random() * 0.3 * exponentialDelay; // Add up to 30% jitter
  const delay = Math.min(exponentialDelay + jitter, maxDelay);

  return Math.floor(delay);
}

/**
 * Sleep for specified milliseconds
 */
export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Execute function with retry logic
 */
export async function withRetry<T>(
  fn: () => Promise<T>,
  options: RetryOptions
): Promise<{ result: T; attempts: number }> {
  const {
    maxRetries,
    initialDelay,
    exponentialBackoff,
    maxDelay,
    shouldRetry = isRetryableError,
    onRetry,
  } = options;

  let lastError: Error | undefined;
  let attempts = 0;

  while (attempts <= maxRetries) {
    try {
      const result = await fn();
      return { result, attempts };
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      attempts++;

      // Don't retry if we've exhausted attempts or error is not retryable
      if (attempts > maxRetries || !shouldRetry(error)) {
        break;
      }

      // Calculate delay and notify
      const delay = calculateRetryDelay(attempts - 1, initialDelay, exponentialBackoff, maxDelay);

      if (onRetry) {
        onRetry(attempts, error);
      }

      // Wait before retrying
      await sleep(delay);
    }
  }

  // All retries exhausted
  throw new RetryExhaustedError(
    `Failed after ${attempts} attempts: ${lastError?.message ?? 'Unknown error'}`,
    attempts,
    lastError
  );
}
