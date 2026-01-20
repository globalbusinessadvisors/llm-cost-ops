/**
 * Decision Event Emitter
 *
 * Creates and emits DecisionEvents to ruvector-service.
 * Each agent invocation MUST emit exactly ONE DecisionEvent.
 *
 * DecisionEvent schema includes:
 * - agent_id
 * - agent_version
 * - decision_type
 * - inputs_hash
 * - outputs
 * - confidence (estimation certainty)
 * - constraints_applied
 * - execution_ref
 * - timestamp (UTC)
 */
import { createHash } from 'crypto';
import { v4 as uuidv4 } from 'uuid';
import { RuvectorServiceClient } from '../services/ruvector-client.js';
const AGENT_ID = 'cost-performance-tradeoff-agent';
const AGENT_VERSION = '1.0.0';
export class DecisionEventEmitter {
    ruvectorClient;
    constructor(ruvectorClient) {
        this.ruvectorClient = ruvectorClient ?? new RuvectorServiceClient();
    }
    /**
     * Create a DecisionEvent from analysis results
     */
    createEvent(options) {
        const inputsHash = this.hashInputs(options.input);
        // Calculate confidence based on data quality and analysis results
        const confidence = this.calculateConfidence(options.input, options.output);
        return {
            agent_id: AGENT_ID,
            agent_version: AGENT_VERSION,
            decision_type: options.decisionType,
            inputs_hash: inputsHash,
            outputs: {
                analysis_id: options.output.analysis_id,
                analysis_scope: options.output.analysis_scope,
                total_records: options.output.summary.total_records_analyzed,
                unique_models: options.output.summary.unique_models,
                best_overall: options.output.summary.best_overall,
                recommendations_count: options.output.recommendations?.length ?? 0,
                pareto_optimal_count: options.output.pareto_frontier?.length ?? 0,
                diminishing_returns_detected: options.output.diminishing_returns?.detected ?? false
            },
            confidence,
            constraints_applied: options.output.constraints_applied,
            execution_ref: options.executionRef ?? uuidv4(),
            timestamp: new Date().toISOString()
        };
    }
    /**
     * Emit a DecisionEvent to ruvector-service
     * Also outputs to stderr for local debugging
     */
    async emit(event) {
        // Output to stderr for local debugging/logging
        process.stderr.write(JSON.stringify({
            type: 'decision_event',
            event
        }) + '\n');
        // Persist to ruvector-service
        try {
            await this.ruvectorClient.persistDecisionEvent(event);
        }
        catch (error) {
            // Log but don't fail - decision events should not break the agent
            process.stderr.write(JSON.stringify({
                type: 'decision_event_error',
                error: error instanceof Error ? error.message : String(error)
            }) + '\n');
        }
    }
    /**
     * Create and emit a DecisionEvent in one call
     */
    async createAndEmit(options) {
        const event = this.createEvent(options);
        await this.emit(event);
        return event;
    }
    /**
     * Hash inputs for deterministic identification
     */
    hashInputs(input) {
        // Create a deterministic string representation
        const inputString = JSON.stringify({
            record_ids: input.records.map(r => r.id).sort(),
            analysis_scope: input.analysis_scope,
            weights: input.weights,
            constraints: input.constraints,
            options: input.options
        });
        return createHash('sha256').update(inputString).digest('hex').substring(0, 16);
    }
    /**
     * Calculate confidence score based on data quality and analysis
     */
    calculateConfidence(input, output) {
        let confidence = 0.5; // Base confidence
        // More records = higher confidence (up to +0.25)
        const recordCount = input.records.length;
        confidence += Math.min(recordCount / 100, 1) * 0.25;
        // Quality data available = higher confidence (+0.1)
        const recordsWithQuality = input.records.filter(r => r.quality !== undefined).length;
        if (recordsWithQuality / recordCount > 0.5) {
            confidence += 0.1;
        }
        // Multiple models/providers = higher confidence (+0.1)
        if (output.summary.unique_models > 2) {
            confidence += 0.05;
        }
        if (output.summary.unique_providers > 1) {
            confidence += 0.05;
        }
        // Pareto frontier identified = higher confidence (+0.05)
        if (output.pareto_frontier && output.pareto_frontier.length > 0) {
            confidence += 0.05;
        }
        return Math.min(confidence, 0.95); // Cap at 0.95
    }
}
/**
 * Export singleton instance for convenience
 */
export const decisionEventEmitter = new DecisionEventEmitter();
//# sourceMappingURL=decision-event.js.map