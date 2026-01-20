/**
 * ROI Estimation Agent - TypeScript Types
 *
 * Types derived from Zod schemas + additional interfaces
 * Classification: ROI ANALYSIS
 */

import { z } from 'zod';
import {
  ProviderSchema,
  CurrencySchema,
  CostRecordSchema,
  CostAggregationSchema,
  OutcomeMetricTypeSchema,
  OutcomeMetricSchema,
  OutcomeAggregationSchema,
  ROIAnalysisScopeSchema,
  ROIInputSchema,
  CorrelationResultSchema,
  ROIMetricSchema,
  EfficiencyMetricSchema,
  ROIRecommendationSchema,
  ROIOutputSchema,
  DecisionEventSchema,
  ROIErrorCodeSchema,
  ROIErrorSchema,
  AnalyzeROIRequestSchema,
  AnalyzeROIResponseSchema,
  HealthCheckResponseSchema
} from './schemas.js';

// ============================================================================
// TYPE EXPORTS (Inferred from Zod schemas)
// ============================================================================

export type Provider = z.infer<typeof ProviderSchema>;
export type Currency = z.infer<typeof CurrencySchema>;
export type CostRecord = z.infer<typeof CostRecordSchema>;
export type CostAggregation = z.infer<typeof CostAggregationSchema>;
export type OutcomeMetricType = z.infer<typeof OutcomeMetricTypeSchema>;
export type OutcomeMetric = z.infer<typeof OutcomeMetricSchema>;
export type OutcomeAggregation = z.infer<typeof OutcomeAggregationSchema>;
export type ROIAnalysisScope = z.infer<typeof ROIAnalysisScopeSchema>;
export type ROIInput = z.infer<typeof ROIInputSchema>;
export type CorrelationResult = z.infer<typeof CorrelationResultSchema>;
export type ROIMetric = z.infer<typeof ROIMetricSchema>;
export type EfficiencyMetric = z.infer<typeof EfficiencyMetricSchema>;
export type ROIRecommendation = z.infer<typeof ROIRecommendationSchema>;
export type ROIOutput = z.infer<typeof ROIOutputSchema>;
export type DecisionEvent = z.infer<typeof DecisionEventSchema>;
export type ROIErrorCode = z.infer<typeof ROIErrorCodeSchema>;
export type ROIError = z.infer<typeof ROIErrorSchema>;
export type AnalyzeROIRequest = z.infer<typeof AnalyzeROIRequestSchema>;
export type AnalyzeROIResponse = z.infer<typeof AnalyzeROIResponseSchema>;
export type HealthCheckResponse = z.infer<typeof HealthCheckResponseSchema>;

// ============================================================================
// AGENT CONFIGURATION
// ============================================================================

export interface ROIAgentConfig {
  /** Unique agent identifier */
  agentId: string;

  /** Semantic version of the agent */
  agentVersion: string;

  /** Enable DecisionEvent logging to ruvector-service */
  enableDecisionLogging: boolean;

  /** Default currency for calculations */
  defaultCurrency: Currency;

  /** Validation strictness level */
  validationLevel: 'strict' | 'moderate' | 'permissive';

  /** Minimum data points required for correlation */
  minDataPointsForCorrelation: number;

  /** P-value threshold for statistical significance */
  significanceThreshold: number;

  /** Maximum processing time before timeout (ms) */
  maxProcessingTimeMs: number;

  /** RuVector service configuration */
  ruvectorService: {
    baseUrl: string;
    timeoutMs: number;
    retryAttempts: number;
    retryDelayMs: number;
  };

  /** Telemetry configuration */
  telemetry: {
    endpoint: string;
    batchSize: number;
    flushIntervalMs: number;
  };
}

export const DEFAULT_AGENT_CONFIG: ROIAgentConfig = {
  agentId: 'roi-estimation-agent',
  agentVersion: '1.0.0',
  enableDecisionLogging: true,
  defaultCurrency: 'USD',
  validationLevel: 'strict',
  minDataPointsForCorrelation: 5,
  significanceThreshold: 0.05,
  maxProcessingTimeMs: 30000,
  ruvectorService: {
    baseUrl: process.env.RUVECTOR_SERVICE_URL || 'http://localhost:8080',
    timeoutMs: 10000,
    retryAttempts: 3,
    retryDelayMs: 1000
  },
  telemetry: {
    endpoint: process.env.TELEMETRY_ENDPOINT || 'http://localhost:4317',
    batchSize: 100,
    flushIntervalMs: 10000
  }
};

