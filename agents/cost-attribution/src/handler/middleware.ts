/**
 * Middleware Functions
 *
 * Input validation, error handling, logging, and rate limiting
 */

import type { Request, Response } from '@google-cloud/functions-framework';
import { ZodError } from 'zod';
import { costAttributionInputSchema } from '../schemas';
import { sendValidationError, sendRateLimitError } from './response';
import type { CostAttributionInput, ValidationError } from '../types';

/**
 * In-memory rate limiter for edge function
 * Maps IP address to array of request timestamps
 */
const rateLimitStore = new Map<string, number[]>();
const RATE_LIMIT_WINDOW_MS = 60_000; // 1 minute
const RATE_LIMIT_MAX_REQUESTS = 100; // 100 requests per minute

/**
 * Validate request input using Zod schema
 */
export function validateInput(
  req: Request,
  res: Response
): CostAttributionInput | null {
  try {
    // Parse request body
    const input = costAttributionInputSchema.parse(req.body);
    return input;
  } catch (error) {
    if (error instanceof ZodError) {
      const details: ValidationError[] = error.errors.map((err) => ({
        field: err.path.join('.'),
        message: err.message,
        value: err.code === 'invalid_type' ? undefined : err.input,
      }));

      sendValidationError(
        res,
        'Invalid input data',
        details,
        req.body?.requestId
      );
      return null;
    }

    sendValidationError(res, 'Invalid request body', undefined, req.body?.requestId);
    return null;
  }
}

/**
 * Rate limiting middleware
 * Returns true if request is allowed, false if rate limit exceeded
 */
export function checkRateLimit(req: Request, res: Response): boolean {
  // Get client IP address
  const ip = getClientIp(req);

  // Get current timestamp
  const now = Date.now();

  // Get or create request history for this IP
  let requestHistory = rateLimitStore.get(ip) || [];

  // Remove timestamps outside the window
  requestHistory = requestHistory.filter(
    (timestamp) => now - timestamp < RATE_LIMIT_WINDOW_MS
  );

  // Check if rate limit exceeded
  if (requestHistory.length >= RATE_LIMIT_MAX_REQUESTS) {
    sendRateLimitError(res, 'Rate limit exceeded. Maximum 100 requests per minute.', req.body?.requestId);
    return false;
  }

  // Add current timestamp
  requestHistory.push(now);

  // Update store
  rateLimitStore.set(ip, requestHistory);

  // Clean up old entries periodically (every 100 requests)
  if (Math.random() < 0.01) {
    cleanupRateLimitStore();
  }

  return true;
}

/**
 * Get client IP address from request
 */
function getClientIp(req: Request): string {
  // Check various headers for IP address
  const forwarded = req.headers['x-forwarded-for'];
  if (typeof forwarded === 'string') {
    return forwarded.split(',')[0].trim();
  }

  const realIp = req.headers['x-real-ip'];
  if (typeof realIp === 'string') {
    return realIp;
  }

  return req.ip || 'unknown';
}

/**
 * Clean up rate limit store by removing old entries
 */
function cleanupRateLimitStore(): void {
  const now = Date.now();
  const cutoff = now - RATE_LIMIT_WINDOW_MS;

  for (const [ip, timestamps] of rateLimitStore.entries()) {
    const validTimestamps = timestamps.filter((ts) => ts > cutoff);
    if (validTimestamps.length === 0) {
      rateLimitStore.delete(ip);
    } else {
      rateLimitStore.set(ip, validTimestamps);
    }
  }
}

/**
 * Logging middleware
 */
export function logRequest(
  req: Request,
  requestId: string,
  level: 'info' | 'warn' | 'error' = 'info'
): void {
  const log = {
    timestamp: new Date().toISOString(),
    level,
    requestId,
    method: req.method,
    path: req.path,
    ip: getClientIp(req),
    userAgent: req.headers['user-agent'],
  };

  // In production, send to structured logging service
  // For now, use console
  console.log(JSON.stringify(log));
}

/**
 * Error handling middleware
 * Catches unhandled errors and returns proper response
 */
export function handleError(
  error: unknown,
  res: Response,
  requestId?: string
): void {
  // Log error
  console.error({
    timestamp: new Date().toISOString(),
    level: 'error',
    requestId,
    error: error instanceof Error ? error.message : 'Unknown error',
    stack: error instanceof Error ? error.stack : undefined,
  });

  // Send error response
  const message = error instanceof Error ? error.message : 'An unexpected error occurred';
  res.status(500).json({
    error: {
      code: 'INTERNAL_ERROR',
      message,
    },
    requestId,
    timestamp: new Date().toISOString(),
  });
}

/**
 * Validate HTTP method
 * Returns true if method is POST, false otherwise
 */
export function validateMethod(req: Request, res: Response): boolean {
  if (req.method !== 'POST') {
    res.status(405).json({
      error: {
        code: 'METHOD_NOT_ALLOWED',
        message: 'Only POST method is allowed',
      },
      timestamp: new Date().toISOString(),
    });
    return false;
  }
  return true;
}

/**
 * Set CORS headers
 */
export function setCorsHeaders(res: Response): void {
  res.set('Access-Control-Allow-Origin', '*');
  res.set('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.set('Access-Control-Allow-Headers', 'Content-Type, Authorization');
  res.set('Access-Control-Max-Age', '3600');
}

/**
 * Handle OPTIONS preflight request
 */
export function handleOptions(req: Request, res: Response): boolean {
  if (req.method === 'OPTIONS') {
    setCorsHeaders(res);
    res.status(204).send('');
    return true;
  }
  return false;
}
