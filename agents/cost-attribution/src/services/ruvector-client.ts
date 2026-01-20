/**
 * RuvectorServiceClient - HTTP client for ruvector-service persistence
 *
 * CONSTITUTION COMPLIANCE:
 * - No local persistence - all data via HTTP to ruvector-service
 * - Never executes SQL directly
 * - Async, non-blocking operations
 * - Retry logic with exponential backoff
 */

export interface DecisionEvent {
  timestamp: string;
  agent_id: string;
  decision_type: string;
  context: Record<string, unknown>;
  outcome?: Record<string, unknown>;
  metadata?: Record<string, unknown>;
}

export interface DecisionEventFilters {
  agent_id?: string;
  decision_type?: string;
  start_time?: string;
  end_time?: string;
  limit?: number;
  offset?: number;
}

export interface RuvectorConfig {
  baseUrl: string;
  apiKey?: string;
  timeout?: number;
  maxRetries?: number;
  retryDelayMs?: number;
}

export interface RetryConfig {
  maxRetries: number;
  delayMs: number;
  backoffMultiplier: number;
}

export class RuvectorServiceClient {
  private readonly baseUrl: string;
  private readonly apiKey?: string;
  private readonly timeout: number;
  private readonly retryConfig: RetryConfig;

  constructor(config: RuvectorConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, ''); // Remove trailing slash
    this.apiKey = config.apiKey;
    this.timeout = config.timeout || 10000; // Default 10s timeout
    this.retryConfig = {
      maxRetries: config.maxRetries || 3,
      delayMs: config.retryDelayMs || 1000,
      backoffMultiplier: 2,
    };
  }

  /**
   * Persist a decision event to ruvector-service
   * Non-blocking, async operation with retry logic
   */
  async persistDecisionEvent(event: DecisionEvent): Promise<void> {
    const endpoint = `${this.baseUrl}/api/v1/events`;

    await this.executeWithRetry(async () => {
      const response = await this.fetch(endpoint, {
        method: 'POST',
        headers: this.buildHeaders(),
        body: JSON.stringify(event),
      });

      if (!response.ok) {
        const errorText = await response.text().catch(() => 'Unknown error');
        throw new Error(
          `Failed to persist decision event: ${response.status} ${response.statusText} - ${errorText}`
        );
      }
    });
  }

  /**
   * Query decision events from ruvector-service
   * Returns filtered events based on provided criteria
   */
  async queryDecisionEvents(filters: DecisionEventFilters): Promise<DecisionEvent[]> {
    const queryParams = this.buildQueryParams(filters);
    const endpoint = `${this.baseUrl}/api/v1/events?${queryParams}`;

    return await this.executeWithRetry(async () => {
      const response = await this.fetch(endpoint, {
        method: 'GET',
        headers: this.buildHeaders(),
      });

      if (!response.ok) {
        const errorText = await response.text().catch(() => 'Unknown error');
        throw new Error(
          `Failed to query decision events: ${response.status} ${response.statusText} - ${errorText}`
        );
      }

      const data = await response.json();
      return data.events || [];
    });
  }

  /**
   * Execute operation with exponential backoff retry logic
   */
  private async executeWithRetry<T>(
    operation: () => Promise<T>,
    attempt: number = 0
  ): Promise<T> {
    try {
      return await operation();
    } catch (error) {
      if (attempt >= this.retryConfig.maxRetries) {
        throw new Error(
          `Operation failed after ${this.retryConfig.maxRetries} retries: ${
            error instanceof Error ? error.message : String(error)
          }`
        );
      }

      // Exponential backoff
      const delay = this.retryConfig.delayMs * Math.pow(this.retryConfig.backoffMultiplier, attempt);

      await this.sleep(delay);
      return this.executeWithRetry(operation, attempt + 1);
    }
  }

  /**
   * Build HTTP headers with optional API key
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
   * Build query parameters from filters
   */
  private buildQueryParams(filters: DecisionEventFilters): string {
    const params = new URLSearchParams();

    if (filters.agent_id) {
      params.append('agent_id', filters.agent_id);
    }
    if (filters.decision_type) {
      params.append('decision_type', filters.decision_type);
    }
    if (filters.start_time) {
      params.append('start_time', filters.start_time);
    }
    if (filters.end_time) {
      params.append('end_time', filters.end_time);
    }
    if (filters.limit !== undefined) {
      params.append('limit', filters.limit.toString());
    }
    if (filters.offset !== undefined) {
      params.append('offset', filters.offset.toString());
    }

    return params.toString();
  }

  /**
   * Wrapper around fetch with timeout support
   */
  private async fetch(url: string, options: RequestInit): Promise<Response> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(url, {
        ...options,
        signal: controller.signal,
      });
      return response;
    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        throw new Error(`Request timeout after ${this.timeout}ms`);
      }
      throw error;
    } finally {
      clearTimeout(timeoutId);
    }
  }

  /**
   * Sleep utility for retry delays
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Health check - verify ruvector-service is reachable
   */
  async healthCheck(): Promise<boolean> {
    try {
      const endpoint = `${this.baseUrl}/health`;
      const response = await this.fetch(endpoint, {
        method: 'GET',
        headers: this.buildHeaders(),
      });
      return response.ok;
    } catch {
      return false;
    }
  }
}