// ============================================================================
// STORAGE ADAPTER INTERFACE
// ============================================================================

export interface DecisionEventFilters {
  agentId?: string;
  decisionType?: string;
  startTime?: string;
  endTime?: string;
  executionRef?: string;
  limit?: number;
  offset?: number;
}

export interface StorageAdapter {
  /**
   * Persist a DecisionEvent to ruvector-service
   * MUST NOT access SQL directly
   */
  saveDecisionEvent(event: DecisionEvent): Promise<void>;

  /**
   * Query DecisionEvents from ruvector-service
   * MUST NOT access SQL directly
   */
  queryDecisionEvents(filters: DecisionEventFilters): Promise<DecisionEvent[]>;

  /**
   * Health check for storage connectivity
   */
  healthCheck(): Promise<boolean>;
}

// ============================================================================
// METRICS ADAPTER INTERFACE
// ============================================================================

export interface CostMetricsQuery {
  organization_id?: string;
  project_id?: string;
  workflow_id?: string;
  agent_id?: string;
  provider?: Provider;
  model?: string;
  start_time: string;
  end_time: string;
  aggregation?: 'hourly' | 'daily' | 'weekly' | 'monthly';
}

export interface OutcomeMetricsQuery {
  organization_id?: string;
  project_id?: string;
  workflow_id?: string;
  agent_id?: string;
  metric_type?: OutcomeMetricType;
  metric_name?: string;
  start_time: string;
  end_time: string;
  aggregation?: 'hourly' | 'daily' | 'weekly' | 'monthly';
}

export interface MetricsAdapter {
  /**
   * Fetch cost records for ROI analysis
   */
  getCostRecords(query: CostMetricsQuery): Promise<CostRecord[]>;

  /**
   * Fetch cost aggregation for ROI analysis
   */
  getCostAggregation(query: CostMetricsQuery): Promise<CostAggregation>;

  /**
   * Fetch outcome metrics for ROI analysis
   */
  getOutcomeMetrics(query: OutcomeMetricsQuery): Promise<OutcomeMetric[]>;

  /**
   * Fetch outcome aggregations for ROI analysis
   */
  getOutcomeAggregations(query: OutcomeMetricsQuery): Promise<OutcomeAggregation[]>;
}

// ============================================================================
// TELEMETRY INTERFACES
// ============================================================================

export interface SpanContext {
  traceId: string;
  spanId: string;
  parentSpanId?: string;
}

export interface TelemetryEvent {
  event_type: 'span' | 'metric' | 'log';
  timestamp: string;
  agent_id: string;
  span_context?: SpanContext;
  name: string;
  duration_ms?: number;
  attributes: Record<string, unknown>;
  status?: 'ok' | 'error';
  error_message?: string;
}

export interface TelemetryEmitter {
  /**
   * Emit a telemetry event to LLM-Observatory
   */
  emit(event: TelemetryEvent): Promise<void>;

  /**
   * Emit a span for ROI analysis operation
   */
  emitSpan(
    name: string,
    agentId: string,
    durationMs: number,
    context: SpanContext,
    attributes?: Record<string, unknown>
  ): Promise<void>;

  /**
   * Emit an error event
   */
  emitError(
    agentId: string,
    error: Error,
    context?: SpanContext
  ): Promise<void>;

  /**
   * Flush pending telemetry events
   */
  flush(): Promise<void>;
}

// ============================================================================
// ROI CALCULATION INTERFACES
// ============================================================================

export interface CorrelationInput {
  costValues: number[];
  outcomeValues: number[];
  timestamps: string[];
}

export interface CorrelationConfig {
  method: 'pearson' | 'spearman' | 'kendall';
  minThreshold: number;
  significanceLevel: number;
}

export interface ROICalculationInput {
  totalCost: string;
  totalGain: string;
  periodDays: number;
  baselineCost?: string;
  baselineGain?: string;
}

