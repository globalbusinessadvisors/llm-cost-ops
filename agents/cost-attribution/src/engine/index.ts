/**
 * Cost Attribution Engine
 *
 * Pure, stateless, deterministic cost calculation and attribution engine.
 *
 * Core Components:
 * - CostCalculator: Computes costs from usage records using pricing tables
 * - CostAttributor: Attributes costs to different scopes (execution/agent/workflow/tenant)
 * - TokenNormalizer: Standardizes token counts across different LLM providers
 *
 * Design Principles:
 * - Stateless: All functions are pure with no side effects
 * - Deterministic: Same inputs always produce same outputs
 * - Precise: Uses decimal.js for currency calculations (no floating-point errors)
 * - Provider-agnostic: Supports multiple LLM providers with normalization
 */

// Calculator exports
export {
  CostCalculator,
  type UsageRecord,
  type CostRecord,
  type PricingTable,
  type PricingTier,
  PricingType,
  Currency,
} from './calculator';

// Attributor exports
export {
  CostAttributor,
  type Attribution,
  type ExecutionAttribution,
  type AgentAttribution,
  type WorkflowAttribution,
  type TenantAttribution,
  type AttributionSummary,
  type ProviderBreakdown,
  type ModelBreakdown,
  type AgentBreakdown,
  type WorkflowBreakdown,
} from './attributor';

// Normalizer exports
export {
  TokenNormalizer,
  type NormalizedUsage,
  type TokenCountingConfig,
} from './normalizer';

/**
 * Convenience function to create a complete attribution pipeline
 */
import { CostCalculator, UsageRecord, PricingTable, CostRecord } from './calculator';
import { CostAttributor, AttributionSummary } from './attributor';
import { TokenNormalizer, NormalizedUsage, TokenCountingConfig } from './normalizer';

export interface AttributionPipelineOptions {
  pricingTable: PricingTable;
  normalizeTokens?: boolean;
  tokenConfigs?: TokenCountingConfig[];
}

export interface AttributionPipelineResult {
  costRecords: CostRecord[];
  normalizedUsages?: NormalizedUsage[];
  summary: AttributionSummary;
}

/**
 * Complete attribution pipeline: normalize -> calculate -> attribute
 */
export function runAttributionPipeline(
  usages: UsageRecord[],
  options: AttributionPipelineOptions
): AttributionPipelineResult {
  const calculator = new CostCalculator();
  const attributor = new CostAttributor();

  let costRecords: CostRecord[];
  let normalizedUsages: NormalizedUsage[] | undefined;

  // Step 1: Normalize tokens (optional)
  if (options.normalizeTokens) {
    const normalizer = new TokenNormalizer(options.tokenConfigs);
    normalizedUsages = normalizer.normalizeBatch(usages);

    // Convert normalized usages back to UsageRecords for cost calculation
    const normalizedUsageRecords: UsageRecord[] = normalizedUsages.map(nu => ({
      ...nu.originalUsage,
      inputTokens: nu.normalizedInputTokens,
      outputTokens: nu.normalizedOutputTokens,
      cachedInputTokens: nu.normalizedCachedInputTokens,
    }));

    costRecords = calculator.calculateBatchCosts(normalizedUsageRecords, options.pricingTable);
  } else {
    // Step 1 (skip normalization): Calculate costs directly
    costRecords = calculator.calculateBatchCosts(usages, options.pricingTable);
  }

  // Step 2: Attribute costs to all scopes
  const executionAttributions = attributor.attributeByExecution(costRecords);
  const agentAttributions = attributor.attributeByAgent(costRecords);
  const workflowAttributions = attributor.attributeByWorkflow(costRecords);
  const tenantAttributions = attributor.attributeByTenant(costRecords);

  // Step 3: Generate comprehensive summary
  const allAttributions = [
    ...executionAttributions,
    ...agentAttributions,
    ...workflowAttributions,
    ...tenantAttributions,
  ];

  const summary = attributor.generateSummary(allAttributions);

  return {
    costRecords,
    normalizedUsages,
    summary,
  };
}

/**
 * Utility: Create a pricing table for common providers
 */
export function createPricingTable(
  provider: 'anthropic' | 'openai' | 'google' | 'cohere',
  model: string,
  options?: {
    inputPricePerMillion?: string;
    outputPricePerMillion?: string;
    cachedInputPricePerMillion?: string;
    currency?: 'USD' | 'EUR' | 'GBP' | 'JPY';
  }
): PricingTable {
  const { Currency, PricingType } = require('./calculator');

  // Default pricing (placeholder - use real pricing in production)
  const defaults = {
    anthropic: {
      'claude-3-5-sonnet-20250219': {
        input: '3.00',
        output: '15.00',
        cached: '0.30',
      },
      'claude-3-5-haiku-20250219': {
        input: '0.80',
        output: '4.00',
        cached: '0.08',
      },
    },
    openai: {
      'gpt-4': {
        input: '30.00',
        output: '60.00',
      },
      'gpt-3.5-turbo': {
        input: '0.50',
        output: '1.50',
      },
    },
    google: {
      'gemini-pro': {
        input: '0.50',
        output: '1.50',
      },
    },
    cohere: {
      'command': {
        input: '1.00',
        output: '2.00',
      },
    },
  };

  const providerDefaults = defaults[provider]?.[model];
  if (!providerDefaults && !options) {
    throw new Error(`No default pricing for ${provider}:${model}. Please provide pricing options.`);
  }

  return {
    provider,
    model,
    type: PricingType.PER_TOKEN,
    currency: options?.currency || Currency.USD,
    inputTokenPrice: options?.inputPricePerMillion || providerDefaults?.input || '0',
    outputTokenPrice: options?.outputPricePerMillion || providerDefaults?.output || '0',
    cachedInputTokenPrice:
      options?.cachedInputPricePerMillion || providerDefaults?.cached || undefined,
    effectiveDate: new Date(),
  };
}

/**
 * Utility: Validate usage records before processing
 */
export function validateUsageRecords(usages: UsageRecord[]): {
  valid: boolean;
  errors: string[];
} {
  const errors: string[] = [];

  for (let i = 0; i < usages.length; i++) {
    const usage = usages[i];

    if (!usage.executionId) {
      errors.push(`Record ${i}: missing executionId`);
    }

    if (!usage.agentId) {
      errors.push(`Record ${i}: missing agentId`);
    }

    if (!usage.provider) {
      errors.push(`Record ${i}: missing provider`);
    }

    if (!usage.model) {
      errors.push(`Record ${i}: missing model`);
    }

    if (usage.inputTokens < 0) {
      errors.push(`Record ${i}: inputTokens cannot be negative`);
    }

    if (usage.outputTokens < 0) {
      errors.push(`Record ${i}: outputTokens cannot be negative`);
    }

    if (usage.cachedInputTokens && usage.cachedInputTokens < 0) {
      errors.push(`Record ${i}: cachedInputTokens cannot be negative`);
    }

    if (!usage.timestamp) {
      errors.push(`Record ${i}: missing timestamp`);
    }
  }

  return {
    valid: errors.length === 0,
    errors,
  };
}
