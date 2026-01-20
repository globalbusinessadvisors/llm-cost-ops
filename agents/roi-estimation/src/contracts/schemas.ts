/**
 * ROI Estimation Agent - Zod Schemas
 *
 * Following agentics-contracts pattern for LLM-CostOps
 * Classification: ROI ANALYSIS
 * Decision Type: roi_estimation
 */

import { z } from 'zod';

// ============================================================================
// PROVIDER & MODEL SCHEMAS (agentics-contracts aligned)
// ============================================================================

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

export const CurrencySchema = z.enum([
  'USD',
  'EUR',
  'GBP',
  'JPY',
  'CAD',
  'AUD',
  'CHF',
  'CNY'
]);

// ============================================================================
// COST INPUT SCHEMAS
// ============================================================================

export const CostRecordSchema = z.object({
  id: z.string().uuid(),
  timestamp: z.string().datetime(),
  provider: ProviderSchema,
  model: z.string().min(1),
  organization_id: z.string().optional(),
  project_id: z.string().optional(),
  workflow_id: z.string().optional(),
  agent_id: z.string().optional(),
  execution_id: z.string().uuid().optional(),
  prompt_tokens: z.number().int().nonnegative(),
  completion_tokens: z.number().int().nonnegative(),
  total_tokens: z.number().int().nonnegative(),
  cached_tokens: z.number().int().nonnegative().optional(),
  reasoning_tokens: z.number().int().nonnegative().optional(),
  cost_amount: z.string(), // Decimal string for precision
  currency: CurrencySchema,
  latency_ms: z.number().nonnegative().optional(),
  tags: z.record(z.string()).optional(),
  metadata: z.record(z.unknown()).optional()
});

export const CostAggregationSchema = z.object({
  period_start: z.string().datetime(),
  period_end: z.string().datetime(),
  total_cost: z.string(), // Decimal string
  currency: CurrencySchema,
  total_tokens: z.number().int().nonnegative(),
  total_requests: z.number().int().nonnegative(),
  by_provider: z.record(z.object({
    cost: z.string(),
    tokens: z.number().int().nonnegative(),
    requests: z.number().int().nonnegative()
  })).optional(),
  by_model: z.record(z.object({
    cost: z.string(),
    tokens: z.number().int().nonnegative(),
    requests: z.number().int().nonnegative()
  })).optional()
});

// ============================================================================
// OUTCOME METRIC SCHEMAS
// ============================================================================

export const OutcomeMetricTypeSchema = z.enum([
  'throughput',           // Operations/requests per unit time
  'latency',              // Response time metrics
  'success_rate',         // Successful operations percentage
  'error_rate',           // Failed operations percentage
  'quality_score',        // Quality metric (e.g., accuracy, relevance)
  'user_satisfaction',    // User feedback/NPS score
  'task_completion',      // Task completion rate
  'revenue_impact',       // Revenue attributed to LLM usage
  'cost_savings',         // Costs saved vs alternative
  'productivity_gain',    // Productivity improvement metric
  'custom'                // Custom business metric
]);

export const OutcomeMetricSchema = z.object({
  id: z.string().uuid(),
  timestamp: z.string().datetime(),
  metric_type: OutcomeMetricTypeSchema,
  metric_name: z.string().min(1),
  value: z.number(),
  unit: z.string().min(1),
  organization_id: z.string().optional(),
  project_id: z.string().optional(),
  workflow_id: z.string().optional(),
  agent_id: z.string().optional(),
  correlation_window_hours: z.number().positive().default(24),
  baseline_value: z.number().optional(),
  target_value: z.number().optional(),
  tags: z.record(z.string()).optional(),
  metadata: z.record(z.unknown()).optional()
});

export const OutcomeAggregationSchema = z.object({
  period_start: z.string().datetime(),
  period_end: z.string().datetime(),
  metric_type: OutcomeMetricTypeSchema,
  metric_name: z.string(),
  aggregate_value: z.number(),
  min_value: z.number(),
  max_value: z.number(),
  avg_value: z.number(),
  sample_count: z.number().int().nonnegative(),
  standard_deviation: z.number().optional()
});

