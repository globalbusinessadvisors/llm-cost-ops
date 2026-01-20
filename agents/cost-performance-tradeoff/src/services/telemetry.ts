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

import { v4 as uuidv4 } from 'uuid';
import type { TelemetryConfig, TelemetryEvent } from '../contracts/index.js';

const DEFAULT_CONFIG: TelemetryConfig = {
  endpoint: process.env.TELEMETRY_ENDPOINT ?? 'http://localhost:8081/api/v1/telemetry',
  batchSize: 10,
  flushIntervalMs: 5000
};

export class TelemetryEmitter {
  private readonly config: TelemetryConfig;
  private buffer: TelemetryEvent[] = [];
  private flushTimer: NodeJS.Timeout | null = null;

  constructor(config: Partial<TelemetryConfig> = {}) {
    this.config = { ...DEFAULT_CONFIG, ...config };
    this.startFlushTimer();
  }

  /**
   * Emit a telemetry event
   */
  async emit(event: TelemetryEvent): Promise<void> {
    this.buffer.push(event);

    if (this.buffer.length >= this.config.batchSize) {
      await this.flush();
    }
  }

  /**
   * Emit a span event for distributed tracing
   */
  async emitSpan(
    name: string,
    agentId: string,
    durationMs: number,
    traceId?: string,
    parentSpanId?: string
  ): Promise<void> {
    const event: TelemetryEvent = {
      event_type: 'span',
      agent_id: agentId,
      timestamp: new Date().toISOString(),
      data: {
        name,
        duration_ms: durationMs,
        status: 'ok'
      },
      trace_id: traceId ?? uuidv4(),
      span_id: uuidv4()
    };

    if (parentSpanId) {
      event.data.parent_span_id = parentSpanId;
    }

    await this.emit(event);
  }

  /**
   * Emit a tradeoff analysis event
   */
  async emitTradeoffAnalysis(
    agentId: string,
    analysisId: string,
    recordsAnalyzed: number,
    durationMs: number,
    traceId?: string
  ): Promise<void> {
    const event: TelemetryEvent = {
      event_type: 'tradeoff_analysis',
      agent_id: agentId,
      timestamp: new Date().toISOString(),
      data: {
        analysis_id: analysisId,
        records_analyzed: recordsAnalyzed,
        duration_ms: durationMs
      },
      trace_id: traceId ?? uuidv4(),
      span_id: uuidv4()
    };

    await this.emit(event);
  }

  /**
   * Emit an error event
   */
  async emitError(
    agentId: string,
    error: Error,
    context?: Record<string, unknown>,
    traceId?: string
  ): Promise<void> {
    const event: TelemetryEvent = {
      event_type: 'error',
      agent_id: agentId,
      timestamp: new Date().toISOString(),
      data: {
        error_name: error.name,
        error_message: error.message,
        error_stack: error.stack,
        context
      },
      trace_id: traceId ?? uuidv4(),
      span_id: uuidv4()
    };

    await this.emit(event);
  }

  /**
   * Flush buffered events to the telemetry endpoint
   */
  async flush(): Promise<void> {
    if (this.buffer.length === 0) {
      return;
    }

    const events = [...this.buffer];
    this.buffer = [];

    try {
      const response = await fetch(this.config.endpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ events })
      });

      if (!response.ok) {
        // Log but don't throw - telemetry should not break the agent
        console.error(`Telemetry flush failed: HTTP ${response.status}`);
      }
    } catch (error) {
      // Log but don't throw
      console.error('Telemetry flush error:', error);
    }
  }

  /**
   * Stop the auto-flush timer
   */
  stop(): void {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
  }

  private startFlushTimer(): void {
    this.flushTimer = setInterval(() => {
      this.flush().catch(console.error);
    }, this.config.flushIntervalMs);
  }
}
