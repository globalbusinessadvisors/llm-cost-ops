/**
 * ROI Estimation Agent - Telemetry Emitter
 *
 * Emits telemetry events to LLM-Observatory for observability
 */

import {
  type TelemetryEvent,
  type TelemetryEmitter as ITelemetryEmitter,
  type SpanContext,
  type ROIAgentConfig
} from '../contracts/index.js';

/**
 * Telemetry Emitter for LLM-Observatory Integration
 *
 * Features:
 * - Event batching for efficiency
 * - Auto-flush on interval
 * - Structured span and metric emission
 */
export class TelemetryEmitter implements ITelemetryEmitter {
  private readonly endpoint: string;
  private readonly batchSize: number;
  private readonly flushIntervalMs: number;
  private eventBuffer: TelemetryEvent[] = [];
  private flushTimer: NodeJS.Timeout | null = null;
  private readonly agentId: string;

  constructor(config: ROIAgentConfig) {
    this.endpoint = config.telemetry.endpoint;
    this.batchSize = config.telemetry.batchSize;
    this.flushIntervalMs = config.telemetry.flushIntervalMs;
    this.agentId = config.agentId;

    // Start auto-flush timer
    this.startAutoFlush();
  }

  /**
   * Emit a generic telemetry event
   */
  async emit(event: TelemetryEvent): Promise<void> {
    this.eventBuffer.push(event);

    if (this.eventBuffer.length >= this.batchSize) {
      await this.flush();
    }
  }

  /**
   * Emit a span event for an operation
   */
  async emitSpan(
    name: string,
    agentId: string,
    durationMs: number,
    context: SpanContext,
    attributes?: Record<string, unknown>
  ): Promise<void> {
    const event: TelemetryEvent = {
      event_type: 'span',
      timestamp: new Date().toISOString(),
      agent_id: agentId,
      span_context: context,
      name,
      duration_ms: durationMs,
      attributes: {
        ...attributes,
        agent_type: 'roi_estimation',
        decision_type: 'roi_estimation'
      },
      status: 'ok'
    };

    await this.emit(event);
  }

  /**
   * Emit a metric event
   */
  async emitMetric(
    name: string,
    value: number,
    unit: string,
    attributes?: Record<string, unknown>
  ): Promise<void> {
    const event: TelemetryEvent = {
      event_type: 'metric',
      timestamp: new Date().toISOString(),
      agent_id: this.agentId,
      name,
      attributes: {
        value,
        unit,
        ...attributes,
        agent_type: 'roi_estimation'
      },
      status: 'ok'
    };

    await this.emit(event);
  }

  /**
   * Emit ROI analysis metrics
   */
  async emitROIMetrics(
    roiPercentage: number,
    confidence: number,
    correlationCount: number,
    processingTimeMs: number,
    context?: SpanContext
  ): Promise<void> {
    const baseAttributes = {
      agent_type: 'roi_estimation',
      decision_type: 'roi_estimation'
    };

    const events: TelemetryEvent[] = [
      {
        event_type: 'metric',
        timestamp: new Date().toISOString(),
        agent_id: this.agentId,
        span_context: context,
        name: 'roi.percentage',
        attributes: { ...baseAttributes, value: roiPercentage, unit: 'percent' },
        status: 'ok'
      },
      {
        event_type: 'metric',
        timestamp: new Date().toISOString(),
        agent_id: this.agentId,
        span_context: context,
        name: 'roi.confidence',
        attributes: { ...baseAttributes, value: confidence, unit: 'ratio' },
        status: 'ok'
      },
      {
        event_type: 'metric',
        timestamp: new Date().toISOString(),
        agent_id: this.agentId,
        span_context: context,
        name: 'roi.correlations_analyzed',
        attributes: { ...baseAttributes, value: correlationCount, unit: 'count' },
        status: 'ok'
      },
      {
        event_type: 'metric',
        timestamp: new Date().toISOString(),
        agent_id: this.agentId,
        span_context: context,
        name: 'roi.processing_time',
        attributes: { ...baseAttributes, value: processingTimeMs, unit: 'milliseconds' },
        status: 'ok'
      }
    ];

    for (const event of events) {
      await this.emit(event);
    }
  }

