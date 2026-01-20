import { z } from 'zod';

/**
 * LLM Provider enum
 */
export const ProviderSchema = z.enum([
  'OpenAI',
  'Anthropic',
  'Google',
  'Azure',
  'AWS',
  'Cohere',
  'Mistral',
  'Custom'
]);

/**
 * Usage record schema capturing token consumption and metadata
 */
export const UsageRecordSchema = z.object({
  id: z.string().uuid(),
  timestamp: z.string().datetime(),
  provider: ProviderSchema,
  model: z.string(),
  organization_id: z.string().optional(),
  project_id: z.string().optional(),
  user_id: z.string().optional(),
  prompt_tokens: z.number().int().nonnegative(),
  completion_tokens: z.number().int().nonnegative(),
  total_tokens: z.number().int().nonnegative(),
  cached_tokens: z.number().int().nonnegative().optional(),
  reasoning_tokens: z.number().int().nonnegative().optional(),
  latency_ms: z.number().nonnegative().optional(),
  tags: z.record(z.string()).optional(),
  metadata: z.record(z.unknown()).optional()
});

/**
 * Cost record schema with pricing breakdown
 */
export const CostRecordSchema = z.object({
  id: z.string().uuid(),
  usage_record_id: z.string().uuid(),
  provider: ProviderSchema,
  model: z.string(),
  prompt_cost: z.number().nonnegative(),
  completion_cost: z.number().nonnegative(),
  cached_cost: z.number().nonnegative().optional(),
  total_cost: z.number().nonnegative(),
  currency: z.string().default('USD'),
  pricing_id: z.string().optional(),
  timestamp: z.string().datetime()
});

/**
 * Attribution scope defines the granularity level
 */
export const AttributionScopeSchema = z.enum([
  'execution',
  'agent',
  'workflow',
  'tenant'
]);

/**
 * Pricing context for cost calculation
 */
export const PricingContextSchema = z.object({
  prompt_price_per_1k: z.number().nonnegative(),
  completion_price_per_1k: z.number().nonnegative(),
  cached_price_per_1k: z.number().nonnegative().optional(),
  currency: z.string().default('USD'),
  effective_date: z.string().datetime().optional()
});

/**
 * Cost attribution input schema
 */
export const CostAttributionInputSchema = z.object({
  usage_records: z.array(UsageRecordSchema).min(1),
  pricing_context: PricingContextSchema.optional(),
  attribution_scope: AttributionScopeSchema.default('execution')
});

/**
 * Individual attribution result
 */
export const AttributionResultSchema = z.object({
  scope_id: z.string(),
  scope_type: AttributionScopeSchema,
  provider: ProviderSchema,
  model: z.string(),
  total_tokens: z.number().int().nonnegative(),
  prompt_tokens: z.number().int().nonnegative(),
  completion_tokens: z.number().int().nonnegative(),
  cached_tokens: z.number().int().nonnegative().optional(),
  reasoning_tokens: z.number().int().nonnegative().optional(),
  total_cost: z.number().nonnegative(),
  prompt_cost: z.number().nonnegative(),
  completion_cost: z.number().nonnegative(),
  cached_cost: z.number().nonnegative().optional(),
  currency: z.string().default('USD'),
  usage_count: z.number().int().positive(),
  avg_latency_ms: z.number().nonnegative().optional()
});

/**
 * Cost attribution summary
 */
export const AttributionSummarySchema = z.object({
  total_cost: z.number().nonnegative(),
  total_tokens: z.number().int().nonnegative(),
  total_requests: z.number().int().nonnegative(),
  currency: z.string().default('USD'),
  providers: z.array(ProviderSchema),
  models: z.array(z.string()),
  time_range: z.object({
    start: z.string().datetime(),
    end: z.string().datetime()
  }).optional()
});

/**
 * Cost attribution output schema
 */
export const CostAttributionOutputSchema = z.object({
  attributions: z.array(AttributionResultSchema),
  summary: AttributionSummarySchema,
  metadata: z.object({
    processed_at: z.string().datetime(),
    attribution_scope: AttributionScopeSchema,
    total_usage_records: z.number().int().nonnegative(),
    processing_time_ms: z.number().nonnegative().optional()
  })
});

/**
 * Decision event schema following agentic constitution requirements
 * Captures agent decision-making for auditability and governance
 */
export const DecisionEventSchema = z.object({
  agent_id: z.string(),
  agent_version: z.string(),
  decision_type: z.enum([
    'cost_attribution',
    'pricing_calculation',
    'scope_determination',
    'aggregation',
    'validation'
  ]),
  inputs_hash: z.string(),
  outputs: z.record(z.unknown()),
  confidence: z.number().min(0).max(1),
  constraints_applied: z.array(z.string()),
  execution_ref: z.string().uuid(),
  timestamp: z.string().datetime(),
  metadata: z.object({
    processing_time_ms: z.number().nonnegative().optional(),
    validation_status: z.enum(['passed', 'failed', 'warning']).optional(),
    error_details: z.string().optional()
  }).optional()
});

/**
 * Error response schema
 */
export const ErrorResponseSchema = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
    details: z.unknown().optional(),
    timestamp: z.string().datetime()
  })
});

/**
 * Health check response schema
 */
export const HealthCheckResponseSchema = z.object({
  status: z.enum(['healthy', 'degraded', 'unhealthy']),
  version: z.string(),
  timestamp: z.string().datetime(),
  checks: z.object({
    database: z.boolean().optional(),
    pricing_service: z.boolean().optional(),
    memory: z.boolean().optional()
  }).optional()
});
