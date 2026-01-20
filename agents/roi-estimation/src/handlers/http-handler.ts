/**
 * ROI Estimation Agent - HTTP Handler
 *
 * Google Cloud Functions entry point
 * Classification: ROI ANALYSIS
 * Decision Type: roi_estimation
 */

import { createHash } from 'crypto';
import { v4 as uuidv4 } from 'uuid';
import type { Request, Response } from '@google-cloud/functions-framework';
import {
  ROIInputSchema,
  AnalyzeROIResponseSchema,
  HealthCheckResponseSchema,
  type ROIInput,
  type ROIOutput,
  type DecisionEvent,
  type AnalyzeROIResponse,
  type HealthCheckResponse,
  type ROIAgentConfig,
  type SpanContext,
  DEFAULT_AGENT_CONFIG,
  AGENT_METADATA
} from '../contracts/index.js';
import { roiCalculator } from '../engine/roi-calculator.js';
import { metricsCorrelator } from '../engine/metrics-correlator.js';
import {
  createRuvectorClient,
  createTelemetryEmitter,
  createSpanContext,
  type RuvectorServiceClient,
  type TelemetryEmitter
} from '../services/index.js';

/**
 * ROI Estimation Agent HTTP Handler
 *
 * Stateless Google Cloud Edge Function handler for ROI analysis.
 *
 * Constitution Compliance:
 * - Stateless execution
 * - Deterministic behavior
 * - No execution interception
 * - No direct SQL access
 * - Async writes via ruvector-service only
 */
export class ROIEstimationHandler {
  private readonly config: ROIAgentConfig;
  private readonly ruvectorClient: RuvectorServiceClient;
  private readonly telemetry: TelemetryEmitter;

  constructor(config: ROIAgentConfig = DEFAULT_AGENT_CONFIG) {
    this.config = config;
    this.ruvectorClient = createRuvectorClient(config);
    this.telemetry = createTelemetryEmitter(config);
  }

  /**
   * Handle incoming HTTP request
   */
  async handleRequest(req: Request, res: Response): Promise<void> {
    const startTime = Date.now();
    const spanContext = createSpanContext();

    // Route based on path
    const path = req.path || '/';

    try {
      if (req.method === 'GET' && path === '/health') {
        await this.handleHealthCheck(req, res);
        return;
      }

      if (req.method === 'POST' && (path === '/analyze' || path === '/')) {
        await this.handleAnalyze(req, res, spanContext, startTime);
        return;
      }

      if (req.method === 'GET' && path === '/inspect') {
        await this.handleInspect(req, res);
        return;
      }

      // 404 for unknown routes
      res.status(404).json({
        success: false,
        error: {
          code: 'INVALID_INPUT',
          message: `Unknown route: ${req.method} ${path}`,
          timestamp: new Date().toISOString()
        }
      });
    } catch (error) {
      await this.handleError(error, res, spanContext);
    }
  }

  /**
   * Handle /analyze endpoint - main ROI analysis
   */
  private async handleAnalyze(
    req: Request,
    res: Response,
    spanContext: SpanContext,
    startTime: number
  ): Promise<void> {
    // 1. Validate input
    const parseResult = ROIInputSchema.safeParse(req.body);
    if (!parseResult.success) {
      const response: AnalyzeROIResponse = {
        success: false,
        error: {
          code: 'INVALID_INPUT',
          message: 'Input validation failed',
          details: { errors: parseResult.error.errors },
          timestamp: new Date().toISOString(),
          request_id: req.body?.request_id
        }
      };
      res.status(400).json(response);
      return;
    }

    const input: ROIInput = parseResult.data;

    try {
      // 2. Perform ROI analysis
      const output = await this.performROIAnalysis(input, spanContext);

      const processingTimeMs = Date.now() - startTime;

      // 3. Create and persist DecisionEvent
      const decisionEvent = this.createDecisionEvent(
        input,
        output,
        processingTimeMs,
        spanContext
      );

      // Persist to ruvector-service (async, non-blocking write)
      if (this.config.enableDecisionLogging) {
        await this.ruvectorClient.saveDecisionEvent(decisionEvent);
      }

      // 4. Emit telemetry
      await this.telemetry.emitROIMetrics(
        output.roi_metrics.roi_percentage,
        decisionEvent.confidence,
        output.correlations.length,
        processingTimeMs,
        spanContext
      );

      await this.telemetry.emitSpan(
        'roi_estimation.analyze',
        this.config.agentId,
        processingTimeMs,
        spanContext,
        {
          analysis_scope: input.analysis_scope,
          cost_records_count: input.cost_records?.length || 0,
          outcome_metrics_count: input.outcome_metrics?.length || 0,
          roi_percentage: output.roi_metrics.roi_percentage,
          overall_assessment: output.summary.overall_assessment
        }
      );

      // 5. Return response
      const response: AnalyzeROIResponse = {
        success: true,
        data: output,
        decision_event_id: decisionEvent.execution_ref
      };

      res.status(200).json(response);
    } catch (error) {
      await this.telemetry.emitError(
        this.config.agentId,
        error instanceof Error ? error : new Error(String(error)),
        spanContext
      );
      throw error;
    }
  }

