/**
 * Cost-Performance Tradeoff Agent - Contract Schemas
 *
 * Classification: TRADEOFF ANALYSIS
 *
 * These schemas define the inputs, outputs, and validation rules
 * for the Cost-Performance Tradeoff Agent.
 *
 * All schemas are imported from agentics-contracts pattern.
 */
import { z } from 'zod';
// ============================================================================
// Provider and Model Schemas
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
export const ModelTierSchema = z.enum([
    'economy', // Low cost, lower quality (e.g., GPT-3.5-turbo)
    'standard', // Balanced cost/quality (e.g., GPT-4o-mini)
    'premium', // High quality, higher cost (e.g., GPT-4o)
    'flagship' // Highest quality (e.g., GPT-4, Claude-3-opus)
]);
// ============================================================================
// Performance Metrics Schemas
// ============================================================================
export const LatencyMetricsSchema = z.object({
    p50_ms: z.number().nonnegative(),
    p95_ms: z.number().nonnegative(),
    p99_ms: z.number().nonnegative(),
    avg_ms: z.number().nonnegative(),
    min_ms: z.number().nonnegative(),
    max_ms: z.number().nonnegative()
});
export const QualityMetricsSchema = z.object({
    accuracy: z.number().min(0).max(1).optional(),
    relevance: z.number().min(0).max(1).optional(),
    coherence: z.number().min(0).max(1).optional(),
    helpfulness: z.number().min(0).max(1).optional(),
    composite_score: z.number().min(0).max(1)
});
export const CostMetricsSchema = z.object({
    cost_per_request_usd: z.number().nonnegative(),
    cost_per_1k_tokens_usd: z.number().nonnegative(),
    total_cost_usd: z.number().nonnegative(),
    token_count: z.number().int().nonnegative()
});
// ============================================================================
// Input Schemas
// ============================================================================
export const PerformanceRecordSchema = z.object({
    id: z.string().uuid(),
    timestamp: z.string().datetime(),
    provider: ProviderSchema,
    model: z.string(),
    model_tier: ModelTierSchema.optional(),
    // Cost metrics
    cost: CostMetricsSchema,
    // Performance metrics
    latency: LatencyMetricsSchema,
    // Quality metrics (optional - may not be available for all records)
    quality: QualityMetricsSchema.optional(),
    // Context
    execution_id: z.string().optional(),
    agent_id: z.string().optional(),
    workflow_id: z.string().optional(),
    tenant_id: z.string().optional(),
    // Tags for filtering/grouping
    tags: z.record(z.string()).optional(),
    metadata: z.record(z.unknown()).optional()
});
export const TradeoffAnalysisInputSchema = z.object({
    records: z.array(PerformanceRecordSchema).min(1),
    analysis_scope: z.enum(['model', 'provider', 'tier', 'execution']).default('model'),
    // Weighting for tradeoff calculation
    weights: z.object({
        cost: z.number().min(0).max(1).default(0.33),
        latency: z.number().min(0).max(1).default(0.33),
        quality: z.number().min(0).max(1).default(0.34)
    }).refine((w) => Math.abs(w.cost + w.latency + w.quality - 1.0) < 0.001, { message: 'Weights must sum to 1.0' }).optional(),
    // Constraints
    constraints: z.object({
        max_cost_per_request_usd: z.number().positive().optional(),
        max_latency_p95_ms: z.number().positive().optional(),
        min_quality_score: z.number().min(0).max(1).optional()
    }).optional(),
    // Analysis options
    options: z.object({
        include_diminishing_returns: z.boolean().default(true),
        include_pareto_frontier: z.boolean().default(true),
        include_recommendations: z.boolean().default(true),
        normalize_metrics: z.boolean().default(true)
    }).optional()
});
// ============================================================================
// Output Schemas
// ============================================================================
export const TradeoffScoreSchema = z.object({
    overall_score: z.number().min(0).max(1),
    cost_score: z.number().min(0).max(1),
    latency_score: z.number().min(0).max(1),
    quality_score: z.number().min(0).max(1),
    efficiency_ratio: z.number().nonnegative() // quality/cost ratio
});
export const DiminishingReturnsAnalysisSchema = z.object({
    detected: z.boolean(),
    threshold_cost_usd: z.number().nonnegative().optional(),
    marginal_quality_gain: z.number().optional(),
    recommendation: z.string()
});
export const ParetoPointSchema = z.object({
    model: z.string(),
    provider: ProviderSchema,
    cost: z.number().nonnegative(),
    latency: z.number().nonnegative(),
    quality: z.number().min(0).max(1).optional(),
    is_optimal: z.boolean()
});
export const TradeoffResultSchema = z.object({
    identifier: z.string(), // model name, provider, or tier
    provider: ProviderSchema,
    model: z.string(),
    model_tier: ModelTierSchema.optional(),
    // Aggregated metrics
    avg_cost: CostMetricsSchema,
    avg_latency: LatencyMetricsSchema,
    avg_quality: QualityMetricsSchema.optional(),
    // Tradeoff scores
    tradeoff_score: TradeoffScoreSchema,
    // Sample size
    record_count: z.number().int().positive()
});
export const TradeoffRecommendationSchema = z.object({
    recommendation_type: z.enum([
        'cost_optimization',
        'latency_optimization',
        'quality_optimization',
        'balanced',
        'constraint_violation'
    ]),
    recommended_model: z.string(),
    recommended_provider: ProviderSchema,
    rationale: z.string(),
    estimated_impact: z.object({
        cost_change_percent: z.number(),
        latency_change_percent: z.number(),
        quality_change_percent: z.number().optional()
    }),
    confidence: z.number().min(0).max(1)
});
export const TradeoffAnalysisOutputSchema = z.object({
    analysis_id: z.string().uuid(),
    analyzed_at: z.string().datetime(),
    analysis_scope: z.string(),
    // Results by identifier (model/provider/tier)
    results: z.array(TradeoffResultSchema),
    // Pareto frontier (efficient options)
    pareto_frontier: z.array(ParetoPointSchema).optional(),
    // Diminishing returns analysis
    diminishing_returns: DiminishingReturnsAnalysisSchema.optional(),
    // Recommendations
    recommendations: z.array(TradeoffRecommendationSchema).optional(),
    // Summary statistics
    summary: z.object({
        total_records_analyzed: z.number().int().nonnegative(),
        unique_models: z.number().int().nonnegative(),
        unique_providers: z.number().int().nonnegative(),
        best_cost_efficiency: z.string().optional(),
        best_latency: z.string().optional(),
        best_quality: z.string().optional(),
        best_overall: z.string().optional()
    }),
    // Constraints applied
    constraints_applied: z.object({
        max_cost_per_request_usd: z.number().optional(),
        max_latency_p95_ms: z.number().optional(),
        min_quality_score: z.number().optional()
    }).optional(),
    // Metadata
    metadata: z.object({
        weights_used: z.object({
            cost: z.number(),
            latency: z.number(),
            quality: z.number()
        }),
        analysis_duration_ms: z.number().nonnegative()
    })
});
// ============================================================================
// Decision Event Schema
// ============================================================================
export const DecisionTypeSchema = z.enum([
    'cost_performance_tradeoff',
    'diminishing_returns_detection',
    'pareto_analysis',
    'model_recommendation'
]);
export const DecisionEventSchema = z.object({
    agent_id: z.string(),
    agent_version: z.string(),
    decision_type: DecisionTypeSchema,
    inputs_hash: z.string(),
    outputs: z.record(z.unknown()),
    confidence: z.number().min(0).max(1),
    constraints_applied: z.object({
        max_cost_per_request_usd: z.number().optional(),
        max_latency_p95_ms: z.number().optional(),
        min_quality_score: z.number().optional()
    }).optional(),
    execution_ref: z.string().optional(),
    timestamp: z.string().datetime()
});
// ============================================================================
// Error Schemas
// ============================================================================
export const TradeoffErrorSchema = z.object({
    code: z.enum([
        'INVALID_INPUT',
        'INSUFFICIENT_DATA',
        'CONSTRAINT_VIOLATION',
        'ANALYSIS_FAILED',
        'SERVICE_UNAVAILABLE'
    ]),
    message: z.string(),
    details: z.record(z.unknown()).optional()
});
//# sourceMappingURL=schemas.js.map