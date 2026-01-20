/**
 * Services module - Export all service clients
 *
 * CONSTITUTION COMPLIANCE:
 * - All persistence via HTTP to ruvector-service
 * - No local SQL execution
 * - Async, non-blocking operations
 */

export {
  RuvectorServiceClient,
  type DecisionEvent,
  type DecisionEventFilters,
  type RuvectorConfig,
  type RetryConfig,
} from './ruvector-client';

export {
  TelemetryEmitter,
  type TelemetryEvent,
  type TelemetryMetrics,
  type TokenCounts,
  type CostMetrics,
  type SpanContext,
  type TelemetryConfig,
} from './telemetry';
