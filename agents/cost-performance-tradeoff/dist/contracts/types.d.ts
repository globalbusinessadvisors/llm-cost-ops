/**
 * Cost-Performance Tradeoff Agent - TypeScript Types
 *
 * Types inferred from Zod schemas for type-safe usage throughout the agent.
 */
import { z } from 'zod';
import { ProviderSchema, ModelTierSchema, LatencyMetricsSchema, QualityMetricsSchema, CostMetricsSchema, PerformanceRecordSchema, TradeoffAnalysisInputSchema, TradeoffScoreSchema, DiminishingReturnsAnalysisSchema, ParetoPointSchema, TradeoffResultSchema, TradeoffRecommendationSchema, TradeoffAnalysisOutputSchema, DecisionTypeSchema, DecisionEventSchema, TradeoffErrorSchema } from './schemas.js';
export type Provider = z.infer<typeof ProviderSchema>;
export type ModelTier = z.infer<typeof ModelTierSchema>;
export type LatencyMetrics = z.infer<typeof LatencyMetricsSchema>;
export type QualityMetrics = z.infer<typeof QualityMetricsSchema>;
export type CostMetrics = z.infer<typeof CostMetricsSchema>;
export type PerformanceRecord = z.infer<typeof PerformanceRecordSchema>;
export type TradeoffAnalysisInput = z.infer<typeof TradeoffAnalysisInputSchema>;
export type TradeoffScore = z.infer<typeof TradeoffScoreSchema>;
export type DiminishingReturnsAnalysis = z.infer<typeof DiminishingReturnsAnalysisSchema>;
export type ParetoPoint = z.infer<typeof ParetoPointSchema>;
export type TradeoffResult = z.infer<typeof TradeoffResultSchema>;
export type TradeoffRecommendation = z.infer<typeof TradeoffRecommendationSchema>;
export type TradeoffAnalysisOutput = z.infer<typeof TradeoffAnalysisOutputSchema>;
export type DecisionType = z.infer<typeof DecisionTypeSchema>;
export type DecisionEvent = z.infer<typeof DecisionEventSchema>;
export type TradeoffError = z.infer<typeof TradeoffErrorSchema>;
export interface AgentConfig {
    agentId: string;
    agentVersion: string;
    enableDecisionLogging: boolean;
    ruvectorServiceUrl: string;
    telemetryEndpoint: string;
    maxBatchSize: number;
    defaultWeights: {
        cost: number;
        latency: number;
        quality: number;
    };
}
export interface RuvectorConfig {
    baseUrl: string;
    apiKey?: string;
    timeout: number;
    maxRetries: number;
}
export interface TelemetryConfig {
    endpoint: string;
    batchSize: number;
    flushIntervalMs: number;
}
export interface TelemetryEvent {
    event_type: string;
    agent_id: string;
    timestamp: string;
    data: Record<string, unknown>;
    trace_id?: string;
    span_id?: string;
}
export interface AnalysisOptions {
    includeDiminishingReturns: boolean;
    includeParetoFrontier: boolean;
    includeRecommendations: boolean;
    normalizeMetrics: boolean;
}
export interface AnalysisWeights {
    cost: number;
    latency: number;
    quality: number;
}
export interface AnalysisConstraints {
    maxCostPerRequestUsd?: number;
    maxLatencyP95Ms?: number;
    minQualityScore?: number;
}
export interface AnalyzeCommandOptions {
    input: string;
    format: 'json' | 'table';
    scope: 'model' | 'provider' | 'tier' | 'execution';
    weights?: string;
    maxCost?: number;
    maxLatency?: number;
    minQuality?: number;
    summary: boolean;
}
export interface InspectCommandOptions {
    model?: string;
    provider?: string;
    format: 'json' | 'table';
    limit: number;
}
export interface AggregatedMetrics {
    identifier: string;
    provider: Provider;
    model: string;
    modelTier?: ModelTier;
    records: PerformanceRecord[];
    avgCost: CostMetrics;
    avgLatency: LatencyMetrics;
    avgQuality?: QualityMetrics;
}
export interface NormalizedMetrics {
    cost: number;
    latency: number;
    quality: number;
}
export type AnalysisResult = {
    success: true;
    output: TradeoffAnalysisOutput;
} | {
    success: false;
    error: TradeoffError;
};
//# sourceMappingURL=types.d.ts.map