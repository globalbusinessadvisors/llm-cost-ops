/**
 * ROI Estimation Agent - Contracts Barrel Export
 */

// Schema exports
export {
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
  DecisionTypeSchema,
  DecisionEventSchema,
  ROIErrorCodeSchema,
  ROIErrorSchema,
  AnalyzeROIRequestSchema,
  AnalyzeROIResponseSchema,
  HealthCheckResponseSchema
} from './schemas.js';

// Type exports
export type {
  Provider,
  Currency,
  CostRecord,
  CostAggregation,
  OutcomeMetricType,
  OutcomeMetric,
  OutcomeAggregation,
  ROIAnalysisScope,
  ROIInput,
  CorrelationResult,
  ROIMetric,
  EfficiencyMetric,
  ROIRecommendation,
  ROIOutput,
  DecisionEvent,
  ROIErrorCode,
  ROIError,
  AnalyzeROIRequest,
  AnalyzeROIResponse,
  HealthCheckResponse,
  ROIAgentConfig,
  DecisionEventFilters,
  StorageAdapter,
  CostMetricsQuery,
  OutcomeMetricsQuery,
  MetricsAdapter,
  SpanContext,
  TelemetryEvent,
  TelemetryEmitter,
  CorrelationInput,
  CorrelationConfig,
  ROICalculationInput,
  ROICalculator,
  AgentBoundaries,
  CLIAnalyzeCommand,
  CLIInspectCommand,
  CLIHistoryCommand,
  CLICommand
} from './types.js';

export { DEFAULT_AGENT_CONFIG, AGENT_METADATA } from './types.js';
