/**
 * RuvectorServiceClient Tests
 *
 * Verifies compliance with LLM-CostOps Constitution:
 * - No local persistence
 * - All persistence via HTTP to ruvector-service
 * - Never executes SQL directly
 * - Async, non-blocking operations
 */

import { RuvectorServiceClient, DecisionEvent } from '../src/services/ruvector-client';

// Mock fetch globally
const mockFetch = jest.fn();
global.fetch = mockFetch;

describe('RuvectorServiceClient', () => {
  let client: RuvectorServiceClient;

  beforeEach(() => {
    mockFetch.mockClear();
    client = new RuvectorServiceClient({
      baseUrl: 'https://ruvector.example.com',
      apiKey: 'test-api-key',
      timeout: 5000,
      maxRetries: 2,
      retryDelayMs: 100,
    });
  });

  describe('Constitution Compliance', () => {
    it('should use HTTP for persistence, not SQL', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ success: true }),
      });

      const event: DecisionEvent = {
        timestamp: new Date().toISOString(),
        agent_id: 'cost-attribution-agent',
        decision_type: 'cost_attribution',
        context: { provider: 'anthropic' },
      };

      await client.persistDecisionEvent(event);

      // Verify HTTP was used
      expect(mockFetch).toHaveBeenCalledWith(
        'https://ruvector.example.com/api/v1/events',
        expect.objectContaining({
          method: 'POST',
          headers: expect.objectContaining({
            'Content-Type': 'application/json',
          }),
        })
      );
    });

    it('should be async and non-blocking', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ success: true }),
      });

      const event: DecisionEvent = {
        timestamp: new Date().toISOString(),
        agent_id: 'cost-attribution-agent',
        decision_type: 'cost_attribution',
        context: {},
      };

      // The call should return a Promise
      const result = client.persistDecisionEvent(event);
      expect(result).toBeInstanceOf(Promise);

      // Await to ensure completion
      await result;
    });
  });

  describe('persistDecisionEvent', () => {
    it('should send event to correct endpoint', async () => {
      mockFetch.mockResolvedValueOnce({ ok: true });

      const event: DecisionEvent = {
        timestamp: new Date().toISOString(),
        agent_id: 'cost-attribution-agent',
        decision_type: 'cost_attribution',
        context: { total_cost: '0.0015' },
      };

      await client.persistDecisionEvent(event);

      expect(mockFetch).toHaveBeenCalledWith(
        'https://ruvector.example.com/api/v1/events',
        expect.objectContaining({
          method: 'POST',
          body: JSON.stringify(event),
        })
      );
    });

    it('should include API key in Authorization header', async () => {
      mockFetch.mockResolvedValueOnce({ ok: true });

      const event: DecisionEvent = {
        timestamp: new Date().toISOString(),
        agent_id: 'test-agent',
        decision_type: 'cost_attribution',
        context: {},
      };

      await client.persistDecisionEvent(event);

      expect(mockFetch).toHaveBeenCalledWith(
        expect.any(String),
        expect.objectContaining({
          headers: expect.objectContaining({
            Authorization: 'Bearer test-api-key',
          }),
        })
      );
    });

    it('should retry on failure with exponential backoff', async () => {
      // Fail first time, succeed second time
      mockFetch
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce({ ok: true });

      const event: DecisionEvent = {
        timestamp: new Date().toISOString(),
        agent_id: 'test-agent',
        decision_type: 'cost_attribution',
        context: {},
      };

      await client.persistDecisionEvent(event);

      expect(mockFetch).toHaveBeenCalledTimes(2);
    });

    it('should throw after max retries exceeded', async () => {
      mockFetch.mockRejectedValue(new Error('Persistent failure'));

      const event: DecisionEvent = {
        timestamp: new Date().toISOString(),
        agent_id: 'test-agent',
        decision_type: 'cost_attribution',
        context: {},
      };

      await expect(client.persistDecisionEvent(event)).rejects.toThrow(
        /failed after 2 retries/i
      );

      // Initial + 2 retries = 3 attempts
      expect(mockFetch).toHaveBeenCalledTimes(3);
    });

    it('should handle HTTP error responses', async () => {
      mockFetch.mockResolvedValue({
        ok: false,
        status: 500,
        statusText: 'Internal Server Error',
        text: () => Promise.resolve('Server error details'),
      });

      const event: DecisionEvent = {
        timestamp: new Date().toISOString(),
        agent_id: 'test-agent',
        decision_type: 'cost_attribution',
        context: {},
      };

      await expect(client.persistDecisionEvent(event)).rejects.toThrow(
        /500 Internal Server Error/
      );
    });
  });

  describe('queryDecisionEvents', () => {
    it('should build query parameters correctly', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ events: [] }),
      });

      await client.queryDecisionEvents({
        agent_id: 'cost-attribution-agent',
        decision_type: 'cost_attribution',
        start_time: '2024-01-01T00:00:00Z',
        end_time: '2024-01-31T23:59:59Z',
        limit: 100,
        offset: 0,
      });

      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('agent_id=cost-attribution-agent'),
        expect.any(Object)
      );
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('decision_type=cost_attribution'),
        expect.any(Object)
      );
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('limit=100'),
        expect.any(Object)
      );
    });

    it('should return events from response', async () => {
      const mockEvents: DecisionEvent[] = [
        {
          timestamp: '2024-01-15T10:00:00Z',
          agent_id: 'cost-attribution-agent',
          decision_type: 'cost_attribution',
          context: { total_cost: '0.0015' },
        },
        {
          timestamp: '2024-01-15T11:00:00Z',
          agent_id: 'cost-attribution-agent',
          decision_type: 'cost_attribution',
          context: { total_cost: '0.0025' },
        },
      ];

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({ events: mockEvents }),
      });

      const result = await client.queryDecisionEvents({
        agent_id: 'cost-attribution-agent',
      });

      expect(result).toEqual(mockEvents);
    });
  });

  describe('healthCheck', () => {
    it('should return true when service is healthy', async () => {
      mockFetch.mockResolvedValueOnce({ ok: true });

      const result = await client.healthCheck();

      expect(result).toBe(true);
      expect(mockFetch).toHaveBeenCalledWith(
        'https://ruvector.example.com/health',
        expect.any(Object)
      );
    });

    it('should return false when service is unhealthy', async () => {
      mockFetch.mockResolvedValueOnce({ ok: false });

      const result = await client.healthCheck();

      expect(result).toBe(false);
    });

    it('should return false on network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      const result = await client.healthCheck();

      expect(result).toBe(false);
    });
  });
});
