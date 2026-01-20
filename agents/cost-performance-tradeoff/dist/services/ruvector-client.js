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
const DEFAULT_CONFIG = {
    baseUrl: process.env.RUVECTOR_SERVICE_URL ?? 'http://localhost:8080',
    apiKey: process.env.RUVECTOR_API_KEY,
    timeout: 5000,
    maxRetries: 3
};
export class RuvectorServiceClient {
    config;
    constructor(config = {}) {
        this.config = { ...DEFAULT_CONFIG, ...config };
    }
    /**
     * Persist a DecisionEvent to ruvector-service
     */
    async persistDecisionEvent(event) {
        const endpoint = `${this.config.baseUrl}/api/v1/decision-events`;
        let lastError;
        for (let attempt = 0; attempt < this.config.maxRetries; attempt++) {
            try {
                const controller = new AbortController();
                const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);
                const response = await fetch(endpoint, {
                    method: 'POST',
                    headers: {
                        'Content-Type': 'application/json',
                        ...(this.config.apiKey && { 'Authorization': `Bearer ${this.config.apiKey}` })
                    },
                    body: JSON.stringify(event),
                    signal: controller.signal
                });
                clearTimeout(timeoutId);
                if (!response.ok) {
                    throw new Error(`HTTP ${response.status}: ${await response.text()}`);
                }
                return; // Success
            }
            catch (error) {
                lastError = error;
                // Don't retry on abort (timeout)
                if (error instanceof DOMException && error.name === 'AbortError') {
                    throw new Error(`Request timeout after ${this.config.timeout}ms`);
                }
                // Exponential backoff
                if (attempt < this.config.maxRetries - 1) {
                    const delay = Math.pow(2, attempt) * 100;
                    await this.sleep(delay);
                }
            }
        }
        throw new Error(`Failed to persist DecisionEvent after ${this.config.maxRetries} attempts: ${lastError?.message}`);
    }
    /**
     * Query decision events with filters
     */
    async queryDecisionEvents(filters) {
        const params = new URLSearchParams();
        if (filters.agentId)
            params.set('agent_id', filters.agentId);
        if (filters.decisionType)
            params.set('decision_type', filters.decisionType);
        if (filters.startTime)
            params.set('start_time', filters.startTime);
        if (filters.endTime)
            params.set('end_time', filters.endTime);
        if (filters.limit)
            params.set('limit', filters.limit.toString());
        const endpoint = `${this.config.baseUrl}/api/v1/decision-events?${params}`;
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);
        try {
            const response = await fetch(endpoint, {
                method: 'GET',
                headers: {
                    ...(this.config.apiKey && { 'Authorization': `Bearer ${this.config.apiKey}` })
                },
                signal: controller.signal
            });
            clearTimeout(timeoutId);
            if (!response.ok) {
                throw new Error(`HTTP ${response.status}: ${await response.text()}`);
            }
            const data = await response.json();
            return data.events;
        }
        catch (error) {
            if (error instanceof DOMException && error.name === 'AbortError') {
                throw new Error(`Request timeout after ${this.config.timeout}ms`);
            }
            throw error;
        }
    }
    /**
     * Health check for ruvector-service
     */
    async healthCheck() {
        const startTime = Date.now();
        const endpoint = `${this.config.baseUrl}/health`;
        try {
            const controller = new AbortController();
            const timeoutId = setTimeout(() => controller.abort(), 2000);
            const response = await fetch(endpoint, {
                method: 'GET',
                signal: controller.signal
            });
            clearTimeout(timeoutId);
            return {
                healthy: response.ok,
                latencyMs: Date.now() - startTime
            };
        }
        catch {
            return {
                healthy: false,
                latencyMs: Date.now() - startTime
            };
        }
    }
    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
}
//# sourceMappingURL=ruvector-client.js.map