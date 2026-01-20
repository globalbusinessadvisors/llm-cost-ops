/**
 * Edge Function Handler
 *
 * Google Cloud Edge Function handler for the Cost-Performance Tradeoff Agent.
 *
 * Deployment:
 *   gcloud functions deploy cost-performance-tradeoff-agent \
 *     --runtime nodejs20 \
 *     --trigger-http \
 *     --allow-unauthenticated \
 *     --entry-point handler
 *
 * This handler:
 * - Receives HTTP requests with performance records
 * - Validates input against schemas
 * - Runs tradeoff analysis
 * - Emits DecisionEvent to ruvector-service
 * - Returns analysis results
 *
 * This handler MUST NOT:
 * - Intercept runtime execution
 * - Trigger retries
 * - Execute workflows
 * - Modify routing or execution behavior
 * - Apply optimizations automatically
 * - Enforce policies directly
 */

import { v4 as uuidv4 } from 'uuid';
import { TradeoffAnalysisInputSchema } from '../contracts/schemas.js';
import { TradeoffAnalyzer } from '../engine/index.js';
import { DecisionEventEmitter } from '../types/index.js';
import { TelemetryEmitter } from '../services/index.js';
import type { TradeoffAnalysisInput, TradeoffAnalysisOutput, TradeoffError } from '../contracts/index.js';

const AGENT_ID = 'cost-performance-tradeoff-agent';
const AGENT_VERSION = '1.0.0';

interface HttpRequest {
  method: string;
  headers: Record<string, string>;
  body: unknown;
  query: Record<string, string>;
}

interface HttpResponse {
  status: (code: number) => HttpResponse;
  json: (data: unknown) => void;
  set: (header: string, value: string) => HttpResponse;
}

type SuccessResponse = {
  success: true;
  data: TradeoffAnalysisOutput;
  metadata: {
    agent_id: string;
    agent_version: string;
    execution_time_ms: number;
    trace_id: string;
  };
};

type ErrorResponse = {
  success: false;
  error: TradeoffError;
  metadata: {
    agent_id: string;
    agent_version: string;
    trace_id: string;
  };
};

/**
 * Main Edge Function handler
 */
export async function handler(req: HttpRequest, res: HttpResponse): Promise<void> {
  const startTime = Date.now();
  const traceId = req.headers['x-trace-id'] ?? uuidv4();

  // Set CORS headers
  res.set('Access-Control-Allow-Origin', '*');
  res.set('Access-Control-Allow-Methods', 'POST, OPTIONS');
  res.set('Access-Control-Allow-Headers', 'Content-Type, X-Trace-ID');

  // Handle preflight
  if (req.method === 'OPTIONS') {
    res.status(204).json({});
    return;
  }

  // Only accept POST
  if (req.method !== 'POST') {
    const error: ErrorResponse = {
      success: false,
      error: {
        code: 'INVALID_INPUT',
        message: 'Only POST method is allowed'
      },
      metadata: {
        agent_id: AGENT_ID,
        agent_version: AGENT_VERSION,
        trace_id: traceId
      }
    };
    res.status(405).json(error);
    return;
  }

  const telemetry = new TelemetryEmitter();
  const decisionEmitter = new DecisionEventEmitter();

  try {
    // Validate input
    const parseResult = TradeoffAnalysisInputSchema.safeParse(req.body);

    if (!parseResult.success) {
      const error: ErrorResponse = {
        success: false,
        error: {
          code: 'INVALID_INPUT',
          message: 'Invalid input schema',
          details: {
            errors: parseResult.error.errors
          }
        },
        metadata: {
          agent_id: AGENT_ID,
          agent_version: AGENT_VERSION,
          trace_id: traceId
        }
      };

      await telemetry.emitError(
        AGENT_ID,
        new Error('Input validation failed'),
        { errors: parseResult.error.errors },
        traceId
      );

      res.status(400).json(error);
      return;
    }

    const input: TradeoffAnalysisInput = parseResult.data;

    // Check for minimum data
    if (input.records.length === 0) {
      const error: ErrorResponse = {
        success: false,
        error: {
          code: 'INSUFFICIENT_DATA',
          message: 'At least one performance record is required'
        },
        metadata: {
          agent_id: AGENT_ID,
          agent_version: AGENT_VERSION,
          trace_id: traceId
        }
      };

      res.status(400).json(error);
      return;
    }

    // Run analysis
    const analyzer = new TradeoffAnalyzer();
    const output = analyzer.analyze(input);

    const executionTimeMs = Date.now() - startTime;

    // Emit telemetry
    await telemetry.emitTradeoffAnalysis(
      AGENT_ID,
      output.analysis_id,
      output.summary.total_records_analyzed,
      executionTimeMs,
      traceId
    );

    // Emit decision event (REQUIRED - exactly ONE per invocation)
    await decisionEmitter.createAndEmit({
      decisionType: 'cost_performance_tradeoff',
      input,
      output,
      executionRef: traceId
    });

    // Return success response
    const response: SuccessResponse = {
      success: true,
      data: output,
      metadata: {
        agent_id: AGENT_ID,
        agent_version: AGENT_VERSION,
        execution_time_ms: executionTimeMs,
        trace_id: traceId
      }
    };

    await telemetry.flush();
    telemetry.stop();

    res.status(200).json(response);
  } catch (error) {
    const executionTimeMs = Date.now() - startTime;

    await telemetry.emitError(
      AGENT_ID,
      error as Error,
      { execution_time_ms: executionTimeMs },
      traceId
    );

    await telemetry.flush();
    telemetry.stop();

    const errorResponse: ErrorResponse = {
      success: false,
      error: {
        code: 'ANALYSIS_FAILED',
        message: error instanceof Error ? error.message : String(error)
      },
      metadata: {
        agent_id: AGENT_ID,
        agent_version: AGENT_VERSION,
        trace_id: traceId
      }
    };

    res.status(500).json(errorResponse);
  }
}

/**
 * Health check endpoint
 */
export async function health(_req: HttpRequest, res: HttpResponse): Promise<void> {
  res.status(200).json({
    status: 'healthy',
    agent_id: AGENT_ID,
    agent_version: AGENT_VERSION,
    timestamp: new Date().toISOString()
  });
}
