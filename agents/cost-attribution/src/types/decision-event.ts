/**
 * Decision Event for RuVector Service
 *
 * Every CLI invocation emits a DecisionEvent to track:
 * - What decision was made
 * - Input context
 * - Output result
 * - Confidence level
 * - Metadata for learning
 */

export interface DecisionEvent {
  /**
   * Unique identifier for this decision
   */
  id: string;

  /**
   * Type of decision made
   */
  type: 'cost-attribution' | 'cost-inspection' | 'batch-processing';

  /**
   * ISO timestamp when decision was made
   */
  timestamp: string;

  /**
   * Input context that led to this decision
   */
  input: {
    command: string;
    args: Record<string, unknown>;
    usageRecords?: number; // Count of usage records processed
  };

  /**
   * Output of the decision
   */
  output: {
    success: boolean;
    recordsProcessed?: number;
    totalCost?: string;
    currency?: string;
    errors?: string[];
  };

  /**
   * Confidence level in the decision (0-1)
   */
  confidence: number;

  /**
   * Additional metadata for learning
   */
  metadata: {
    executionTimeMs: number;
    memoryUsedMb?: number;
    [key: string]: unknown;
  };
}

/**
 * Emit a decision event to RuVector Service
 * In production, this would send to the actual service
 */
export class DecisionEventEmitter {
  private ruvectorServiceUrl: string;

  constructor(ruvectorServiceUrl = process.env.RUVECTOR_SERVICE_URL || 'http://localhost:8080') {
    this.ruvectorServiceUrl = ruvectorServiceUrl;
  }

  /**
   * Emit a decision event
   * For now, this logs to stderr in JSON format
   * In production, would POST to ruvector-service
   */
  async emit(event: DecisionEvent): Promise<void> {
    // Deterministic output to stderr for machine readability
    const eventJson = JSON.stringify({
      _type: 'decision_event',
      ...event,
    });

    // Log to stderr so it doesn't interfere with stdout results
    console.error(eventJson);

    // In production, would do:
    // await fetch(`${this.ruvectorServiceUrl}/api/v1/decisions`, {
    //   method: 'POST',
    //   headers: { 'Content-Type': 'application/json' },
    //   body: eventJson
    // });
  }

  /**
   * Create a decision event from CLI execution
   */
  createEvent(
    type: DecisionEvent['type'],
    command: string,
    args: Record<string, unknown>,
    output: DecisionEvent['output'],
    metadata: DecisionEvent['metadata']
  ): DecisionEvent {
    return {
      id: this.generateEventId(),
      type,
      timestamp: new Date().toISOString(),
      input: {
        command,
        args,
        usageRecords: typeof args.input === 'string' ? undefined : 0,
      },
      output,
      confidence: output.success ? 0.95 : 0.5,
      metadata,
    };
  }

  private generateEventId(): string {
    // Deterministic ID based on timestamp + random
    const timestamp = Date.now();
    const random = Math.floor(Math.random() * 1000000);
    return `evt_${timestamp}_${random}`;
  }
}
