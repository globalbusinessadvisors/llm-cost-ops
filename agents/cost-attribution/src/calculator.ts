/**
 * Cost Calculator
 *
 * Calculates costs based on usage data and pricing models
 */

import { UsageData, PricingContext, CostBreakdown } from './types';

/**
 * Default pricing models per provider/model
 * Prices are per million tokens (converted to per-token internally)
 */
const DEFAULT_PRICING: Record<
  string,
  Record<string, { input: number; output: number; cached?: number }>
> = {
  openai: {
    'gpt-4': { input: 30.0, output: 60.0 },
    'gpt-4-turbo': { input: 10.0, output: 30.0 },
    'gpt-3.5-turbo': { input: 0.5, output: 1.5 },
    'gpt-4o': { input: 5.0, output: 15.0 },
    'gpt-4o-mini': { input: 0.15, output: 0.6 },
  },
  anthropic: {
    'claude-opus-4': { input: 15.0, output: 75.0, cached: 1.5 },
    'claude-sonnet-4': { input: 3.0, output: 15.0, cached: 0.3 },
    'claude-haiku-4': { input: 0.8, output: 4.0, cached: 0.08 },
    'claude-3-opus': { input: 15.0, output: 75.0, cached: 1.5 },
    'claude-3-sonnet': { input: 3.0, output: 15.0, cached: 0.3 },
    'claude-3-haiku': { input: 0.25, output: 1.25, cached: 0.025 },
  },
  google: {
    'gemini-pro': { input: 0.5, output: 1.5 },
    'gemini-ultra': { input: 10.0, output: 30.0 },
    'gemini-flash': { input: 0.075, output: 0.3 },
  },
};

export class CostCalculator {
  /**
   * Calculate cost breakdown for given usage
   */
  calculate(
    usage: UsageData,
    pricingContext?: PricingContext
  ): CostBreakdown {
    const currency = pricingContext?.currency || 'USD';

    // Get pricing rates (per million tokens)
    const pricing = this.getPricing(usage, pricingContext);

    // Convert to per-token pricing
    const inputPricePerToken = pricing.input / 1_000_000;
    const outputPricePerToken = pricing.output / 1_000_000;
    const cachedPricePerToken = pricing.cached ? pricing.cached / 1_000_000 : 0;

    // Calculate costs
    const inputCost = usage.inputTokens * inputPricePerToken;
    const outputCost = usage.outputTokens * outputPricePerToken;
    const cachedCost = usage.cachedTokens
      ? usage.cachedTokens * cachedPricePerToken
      : 0;

    const totalCost = inputCost + outputCost + cachedCost;
    const totalTokens =
      usage.inputTokens + usage.outputTokens + (usage.cachedTokens || 0);
    const costPer1kTokens = totalTokens > 0 ? (totalCost / totalTokens) * 1000 : 0;

    return {
      totalCost: this.round(totalCost),
      inputCost: this.round(inputCost),
      outputCost: this.round(outputCost),
      cachedCost: cachedCost > 0 ? this.round(cachedCost) : undefined,
      currency,
      costPer1kTokens: this.round(costPer1kTokens),
    };
  }

  /**
   * Get pricing for provider/model combination
   */
  private getPricing(
    usage: UsageData,
    pricingContext?: PricingContext
  ): { input: number; output: number; cached?: number } {
    // Use custom pricing if provided
    if (pricingContext?.customPricing) {
      const custom = pricingContext.customPricing;
      return {
        input: custom.inputPricePerToken ? custom.inputPricePerToken * 1_000_000 : 0,
        output: custom.outputPricePerToken
          ? custom.outputPricePerToken * 1_000_000
          : 0,
        cached: custom.cachedPricePerToken
          ? custom.cachedPricePerToken * 1_000_000
          : undefined,
      };
    }

    // Lookup default pricing
    const provider = usage.provider.toLowerCase();
    const model = usage.model.toLowerCase();

    const providerPricing = DEFAULT_PRICING[provider];
    if (!providerPricing) {
      throw new Error(`Unknown provider: ${usage.provider}`);
    }

    // Try exact match first
    let modelPricing = providerPricing[model];

    // If no exact match, try prefix matching
    if (!modelPricing) {
      const matchingModel = Object.keys(providerPricing).find((key) =>
        model.startsWith(key)
      );
      if (matchingModel) {
        modelPricing = providerPricing[matchingModel];
      }
    }

    if (!modelPricing) {
      throw new Error(`Unknown model: ${usage.model} for provider ${usage.provider}`);
    }

    return modelPricing;
  }

  /**
   * Round to 6 decimal places for currency precision
   */
  private round(value: number): number {
    return Math.round(value * 1_000_000) / 1_000_000;
  }
}