// ============================================================================
// ROI INPUT SCHEMA
// ============================================================================

export const ROIAnalysisScopeSchema = z.enum([
  'organization',
  'project',
  'workflow',
  'agent',
  'model',
  'provider',
  'custom'
]);

export const ROIInputSchema = z.object({
  // Required: Cost data
  cost_records: z.array(CostRecordSchema).min(1).optional(),
  cost_aggregation: CostAggregationSchema.optional(),

  // Required: At least one outcome metric
  outcome_metrics: z.array(OutcomeMetricSchema).optional(),
  outcome_aggregations: z.array(OutcomeAggregationSchema).optional(),

  // Analysis parameters
  analysis_scope: ROIAnalysisScopeSchema,
  scope_id: z.string().optional(),
  period_start: z.string().datetime(),
  period_end: z.string().datetime(),

  // Correlation settings
  correlation_method: z.enum(['pearson', 'spearman', 'kendall']).default('pearson'),
  min_correlation_threshold: z.number().min(-1).max(1).default(0.3),

  // ROI calculation settings
  roi_calculation_method: z.enum([
    'simple',           // (gain - cost) / cost
    'net_present_value', // NPV over time
    'payback_period',   // Time to recover cost
    'cost_efficiency'   // Outcome per unit cost
  ]).default('simple'),

  // Optional: Baseline for comparison
  baseline_period_start: z.string().datetime().optional(),
  baseline_period_end: z.string().datetime().optional(),
  baseline_cost: z.string().optional(),
  baseline_outcome: z.number().optional(),

  // Optional: Business context
  business_value_per_unit: z.string().optional(), // e.g., $10 per successful task
  opportunity_cost_per_unit: z.string().optional(),

  // Request metadata
  request_id: z.string().uuid(),
  requester_id: z.string().optional(),
  tags: z.record(z.string()).optional(),
  metadata: z.record(z.unknown()).optional()
}).refine(
  (data) => data.cost_records || data.cost_aggregation,
  { message: 'Either cost_records or cost_aggregation must be provided' }
).refine(
  (data) => data.outcome_metrics || data.outcome_aggregations,
  { message: 'Either outcome_metrics or outcome_aggregations must be provided' }
);

// ============================================================================
// ROI OUTPUT SCHEMAS
// ============================================================================

export const CorrelationResultSchema = z.object({
  metric_name: z.string(),
  metric_type: OutcomeMetricTypeSchema,
  correlation_coefficient: z.number().min(-1).max(1),
  correlation_strength: z.enum(['strong_positive', 'moderate_positive', 'weak_positive', 'none', 'weak_negative', 'moderate_negative', 'strong_negative']),
  p_value: z.number().min(0).max(1).optional(),
  sample_size: z.number().int().positive(),
  is_significant: z.boolean()
});

export const ROIMetricSchema = z.object({
  roi_percentage: z.number(), // Can be negative
  roi_ratio: z.number(),      // gain/cost ratio
  net_value: z.string(),      // gain - cost (decimal string)
  total_cost: z.string(),     // decimal string
  total_gain: z.string(),     // estimated monetary gain (decimal string)
  cost_per_outcome_unit: z.string(), // cost efficiency
  gain_per_cost_unit: z.string(),    // inverse efficiency
  payback_period_days: z.number().optional(),
  break_even_point: z.string().optional()
});

export const EfficiencyMetricSchema = z.object({
  metric_name: z.string(),
  current_efficiency: z.number(),
  baseline_efficiency: z.number().optional(),
  efficiency_change_percentage: z.number().optional(),
  unit: z.string(),
  interpretation: z.string()
});

export const ROIRecommendationSchema = z.object({
  category: z.enum([
    'cost_optimization',
    'outcome_improvement',
    'model_selection',
    'scaling_decision',
    'budget_adjustment',
    'process_improvement'
  ]),
  priority: z.enum(['high', 'medium', 'low']),
  recommendation: z.string(),
  estimated_impact: z.string().optional(),
  confidence: z.number().min(0).max(1)
});