  /**
   * Perform the actual ROI analysis
   */
  private async performROIAnalysis(
    input: ROIInput,
    spanContext: SpanContext
  ): Promise<ROIOutput> {
    const analysisStart = Date.now();

    // Prepare data for correlation analysis
    const costRecords = input.cost_records || [];
    const outcomeMetrics = input.outcome_metrics || [];

    // Calculate correlations
    let correlations = [];
    if (costRecords.length > 0 && outcomeMetrics.length > 0) {
      const correlationData = metricsCorrelator.prepareCorrelationData(
        costRecords,
        outcomeMetrics,
        24 // Default window hours
      );

      correlations = metricsCorrelator.calculateMultipleCorrelations(
        correlationData.costTimeSeries,
        correlationData.outcomeTimeSeries,
        {
          method: input.correlation_method,
          minThreshold: input.min_correlation_threshold,
          significanceLevel: 0.05
        }
      );
    }

    // Calculate total cost
    let totalCost: string;
    if (input.cost_aggregation) {
      totalCost = input.cost_aggregation.total_cost;
    } else if (costRecords.length > 0) {
      totalCost = costRecords
        .reduce((sum, r) => sum + parseFloat(r.cost_amount), 0)
        .toFixed(6);
    } else {
      totalCost = '0';
    }

    // Estimate total gain from outcome metrics
    let totalGain = '0';
    if (input.business_value_per_unit && outcomeMetrics.length > 0) {
      const valuePerUnit = parseFloat(input.business_value_per_unit);
      const totalOutcome = outcomeMetrics.reduce((sum, m) => sum + m.value, 0);
      totalGain = (totalOutcome * valuePerUnit).toFixed(6);
    } else if (outcomeMetrics.length > 0) {
      // Use a heuristic: sum of outcome values as gain indicator
      totalGain = outcomeMetrics
        .filter(m => ['revenue_impact', 'cost_savings', 'productivity_gain'].includes(m.metric_type))
        .reduce((sum, m) => sum + m.value, 0)
        .toFixed(6);
    }

    // Calculate period in days
    const periodStart = new Date(input.period_start);
    const periodEnd = new Date(input.period_end);
    const periodDays = Math.max(1, (periodEnd.getTime() - periodStart.getTime()) / (1000 * 60 * 60 * 24));

    // Calculate ROI metrics
    const roiMetrics = roiCalculator.calculateROI({
      totalCost,
      totalGain,
      periodDays,
      baselineCost: input.baseline_cost,
      baselineGain: input.baseline_outcome?.toString()
    });

    // Calculate efficiency metrics
    const efficiencyMetrics = roiCalculator.calculateEfficiency(
      input.cost_aggregation || costRecords,
      input.outcome_aggregations || outcomeMetrics
    );

    // Generate recommendations
    const recommendations = roiCalculator.generateRecommendations(
      roiMetrics,
      correlations,
      efficiencyMetrics
    );

    // Calculate data quality score
    const dataQualityScore = roiCalculator.calculateDataQualityScore(
      costRecords.length,
      outcomeMetrics.length,
      correlations
    );

    // Calculate overall assessment
    const assessment = roiCalculator.calculateOverallAssessment(
      roiMetrics,
      correlations,
      dataQualityScore
    );

    // Find primary correlation (strongest significant correlation)
    const significantCorrelations = correlations.filter(c => c.is_significant);
    const primaryCorrelation = significantCorrelations.length > 0
      ? significantCorrelations.reduce((best, current) =>
          Math.abs(current.correlation_coefficient) > Math.abs(best.correlation_coefficient)
            ? current
            : best
        )
      : undefined;

    const processingTimeMs = Date.now() - analysisStart;

    return {
      roi_metrics: roiMetrics,
      correlations,
      primary_correlation: primaryCorrelation,
      efficiency_metrics: efficiencyMetrics,
      recommendations,
      summary: {
        overall_assessment: assessment.overall_assessment,
        key_insight: assessment.key_insight,
        confidence_level: assessment.confidence_level,
        data_quality_score: dataQualityScore
      },
      metadata: {
        analyzed_at: new Date().toISOString(),
        analysis_scope: input.analysis_scope,
        scope_id: input.scope_id,
        period_start: input.period_start,
        period_end: input.period_end,
        cost_records_analyzed: costRecords.length + (input.cost_aggregation ? 1 : 0),
        outcome_metrics_analyzed: outcomeMetrics.length + (input.outcome_aggregations?.length || 0),
        processing_time_ms: processingTimeMs,
        calculation_method: input.roi_calculation_method
      }
    };
  }