  /**
   * Emit an error event
   */
  async emitError(
    agentId: string,
    error: Error,
    context?: SpanContext
  ): Promise<void> {
    const event: TelemetryEvent = {
      event_type: 'log',
      timestamp: new Date().toISOString(),
      agent_id: agentId,
      span_context: context,
      name: 'error',
      attributes: {
        error_name: error.name,
        error_message: error.message,
        error_stack: error.stack,
        agent_type: 'roi_estimation',
        decision_type: 'roi_estimation'
      },
      status: 'error',
      error_message: error.message
    };

    await this.emit(event);
  }

  /**
   * Emit log event
   */
  async emitLog(
    level: 'debug' | 'info' | 'warn' | 'error',
    message: string,
    attributes?: Record<string, unknown>,
    context?: SpanContext
  ): Promise<void> {
    const event: TelemetryEvent = {
      event_type: 'log',
      timestamp: new Date().toISOString(),
      agent_id: this.agentId,
      span_context: context,
      name: `log.${level}`,
      attributes: {
        level,
        message,
        ...attributes,
        agent_type: 'roi_estimation'
      },
      status: level === 'error' ? 'error' : 'ok'
    };

    await this.emit(event);
  }

  /**
   * Flush all buffered events to the telemetry endpoint
   */
  async flush(): Promise<void> {
    if (this.eventBuffer.length === 0) {
      return;
    }

    const eventsToSend = [...this.eventBuffer];
    this.eventBuffer = [];

    try {
      const response = await fetch(`${this.endpoint}/api/v1/telemetry/batch`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'X-Agent-ID': this.agentId,
          'X-Agent-Type': 'roi_estimation'
        },
        body: JSON.stringify({ events: eventsToSend })
      });

      if (!response.ok) {
        // Log error but don't throw - telemetry should not break the agent
        console.error(
          `Telemetry flush failed: ${response.status} ${response.statusText}`
        );
        // Re-add events to buffer for retry (up to max buffer size)
        const remainingCapacity = this.batchSize * 2 - this.eventBuffer.length;
        if (remainingCapacity > 0) {
          this.eventBuffer.unshift(...eventsToSend.slice(0, remainingCapacity));
        }
      }
    } catch (error) {
      // Log error but don't throw - telemetry should not break the agent
      console.error('Telemetry flush error:', error);
      // Re-add events to buffer for retry
      const remainingCapacity = this.batchSize * 2 - this.eventBuffer.length;
      if (remainingCapacity > 0) {
        this.eventBuffer.unshift(...eventsToSend.slice(0, remainingCapacity));
      }
    }
  }

  /**
   * Get connection status
   */
  async getConnectionStatus(): Promise<'connected' | 'disconnected' | 'unknown'> {
    try {
      const response = await fetch(`${this.endpoint}/health`, {
        method: 'GET',
        headers: { 'Accept': 'application/json' }
      });
      return response.ok ? 'connected' : 'disconnected';
    } catch {
      return 'unknown';
    }
  }

  /**
   * Shutdown the emitter
   */
  async shutdown(): Promise<void> {
    this.stopAutoFlush();
    await this.flush();
  }

  // ============================================================================
  // PRIVATE HELPERS
  // ============================================================================

  private startAutoFlush(): void {
    if (this.flushTimer) {
      return;
    }

    this.flushTimer = setInterval(() => {
      this.flush().catch(error => {
        console.error('Auto-flush error:', error);
      });
    }, this.flushIntervalMs);

    // Don't let this timer keep the process alive
    if (this.flushTimer.unref) {
      this.flushTimer.unref();
    }
  }

  private stopAutoFlush(): void {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = null;
    }
  }
}

/**
 * Create a TelemetryEmitter from agent config
 */
export function createTelemetryEmitter(config: ROIAgentConfig): TelemetryEmitter {
  return new TelemetryEmitter(config);
}

/**
 * Generate a new span context
 */
export function createSpanContext(parentContext?: SpanContext): SpanContext {
  const generateId = (length: number): string => {
    const chars = '0123456789abcdef';
    let result = '';
    for (let i = 0; i < length; i++) {
      result += chars[Math.floor(Math.random() * chars.length)];
    }
    return result;
  };

  return {
    traceId: parentContext?.traceId || generateId(32),
    spanId: generateId(16),
    parentSpanId: parentContext?.spanId
  };
}
