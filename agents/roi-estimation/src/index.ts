/**
 * ROI Estimation Agent
 *
 * LLM-CostOps Agent for estimating return on investment
 * by correlating cost with business or system outcomes.
 *
 * Classification: ROI ANALYSIS
 * Decision Type: roi_estimation
 *
 * @module @llm-costops/roi-estimation-agent
 */

// ============================================================================
// GOOGLE CLOUD FUNCTIONS ENTRY POINT
// ============================================================================

export { roiEstimationAgent } from './handlers/index.js';

// ============================================================================
// CONTRACT EXPORTS
// ============================================================================

// Schemas
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
} from './contracts/index.js';

// Types
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
} from './contracts/index.js';

export { DEFAULT_AGENT_CONFIG, AGENT_METADATA } from './contracts/index.js';

// ============================================================================
// ENGINE EXPORTS
// ============================================================================

export { ROICalculatorEngine, roiCalculator } from './engine/index.js';
export { MetricsCorrelator, metricsCorrelator } from './engine/index.js';

// ============================================================================
// SERVICE EXPORTS
// ============================================================================

export {
  RuvectorServiceClient,
  createRuvectorClient,
  TelemetryEmitter as TelemetryEmitterService,
  createTelemetryEmitter,
  createSpanContext
} from './services/index.js';

// ============================================================================
// HANDLER EXPORTS
// ============================================================================

export { ROIEstimationHandler, getHandler } from './handlers/index.js';

// ============================================================================
// CLI EXPORTS (for CLI integration)
// ============================================================================

/**
 * CLI invocation shape for ROI Estimation Agent
 *
 * Commands:
 * - analyze: Perform ROI analysis
 * - inspect: Inspect a previous DecisionEvent
 * - history: List historical ROI analyses
 *
 * Usage:
 *   npx @llm-costops/roi-estimation-agent analyze \
 *     --scope organization \
 *     --start-date 2024-01-01 \
 *     --end-date 2024-01-31 \
 *     --correlation-method pearson \
 *     --output-format json
 *
 *   npx @llm-costops/roi-estimation-agent inspect \
 *     --id <decision-event-uuid>
 *
 *   npx @llm-costops/roi-estimation-agent history \
 *     --scope organization \
 *     --limit 10
 */
export const CLI_SPEC = {
  name: 'roi-estimation-agent',
  version: AGENT_METADATA.version,
  description: AGENT_METADATA.description,
  commands: {
    analyze: {
      description: 'Perform ROI analysis on cost and outcome data',
      options: {
        scope: {
          type: 'string' as const,
          required: true,
          choices: ['organization', 'project', 'workflow', 'agent', 'model', 'provider', 'custom'],
          description: 'Analysis scope'
        },
        'scope-id': {
          type: 'string' as const,
          description: 'Scope identifier'
        },
        'start-date': {
          type: 'string' as const,
          required: true,
          description: 'Period start date (ISO 8601)'
        },
        'end-date': {
          type: 'string' as const,
          required: true,
          description: 'Period end date (ISO 8601)'
        },
        'correlation-method': {
          type: 'string' as const,
          choices: ['pearson', 'spearman', 'kendall'],
          default: 'pearson',
          description: 'Correlation method'
        },
        'calculation-method': {
          type: 'string' as const,
          choices: ['simple', 'net_present_value', 'payback_period', 'cost_efficiency'],
          default: 'simple',
          description: 'ROI calculation method'
        },
        'output-format': {
          type: 'string' as const,
          choices: ['json', 'table', 'summary'],
          default: 'json',
          description: 'Output format'
        },
        verbose: {
          type: 'boolean' as const,
          default: false,
          description: 'Verbose output'
        }
      }
    },
    inspect: {
      description: 'Inspect a previous DecisionEvent',
      options: {
        id: {
          type: 'string' as const,
          required: true,
          description: 'DecisionEvent ID'
        },
        'output-format': {
          type: 'string' as const,
          choices: ['json', 'table'],
          default: 'json',
          description: 'Output format'
        }
      }
    },
    history: {
      description: 'List historical ROI analyses',
      options: {
        scope: {
          type: 'string' as const,
          choices: ['organization', 'project', 'workflow', 'agent', 'model', 'provider', 'custom'],
          description: 'Filter by analysis scope'
        },
        'scope-id': {
          type: 'string' as const,
          description: 'Filter by scope identifier'
        },
        'start-date': {
          type: 'string' as const,
          description: 'Filter by start date (ISO 8601)'
        },
        'end-date': {
          type: 'string' as const,
          description: 'Filter by end date (ISO 8601)'
        },
        limit: {
          type: 'number' as const,
          default: 10,
          description: 'Maximum results to return'
        },
        'output-format': {
          type: 'string' as const,
          choices: ['json', 'table'],
          default: 'json',
          description: 'Output format'
        }
      }
    }
  }
} as const;

// ============================================================================
// AGENT BOUNDARY DOCUMENTATION
// ============================================================================

/**
 * ROI Estimation Agent - Explicit Non-Responsibilities
 *
 * Per LLM-CostOps Constitution, this agent MUST NOT:
 *
 * 1. INTERCEPT runtime execution
 *    - Analysis-only agent, not in execution path
 *
 * 2. TRIGGER retries or recovery actions
 *    - No execution control capabilities
 *
 * 3. EXECUTE workflows
 *    - Analyzes historical data only
 *
 * 4. MODIFY routing or execution behavior
 *    - Emits advisory signals, never enforces
 *
 * 5. APPLY optimizations automatically
 *    - Recommendations only, no auto-apply
 *
 * 6. ENFORCE policies directly
 *    - Advisory signals, not enforcement
 *
 * 7. CONNECT directly to Google SQL
 *    - All persistence via ruvector-service
 *
 * 8. EXECUTE SQL queries
 *    - HTTP client to ruvector-service only
 *
 * 9. INVOKE other CostOps agents
 *    - Analysis-only, no agent chaining
 *
 * This agent MAY:
 *
 * 1. Correlate cost signals with performance/outcome metrics
 * 2. Compute ROI and efficiency scores
 * 3. Emit ROI assessments and recommendations
 * 4. Persist DecisionEvents via ruvector-service
 * 5. Emit telemetry to LLM-Observatory
 */
export const AGENT_BOUNDARIES = {
  allowed: [
    'analyze_cost_data',
    'correlate_metrics',
    'calculate_roi',
    'emit_advisory_signals',
    'persist_decision_events',
    'emit_telemetry'
  ],
  forbidden: [
    'intercept_execution',
    'trigger_retries',
    'execute_workflows',
    'modify_routing',
    'apply_optimizations',
    'enforce_policies',
    'direct_sql_access',
    'invoke_other_agents'
  ]
} as const;