  /**
   * Create DecisionEvent for persistence
   */
  private createDecisionEvent(
    input: ROIInput,
    output: ROIOutput,
    processingDurationMs: number,
    spanContext: SpanContext
  ): DecisionEvent {
    // Calculate confidence based on data quality and correlation significance
    const statisticalSignificance = output.correlations.some(c => c.is_significant);
    const confidence = Math.min(
      1,
      output.summary.data_quality_score * 0.6 +
      (statisticalSignificance ? 0.3 : 0) +
      (output.correlations.length > 0 ? 0.1 : 0)
    );

    // Determine constraints applied
    const constraintsApplied: string[] = [];
    if (input.min_correlation_threshold !== 0.3) {
      constraintsApplied.push(`correlation_threshold:${input.min_correlation_threshold}`);
    }
    if (input.baseline_cost) {
      constraintsApplied.push(`baseline_cost:${input.baseline_cost}`);
    }
    if (input.business_value_per_unit) {
      constraintsApplied.push(`business_value_per_unit:${input.business_value_per_unit}`);
    }

    // Calculate input hash for determinism verification
    const inputsHash = this.hashInputs(input);

    return {
      agent_id: this.config.agentId,
      agent_version: this.config.agentVersion,
      decision_type: 'roi_estimation',
      inputs_hash: inputsHash,
      outputs: output,
      confidence,
      constraints_applied: constraintsApplied,
      execution_ref: input.request_id,
      timestamp: new Date().toISOString(),
      metadata: {
        request_id: input.request_id,
        processing_duration_ms: processingDurationMs,
        input_validation_passed: true,
        data_completeness_score: output.summary.data_quality_score,
        statistical_significance: statisticalSignificance,
        correlation_method_used: input.correlation_method
      }
    };
  }

  /**
   * Handle /health endpoint
   */
  private async handleHealthCheck(req: Request, res: Response): Promise<void> {
    const [ruvectorStatus, telemetryStatus] = await Promise.all([
      this.ruvectorClient.getConnectionStatus(),
      this.telemetry.getConnectionStatus()
    ]);

    const isHealthy =
      ruvectorStatus === 'connected' || ruvectorStatus === 'unknown';

    const response: HealthCheckResponse = {
      status: isHealthy ? 'healthy' : 'degraded',
      version: AGENT_METADATA.version,
      timestamp: new Date().toISOString(),
      dependencies: {
        ruvector_service: ruvectorStatus,
        telemetry: telemetryStatus
      }
    };

    res.status(isHealthy ? 200 : 503).json(response);
  }

  /**
   * Handle /inspect endpoint - inspect a previous DecisionEvent
   */
  private async handleInspect(req: Request, res: Response): Promise<void> {
    const decisionEventId = req.query.id as string;

    if (!decisionEventId) {
      res.status(400).json({
        success: false,
        error: {
          code: 'INVALID_INPUT',
          message: 'Missing required query parameter: id',
          timestamp: new Date().toISOString()
        }
      });
      return;
    }

    const events = await this.ruvectorClient.queryDecisionEvents({
      executionRef: decisionEventId,
      limit: 1
    });

    if (events.length === 0) {
      res.status(404).json({
        success: false,
        error: {
          code: 'INVALID_INPUT',
          message: `DecisionEvent not found: ${decisionEventId}`,
          timestamp: new Date().toISOString()
        }
      });
      return;
    }

    res.status(200).json({
      success: true,
      data: events[0]
    });
  }

