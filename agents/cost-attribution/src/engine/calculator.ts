import Decimal from 'decimal.js';

export { Decimal };

/**
 * Pricing structure types supported by the calculator
 */
export enum PricingType {
  PER_TOKEN = 'per_token',
  PER_REQUEST = 'per_request',
  TIERED = 'tiered',
}

/**
 * Currency codes supported for multi-currency calculations
 */
export enum Currency {
  USD = 'USD',
  EUR = 'EUR',
  GBP = 'GBP',
  JPY = 'JPY',
}

/**
 * Pricing tier for tiered pricing models
 */
export interface PricingTier {
  minTokens: number;
  maxTokens: number | null; // null = infinity
  pricePerToken: string; // Decimal string for precision
}

/**
 * Pricing table structure
 */
export interface PricingTable {
  provider: string;
  model: string;
  type: PricingType;
  currency: Currency;

  // For PER_TOKEN pricing
  inputTokenPrice?: string; // Price per 1M tokens
  outputTokenPrice?: string;
  cachedInputTokenPrice?: string; // Discounted price for cached tokens

  // For PER_REQUEST pricing
  requestPrice?: string;

  // For TIERED pricing
  tiers?: PricingTier[];

  effectiveDate: Date;
}

/**
 * Usage record from telemetry
 */
export interface UsageRecord {
  executionId: string;
  agentId: string;
  workflowId?: string;
  tenantId?: string;

  provider: string;
  model: string;

  inputTokens: number;
  outputTokens: number;
  cachedInputTokens?: number;

  requestCount: number;
  timestamp: Date;

  metadata?: Record<string, unknown>;
}

/**
 * Computed cost record
 */
export interface CostRecord {
  executionId: string;
  agentId: string;
  workflowId?: string;
  tenantId?: string;

  provider: string;
  model: string;

  inputTokens: number;
  outputTokens: number;
  cachedInputTokens: number;

  inputTokenCost: string; // Decimal string
  outputTokenCost: string;
  cachedInputTokenCost: string;
  requestCost: string;

  totalCost: string;
  currency: Currency;

  timestamp: Date;
  calculatedAt: Date;
}

/**
 * CostCalculator - Pure, deterministic cost computation engine
 *
 * Uses decimal.js for precise currency calculations to avoid floating-point errors.
 * All monetary values are stored as decimal strings for precision.
 */
export class CostCalculator {
  /**
   * Calculate cost for a single usage record
   */
  calculateCost(usage: UsageRecord, pricing: PricingTable): CostRecord {
    this.validateInputs(usage, pricing);

    let inputTokenCost = new Decimal(0);
    let outputTokenCost = new Decimal(0);
    let cachedInputTokenCost = new Decimal(0);
    let requestCost = new Decimal(0);

    switch (pricing.type) {
      case PricingType.PER_TOKEN:
        ({ inputTokenCost, outputTokenCost, cachedInputTokenCost } =
          this.calculatePerTokenCosts(usage, pricing));
        break;

      case PricingType.PER_REQUEST:
        requestCost = this.calculateRequestCost(usage, pricing);
        break;

      case PricingType.TIERED:
        ({ inputTokenCost, outputTokenCost, cachedInputTokenCost } =
          this.calculateTieredCosts(usage, pricing));
        break;

      default:
        throw new Error(`Unsupported pricing type: ${pricing.type}`);
    }

    const totalCost = inputTokenCost
      .plus(outputTokenCost)
      .plus(cachedInputTokenCost)
      .plus(requestCost);

    return {
      executionId: usage.executionId,
      agentId: usage.agentId,
      workflowId: usage.workflowId,
      tenantId: usage.tenantId,

      provider: usage.provider,
      model: usage.model,

      inputTokens: usage.inputTokens,
      outputTokens: usage.outputTokens,
      cachedInputTokens: usage.cachedInputTokens || 0,

      inputTokenCost: inputTokenCost.toFixed(10),
      outputTokenCost: outputTokenCost.toFixed(10),
      cachedInputTokenCost: cachedInputTokenCost.toFixed(10),
      requestCost: requestCost.toFixed(10),

      totalCost: totalCost.toFixed(10),
      currency: pricing.currency,

      timestamp: usage.timestamp,
      calculatedAt: new Date(),
    };
  }

  /**
   * Calculate costs for multiple usage records (batch processing)
   */
  calculateBatchCosts(usages: UsageRecord[], pricing: PricingTable): CostRecord[] {
    return usages.map(usage => this.calculateCost(usage, pricing));
  }

