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
export declare const ProviderSchema: z.ZodEnum<["OpenAI", "Anthropic", "Google", "Azure", "AWS", "Cohere", "Mistral", "Custom"]>;
export declare const ModelTierSchema: z.ZodEnum<["economy", "standard", "premium", "flagship"]>;
export declare const LatencyMetricsSchema: z.ZodObject<{
    p50_ms: z.ZodNumber;
    p95_ms: z.ZodNumber;
    p99_ms: z.ZodNumber;
    avg_ms: z.ZodNumber;
    min_ms: z.ZodNumber;
    max_ms: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    p50_ms: number;
    p95_ms: number;
    p99_ms: number;
    avg_ms: number;
    min_ms: number;
    max_ms: number;
}, {
    p50_ms: number;
    p95_ms: number;
    p99_ms: number;
    avg_ms: number;
    min_ms: number;
    max_ms: number;
}>;
export declare const QualityMetricsSchema: z.ZodObject<{
    accuracy: z.ZodOptional<z.ZodNumber>;
    relevance: z.ZodOptional<z.ZodNumber>;
    coherence: z.ZodOptional<z.ZodNumber>;
    helpfulness: z.ZodOptional<z.ZodNumber>;
    composite_score: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    composite_score: number;
    accuracy?: number | undefined;
    relevance?: number | undefined;
    coherence?: number | undefined;
    helpfulness?: number | undefined;
}, {
    composite_score: number;
    accuracy?: number | undefined;
    relevance?: number | undefined;
    coherence?: number | undefined;
    helpfulness?: number | undefined;
}>;
export declare const CostMetricsSchema: z.ZodObject<{
    cost_per_request_usd: z.ZodNumber;
    cost_per_1k_tokens_usd: z.ZodNumber;
    total_cost_usd: z.ZodNumber;
    token_count: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    cost_per_request_usd: number;
    cost_per_1k_tokens_usd: number;
    total_cost_usd: number;
    token_count: number;
}, {
    cost_per_request_usd: number;
    cost_per_1k_tokens_usd: number;
    total_cost_usd: number;
    token_count: number;
}>;
export declare const PerformanceRecordSchema: z.ZodObject<{
    id: z.ZodString;
    timestamp: z.ZodString;
    provider: z.ZodEnum<["OpenAI", "Anthropic", "Google", "Azure", "AWS", "Cohere", "Mistral", "Custom"]>;
    model: z.ZodString;
    model_tier: z.ZodOptional<z.ZodEnum<["economy", "standard", "premium", "flagship"]>>;
    cost: z.ZodObject<{
        cost_per_request_usd: z.ZodNumber;
        cost_per_1k_tokens_usd: z.ZodNumber;
        total_cost_usd: z.ZodNumber;
        token_count: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        cost_per_request_usd: number;
        cost_per_1k_tokens_usd: number;
        total_cost_usd: number;
        token_count: number;
    }, {
        cost_per_request_usd: number;
        cost_per_1k_tokens_usd: number;
        total_cost_usd: number;
        token_count: number;
    }>;
    latency: z.ZodObject<{
        p50_ms: z.ZodNumber;
        p95_ms: z.ZodNumber;
        p99_ms: z.ZodNumber;
        avg_ms: z.ZodNumber;
        min_ms: z.ZodNumber;
        max_ms: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        p50_ms: number;
        p95_ms: number;
        p99_ms: number;
        avg_ms: number;
        min_ms: number;
        max_ms: number;
    }, {
        p50_ms: number;
        p95_ms: number;
        p99_ms: number;
        avg_ms: number;
        min_ms: number;
        max_ms: number;
    }>;
    quality: z.ZodOptional<z.ZodObject<{
        accuracy: z.ZodOptional<z.ZodNumber>;
        relevance: z.ZodOptional<z.ZodNumber>;
        coherence: z.ZodOptional<z.ZodNumber>;
        helpfulness: z.ZodOptional<z.ZodNumber>;
        composite_score: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        composite_score: number;
        accuracy?: number | undefined;
        relevance?: number | undefined;
        coherence?: number | undefined;
        helpfulness?: number | undefined;
    }, {
        composite_score: number;
        accuracy?: number | undefined;
        relevance?: number | undefined;
        coherence?: number | undefined;
        helpfulness?: number | undefined;
    }>>;
    execution_id: z.ZodOptional<z.ZodString>;
    agent_id: z.ZodOptional<z.ZodString>;
    workflow_id: z.ZodOptional<z.ZodString>;
    tenant_id: z.ZodOptional<z.ZodString>;
    tags: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
    metadata: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
}, "strip", z.ZodTypeAny, {
    id: string;
    timestamp: string;
    provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
    model: string;
    cost: {
        cost_per_request_usd: number;
        cost_per_1k_tokens_usd: number;
        total_cost_usd: number;
        token_count: number;
    };
    latency: {
        p50_ms: number;
        p95_ms: number;
        p99_ms: number;
        avg_ms: number;
        min_ms: number;
        max_ms: number;
    };
    model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
    quality?: {
        composite_score: number;
        accuracy?: number | undefined;
        relevance?: number | undefined;
        coherence?: number | undefined;
        helpfulness?: number | undefined;
    } | undefined;
    execution_id?: string | undefined;
    agent_id?: string | undefined;
    workflow_id?: string | undefined;
    tenant_id?: string | undefined;
    tags?: Record<string, string> | undefined;
    metadata?: Record<string, unknown> | undefined;
}, {
    id: string;
    timestamp: string;
    provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
    model: string;
    cost: {
        cost_per_request_usd: number;
        cost_per_1k_tokens_usd: number;
        total_cost_usd: number;
        token_count: number;
    };
    latency: {
        p50_ms: number;
        p95_ms: number;
        p99_ms: number;
        avg_ms: number;
        min_ms: number;
        max_ms: number;
    };
    model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
    quality?: {
        composite_score: number;
        accuracy?: number | undefined;
        relevance?: number | undefined;
        coherence?: number | undefined;
        helpfulness?: number | undefined;
    } | undefined;
    execution_id?: string | undefined;
    agent_id?: string | undefined;
    workflow_id?: string | undefined;
    tenant_id?: string | undefined;
    tags?: Record<string, string> | undefined;
    metadata?: Record<string, unknown> | undefined;
}>;
export declare const TradeoffAnalysisInputSchema: z.ZodObject<{
    records: z.ZodArray<z.ZodObject<{
        id: z.ZodString;
        timestamp: z.ZodString;
        provider: z.ZodEnum<["OpenAI", "Anthropic", "Google", "Azure", "AWS", "Cohere", "Mistral", "Custom"]>;
        model: z.ZodString;
        model_tier: z.ZodOptional<z.ZodEnum<["economy", "standard", "premium", "flagship"]>>;
        cost: z.ZodObject<{
            cost_per_request_usd: z.ZodNumber;
            cost_per_1k_tokens_usd: z.ZodNumber;
            total_cost_usd: z.ZodNumber;
            token_count: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        }, {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        }>;
        latency: z.ZodObject<{
            p50_ms: z.ZodNumber;
            p95_ms: z.ZodNumber;
            p99_ms: z.ZodNumber;
            avg_ms: z.ZodNumber;
            min_ms: z.ZodNumber;
            max_ms: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        }, {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        }>;
        quality: z.ZodOptional<z.ZodObject<{
            accuracy: z.ZodOptional<z.ZodNumber>;
            relevance: z.ZodOptional<z.ZodNumber>;
            coherence: z.ZodOptional<z.ZodNumber>;
            helpfulness: z.ZodOptional<z.ZodNumber>;
            composite_score: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        }, {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        }>>;
        execution_id: z.ZodOptional<z.ZodString>;
        agent_id: z.ZodOptional<z.ZodString>;
        workflow_id: z.ZodOptional<z.ZodString>;
        tenant_id: z.ZodOptional<z.ZodString>;
        tags: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodString>>;
        metadata: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
    }, "strip", z.ZodTypeAny, {
        id: string;
        timestamp: string;
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        cost: {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        };
        latency: {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        };
        model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
        quality?: {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        } | undefined;
        execution_id?: string | undefined;
        agent_id?: string | undefined;
        workflow_id?: string | undefined;
        tenant_id?: string | undefined;
        tags?: Record<string, string> | undefined;
        metadata?: Record<string, unknown> | undefined;
    }, {
        id: string;
        timestamp: string;
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        cost: {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        };
        latency: {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        };
        model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
        quality?: {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        } | undefined;
        execution_id?: string | undefined;
        agent_id?: string | undefined;
        workflow_id?: string | undefined;
        tenant_id?: string | undefined;
        tags?: Record<string, string> | undefined;
        metadata?: Record<string, unknown> | undefined;
    }>, "many">;
    analysis_scope: z.ZodDefault<z.ZodEnum<["model", "provider", "tier", "execution"]>>;
    weights: z.ZodOptional<z.ZodEffects<z.ZodObject<{
        cost: z.ZodDefault<z.ZodNumber>;
        latency: z.ZodDefault<z.ZodNumber>;
        quality: z.ZodDefault<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        cost: number;
        latency: number;
        quality: number;
    }, {
        cost?: number | undefined;
        latency?: number | undefined;
        quality?: number | undefined;
    }>, {
        cost: number;
        latency: number;
        quality: number;
    }, {
        cost?: number | undefined;
        latency?: number | undefined;
        quality?: number | undefined;
    }>>;
    constraints: z.ZodOptional<z.ZodObject<{
        max_cost_per_request_usd: z.ZodOptional<z.ZodNumber>;
        max_latency_p95_ms: z.ZodOptional<z.ZodNumber>;
        min_quality_score: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    }, {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    }>>;
    options: z.ZodOptional<z.ZodObject<{
        include_diminishing_returns: z.ZodDefault<z.ZodBoolean>;
        include_pareto_frontier: z.ZodDefault<z.ZodBoolean>;
        include_recommendations: z.ZodDefault<z.ZodBoolean>;
        normalize_metrics: z.ZodDefault<z.ZodBoolean>;
    }, "strip", z.ZodTypeAny, {
        include_diminishing_returns: boolean;
        include_pareto_frontier: boolean;
        include_recommendations: boolean;
        normalize_metrics: boolean;
    }, {
        include_diminishing_returns?: boolean | undefined;
        include_pareto_frontier?: boolean | undefined;
        include_recommendations?: boolean | undefined;
        normalize_metrics?: boolean | undefined;
    }>>;
}, "strip", z.ZodTypeAny, {
    records: {
        id: string;
        timestamp: string;
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        cost: {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        };
        latency: {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        };
        model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
        quality?: {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        } | undefined;
        execution_id?: string | undefined;
        agent_id?: string | undefined;
        workflow_id?: string | undefined;
        tenant_id?: string | undefined;
        tags?: Record<string, string> | undefined;
        metadata?: Record<string, unknown> | undefined;
    }[];
    analysis_scope: "provider" | "model" | "tier" | "execution";
    options?: {
        include_diminishing_returns: boolean;
        include_pareto_frontier: boolean;
        include_recommendations: boolean;
        normalize_metrics: boolean;
    } | undefined;
    weights?: {
        cost: number;
        latency: number;
        quality: number;
    } | undefined;
    constraints?: {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    } | undefined;
}, {
    records: {
        id: string;
        timestamp: string;
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        cost: {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        };
        latency: {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        };
        model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
        quality?: {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        } | undefined;
        execution_id?: string | undefined;
        agent_id?: string | undefined;
        workflow_id?: string | undefined;
        tenant_id?: string | undefined;
        tags?: Record<string, string> | undefined;
        metadata?: Record<string, unknown> | undefined;
    }[];
    options?: {
        include_diminishing_returns?: boolean | undefined;
        include_pareto_frontier?: boolean | undefined;
        include_recommendations?: boolean | undefined;
        normalize_metrics?: boolean | undefined;
    } | undefined;
    analysis_scope?: "provider" | "model" | "tier" | "execution" | undefined;
    weights?: {
        cost?: number | undefined;
        latency?: number | undefined;
        quality?: number | undefined;
    } | undefined;
    constraints?: {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    } | undefined;
}>;
export declare const TradeoffScoreSchema: z.ZodObject<{
    overall_score: z.ZodNumber;
    cost_score: z.ZodNumber;
    latency_score: z.ZodNumber;
    quality_score: z.ZodNumber;
    efficiency_ratio: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    overall_score: number;
    cost_score: number;
    latency_score: number;
    quality_score: number;
    efficiency_ratio: number;
}, {
    overall_score: number;
    cost_score: number;
    latency_score: number;
    quality_score: number;
    efficiency_ratio: number;
}>;
export declare const DiminishingReturnsAnalysisSchema: z.ZodObject<{
    detected: z.ZodBoolean;
    threshold_cost_usd: z.ZodOptional<z.ZodNumber>;
    marginal_quality_gain: z.ZodOptional<z.ZodNumber>;
    recommendation: z.ZodString;
}, "strip", z.ZodTypeAny, {
    detected: boolean;
    recommendation: string;
    threshold_cost_usd?: number | undefined;
    marginal_quality_gain?: number | undefined;
}, {
    detected: boolean;
    recommendation: string;
    threshold_cost_usd?: number | undefined;
    marginal_quality_gain?: number | undefined;
}>;
export declare const ParetoPointSchema: z.ZodObject<{
    model: z.ZodString;
    provider: z.ZodEnum<["OpenAI", "Anthropic", "Google", "Azure", "AWS", "Cohere", "Mistral", "Custom"]>;
    cost: z.ZodNumber;
    latency: z.ZodNumber;
    quality: z.ZodOptional<z.ZodNumber>;
    is_optimal: z.ZodBoolean;
}, "strip", z.ZodTypeAny, {
    provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
    model: string;
    cost: number;
    latency: number;
    is_optimal: boolean;
    quality?: number | undefined;
}, {
    provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
    model: string;
    cost: number;
    latency: number;
    is_optimal: boolean;
    quality?: number | undefined;
}>;
export declare const TradeoffResultSchema: z.ZodObject<{
    identifier: z.ZodString;
    provider: z.ZodEnum<["OpenAI", "Anthropic", "Google", "Azure", "AWS", "Cohere", "Mistral", "Custom"]>;
    model: z.ZodString;
    model_tier: z.ZodOptional<z.ZodEnum<["economy", "standard", "premium", "flagship"]>>;
    avg_cost: z.ZodObject<{
        cost_per_request_usd: z.ZodNumber;
        cost_per_1k_tokens_usd: z.ZodNumber;
        total_cost_usd: z.ZodNumber;
        token_count: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        cost_per_request_usd: number;
        cost_per_1k_tokens_usd: number;
        total_cost_usd: number;
        token_count: number;
    }, {
        cost_per_request_usd: number;
        cost_per_1k_tokens_usd: number;
        total_cost_usd: number;
        token_count: number;
    }>;
    avg_latency: z.ZodObject<{
        p50_ms: z.ZodNumber;
        p95_ms: z.ZodNumber;
        p99_ms: z.ZodNumber;
        avg_ms: z.ZodNumber;
        min_ms: z.ZodNumber;
        max_ms: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        p50_ms: number;
        p95_ms: number;
        p99_ms: number;
        avg_ms: number;
        min_ms: number;
        max_ms: number;
    }, {
        p50_ms: number;
        p95_ms: number;
        p99_ms: number;
        avg_ms: number;
        min_ms: number;
        max_ms: number;
    }>;
    avg_quality: z.ZodOptional<z.ZodObject<{
        accuracy: z.ZodOptional<z.ZodNumber>;
        relevance: z.ZodOptional<z.ZodNumber>;
        coherence: z.ZodOptional<z.ZodNumber>;
        helpfulness: z.ZodOptional<z.ZodNumber>;
        composite_score: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        composite_score: number;
        accuracy?: number | undefined;
        relevance?: number | undefined;
        coherence?: number | undefined;
        helpfulness?: number | undefined;
    }, {
        composite_score: number;
        accuracy?: number | undefined;
        relevance?: number | undefined;
        coherence?: number | undefined;
        helpfulness?: number | undefined;
    }>>;
    tradeoff_score: z.ZodObject<{
        overall_score: z.ZodNumber;
        cost_score: z.ZodNumber;
        latency_score: z.ZodNumber;
        quality_score: z.ZodNumber;
        efficiency_ratio: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        overall_score: number;
        cost_score: number;
        latency_score: number;
        quality_score: number;
        efficiency_ratio: number;
    }, {
        overall_score: number;
        cost_score: number;
        latency_score: number;
        quality_score: number;
        efficiency_ratio: number;
    }>;
    record_count: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
    model: string;
    identifier: string;
    avg_cost: {
        cost_per_request_usd: number;
        cost_per_1k_tokens_usd: number;
        total_cost_usd: number;
        token_count: number;
    };
    avg_latency: {
        p50_ms: number;
        p95_ms: number;
        p99_ms: number;
        avg_ms: number;
        min_ms: number;
        max_ms: number;
    };
    tradeoff_score: {
        overall_score: number;
        cost_score: number;
        latency_score: number;
        quality_score: number;
        efficiency_ratio: number;
    };
    record_count: number;
    model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
    avg_quality?: {
        composite_score: number;
        accuracy?: number | undefined;
        relevance?: number | undefined;
        coherence?: number | undefined;
        helpfulness?: number | undefined;
    } | undefined;
}, {
    provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
    model: string;
    identifier: string;
    avg_cost: {
        cost_per_request_usd: number;
        cost_per_1k_tokens_usd: number;
        total_cost_usd: number;
        token_count: number;
    };
    avg_latency: {
        p50_ms: number;
        p95_ms: number;
        p99_ms: number;
        avg_ms: number;
        min_ms: number;
        max_ms: number;
    };
    tradeoff_score: {
        overall_score: number;
        cost_score: number;
        latency_score: number;
        quality_score: number;
        efficiency_ratio: number;
    };
    record_count: number;
    model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
    avg_quality?: {
        composite_score: number;
        accuracy?: number | undefined;
        relevance?: number | undefined;
        coherence?: number | undefined;
        helpfulness?: number | undefined;
    } | undefined;
}>;
export declare const TradeoffRecommendationSchema: z.ZodObject<{
    recommendation_type: z.ZodEnum<["cost_optimization", "latency_optimization", "quality_optimization", "balanced", "constraint_violation"]>;
    recommended_model: z.ZodString;
    recommended_provider: z.ZodEnum<["OpenAI", "Anthropic", "Google", "Azure", "AWS", "Cohere", "Mistral", "Custom"]>;
    rationale: z.ZodString;
    estimated_impact: z.ZodObject<{
        cost_change_percent: z.ZodNumber;
        latency_change_percent: z.ZodNumber;
        quality_change_percent: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        cost_change_percent: number;
        latency_change_percent: number;
        quality_change_percent?: number | undefined;
    }, {
        cost_change_percent: number;
        latency_change_percent: number;
        quality_change_percent?: number | undefined;
    }>;
    confidence: z.ZodNumber;
}, "strip", z.ZodTypeAny, {
    recommendation_type: "cost_optimization" | "latency_optimization" | "quality_optimization" | "balanced" | "constraint_violation";
    recommended_model: string;
    recommended_provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
    rationale: string;
    estimated_impact: {
        cost_change_percent: number;
        latency_change_percent: number;
        quality_change_percent?: number | undefined;
    };
    confidence: number;
}, {
    recommendation_type: "cost_optimization" | "latency_optimization" | "quality_optimization" | "balanced" | "constraint_violation";
    recommended_model: string;
    recommended_provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
    rationale: string;
    estimated_impact: {
        cost_change_percent: number;
        latency_change_percent: number;
        quality_change_percent?: number | undefined;
    };
    confidence: number;
}>;
export declare const TradeoffAnalysisOutputSchema: z.ZodObject<{
    analysis_id: z.ZodString;
    analyzed_at: z.ZodString;
    analysis_scope: z.ZodString;
    results: z.ZodArray<z.ZodObject<{
        identifier: z.ZodString;
        provider: z.ZodEnum<["OpenAI", "Anthropic", "Google", "Azure", "AWS", "Cohere", "Mistral", "Custom"]>;
        model: z.ZodString;
        model_tier: z.ZodOptional<z.ZodEnum<["economy", "standard", "premium", "flagship"]>>;
        avg_cost: z.ZodObject<{
            cost_per_request_usd: z.ZodNumber;
            cost_per_1k_tokens_usd: z.ZodNumber;
            total_cost_usd: z.ZodNumber;
            token_count: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        }, {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        }>;
        avg_latency: z.ZodObject<{
            p50_ms: z.ZodNumber;
            p95_ms: z.ZodNumber;
            p99_ms: z.ZodNumber;
            avg_ms: z.ZodNumber;
            min_ms: z.ZodNumber;
            max_ms: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        }, {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        }>;
        avg_quality: z.ZodOptional<z.ZodObject<{
            accuracy: z.ZodOptional<z.ZodNumber>;
            relevance: z.ZodOptional<z.ZodNumber>;
            coherence: z.ZodOptional<z.ZodNumber>;
            helpfulness: z.ZodOptional<z.ZodNumber>;
            composite_score: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        }, {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        }>>;
        tradeoff_score: z.ZodObject<{
            overall_score: z.ZodNumber;
            cost_score: z.ZodNumber;
            latency_score: z.ZodNumber;
            quality_score: z.ZodNumber;
            efficiency_ratio: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            overall_score: number;
            cost_score: number;
            latency_score: number;
            quality_score: number;
            efficiency_ratio: number;
        }, {
            overall_score: number;
            cost_score: number;
            latency_score: number;
            quality_score: number;
            efficiency_ratio: number;
        }>;
        record_count: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        identifier: string;
        avg_cost: {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        };
        avg_latency: {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        };
        tradeoff_score: {
            overall_score: number;
            cost_score: number;
            latency_score: number;
            quality_score: number;
            efficiency_ratio: number;
        };
        record_count: number;
        model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
        avg_quality?: {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        } | undefined;
    }, {
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        identifier: string;
        avg_cost: {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        };
        avg_latency: {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        };
        tradeoff_score: {
            overall_score: number;
            cost_score: number;
            latency_score: number;
            quality_score: number;
            efficiency_ratio: number;
        };
        record_count: number;
        model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
        avg_quality?: {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        } | undefined;
    }>, "many">;
    pareto_frontier: z.ZodOptional<z.ZodArray<z.ZodObject<{
        model: z.ZodString;
        provider: z.ZodEnum<["OpenAI", "Anthropic", "Google", "Azure", "AWS", "Cohere", "Mistral", "Custom"]>;
        cost: z.ZodNumber;
        latency: z.ZodNumber;
        quality: z.ZodOptional<z.ZodNumber>;
        is_optimal: z.ZodBoolean;
    }, "strip", z.ZodTypeAny, {
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        cost: number;
        latency: number;
        is_optimal: boolean;
        quality?: number | undefined;
    }, {
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        cost: number;
        latency: number;
        is_optimal: boolean;
        quality?: number | undefined;
    }>, "many">>;
    diminishing_returns: z.ZodOptional<z.ZodObject<{
        detected: z.ZodBoolean;
        threshold_cost_usd: z.ZodOptional<z.ZodNumber>;
        marginal_quality_gain: z.ZodOptional<z.ZodNumber>;
        recommendation: z.ZodString;
    }, "strip", z.ZodTypeAny, {
        detected: boolean;
        recommendation: string;
        threshold_cost_usd?: number | undefined;
        marginal_quality_gain?: number | undefined;
    }, {
        detected: boolean;
        recommendation: string;
        threshold_cost_usd?: number | undefined;
        marginal_quality_gain?: number | undefined;
    }>>;
    recommendations: z.ZodOptional<z.ZodArray<z.ZodObject<{
        recommendation_type: z.ZodEnum<["cost_optimization", "latency_optimization", "quality_optimization", "balanced", "constraint_violation"]>;
        recommended_model: z.ZodString;
        recommended_provider: z.ZodEnum<["OpenAI", "Anthropic", "Google", "Azure", "AWS", "Cohere", "Mistral", "Custom"]>;
        rationale: z.ZodString;
        estimated_impact: z.ZodObject<{
            cost_change_percent: z.ZodNumber;
            latency_change_percent: z.ZodNumber;
            quality_change_percent: z.ZodOptional<z.ZodNumber>;
        }, "strip", z.ZodTypeAny, {
            cost_change_percent: number;
            latency_change_percent: number;
            quality_change_percent?: number | undefined;
        }, {
            cost_change_percent: number;
            latency_change_percent: number;
            quality_change_percent?: number | undefined;
        }>;
        confidence: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        recommendation_type: "cost_optimization" | "latency_optimization" | "quality_optimization" | "balanced" | "constraint_violation";
        recommended_model: string;
        recommended_provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        rationale: string;
        estimated_impact: {
            cost_change_percent: number;
            latency_change_percent: number;
            quality_change_percent?: number | undefined;
        };
        confidence: number;
    }, {
        recommendation_type: "cost_optimization" | "latency_optimization" | "quality_optimization" | "balanced" | "constraint_violation";
        recommended_model: string;
        recommended_provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        rationale: string;
        estimated_impact: {
            cost_change_percent: number;
            latency_change_percent: number;
            quality_change_percent?: number | undefined;
        };
        confidence: number;
    }>, "many">>;
    summary: z.ZodObject<{
        total_records_analyzed: z.ZodNumber;
        unique_models: z.ZodNumber;
        unique_providers: z.ZodNumber;
        best_cost_efficiency: z.ZodOptional<z.ZodString>;
        best_latency: z.ZodOptional<z.ZodString>;
        best_quality: z.ZodOptional<z.ZodString>;
        best_overall: z.ZodOptional<z.ZodString>;
    }, "strip", z.ZodTypeAny, {
        total_records_analyzed: number;
        unique_models: number;
        unique_providers: number;
        best_cost_efficiency?: string | undefined;
        best_latency?: string | undefined;
        best_quality?: string | undefined;
        best_overall?: string | undefined;
    }, {
        total_records_analyzed: number;
        unique_models: number;
        unique_providers: number;
        best_cost_efficiency?: string | undefined;
        best_latency?: string | undefined;
        best_quality?: string | undefined;
        best_overall?: string | undefined;
    }>;
    constraints_applied: z.ZodOptional<z.ZodObject<{
        max_cost_per_request_usd: z.ZodOptional<z.ZodNumber>;
        max_latency_p95_ms: z.ZodOptional<z.ZodNumber>;
        min_quality_score: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    }, {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    }>>;
    metadata: z.ZodObject<{
        weights_used: z.ZodObject<{
            cost: z.ZodNumber;
            latency: z.ZodNumber;
            quality: z.ZodNumber;
        }, "strip", z.ZodTypeAny, {
            cost: number;
            latency: number;
            quality: number;
        }, {
            cost: number;
            latency: number;
            quality: number;
        }>;
        analysis_duration_ms: z.ZodNumber;
    }, "strip", z.ZodTypeAny, {
        weights_used: {
            cost: number;
            latency: number;
            quality: number;
        };
        analysis_duration_ms: number;
    }, {
        weights_used: {
            cost: number;
            latency: number;
            quality: number;
        };
        analysis_duration_ms: number;
    }>;
}, "strip", z.ZodTypeAny, {
    metadata: {
        weights_used: {
            cost: number;
            latency: number;
            quality: number;
        };
        analysis_duration_ms: number;
    };
    analysis_scope: string;
    analysis_id: string;
    analyzed_at: string;
    results: {
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        identifier: string;
        avg_cost: {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        };
        avg_latency: {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        };
        tradeoff_score: {
            overall_score: number;
            cost_score: number;
            latency_score: number;
            quality_score: number;
            efficiency_ratio: number;
        };
        record_count: number;
        model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
        avg_quality?: {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        } | undefined;
    }[];
    summary: {
        total_records_analyzed: number;
        unique_models: number;
        unique_providers: number;
        best_cost_efficiency?: string | undefined;
        best_latency?: string | undefined;
        best_quality?: string | undefined;
        best_overall?: string | undefined;
    };
    pareto_frontier?: {
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        cost: number;
        latency: number;
        is_optimal: boolean;
        quality?: number | undefined;
    }[] | undefined;
    diminishing_returns?: {
        detected: boolean;
        recommendation: string;
        threshold_cost_usd?: number | undefined;
        marginal_quality_gain?: number | undefined;
    } | undefined;
    recommendations?: {
        recommendation_type: "cost_optimization" | "latency_optimization" | "quality_optimization" | "balanced" | "constraint_violation";
        recommended_model: string;
        recommended_provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        rationale: string;
        estimated_impact: {
            cost_change_percent: number;
            latency_change_percent: number;
            quality_change_percent?: number | undefined;
        };
        confidence: number;
    }[] | undefined;
    constraints_applied?: {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    } | undefined;
}, {
    metadata: {
        weights_used: {
            cost: number;
            latency: number;
            quality: number;
        };
        analysis_duration_ms: number;
    };
    analysis_scope: string;
    analysis_id: string;
    analyzed_at: string;
    results: {
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        identifier: string;
        avg_cost: {
            cost_per_request_usd: number;
            cost_per_1k_tokens_usd: number;
            total_cost_usd: number;
            token_count: number;
        };
        avg_latency: {
            p50_ms: number;
            p95_ms: number;
            p99_ms: number;
            avg_ms: number;
            min_ms: number;
            max_ms: number;
        };
        tradeoff_score: {
            overall_score: number;
            cost_score: number;
            latency_score: number;
            quality_score: number;
            efficiency_ratio: number;
        };
        record_count: number;
        model_tier?: "economy" | "standard" | "premium" | "flagship" | undefined;
        avg_quality?: {
            composite_score: number;
            accuracy?: number | undefined;
            relevance?: number | undefined;
            coherence?: number | undefined;
            helpfulness?: number | undefined;
        } | undefined;
    }[];
    summary: {
        total_records_analyzed: number;
        unique_models: number;
        unique_providers: number;
        best_cost_efficiency?: string | undefined;
        best_latency?: string | undefined;
        best_quality?: string | undefined;
        best_overall?: string | undefined;
    };
    pareto_frontier?: {
        provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        model: string;
        cost: number;
        latency: number;
        is_optimal: boolean;
        quality?: number | undefined;
    }[] | undefined;
    diminishing_returns?: {
        detected: boolean;
        recommendation: string;
        threshold_cost_usd?: number | undefined;
        marginal_quality_gain?: number | undefined;
    } | undefined;
    recommendations?: {
        recommendation_type: "cost_optimization" | "latency_optimization" | "quality_optimization" | "balanced" | "constraint_violation";
        recommended_model: string;
        recommended_provider: "OpenAI" | "Anthropic" | "Google" | "Azure" | "AWS" | "Cohere" | "Mistral" | "Custom";
        rationale: string;
        estimated_impact: {
            cost_change_percent: number;
            latency_change_percent: number;
            quality_change_percent?: number | undefined;
        };
        confidence: number;
    }[] | undefined;
    constraints_applied?: {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    } | undefined;
}>;
export declare const DecisionTypeSchema: z.ZodEnum<["cost_performance_tradeoff", "diminishing_returns_detection", "pareto_analysis", "model_recommendation"]>;
export declare const DecisionEventSchema: z.ZodObject<{
    agent_id: z.ZodString;
    agent_version: z.ZodString;
    decision_type: z.ZodEnum<["cost_performance_tradeoff", "diminishing_returns_detection", "pareto_analysis", "model_recommendation"]>;
    inputs_hash: z.ZodString;
    outputs: z.ZodRecord<z.ZodString, z.ZodUnknown>;
    confidence: z.ZodNumber;
    constraints_applied: z.ZodOptional<z.ZodObject<{
        max_cost_per_request_usd: z.ZodOptional<z.ZodNumber>;
        max_latency_p95_ms: z.ZodOptional<z.ZodNumber>;
        min_quality_score: z.ZodOptional<z.ZodNumber>;
    }, "strip", z.ZodTypeAny, {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    }, {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    }>>;
    execution_ref: z.ZodOptional<z.ZodString>;
    timestamp: z.ZodString;
}, "strip", z.ZodTypeAny, {
    timestamp: string;
    agent_id: string;
    confidence: number;
    agent_version: string;
    decision_type: "cost_performance_tradeoff" | "diminishing_returns_detection" | "pareto_analysis" | "model_recommendation";
    inputs_hash: string;
    outputs: Record<string, unknown>;
    constraints_applied?: {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    } | undefined;
    execution_ref?: string | undefined;
}, {
    timestamp: string;
    agent_id: string;
    confidence: number;
    agent_version: string;
    decision_type: "cost_performance_tradeoff" | "diminishing_returns_detection" | "pareto_analysis" | "model_recommendation";
    inputs_hash: string;
    outputs: Record<string, unknown>;
    constraints_applied?: {
        max_cost_per_request_usd?: number | undefined;
        max_latency_p95_ms?: number | undefined;
        min_quality_score?: number | undefined;
    } | undefined;
    execution_ref?: string | undefined;
}>;
export declare const TradeoffErrorSchema: z.ZodObject<{
    code: z.ZodEnum<["INVALID_INPUT", "INSUFFICIENT_DATA", "CONSTRAINT_VIOLATION", "ANALYSIS_FAILED", "SERVICE_UNAVAILABLE"]>;
    message: z.ZodString;
    details: z.ZodOptional<z.ZodRecord<z.ZodString, z.ZodUnknown>>;
}, "strip", z.ZodTypeAny, {
    code: "INVALID_INPUT" | "INSUFFICIENT_DATA" | "CONSTRAINT_VIOLATION" | "ANALYSIS_FAILED" | "SERVICE_UNAVAILABLE";
    message: string;
    details?: Record<string, unknown> | undefined;
}, {
    code: "INVALID_INPUT" | "INSUFFICIENT_DATA" | "CONSTRAINT_VIOLATION" | "ANALYSIS_FAILED" | "SERVICE_UNAVAILABLE";
    message: string;
    details?: Record<string, unknown> | undefined;
}>;
//# sourceMappingURL=schemas.d.ts.map