export interface ROICalculator {
  /**
   * Calculate correlation between cost and outcome metrics
   */
  calculateCorrelation(
    input: CorrelationInput,
    config: CorrelationConfig
  ): CorrelationResult;

  /**
   * Calculate ROI metrics
   */
  calculateROI(input: ROICalculationInput): ROIMetric;

  /**
   * Calculate efficiency metrics
   */
  calculateEfficiency(
    costs: CostRecord[] | CostAggregation,
    outcomes: OutcomeMetric[] | OutcomeAggregation[]
  ): EfficiencyMetric[];

  /**
   * Generate recommendations based on ROI analysis
   */
  generateRecommendations(
    roiMetrics: ROIMetric,
    correlations: CorrelationResult[],
    efficiencyMetrics: EfficiencyMetric[]
  ): ROIRecommendation[];
}

// ============================================================================
// AGENT BOUNDARIES (EXPLICIT NON-RESPONSIBILITIES)
// ============================================================================

/**
 * ROI Estimation Agent MUST NOT:
 *
 * 1. INTERCEPT runtime execution
 *    - This is NOT an execution layer agent
 *    - NEVER modify request/response flow
 *
 * 2. TRIGGER retries or recovery actions
 *    - Analysis only, no execution control
 *
 * 3. EXECUTE workflows
 *    - Only analyze historical data
 *
 * 4. MODIFY routing or execution behavior
 *    - Emit signals only, never enforce
 *
 * 5. APPLY optimizations automatically
 *    - Recommend only, never apply
 *
 * 6. ENFORCE policies directly
 *    - Emit advisories, not enforcement
 *
 * 7. CONNECT directly to Google SQL
 *    - All persistence via ruvector-service
 *
 * 8. EXECUTE SQL queries
 *    - Use ruvector-service client only
 *
 * 9. INVOKE other CostOps agents
 *    - Analysis-only, no agent chaining
 */
export interface AgentBoundaries {
  readonly allowedOperations: readonly [
    'analyze_cost_data',
    'correlate_metrics',
    'calculate_roi',
    'emit_advisory_signals',
    'persist_decision_events',
    'emit_telemetry'
  ];

  readonly forbiddenOperations: readonly [
    'intercept_execution',
    'trigger_retries',
    'execute_workflows',
    'modify_routing',
    'apply_optimizations',
    'enforce_policies',
    'direct_sql_access',
    'invoke_other_agents'
  ];
}

// ============================================================================
// CLI INTERFACES
// ============================================================================

export interface CLIAnalyzeCommand {
  command: 'analyze';
  options: {
    scope: ROIAnalysisScope;
    scopeId?: string;
    startDate: string;
    endDate: string;
    correlationMethod?: 'pearson' | 'spearman' | 'kendall';
    calculationMethod?: 'simple' | 'net_present_value' | 'payback_period' | 'cost_efficiency';
    outputFormat?: 'json' | 'table' | 'summary';
    verbose?: boolean;
  };
}

export interface CLIInspectCommand {
  command: 'inspect';
  options: {
    decisionEventId: string;
    outputFormat?: 'json' | 'table';
  };
}

export interface CLIHistoryCommand {
  command: 'history';
  options: {
    scope?: ROIAnalysisScope;
    scopeId?: string;
    startDate?: string;
    endDate?: string;
    limit?: number;
    outputFormat?: 'json' | 'table';
  };
}

export type CLICommand = CLIAnalyzeCommand | CLIInspectCommand | CLIHistoryCommand;

// ============================================================================
// VERSION & METADATA
// ============================================================================

export const AGENT_METADATA = {
  name: 'roi-estimation-agent',
  version: '1.0.0',
  classification: 'ROI_ANALYSIS' as const,
  decisionType: 'roi_estimation' as const,
  description: 'Estimate return on investment by correlating cost with business or system outcomes',
  contractVersion: '1.0.0',
  capabilities: [
    'cost_outcome_correlation',
    'roi_calculation',
    'efficiency_analysis',
    'recommendation_generation'
  ],
  dependencies: [
    'ruvector-service',
    'llm-observatory'
  ]
} as const;
