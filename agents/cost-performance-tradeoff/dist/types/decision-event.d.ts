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
import type { DecisionEvent, DecisionType, TradeoffAnalysisInput, TradeoffAnalysisOutput } from '../contracts/index.js';
import { RuvectorServiceClient } from '../services/ruvector-client.js';
export interface DecisionEventOptions {
    decisionType: DecisionType;
    input: TradeoffAnalysisInput;
    output: TradeoffAnalysisOutput;
    executionRef?: string;
    ruvectorClient?: RuvectorServiceClient;
}
export declare class DecisionEventEmitter {
    private readonly ruvectorClient;
    constructor(ruvectorClient?: RuvectorServiceClient);
    /**
     * Create a DecisionEvent from analysis results
     */
    createEvent(options: DecisionEventOptions): DecisionEvent;
    /**
     * Emit a DecisionEvent to ruvector-service
     * Also outputs to stderr for local debugging
     */
    emit(event: DecisionEvent): Promise<void>;
    /**
     * Create and emit a DecisionEvent in one call
     */
    createAndEmit(options: DecisionEventOptions): Promise<DecisionEvent>;
    /**
     * Hash inputs for deterministic identification
     */
    private hashInputs;
    /**
     * Calculate confidence score based on data quality and analysis
     */
    private calculateConfidence;
}
/**
 * Export singleton instance for convenience
 */
export declare const decisionEventEmitter: DecisionEventEmitter;
//# sourceMappingURL=decision-event.d.ts.map