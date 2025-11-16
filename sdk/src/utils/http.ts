/**
 * HTTP utility functions for making requests
 */

import { NetworkError, TimeoutError } from '../errors/index.js';

/**
 * Fetch with timeout support
 */
export async function fetchWithTimeout(
  url: string,
  options: RequestInit = {},
  timeout: number = 30000
): Promise<Response> {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeout);

  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal,
    });
    clearTimeout(timeoutId);
    return response;
  } catch (error) {
    clearTimeout(timeoutId);

    if (error instanceof Error) {
      if (error.name === 'AbortError') {
        throw new TimeoutError(`Request timeout after ${timeout}ms`, timeout);
      }
      throw new NetworkError('Network request failed', error);
    }

    throw new NetworkError('Unknown network error');
  }
}

/**
 * Build URL with query parameters
 */
export function buildUrl(
  baseUrl: string,
  path: string,
  query?: Record<string, string | number | boolean | undefined>
): string {
  const url = new URL(path, baseUrl);

  if (query) {
    Object.entries(query).forEach(([key, value]) => {
      if (value !== undefined && value !== null) {
        url.searchParams.append(key, String(value));
      }
    });
  }

  return url.toString();
}

/**
 * Safely parse JSON response
 */
export async function parseJsonResponse<T>(response: Response): Promise<T> {
  const text = await response.text();

  if (text === '' || text.trim() === '') {
    return {} as T;
  }

  try {
    return JSON.parse(text) as T;
  } catch (error) {
    throw new Error(`Failed to parse JSON response: ${text.substring(0, 100)}`);
  }
}

/**
 * Extract headers as plain object
 */
export function extractHeaders(headers: Headers): Record<string, string> {
  const result: Record<string, string> = {};
  headers.forEach((value, key) => {
    result[key] = value;
  });
  return result;
}

/**
 * Check if running in browser environment
 */
export function isBrowser(): boolean {
  return typeof window !== 'undefined' && typeof document !== 'undefined';
}

/**
 * Check if running in Node.js environment
 */
export function isNode(): boolean {
  return (
    typeof process !== 'undefined' &&
    process.versions != null &&
    process.versions.node != null
  );
}

/**
 * Get user agent string based on environment
 */
export function getUserAgent(): string {
  const version = '1.0.0'; // This would be injected during build

  if (isNode()) {
    return `llm-cost-ops-sdk-node/${version}`;
  }

  if (isBrowser()) {
    return `llm-cost-ops-sdk-browser/${version}`;
  }

  return `llm-cost-ops-sdk/${version}`;
}
