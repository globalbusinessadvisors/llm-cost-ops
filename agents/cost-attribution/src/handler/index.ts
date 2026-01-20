/**
 * Cost Attribution Agent - Main Handler
 *
 * Google Cloud Edge Function handler for cost attribution analysis
 */

import type { Request, Response } from '@google-cloud/functions-framework';
import { randomUUID } from 'crypto';
import { CostCalculator } from '../calculator';
import { CostAttributor } from '../attributor';
import type { CostAttributionOutput, DecisionEventMetadata, TelemetryMetadata } from '../types';
import {
  validateInput,
  checkRateLimit,
  logRequest,
  handleError,
  validateMethod,
  setCorsHeaders,
  handleOptions,
} from './middleware';
import { sendSuccess } from './response';

// Agent configuration
const AGENT_ID = process.env.AGENT_ID || 'cost-attribution-agent';
const RUVECTOR_SERVICE_URL = process.env.RUVECTOR_SERVICE_URL;
const OBSERVATORY_URL = process.env.OBSERVATORY_URL;

// Instantiate service classes
const costCalculator = new CostCalculator();
const costAttributor = new CostAttributor();

/**
 * Main HTTP handler for Cost Attribution Agent
 *
 * This is a stateless edge function that:
 * 1. Validates input using Zod schemas
 * 2. Calculates costs using CostCalculator
 * 3. Attributes costs using CostAttributor
 * 4. Emits DecisionEvent to ruvector-service
 * 5. Emits telemetry to LLM-Observatory
 * 6. Returns structured CostAttributionOutput
 */
export async function costAttributionHandler(
  req: Request,
  res: Response
): Promise<void> {
  const startTime = Date.now();

  try {
    // Set CORS headers
    setCorsHeaders(res);

    // Handle OPTIONS preflight
    if (handleOptions(req, res)) {
      return;
    }

    // Validate HTTP method
    if (!validateMethod(req, res)) {
      return;
    }

    // Check rate limit
    if (!checkRateLimit(req, res)) {
      return;
    }

    // Validate input
    const input = validateInput(req, res);
    if (!input) {
      return; // Response already sent by validateInput
    }

    // Log request
    logRequest(req, input.requestId, 'info');

    // Calculate costs
    const costs = costCalculator.calculate(input.usage, input.pricingContext);

    // Attribute costs to dimensions
    const attribution = costAttributor.attribute(input.dimensions);

    // Generate event IDs
    const eventId = randomUUID();
    const telemetryId = randomUUID();
    const analysisTimestamp = new Date().toISOString();

    // Calculate processing duration
    const processingDurationMs = Date.now() - startTime;

    // Build decision event
    const decisionEvent: DecisionEventMetadata = {
      eventId,
      eventType: 'cost_attribution',
      timestamp: analysisTimestamp,
      agentId: AGENT_ID,
      decision: {
        action: 'attribute_cost',
        result: 'success',
        confidence: attribution.confidence,
      },
      context: {
        provider: input.usage.provider,
        model: input.usage.model,
        totalTokens:
          input.usage.inputTokens +
          input.usage.outputTokens +
          (input.usage.cachedTokens || 0),
        totalCost: costs.totalCost,
      },
    };

    // Build telemetry metadata
    const telemetry: TelemetryMetadata = {
      telemetryId,
      agentId: AGENT_ID,
      timestamp: analysisTimestamp,
      metrics: {
        processingDurationMs,
        dimensionCount: Object.values(attribution.dimensions).filter(Boolean).length,
      },
      trace: req.headers['x-cloud-trace-context']
        ? {
            traceId: req.headers['x-cloud-trace-context'] as string,
            spanId: randomUUID(),
          }
        : undefined,
    };

    // Build output
    const output: CostAttributionOutput = {
      requestId: input.requestId,
      analysisTimestamp,
      costs,
      attribution,
      decisionEvent,
      telemetry,
    };

    // Emit decision event to ruvector-service (fire-and-forget)
    if (RUVECTOR_SERVICE_URL) {
      emitDecisionEvent(decisionEvent, output).catch((error) => {
        console.error('Failed to emit decision event:', error);
      });
    }

    // Emit telemetry to LLM-Observatory (fire-and-forget)
    if (OBSERVATORY_URL) {
      emitTelemetry(telemetry, output).catch((error) => {
        console.error('Failed to emit telemetry:', error);
      });
    }

    // Send success response
    sendSuccess(res, output);
  } catch (error) {
    // Handle unexpected errors
    handleError(error, res, req.body?.requestId);
  }
}

/**
 * Emit decision event to ruvector-service
 * This is a fire-and-forget operation
 */
async function emitDecisionEvent(
  event: DecisionEventMetadata,
  output: CostAttributionOutput
): Promise<void> {
  if (!RUVECTOR_SERVICE_URL) {
    return;
  }

  const payload = {
    event,
    data: {
      requestId: output.requestId,
      costs: output.costs,
      attribution: output.attribution,
    },
  };

  const response = await fetch(`${RUVECTOR_SERVICE_URL}/events`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(payload),
    signal: AbortSignal.timeout(5000), // 5 second timeout
  });

  if (!response.ok) {
    throw new Error(
      `Failed to emit decision event: ${response.status} ${response.statusText}`
    );
  }
}

/**
 * Emit telemetry to LLM-Observatory
 * This is a fire-and-forget operation
 */
async function emitTelemetry(
  telemetry: TelemetryMetadata,
  output: CostAttributionOutput
): Promise<void> {
  if (!OBSERVATORY_URL) {
    return;
  }

  const payload = {
    telemetry,
    agent: {
      id: AGENT_ID,
      type: 'cost-attribution',
      version: process.env.AGENT_VERSION || '1.0.0',
    },
    metrics: {
      ...telemetry.metrics,
      costMetrics: {
        totalCost: output.costs.totalCost,
        currency: output.costs.currency,
        costPer1kTokens: output.costs.costPer1kTokens,
      },
    },
  };

  const response = await fetch(`${OBSERVATORY_URL}/telemetry`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(payload),
    signal: AbortSignal.timeout(5000), // 5 second timeout
  });

  if (!response.ok) {
    throw new Error(
      `Failed to emit telemetry: ${response.status} ${response.statusText}`
    );
  }
}
