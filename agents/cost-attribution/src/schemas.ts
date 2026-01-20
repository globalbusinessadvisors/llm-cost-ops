/**
 * Cost Attribution Agent Schemas
 *
 * Zod validation schemas for input/output validation
 */

import { z } from 'zod';

/**
 * Usage Data Schema
 */
export const usageDataSchema = z.object({
  provider: z.string().min(1, 'Provider is required'),
  model: z.string().min(1, 'Model is required'),
  inputTokens: z.number().int().nonnegative('Input tokens must be non-negative'),
  outputTokens: z.number().int().nonnegative('Output tokens must be non-negative'),
  cachedTokens: z.number().int().nonnegative().optional(),
  latencyMs: z.number().nonnegative().optional(),
});

/**
 * Pricing Context Schema
 */
export const pricingContextSchema = z.object({
  tier: z.string().optional(),
  currency: z.string().length(3).default('USD'),
  customPricing: z
    .object({
      inputPricePerToken: z.number().nonnegative().optional(),
      outputPricePerToken: z.number().nonnegative().optional(),
      cachedPricePerToken: z.number().nonnegative().optional(),
    })
    .optional(),
});

/**
 * Attribution Dimensions Schema
 */
export const attributionDimensionsSchema = z.object({
  userId: z.string().optional(),
  projectId: z.string().optional(),
  organizationId: z.string().optional(),
  environment: z.string().optional(),
  tags: z.record(z.string(), z.string()).optional(),
});

/**
 * Cost Attribution Input Schema
 */
export const costAttributionInputSchema = z.object({
  requestId: z.string().uuid('Request ID must be a valid UUID'),
  timestamp: z.string().datetime('Timestamp must be ISO 8601 format'),
  usage: usageDataSchema,
  pricingContext: pricingContextSchema.optional(),
  dimensions: attributionDimensionsSchema.optional(),
});

/**
 * Cost Breakdown Schema
 */
export const costBreakdownSchema = z.object({
  totalCost: z.number().nonnegative(),
  inputCost: z.number().nonnegative(),
  outputCost: z.number().nonnegative(),
  cachedCost: z.number().nonnegative().optional(),
  currency: z.string().length(3),
  costPer1kTokens: z.number().nonnegative(),
});

/**
 * Attribution Result Schema
 */
export const attributionResultSchema = z.object({
  primary: z.string(),
  dimensions: z.object({
    userId: z.string().optional(),
    projectId: z.string().optional(),
    organizationId: z.string().optional(),
    environment: z.string().optional(),
  }),
  tags: z.record(z.string(), z.string()),
  confidence: z.number().min(0).max(1),
});

/**
 * Decision Event Metadata Schema
 */
export const decisionEventMetadataSchema = z.object({
  eventId: z.string().uuid(),
  eventType: z.literal('cost_attribution'),
  timestamp: z.string().datetime(),
  agentId: z.string(),
  decision: z.object({
    action: z.literal('attribute_cost'),
    result: z.enum(['success', 'error']),
    confidence: z.number().min(0).max(1),
  }),
  context: z.object({
    provider: z.string(),
    model: z.string(),
    totalTokens: z.number().int().nonnegative(),
    totalCost: z.number().nonnegative(),
  }),
});

/**
 * Telemetry Metadata Schema
 */
export const telemetryMetadataSchema = z.object({
  telemetryId: z.string().uuid(),
  agentId: z.string(),
  timestamp: z.string().datetime(),
  metrics: z.object({
    processingDurationMs: z.number().nonnegative(),
    accuracy: z.number().min(0).max(1).optional(),
    dimensionCount: z.number().int().nonnegative(),
  }),
  trace: z
    .object({
      traceId: z.string(),
      spanId: z.string(),
    })
    .optional(),
});

/**
 * Cost Attribution Output Schema
 */
export const costAttributionOutputSchema = z.object({
  requestId: z.string().uuid(),
  analysisTimestamp: z.string().datetime(),
  costs: costBreakdownSchema,
  attribution: attributionResultSchema,
  decisionEvent: decisionEventMetadataSchema,
  telemetry: telemetryMetadataSchema,
});

/**
 * Validation Error Schema
 */
export const validationErrorSchema = z.object({
  field: z.string(),
  message: z.string(),
  value: z.unknown().optional(),
});

/**
 * Error Response Schema
 */
export const errorResponseSchema = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
    details: z.array(validationErrorSchema).optional(),
  }),
  requestId: z.string().optional(),
  timestamp: z.string().datetime(),
});

/**
 * Type exports from schemas
 */
export type CostAttributionInput = z.infer<typeof costAttributionInputSchema>;
export type CostAttributionOutput = z.infer<typeof costAttributionOutputSchema>;
export type ValidationError = z.infer<typeof validationErrorSchema>;
export type ErrorResponse = z.infer<typeof errorResponseSchema>;