export const ROIOutputSchema = z.object({
  // Core ROI results
  roi_metrics: ROIMetricSchema,

  // Correlation analysis
  correlations: z.array(CorrelationResultSchema),
  primary_correlation: CorrelationResultSchema.optional(),

  // Efficiency breakdown
  efficiency_metrics: z.array(EfficiencyMetricSchema),

  // Recommendations
  recommendations: z.array(ROIRecommendationSchema),

  // Summary
  summary: z.object({
    overall_assessment: z.enum([
      'highly_positive',
      'positive',
      'neutral',
      'negative',
      'highly_negative',
      'insufficient_data'
    ]),
    key_insight: z.string(),
    confidence_level: z.enum(['high', 'medium', 'low']),
    data_quality_score: z.number().min(0).max(1)
  }),

  // Metadata
  metadata: z.object({
    analyzed_at: z.string().datetime(),
    analysis_scope: ROIAnalysisScopeSchema,
    scope_id: z.string().optional(),
    period_start: z.string().datetime(),
    period_end: z.string().datetime(),
    cost_records_analyzed: z.number().int().nonnegative(),
    outcome_metrics_analyzed: z.number().int().nonnegative(),
    processing_time_ms: z.number().nonnegative(),
    calculation_method: z.string()
  })
});

// ============================================================================
// DECISION EVENT SCHEMA (agentics-contracts aligned)
// ============================================================================

export const DecisionTypeSchema = z.literal('roi_estimation');

export const DecisionEventSchema = z.object({
  agent_id: z.string(),
  agent_version: z.string(),
  decision_type: DecisionTypeSchema,
  inputs_hash: z.string(), // SHA-256 hash of inputs
  outputs: ROIOutputSchema,
  confidence: z.number().min(0).max(1), // Statistical confidence in ROI calculation
  constraints_applied: z.array(z.string()), // ROI thresholds, cost caps, etc.
  execution_ref: z.string().uuid(),
  timestamp: z.string().datetime(),
  metadata: z.object({
    request_id: z.string().uuid(),
    processing_duration_ms: z.number().nonnegative(),
    input_validation_passed: z.boolean(),
    data_completeness_score: z.number().min(0).max(1),
    statistical_significance: z.boolean(),
    correlation_method_used: z.string(),
    error_details: z.string().optional()
  })
});

// ============================================================================
// ERROR SCHEMAS
// ============================================================================

export const ROIErrorCodeSchema = z.enum([
  'INVALID_INPUT',
  'INSUFFICIENT_DATA',
  'CORRELATION_FAILED',
  'CALCULATION_ERROR',
  'PERSISTENCE_ERROR',
  'TIMEOUT',
  'INTERNAL_ERROR'
]);

export const ROIErrorSchema = z.object({
  code: ROIErrorCodeSchema,
  message: z.string(),
  details: z.record(z.unknown()).optional(),
  timestamp: z.string().datetime(),
  request_id: z.string().uuid().optional()
});

// ============================================================================
// HTTP REQUEST/RESPONSE SCHEMAS
// ============================================================================

export const AnalyzeROIRequestSchema = ROIInputSchema;

export const AnalyzeROIResponseSchema = z.discriminatedUnion('success', [
  z.object({
    success: z.literal(true),
    data: ROIOutputSchema,
    decision_event_id: z.string().uuid()
  }),
  z.object({
    success: z.literal(false),
    error: ROIErrorSchema
  })
]);

export const HealthCheckResponseSchema = z.object({
  status: z.enum(['healthy', 'degraded', 'unhealthy']),
  version: z.string(),
  timestamp: z.string().datetime(),
  dependencies: z.object({
    ruvector_service: z.enum(['connected', 'disconnected', 'unknown']),
    telemetry: z.enum(['connected', 'disconnected', 'unknown'])
  })
});