  /**
   * Handle errors
   */
  private async handleError(
    error: unknown,
    res: Response,
    spanContext?: SpanContext
  ): Promise<void> {
    const errorMessage = error instanceof Error ? error.message : String(error);
    const errorCode = this.classifyError(error);

    if (spanContext) {
      await this.telemetry.emitError(
        this.config.agentId,
        error instanceof Error ? error : new Error(errorMessage),
        spanContext
      );
    }

    const response: AnalyzeROIResponse = {
      success: false,
      error: {
        code: errorCode,
        message: errorMessage,
        timestamp: new Date().toISOString()
      }
    };

    const statusCode = this.getStatusCode(errorCode);
    res.status(statusCode).json(response);
  }

  /**
   * Classify error type
   */
  private classifyError(error: unknown): 'INVALID_INPUT' | 'INSUFFICIENT_DATA' | 'CORRELATION_FAILED' | 'CALCULATION_ERROR' | 'PERSISTENCE_ERROR' | 'TIMEOUT' | 'INTERNAL_ERROR' {
    const message = error instanceof Error ? error.message.toLowerCase() : '';

    if (message.includes('validation') || message.includes('invalid')) {
      return 'INVALID_INPUT';
    }
    if (message.includes('insufficient') || message.includes('not enough')) {
      return 'INSUFFICIENT_DATA';
    }
    if (message.includes('correlation')) {
      return 'CORRELATION_FAILED';
    }
    if (message.includes('calculation') || message.includes('compute')) {
      return 'CALCULATION_ERROR';
    }
    if (message.includes('persist') || message.includes('storage') || message.includes('ruvector')) {
      return 'PERSISTENCE_ERROR';
    }
    if (message.includes('timeout') || message.includes('aborted')) {
      return 'TIMEOUT';
    }

    return 'INTERNAL_ERROR';
  }

  /**
   * Get HTTP status code for error type
   */
  private getStatusCode(errorCode: string): number {
    switch (errorCode) {
      case 'INVALID_INPUT':
        return 400;
      case 'INSUFFICIENT_DATA':
        return 422;
      case 'TIMEOUT':
        return 408;
      case 'PERSISTENCE_ERROR':
        return 503;
      default:
        return 500;
    }
  }

  /**
   * Hash inputs for determinism verification
   */
  private hashInputs(input: ROIInput): string {
    const normalized = JSON.stringify(input, Object.keys(input).sort());
    return createHash('sha256').update(normalized).digest('hex').substring(0, 16);
  }

  /**
   * Shutdown handler
   */
  async shutdown(): Promise<void> {
    await this.telemetry.shutdown();
  }
}

// ============================================================================
// GOOGLE CLOUD FUNCTIONS ENTRY POINT
// ============================================================================

let handlerInstance: ROIEstimationHandler | null = null;

/**
 * Get or create handler instance (singleton for function warm starts)
 */
function getHandler(): ROIEstimationHandler {
  if (!handlerInstance) {
    const config: ROIAgentConfig = {
      ...DEFAULT_AGENT_CONFIG,
      agentId: process.env.AGENT_ID || DEFAULT_AGENT_CONFIG.agentId,
      agentVersion: process.env.AGENT_VERSION || DEFAULT_AGENT_CONFIG.agentVersion,
      ruvectorService: {
        ...DEFAULT_AGENT_CONFIG.ruvectorService,
        baseUrl: process.env.RUVECTOR_SERVICE_URL || DEFAULT_AGENT_CONFIG.ruvectorService.baseUrl
      },
      telemetry: {
        ...DEFAULT_AGENT_CONFIG.telemetry,
        endpoint: process.env.TELEMETRY_ENDPOINT || DEFAULT_AGENT_CONFIG.telemetry.endpoint
      }
    };
    handlerInstance = new ROIEstimationHandler(config);
  }
  return handlerInstance;
}

/**
 * Google Cloud Functions HTTP entry point
 */
export async function roiEstimationAgent(
  req: Request,
  res: Response
): Promise<void> {
  const handler = getHandler();
  await handler.handleRequest(req, res);
}

// Export for testing
export { getHandler };
