/**
 * Response Formatters
 *
 * Standard response formatting for success and error cases
 */

import type { Response } from '@google-cloud/functions-framework';
import { CostAttributionOutput, ErrorResponse, ValidationError } from '../types';

/**
 * Send success response with proper headers and formatting
 */
export function sendSuccess(
  res: Response,
  output: CostAttributionOutput
): void {
  res.status(200).json(output);
}

/**
 * Send error response with proper HTTP status code
 */
export function sendError(
  res: Response,
  statusCode: number,
  code: string,
  message: string,
  details?: ValidationError[],
  requestId?: string
): void {
  const errorResponse: ErrorResponse = {
    error: {
      code,
      message,
      details,
    },
    requestId,
    timestamp: new Date().toISOString(),
  };

  res.status(statusCode).json(errorResponse);
}

/**
 * Send validation error (400 Bad Request)
 */
export function sendValidationError(
  res: Response,
  message: string,
  details?: ValidationError[],
  requestId?: string
): void {
  sendError(res, 400, 'VALIDATION_ERROR', message, details, requestId);
}

/**
 * Send internal server error (500)
 */
export function sendInternalError(
  res: Response,
  message: string,
  requestId?: string
): void {
  sendError(res, 500, 'INTERNAL_ERROR', message, undefined, requestId);
}

/**
 * Send service unavailable error (503)
 */
export function sendServiceUnavailable(
  res: Response,
  message: string,
  requestId?: string
): void {
  sendError(res, 503, 'SERVICE_UNAVAILABLE', message, undefined, requestId);
}

/**
 * Send rate limit error (429)
 */
export function sendRateLimitError(
  res: Response,
  message: string = 'Rate limit exceeded',
  requestId?: string
): void {
  sendError(res, 429, 'RATE_LIMIT_EXCEEDED', message, undefined, requestId);
}
