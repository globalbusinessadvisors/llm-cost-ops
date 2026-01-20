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
import { v4 as uuidv4 } from 'uuid';
// Default weights for tradeoff calculation
const DEFAULT_WEIGHTS = {
    cost: 0.33,
    latency: 0.33,
    quality: 0.34
};
export class TradeoffAnalyzer {
    /**
     * Analyze tradeoffs between cost, latency, and quality
     */
    analyze(input) {
        const startTime = Date.now();
        // Validate and set defaults
        const weights = input.weights ?? DEFAULT_WEIGHTS;
        const scope = input.analysis_scope ?? 'model';
        const options = {
            includeDiminishingReturns: input.options?.include_diminishing_returns ?? true,
            includeParetoFrontier: input.options?.include_pareto_frontier ?? true,
            includeRecommendations: input.options?.include_recommendations ?? true,
            normalizeMetrics: input.options?.normalize_metrics ?? true
        };
        // Step 1: Aggregate records by scope (model, provider, tier, execution)
        const aggregated = this.aggregateByScope(input.records, scope);
        // Step 2: Calculate tradeoff scores for each aggregation
        const results = this.calculateTradeoffScores(aggregated, weights, options.normalizeMetrics);
        // Step 3: Compute Pareto frontier if requested
        const paretoFrontier = options.includeParetoFrontier
            ? this.computeParetoFrontier(results)
            : undefined;
        // Step 4: Detect diminishing returns if requested
        const diminishingReturns = options.includeDiminishingReturns
            ? this.detectDiminishingReturns(results)
            : undefined;
        // Step 5: Generate recommendations if requested
        // Convert snake_case constraints to camelCase for internal use
        const internalConstraints = input.constraints ? {
            maxCostPerRequestUsd: input.constraints.max_cost_per_request_usd,
            maxLatencyP95Ms: input.constraints.max_latency_p95_ms,
            minQualityScore: input.constraints.min_quality_score
        } : undefined;
        const recommendations = options.includeRecommendations
            ? this.generateRecommendations(results, internalConstraints)
            : undefined;
        // Step 6: Build summary
        const summary = this.buildSummary(input.records, results);
        const analysisId = uuidv4();
        const analyzedAt = new Date().toISOString();
        const durationMs = Date.now() - startTime;
        return {
            analysis_id: analysisId,
            analyzed_at: analyzedAt,
            analysis_scope: scope,
            results,
            pareto_frontier: paretoFrontier,
            diminishing_returns: diminishingReturns,
            recommendations,
            summary,
            constraints_applied: input.constraints ? {
                max_cost_per_request_usd: input.constraints.max_cost_per_request_usd,
                max_latency_p95_ms: input.constraints.max_latency_p95_ms,
                min_quality_score: input.constraints.min_quality_score
            } : undefined,
            metadata: {
                weights_used: weights,
                analysis_duration_ms: durationMs
            }
        };
    }
    /**
     * Aggregate records by the specified scope
     */
    aggregateByScope(records, scope) {
        const groups = new Map();
        for (const record of records) {
            let key;
            switch (scope) {
                case 'model':
                    key = `${record.provider}:${record.model}`;
                    break;
                case 'provider':
                    key = record.provider;
                    break;
                case 'tier':
                    key = record.model_tier ?? 'unknown';
                    break;
                case 'execution':
                    key = record.execution_id ?? record.id;
                    break;
                default:
                    key = `${record.provider}:${record.model}`;
            }
            const existing = groups.get(key) ?? [];
            existing.push(record);
            groups.set(key, existing);
        }
        const aggregated = [];
        for (const [identifier, groupRecords] of groups) {
            const firstRecord = groupRecords[0];
            aggregated.push({
                identifier,
                provider: firstRecord.provider,
                model: firstRecord.model,
                modelTier: firstRecord.model_tier,
                records: groupRecords,
                avgCost: this.averageCostMetrics(groupRecords),
                avgLatency: this.averageLatencyMetrics(groupRecords),
                avgQuality: this.averageQualityMetrics(groupRecords)
            });
        }
        return aggregated;
    }
    /**
     * Calculate average cost metrics
     */
    averageCostMetrics(records) {
        const n = records.length;
        const totalCost = records.reduce((sum, r) => sum + r.cost.total_cost_usd, 0);
        const totalTokens = records.reduce((sum, r) => sum + r.cost.token_count, 0);
        const avgCostPerRequest = records.reduce((sum, r) => sum + r.cost.cost_per_request_usd, 0) / n;
        const avgCostPer1kTokens = records.reduce((sum, r) => sum + r.cost.cost_per_1k_tokens_usd, 0) / n;
        return {
            cost_per_request_usd: avgCostPerRequest,
            cost_per_1k_tokens_usd: avgCostPer1kTokens,
            total_cost_usd: totalCost,
            token_count: totalTokens
        };
    }
    /**
     * Calculate average latency metrics
     */
    averageLatencyMetrics(records) {
        const n = records.length;
        // Collect all P values and compute averages
        const p50Sum = records.reduce((s, r) => s + r.latency.p50_ms, 0);
        const p95Sum = records.reduce((s, r) => s + r.latency.p95_ms, 0);
        const p99Sum = records.reduce((s, r) => s + r.latency.p99_ms, 0);
        const avgSum = records.reduce((s, r) => s + r.latency.avg_ms, 0);
        const minVal = Math.min(...records.map(r => r.latency.min_ms));
        const maxVal = Math.max(...records.map(r => r.latency.max_ms));
        return {
            p50_ms: p50Sum / n,
            p95_ms: p95Sum / n,
            p99_ms: p99Sum / n,
            avg_ms: avgSum / n,
            min_ms: minVal,
            max_ms: maxVal
        };
    }
    /**
     * Calculate average quality metrics
     */
    averageQualityMetrics(records) {
        const recordsWithQuality = records.filter(r => r.quality !== undefined);
        if (recordsWithQuality.length === 0) {
            return undefined;
        }
        const n = recordsWithQuality.length;
        const qualities = recordsWithQuality.map(r => r.quality);
        const avgComposite = qualities.reduce((s, q) => s + q.composite_score, 0) / n;
        // Average optional fields if present
        const accuracyRecords = qualities.filter(q => q.accuracy !== undefined);
        const relevanceRecords = qualities.filter(q => q.relevance !== undefined);
        const coherenceRecords = qualities.filter(q => q.coherence !== undefined);
        const helpfulnessRecords = qualities.filter(q => q.helpfulness !== undefined);
        return {
            accuracy: accuracyRecords.length > 0
                ? accuracyRecords.reduce((s, q) => s + q.accuracy, 0) / accuracyRecords.length
                : undefined,
            relevance: relevanceRecords.length > 0
                ? relevanceRecords.reduce((s, q) => s + q.relevance, 0) / relevanceRecords.length
                : undefined,
            coherence: coherenceRecords.length > 0
                ? coherenceRecords.reduce((s, q) => s + q.coherence, 0) / coherenceRecords.length
                : undefined,
            helpfulness: helpfulnessRecords.length > 0
                ? helpfulnessRecords.reduce((s, q) => s + q.helpfulness, 0) / helpfulnessRecords.length
                : undefined,
            composite_score: avgComposite
        };
    }
    /**
     * Calculate tradeoff scores for aggregated metrics
     */
    calculateTradeoffScores(aggregated, weights, normalize) {
        // Find min/max for normalization
        const allCosts = aggregated.map(a => a.avgCost.cost_per_request_usd);
        const allLatencies = aggregated.map(a => a.avgLatency.p95_ms);
        const allQualities = aggregated
            .filter(a => a.avgQuality !== undefined)
            .map(a => a.avgQuality.composite_score);
        const minCost = Math.min(...allCosts);
        const maxCost = Math.max(...allCosts);
        const minLatency = Math.min(...allLatencies);
        const maxLatency = Math.max(...allLatencies);
        const minQuality = allQualities.length > 0 ? Math.min(...allQualities) : 0;
        const maxQuality = allQualities.length > 0 ? Math.max(...allQualities) : 1;
        return aggregated.map(agg => {
            // Normalize metrics to 0-1 range
            let normalized;
            if (normalize && maxCost !== minCost && maxLatency !== minLatency) {
                normalized = {
                    cost: (agg.avgCost.cost_per_request_usd - minCost) / (maxCost - minCost),
                    latency: (agg.avgLatency.p95_ms - minLatency) / (maxLatency - minLatency),
                    quality: agg.avgQuality
                        ? (maxQuality !== minQuality
                            ? (agg.avgQuality.composite_score - minQuality) / (maxQuality - minQuality)
                            : 0.5)
                        : 0.5 // Default quality when not available
                };
            }
            else {
                // Use raw values scaled to reasonable ranges
                normalized = {
                    cost: Math.min(agg.avgCost.cost_per_request_usd / 0.1, 1), // $0.10 = max
                    latency: Math.min(agg.avgLatency.p95_ms / 5000, 1), // 5000ms = max
                    quality: agg.avgQuality?.composite_score ?? 0.5
                };
            }
            // Calculate component scores (invert cost and latency so higher is better)
            const costScore = 1 - normalized.cost;
            const latencyScore = 1 - normalized.latency;
            const qualityScore = normalized.quality;
            // Calculate weighted overall score
            const overallScore = weights.cost * costScore +
                weights.latency * latencyScore +
                weights.quality * qualityScore;
            // Calculate efficiency ratio (quality per dollar)
            const efficiencyRatio = agg.avgCost.cost_per_request_usd > 0
                ? (agg.avgQuality?.composite_score ?? 0.5) / agg.avgCost.cost_per_request_usd
                : 0;
            const tradeoffScore = {
                overall_score: overallScore,
                cost_score: costScore,
                latency_score: latencyScore,
                quality_score: qualityScore,
                efficiency_ratio: efficiencyRatio
            };
            return {
                identifier: agg.identifier,
                provider: agg.provider,
                model: agg.model,
                model_tier: agg.modelTier,
                avg_cost: agg.avgCost,
                avg_latency: agg.avgLatency,
                avg_quality: agg.avgQuality,
                tradeoff_score: tradeoffScore,
                record_count: agg.records.length
            };
        });
    }
    /**
     * Compute Pareto frontier (set of non-dominated options)
     */
    computeParetoFrontier(results) {
        const points = results.map(r => ({
            model: r.model,
            provider: r.provider,
            cost: r.avg_cost.cost_per_request_usd,
            latency: r.avg_latency.p95_ms,
            quality: r.avg_quality?.composite_score,
            is_optimal: false // Will be set below
        }));
        // Find Pareto-optimal points (non-dominated)
        for (let i = 0; i < points.length; i++) {
            let isDominated = false;
            for (let j = 0; j < points.length; j++) {
                if (i === j)
                    continue;
                // Check if point j dominates point i
                // A point dominates if it's better or equal in all dimensions and strictly better in at least one
                const jBetterCost = points[j].cost <= points[i].cost;
                const jBetterLatency = points[j].latency <= points[i].latency;
                const jBetterQuality = (points[j].quality ?? 0) >= (points[i].quality ?? 0);
                const jStrictlyBetterCost = points[j].cost < points[i].cost;
                const jStrictlyBetterLatency = points[j].latency < points[i].latency;
                const jStrictlyBetterQuality = (points[j].quality ?? 0) > (points[i].quality ?? 0);
                if (jBetterCost && jBetterLatency && jBetterQuality &&
                    (jStrictlyBetterCost || jStrictlyBetterLatency || jStrictlyBetterQuality)) {
                    isDominated = true;
                    break;
                }
            }
            points[i].is_optimal = !isDominated;
        }
        // Return only Pareto-optimal points
        return points.filter(p => p.is_optimal);
    }
    /**
     * Detect diminishing returns in cost vs quality relationship
     */
    detectDiminishingReturns(results) {
        // Sort by cost
        const sorted = [...results]
            .filter(r => r.avg_quality !== undefined)
            .sort((a, b) => a.avg_cost.cost_per_request_usd - b.avg_cost.cost_per_request_usd);
        if (sorted.length < 3) {
            return {
                detected: false,
                recommendation: 'Insufficient data points to detect diminishing returns (need at least 3)'
            };
        }
        // Calculate marginal quality gains
        const marginalGains = [];
        for (let i = 1; i < sorted.length; i++) {
            const costIncrease = sorted[i].avg_cost.cost_per_request_usd - sorted[i - 1].avg_cost.cost_per_request_usd;
            const qualityGain = sorted[i].avg_quality.composite_score - sorted[i - 1].avg_quality.composite_score;
            if (costIncrease > 0) {
                marginalGains.push({
                    cost: sorted[i].avg_cost.cost_per_request_usd,
                    gain: qualityGain / costIncrease // Quality gain per dollar
                });
            }
        }
        // Detect if marginal gains are decreasing significantly
        let diminishingDetected = false;
        let thresholdCost;
        let lastPositiveMarginalGain;
        for (let i = 1; i < marginalGains.length; i++) {
            const previousGain = marginalGains[i - 1].gain;
            const currentGain = marginalGains[i].gain;
            // Diminishing returns if current marginal gain is less than 50% of previous
            if (previousGain > 0 && currentGain < previousGain * 0.5) {
                diminishingDetected = true;
                thresholdCost = marginalGains[i].cost;
                lastPositiveMarginalGain = currentGain;
                break;
            }
        }
        if (diminishingDetected) {
            return {
                detected: true,
                threshold_cost_usd: thresholdCost,
                marginal_quality_gain: lastPositiveMarginalGain,
                recommendation: `Diminishing returns detected above $${thresholdCost?.toFixed(4)} per request. ` +
                    `Consider models at or below this cost threshold for optimal value.`
            };
        }
        return {
            detected: false,
            recommendation: 'No significant diminishing returns detected. Quality scales roughly linearly with cost.'
        };
    }
    /**
     * Generate recommendations based on analysis results
     */
    generateRecommendations(results, constraints) {
        const recommendations = [];
        // Filter results that meet constraints
        let eligibleResults = results;
        if (constraints) {
            eligibleResults = results.filter(r => {
                if (constraints.maxCostPerRequestUsd && r.avg_cost.cost_per_request_usd > constraints.maxCostPerRequestUsd) {
                    return false;
                }
                if (constraints.maxLatencyP95Ms && r.avg_latency.p95_ms > constraints.maxLatencyP95Ms) {
                    return false;
                }
                if (constraints.minQualityScore && r.avg_quality && r.avg_quality.composite_score < constraints.minQualityScore) {
                    return false;
                }
                return true;
            });
            if (eligibleResults.length === 0) {
                recommendations.push({
                    recommendation_type: 'constraint_violation',
                    recommended_model: results[0]?.model ?? 'unknown',
                    recommended_provider: results[0]?.provider ?? 'OpenAI',
                    rationale: 'No models meet all specified constraints. Consider relaxing constraints.',
                    estimated_impact: {
                        cost_change_percent: 0,
                        latency_change_percent: 0,
                        quality_change_percent: 0
                    },
                    confidence: 0.3
                });
                return recommendations;
            }
        }
        // Find best options for different optimization goals
        const bestCost = this.findBestByMetric(eligibleResults, 'cost');
        const bestLatency = this.findBestByMetric(eligibleResults, 'latency');
        const bestQuality = this.findBestByMetric(eligibleResults, 'quality');
        const bestOverall = this.findBestByMetric(eligibleResults, 'overall');
        // Cost optimization recommendation
        if (bestCost) {
            recommendations.push(this.createRecommendation('cost_optimization', bestCost, eligibleResults, 'Optimizes for lowest cost while maintaining acceptable performance.'));
        }
        // Latency optimization recommendation
        if (bestLatency) {
            recommendations.push(this.createRecommendation('latency_optimization', bestLatency, eligibleResults, 'Optimizes for lowest latency, ideal for real-time applications.'));
        }
        // Quality optimization recommendation
        if (bestQuality && bestQuality.avg_quality) {
            recommendations.push(this.createRecommendation('quality_optimization', bestQuality, eligibleResults, 'Optimizes for highest quality output, suitable for critical applications.'));
        }
        // Balanced recommendation
        if (bestOverall) {
            recommendations.push(this.createRecommendation('balanced', bestOverall, eligibleResults, 'Balanced tradeoff between cost, latency, and quality based on specified weights.'));
        }
        return recommendations;
    }
    /**
     * Find best result by a specific metric
     */
    findBestByMetric(results, metric) {
        if (results.length === 0)
            return undefined;
        switch (metric) {
            case 'cost':
                return results.reduce((best, r) => r.avg_cost.cost_per_request_usd < best.avg_cost.cost_per_request_usd ? r : best);
            case 'latency':
                return results.reduce((best, r) => r.avg_latency.p95_ms < best.avg_latency.p95_ms ? r : best);
            case 'quality':
                return results
                    .filter(r => r.avg_quality !== undefined)
                    .reduce((best, r) => (r.avg_quality?.composite_score ?? 0) > (best.avg_quality?.composite_score ?? 0) ? r : best, results[0]);
            case 'overall':
                return results.reduce((best, r) => r.tradeoff_score.overall_score > best.tradeoff_score.overall_score ? r : best);
        }
    }
    /**
     * Create a recommendation object
     */
    createRecommendation(type, best, allResults, rationale) {
        // Calculate average metrics for comparison
        const avgCost = allResults.reduce((s, r) => s + r.avg_cost.cost_per_request_usd, 0) / allResults.length;
        const avgLatency = allResults.reduce((s, r) => s + r.avg_latency.p95_ms, 0) / allResults.length;
        const resultsWithQuality = allResults.filter(r => r.avg_quality !== undefined);
        const avgQuality = resultsWithQuality.length > 0
            ? resultsWithQuality.reduce((s, r) => s + r.avg_quality.composite_score, 0) / resultsWithQuality.length
            : undefined;
        const costChange = avgCost > 0 ? ((best.avg_cost.cost_per_request_usd - avgCost) / avgCost) * 100 : 0;
        const latencyChange = avgLatency > 0 ? ((best.avg_latency.p95_ms - avgLatency) / avgLatency) * 100 : 0;
        const qualityChange = avgQuality && best.avg_quality
            ? ((best.avg_quality.composite_score - avgQuality) / avgQuality) * 100
            : undefined;
        // Confidence based on sample size and data consistency
        const confidence = Math.min(0.95, 0.5 + (best.record_count / 100) * 0.45);
        return {
            recommendation_type: type,
            recommended_model: best.model,
            recommended_provider: best.provider,
            rationale,
            estimated_impact: {
                cost_change_percent: Number(costChange.toFixed(2)),
                latency_change_percent: Number(latencyChange.toFixed(2)),
                quality_change_percent: qualityChange !== undefined ? Number(qualityChange.toFixed(2)) : undefined
            },
            confidence
        };
    }
    /**
     * Build summary statistics
     */
    buildSummary(records, results) {
        const uniqueModels = new Set(records.map(r => r.model)).size;
        const uniqueProviders = new Set(records.map(r => r.provider)).size;
        // Find best performers
        const bestEfficiency = results.reduce((best, r) => r.tradeoff_score.efficiency_ratio > (best?.tradeoff_score.efficiency_ratio ?? 0) ? r : best, results[0]);
        const bestLatency = results.reduce((best, r) => r.avg_latency.p95_ms < (best?.avg_latency.p95_ms ?? Infinity) ? r : best, results[0]);
        const resultsWithQuality = results.filter(r => r.avg_quality !== undefined);
        const bestQuality = resultsWithQuality.length > 0
            ? resultsWithQuality.reduce((best, r) => (r.avg_quality?.composite_score ?? 0) > (best.avg_quality?.composite_score ?? 0) ? r : best)
            : undefined;
        const bestOverall = results.reduce((best, r) => r.tradeoff_score.overall_score > best.tradeoff_score.overall_score ? r : best, results[0]);
        return {
            total_records_analyzed: records.length,
            unique_models: uniqueModels,
            unique_providers: uniqueProviders,
            best_cost_efficiency: bestEfficiency?.identifier,
            best_latency: bestLatency?.identifier,
            best_quality: bestQuality?.identifier,
            best_overall: bestOverall?.identifier
        };
    }
}
//# sourceMappingURL=analyzer.js.map