  /**
   * Calculate per-token costs with cached token discount support
   */
  private calculatePerTokenCosts(
    usage: UsageRecord,
    pricing: PricingTable
  ): {
    inputTokenCost: Decimal;
    outputTokenCost: Decimal;
    cachedInputTokenCost: Decimal;
  } {
    if (!pricing.inputTokenPrice || !pricing.outputTokenPrice) {
      throw new Error('Per-token pricing requires inputTokenPrice and outputTokenPrice');
    }

    const inputPrice = new Decimal(pricing.inputTokenPrice).div(1_000_000);
    const outputPrice = new Decimal(pricing.outputTokenPrice).div(1_000_000);

    const inputTokenCost = new Decimal(usage.inputTokens).times(inputPrice);
    const outputTokenCost = new Decimal(usage.outputTokens).times(outputPrice);

    let cachedInputTokenCost = new Decimal(0);
    if (usage.cachedInputTokens && pricing.cachedInputTokenPrice) {
      const cachedPrice = new Decimal(pricing.cachedInputTokenPrice).div(1_000_000);
      cachedInputTokenCost = new Decimal(usage.cachedInputTokens).times(cachedPrice);
    }

    return { inputTokenCost, outputTokenCost, cachedInputTokenCost };
  }

  /**
   * Calculate per-request costs
   */
  private calculateRequestCost(usage: UsageRecord, pricing: PricingTable): Decimal {
    if (!pricing.requestPrice) {
      throw new Error('Per-request pricing requires requestPrice');
    }

    const requestPrice = new Decimal(pricing.requestPrice);
    return new Decimal(usage.requestCount).times(requestPrice);
  }

  /**
   * Calculate tiered costs based on token volume
   */
  private calculateTieredCosts(
    usage: UsageRecord,
    pricing: PricingTable
  ): {
    inputTokenCost: Decimal;
    outputTokenCost: Decimal;
    cachedInputTokenCost: Decimal;
  } {
    if (!pricing.tiers || pricing.tiers.length === 0) {
      throw new Error('Tiered pricing requires tiers configuration');
    }

    const totalTokens = usage.inputTokens + usage.outputTokens + (usage.cachedInputTokens || 0);
    const tier = this.findApplicableTier(totalTokens, pricing.tiers);

    if (!tier) {
      throw new Error(`No applicable tier found for ${totalTokens} tokens`);
    }

    const tierPrice = new Decimal(tier.pricePerToken).div(1_000_000);

    const inputTokenCost = new Decimal(usage.inputTokens).times(tierPrice);
    const outputTokenCost = new Decimal(usage.outputTokens).times(tierPrice);

    let cachedInputTokenCost = new Decimal(0);
    if (usage.cachedInputTokens) {
      // Apply same tier price to cached tokens (or could use separate tier logic)
      cachedInputTokenCost = new Decimal(usage.cachedInputTokens).times(tierPrice);
    }

    return { inputTokenCost, outputTokenCost, cachedInputTokenCost };
  }

  /**
   * Find the applicable pricing tier for given token count
   */
  private findApplicableTier(tokens: number, tiers: PricingTier[]): PricingTier | null {
    // Sort tiers by minTokens ascending
    const sortedTiers = [...tiers].sort((a, b) => a.minTokens - b.minTokens);

    for (const tier of sortedTiers) {
      const inRange = tokens >= tier.minTokens &&
        (tier.maxTokens === null || tokens <= tier.maxTokens);

      if (inRange) {
        return tier;
      }
    }

    return null;
  }

  /**
   * Validate inputs before calculation
   */
  private validateInputs(usage: UsageRecord, pricing: PricingTable): void {
    if (usage.provider !== pricing.provider) {
      throw new Error(
        `Provider mismatch: usage=${usage.provider}, pricing=${pricing.provider}`
      );
    }

    if (usage.model !== pricing.model) {
      throw new Error(
        `Model mismatch: usage=${usage.model}, pricing=${pricing.model}`
      );
    }

    if (usage.inputTokens < 0 || usage.outputTokens < 0) {
      throw new Error('Token counts cannot be negative');
    }

    if (usage.cachedInputTokens && usage.cachedInputTokens < 0) {
      throw new Error('Cached token count cannot be negative');
    }
  }

  /**
   * Convert cost to different currency (requires exchange rates)
   * This is a placeholder - in production, use real-time exchange rate service
   */
  convertCurrency(
    amount: string,
    fromCurrency: Currency,
    toCurrency: Currency,
    exchangeRates: Map<string, Decimal>
  ): string {
    if (fromCurrency === toCurrency) {
      return amount;
    }

    const rateKey = `${fromCurrency}_${toCurrency}`;
    const rate = exchangeRates.get(rateKey);

    if (!rate) {
      throw new Error(`Exchange rate not found for ${rateKey}`);
    }

    return new Decimal(amount).times(rate).toFixed(10);
  }
}
