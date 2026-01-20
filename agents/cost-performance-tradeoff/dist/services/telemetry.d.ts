/**
 * Telemetry Emitter
 *
 * Emits telemetry events compatible with LLM-Observatory format.
 *
 * Features:
 * - Event batching
 * - Auto-flush timer
 * - Distributed tracing support
 */
import type { TelemetryConfig, TelemetryEvent } from '../contracts/index.js';
export declare class TelemetryEmitter {
    private readonly config;
    private buffer;
    private flushTimer;
    constructor(config?: Partial<TelemetryConfig>);
    /**
     * Emit a telemetry event
     */
    emit(event: TelemetryEvent): Promise<void>;
    /**
     * Emit a span event for distributed tracing
     */
    emitSpan(name: string, agentId: string, durationMs: number, traceId?: string, parentSpanId?: string): Promise<void>;
    /**
     * Emit a tradeoff analysis event
     */
    emitTradeoffAnalysis(agentId: string, analysisId: string, recordsAnalyzed: number, durationMs: number, traceId?: string): Promise<void>;
    /**
     * Emit an error event
     */
    emitError(agentId: string, error: Error, context?: Record<string, unknown>, traceId?: string): Promise<void>;
    /**
     * Flush buffered events to the telemetry endpoint
     */
    flush(): Promise<void>;
    /**
     * Stop the auto-flush timer
     */
    stop(): void;
    private startFlushTimer;
}
//# sourceMappingURL=telemetry.d.ts.map