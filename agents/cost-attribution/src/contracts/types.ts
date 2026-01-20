import { z } from 'zod';
import {
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

/**
 * Provider type
 */
export type Provider = z.infer<typeof ProviderSchema>;

/**
 * Usage record type
 */
export type UsageRecord = z.infer<typeof UsageRecordSchema>;

/**
 * Cost record type
 */
export type CostRecord = z.infer<typeof CostRecordSchema>;

/**
 * Attribution scope type
 */
export type AttributionScope = z.infer<typeof AttributionScopeSchema>;

/**
 * Pricing context type
 */
export type PricingContext = z.infer<typeof PricingContextSchema>;

/**
 * Cost attribution input type
 */
export type CostAttributionInput = z.infer<typeof CostAttributionInputSchema>;

/**
 * Individual attribution result type
 */
export type AttributionResult = z.infer<typeof AttributionResultSchema>;

/**
 * Attribution summary type
 */
export type AttributionSummary = z.infer<typeof AttributionSummarySchema>;

/**
 * Cost attribution output type
 */
export type CostAttributionOutput = z.infer<typeof CostAttributionOutputSchema>;

/**
 * Decision event type
 */
export type DecisionEvent = z.infer<typeof DecisionEventSchema>;

/**
 * Error response type
 */
export type ErrorResponse = z.infer<typeof ErrorResponseSchema>;

/**
 * Health check response type
 */
export type HealthCheckResponse = z.infer<typeof HealthCheckResponseSchema>;

/**
 * Agent configuration type
 */
export interface AgentConfig {
  agentId: string;
  agentVersion: string;
  enableDecisionLogging: boolean;
  defaultCurrency: string;
  maxBatchSize: number;
  validationLevel: 'strict' | 'moderate' | 'permissive';
}

/**
 * Pricing lookup interface
 */
export interface PricingLookup {
  getProviderPricing(provider: Provider, model: string): Promise<PricingContext | null>;
  updatePricing(provider: Provider, model: string, pricing: PricingContext): Promise<void>;
}

/**
 * Storage interface for persistence
 */
export interface StorageAdapter {
  saveUsageRecord(record: UsageRecord): Promise<void>;
  saveCostRecord(record: CostRecord): Promise<void>;
  saveDecisionEvent(event: DecisionEvent): Promise<void>;
  queryUsageRecords(filter: UsageRecordFilter): Promise<UsageRecord[]>;
  queryCostRecords(filter: CostRecordFilter): Promise<CostRecord[]>;
}

/**
 * Usage record filter
 */
export interface UsageRecordFilter {
  provider?: Provider;
  model?: string;
  organizationId?: string;
  projectId?: string;
  userId?: string;
  startTime?: string;
  endTime?: string;
  tags?: Record<string, string>;
}

/**
 * Cost record filter
 */
export interface CostRecordFilter {
  provider?: Provider;
  model?: string;
  startTime?: string;
  endTime?: string;
  minCost?: number;
  maxCost?: number;
}
