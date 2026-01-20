/**
 * ROI Estimation Agent - RuVector Service Client
 *
 * Persistence layer via ruvector-service
 * MUST NOT access Google SQL directly per constitution
 */

import {
  type DecisionEvent,
  type DecisionEventFilters,
  type StorageAdapter,
  type ROIAgentConfig
} from '../contracts/index.js';

/**
 * RuVector Service Client
 *
 * Handles all persistence operations via ruvector-service HTTP API.
 * Constitution requirement: NO direct SQL access.
 */
export class RuvectorServiceClient implements StorageAdapter {
  private readonly baseUrl: string;
  private readonly timeoutMs: number;
  private readonly retryAttempts: number;
  private readonly retryDelayMs: number;

  constructor(config: ROIAgentConfig['ruvectorService']) {
    this.baseUrl = config.baseUrl;
    this.timeoutMs = config.timeoutMs;
    this.retryAttempts = config.retryAttempts;
    this.retryDelayMs = config.retryDelayMs;
  }

  /**
   * Persist a DecisionEvent to ruvector-service
   *
   * Constitution: All cost records, forecasts, budgets, and ROI analyses
   * are persisted via ruvector-service only.
   */
  async saveDecisionEvent(event: DecisionEvent): Promise<void> {
    const payload = {
      timestamp: event.timestamp,
      agent_id: event.agent_id,
      agent_version: event.agent_version,
      decision_type: event.decision_type,
      execution_ref: event.execution_ref,
      inputs_hash: event.inputs_hash,
      outputs: event.outputs,
      confidence: event.confidence,
      constraints_applied: event.constraints_applied,
      metadata: event.metadata
    };

    await this.executeWithRetry(async () => {
      const response = await this.fetchWithTimeout(
        `${this.baseUrl}/api/v1/decisions`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'X-Agent-ID': event.agent_id,
            'X-Agent-Version': event.agent_version
          },
          body: JSON.stringify(payload)
        }
      );

      if (!response.ok) {
        const errorBody = await response.text().catch(() => 'Unknown error');
        throw new Error(
          `Failed to persist DecisionEvent: ${response.status} ${response.statusText} - ${errorBody}`
        );
      }
    });
  }

  /**
   * Query DecisionEvents from ruvector-service
   */
  async queryDecisionEvents(filters: DecisionEventFilters): Promise<DecisionEvent[]> {
    const params = new URLSearchParams();

    if (filters.agentId) params.append('agent_id', filters.agentId);
    if (filters.decisionType) params.append('decision_type', filters.decisionType);
    if (filters.startTime) params.append('start_time', filters.startTime);
    if (filters.endTime) params.append('end_time', filters.endTime);
    if (filters.executionRef) params.append('execution_ref', filters.executionRef);
    if (filters.limit) params.append('limit', filters.limit.toString());
    if (filters.offset) params.append('offset', filters.offset.toString());

    const url = `${this.baseUrl}/api/v1/decisions?${params.toString()}`;

    return await this.executeWithRetry(async () => {
      const response = await this.fetchWithTimeout(url, {
        method: 'GET',
        headers: {
          'Accept': 'application/json'
        }
      });

      if (!response.ok) {
        throw new Error(
          `Failed to query DecisionEvents: ${response.status} ${response.statusText}`
        );
      }

      const data = await response.json();
      return data.items || data;
    });
  }

  /**
   * Health check for ruvector-service connectivity
   */
  async healthCheck(): Promise<boolean> {
    try {
      const response = await this.fetchWithTimeout(
        `${this.baseUrl}/health`,
        {
          method: 'GET',
          headers: {
            'Accept': 'application/json'
          }
        }
      );

      return response.ok;
    } catch {
      return false;
    }
  }

  /**
   * Get connection status string for health reporting
   */
  async getConnectionStatus(): Promise<'connected' | 'disconnected' | 'unknown'> {
    try {
      const isHealthy = await this.healthCheck();
      return isHealthy ? 'connected' : 'disconnected';
    } catch {
      return 'unknown';
    }
  }

  // ============================================================================
  // PRIVATE HELPERS
  // ============================================================================

  /**
   * Execute operation with exponential backoff retry
   */
  private async executeWithRetry<T>(
    operation: () => Promise<T>,
    attempt: number = 1
  ): Promise<T> {
    try {
      return await operation();
    } catch (error) {
      if (attempt >= this.retryAttempts) {
        throw error;
      }

      // Exponential backoff: delay * 2^(attempt-1)
      const delay = this.retryDelayMs * Math.pow(2, attempt - 1);

      // Add jitter (0-25% of delay)
      const jitter = Math.random() * delay * 0.25;

      await this.sleep(delay + jitter);

      return this.executeWithRetry(operation, attempt + 1);
    }
  }

  /**
   * Fetch with timeout
   */
  private async fetchWithTimeout(
    url: string,
    options: RequestInit
  ): Promise<Response> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeoutMs);

    try {
      const response = await fetch(url, {
        ...options,
        signal: controller.signal
      });
      return response;
    } finally {
      clearTimeout(timeoutId);
    }
  }

  /**
   * Sleep helper
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Create a RuvectorServiceClient from agent config
 */
export function createRuvectorClient(
  config: ROIAgentConfig
): RuvectorServiceClient {
  return new RuvectorServiceClient(config.ruvectorService);
}
