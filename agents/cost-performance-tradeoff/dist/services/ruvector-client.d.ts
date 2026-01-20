/**
 * RuVector Service Client
 *
 * HTTP client for persisting DecisionEvents to ruvector-service.
 * This is the ONLY way LLM-CostOps agents persist data - never direct SQL.
 *
 * Features:
 * - Async, non-blocking writes
 * - Exponential backoff retry
 * - Request timeout handling
 * - Health check endpoint
 */
import type { DecisionEvent, RuvectorConfig } from '../contracts/index.js';
export declare class RuvectorServiceClient {
    private readonly config;
    constructor(config?: Partial<RuvectorConfig>);
    /**
     * Persist a DecisionEvent to ruvector-service
     */
    persistDecisionEvent(event: DecisionEvent): Promise<void>;
    /**
     * Query decision events with filters
     */
    queryDecisionEvents(filters: {
        agentId?: string;
        decisionType?: string;
        startTime?: string;
        endTime?: string;
        limit?: number;
    }): Promise<DecisionEvent[]>;
    /**
     * Health check for ruvector-service
     */
    healthCheck(): Promise<{
        healthy: boolean;
        latencyMs: number;
    }>;
    private sleep;
}
//# sourceMappingURL=ruvector-client.d.ts.map