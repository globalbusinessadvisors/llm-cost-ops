/**
 * Cost Attribution Agent Contracts
 *
 * Exports all schemas, types, and interfaces for the cost attribution agent.
 * Following the agentics-contracts pattern for type-safe inter-agent communication.
 */

// Export all schemas
export {
  ProviderSchema,
  UsageRecordSchema,
  CostRecordSchema,
  AttributionScopeSchema,
  PricingContextSchema,
  CostAttributionInputSchema,
  AttributionResultSchema,
  AttributionSummarySchema,
  CostAttributionOutputSchema,
  DecisionEventSchema,
  ErrorResponseSchema,
  HealthCheckResponseSchema
} from './schemas.js';

// Export all types
export type {
  Provider,
  UsageRecord,
  CostRecord,
  AttributionScope,
  PricingContext,
  CostAttributionInput,
  AttributionResult,
  AttributionSummary,
  CostAttributionOutput,
  DecisionEvent,
  ErrorResponse,
  HealthCheckResponse,
  AgentConfig,
  PricingLookup,
  StorageAdapter,
  UsageRecordFilter,
  CostRecordFilter
} from './types.js';

// Export version
export const CONTRACT_VERSION = '1.0.0';
