/**
 * TelemetryEmitter - Emit telemetry events to LLM-Observatory
 *
 * CONSTITUTION COMPLIANCE:
 * - Async, non-blocking emission
 * - Compatible with LLM-Observatory format
 * - Structured spans and traces
 */

export interface TelemetryEvent {
  timestamp: string;
  event_type: string;
  agent_id: string;
  trace_id?: string;
  span_id?: string;
  parent_span_id?: string;
  metrics?: TelemetryMetrics;
  attributes?: Record<string, unknown>;
}

export interface TelemetryMetrics {
  latency_ms?: number;
  token_counts?: TokenCounts;
  cost_metrics?: CostMetrics;
  error_count?: number;
  success_count?: number;
}

export interface TokenCounts {
  prompt_tokens?: number;
  completion_tokens?: number;
  total_tokens?: number;
}

export interface CostMetrics {
  input_cost_usd?: number;
  output_cost_usd?: number;
  total_cost_usd?: number;
  provider?: string;
  model?: string;
}

export interface SpanContext {
  trace_id: string;
  span_id: string;
  parent_span_id?: string;
}

export interface TelemetryConfig {
  endpoint: string;
  apiKey?: string;
  timeout?: number;
  batchSize?: number;
  flushIntervalMs?: number;
}

export class TelemetryEmitter {
  private readonly endpoint: string;
  private readonly apiKey?: string;
  private readonly timeout: number;
  private readonly batchSize: number;
  private readonly flushIntervalMs: number;
  private eventBuffer: TelemetryEvent[] = [];
  private flushTimer?: NodeJS.Timeout;

  constructor(config: TelemetryConfig) {
    this.endpoint = config.endpoint.replace(/\/$/, '');
    this.apiKey = config.apiKey;
    this.timeout = config.timeout || 5000; // Default 5s timeout
    this.batchSize = config.batchSize || 10;
    this.flushIntervalMs = config.flushIntervalMs || 10000; // Default 10s

    // Start auto-flush timer
    this.startAutoFlush();
  }

  /**
   * Emit a telemetry event
   * Batches events and flushes periodically
   */
  async emit(event: TelemetryEvent): Promise<void> {
    this.eventBuffer.push(event);

    // Flush if batch size reached
    if (this.eventBuffer.length >= this.batchSize) {
      await this.flush();
    }
  }

  /**
   * Emit a span event with timing information
   */
  async emitSpan(
    name: string,
    agent_id: string,
    duration_ms: number,
    context: SpanContext,
    attributes?: Record<string, unknown>
  ): Promise<void> {
    await this.emit({
      timestamp: new Date().toISOString(),
      event_type: 'span',
      agent_id,
      trace_id: context.trace_id,
      span_id: context.span_id,
      parent_span_id: context.parent_span_id,
      metrics: {
        latency_ms: duration_ms,
      },
      attributes: {
        span_name: name,
        ...attributes,
      },
    });
  }

  /**
   * Emit LLM call metrics
   */
  async emitLLMCall(
    agent_id: string,
    provider: string,
    model: string,
    tokens: TokenCounts,
    cost: CostMetrics,
    latency_ms: number,
    context?: SpanContext
  ): Promise<void> {
    await this.emit({
      timestamp: new Date().toISOString(),
      event_type: 'llm_call',
      agent_id,
      trace_id: context?.trace_id,
      span_id: context?.span_id,
      parent_span_id: context?.parent_span_id,
      metrics: {
        latency_ms,
        token_counts: tokens,
        cost_metrics: cost,
      },
      attributes: {
        provider,
        model,
      },
    });
  }

  /**
   * Emit error event
   */
  async emitError(
    agent_id: string,
    error: Error,
    context?: SpanContext,
    attributes?: Record<string, unknown>
  ): Promise<void> {
    await this.emit({
      timestamp: new Date().toISOString(),
      event_type: 'error',
      agent_id,
      trace_id: context?.trace_id,
      span_id: context?.span_id,
      parent_span_id: context?.parent_span_id,
      metrics: {
        error_count: 1,
      },
      attributes: {
        error_name: error.name,
        error_message: error.message,
        error_stack: error.stack,
        ...attributes,
      },
    });
  }

  /**
   * Flush buffered events to LLM-Observatory
   */
  async flush(): Promise<void> {
    if (this.eventBuffer.length === 0) {
      return;
    }

    const eventsToFlush = [...this.eventBuffer];
    this.eventBuffer = [];

    try {
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), this.timeout);

      try {
        const response = await fetch(`${this.endpoint}/api/v1/telemetry`, {
          method: 'POST',
          headers: this.buildHeaders(),
          body: JSON.stringify({ events: eventsToFlush }),
          signal: controller.signal,
        });

        if (!response.ok) {
          const errorText = await response.text().catch(() => 'Unknown error');
          console.error(
            `Failed to flush telemetry: ${response.status} ${response.statusText} - ${errorText}`
          );
          // Re-add events to buffer on failure
          this.eventBuffer.push(...eventsToFlush);
        }
      } finally {
        clearTimeout(timeoutId);
      }
    } catch (error) {
      console.error(
        `Failed to flush telemetry: ${error instanceof Error ? error.message : String(error)}`
      );
      // Re-add events to buffer on failure
      this.eventBuffer.push(...eventsToFlush);
    }
  }

  /**
   * Start auto-flush timer
   */
  private startAutoFlush(): void {
    this.flushTimer = setInterval(() => {
      this.flush().catch(error => {
        console.error('Auto-flush failed:', error);
      });
    }, this.flushIntervalMs);
  }

  /**
   * Stop auto-flush and flush remaining events
   */
  async shutdown(): Promise<void> {
    if (this.flushTimer) {
      clearInterval(this.flushTimer);
      this.flushTimer = undefined;
    }
    await this.flush();
  }

  /**
   * Build HTTP headers
   */
  private buildHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    };

    if (this.apiKey) {
      headers['Authorization'] = `Bearer ${this.apiKey}`;
    }

    return headers;
  }

  /**
   * Generate unique trace ID
   */
  static generateTraceId(): string {
    return `trace-${Date.now()}-${Math.random().toString(36).substring(2, 15)}`;
  }

  /**
   * Generate unique span ID
   */
  static generateSpanId(): string {
    return `span-${Date.now()}-${Math.random().toString(36).substring(2, 15)}`;
  }

  /**
   * Create span context
   */
  static createSpanContext(parent_span_id?: string): SpanContext {
    const trace_id = this.generateTraceId();
    const span_id = this.generateSpanId();
    return {
      trace_id,
      span_id,
      parent_span_id,
    };
  }
}
