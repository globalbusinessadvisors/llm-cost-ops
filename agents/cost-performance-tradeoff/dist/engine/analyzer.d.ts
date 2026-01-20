/**
 * Cost-Performance Tradeoff Analyzer
 *
 * Core analysis engine for evaluating tradeoffs between cost, latency, and quality.
 * This engine is deterministic and produces machine-readable output.
 *
 * Classification: TRADEOFF ANALYSIS
 *
 * This agent:
 * - Analyzes cost vs latency vs quality metrics
 * - Identifies diminishing returns
 * - Computes Pareto frontier
 * - Emits tradeoff recommendations (non-executing)
 *
 * This agent MUST NOT:
 * - Enforce budgets
 * - Modify execution behavior
 * - Apply optimizations automatically
 * - Intercept runtime execution
 */
import type { TradeoffAnalysisInput, TradeoffAnalysisOutput } from '../contracts/index.js';
export declare class TradeoffAnalyzer {
    /**
     * Analyze tradeoffs between cost, latency, and quality
     */
    analyze(input: TradeoffAnalysisInput): TradeoffAnalysisOutput;
    /**
     * Aggregate records by the specified scope
     */
    private aggregateByScope;
    /**
     * Calculate average cost metrics
     */
    private averageCostMetrics;
    /**
     * Calculate average latency metrics
     */
    private averageLatencyMetrics;
    /**
     * Calculate average quality metrics
     */
    private averageQualityMetrics;
    /**
     * Calculate tradeoff scores for aggregated metrics
     */
    private calculateTradeoffScores;
    /**
     * Compute Pareto frontier (set of non-dominated options)
     */
    private computeParetoFrontier;
    /**
     * Detect diminishing returns in cost vs quality relationship
     */
    private detectDiminishingReturns;
    /**
     * Generate recommendations based on analysis results
     */
    private generateRecommendations;
    /**
     * Find best result by a specific metric
     */
    private findBestByMetric;
    /**
     * Create a recommendation object
     */
    private createRecommendation;
    /**
     * Build summary statistics
     */
    private buildSummary;
}
//# sourceMappingURL=analyzer.d.ts.map