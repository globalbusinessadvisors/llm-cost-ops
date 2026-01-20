import { UsageRecord } from './calculator';

/**
 * Normalized usage record with standardized token counts
 */
export interface NormalizedUsage {
  originalUsage: UsageRecord;
  normalizedInputTokens: number;
  normalizedOutputTokens: number;
  normalizedCachedInputTokens: number;
  totalNormalizedTokens: number;
  normalizationFactor: number;
  normalizationMethod: string;
}

/**
 * Provider-specific token counting configuration
 */
export interface TokenCountingConfig {
  provider: string;
  model?: string; // Optional model-specific override

  // Normalization factors relative to base (OpenAI GPT-4)
  inputTokenFactor: number;
  outputTokenFactor: number;
  cachedInputTokenFactor?: number;

  // Provider-specific quirks
  countSystemPromptSeparately?: boolean;
  includeWhitespaceInCount?: boolean;
  useByteCountEstimate?: boolean;

  // Character-to-token ratio estimation (if tokenizer unavailable)
  averageCharsPerToken?: number;
}

/**
 * Default token counting configurations for major providers
 */
const DEFAULT_CONFIGS: TokenCountingConfig[] = [
  {
    provider: 'anthropic',
    model: 'claude-3-5-sonnet-20250219',
    inputTokenFactor: 1.0, // Claude uses similar tokenization to GPT-4
    outputTokenFactor: 1.0,
    cachedInputTokenFactor: 1.0,
    averageCharsPerToken: 4.0,
  },
  {
    provider: 'anthropic',
    model: 'claude-3-5-haiku-20250219',
    inputTokenFactor: 1.0,
    outputTokenFactor: 1.0,
    cachedInputTokenFactor: 1.0,
    averageCharsPerToken: 4.0,
  },
  {
    provider: 'openai',
    model: 'gpt-4',
    inputTokenFactor: 1.0, // Base reference
    outputTokenFactor: 1.0,
    averageCharsPerToken: 4.0,
  },
  {
    provider: 'openai',
    model: 'gpt-3.5-turbo',
    inputTokenFactor: 1.0,
    outputTokenFactor: 1.0,
    averageCharsPerToken: 4.0,
  },
  {
    provider: 'google',
    model: 'gemini-pro',
    inputTokenFactor: 1.05, // Gemini tends to count ~5% more tokens
    outputTokenFactor: 1.05,
    averageCharsPerToken: 3.8,
  },
  {
    provider: 'cohere',
    model: 'command',
    inputTokenFactor: 0.95, // Cohere tends to count slightly fewer tokens
    outputTokenFactor: 0.95,
    averageCharsPerToken: 4.2,
  },
  {
    provider: 'meta',
    model: 'llama-2',
    inputTokenFactor: 1.02,
    outputTokenFactor: 1.02,
    averageCharsPerToken: 4.1,
  },
  {
    provider: 'mistral',
    model: 'mistral-large',
    inputTokenFactor: 0.98,
    outputTokenFactor: 0.98,
    averageCharsPerToken: 4.0,
  },
];

/**
 * TokenNormalizer - Standardizes token counts across providers
 *
 * Different LLM providers use different tokenization algorithms:
 * - OpenAI: tiktoken (BPE with GPT-4 vocab)
 * - Anthropic: Claude tokenizer (similar to GPT-4 but optimized)
 * - Google: SentencePiece (different encoding)
 * - Cohere: Custom BPE
 *
 * This normalizer ensures fair cost comparison by standardizing to a common base.
 */
export class TokenNormalizer {
  private configs: Map<string, TokenCountingConfig>;

  constructor(customConfigs: TokenCountingConfig[] = []) {
    this.configs = new Map();

    // Load default configs
    for (const config of DEFAULT_CONFIGS) {
      const key = this.makeConfigKey(config.provider, config.model);
      this.configs.set(key, config);
    }

    // Override with custom configs
    for (const config of customConfigs) {
      const key = this.makeConfigKey(config.provider, config.model);
      this.configs.set(key, config);
    }
  }

  /**
   * Normalize usage record to standardized token counts
   */
  normalize(usage: UsageRecord): NormalizedUsage {
    const config = this.findConfig(usage.provider, usage.model);

    if (!config) {
      // No normalization config found, use raw counts
      return this.createNormalizedUsage(usage, 1.0, 'raw');
    }

    // Apply provider-specific normalization
    const method = this.determineNormalizationMethod(config);

    switch (method) {
      case 'factor':
        return this.normalizeByFactor(usage, config);

      case 'estimate':
        return this.normalizeByEstimate(usage, config);

      default:
        return this.createNormalizedUsage(usage, 1.0, 'raw');
    }
  }

  /**
   * Batch normalize multiple usage records
   */
  normalizeBatch(usages: UsageRecord[]): NormalizedUsage[] {
    return usages.map(usage => this.normalize(usage));
  }

  /**
   * Add or update a token counting configuration
   */
  addConfig(config: TokenCountingConfig): void {
    const key = this.makeConfigKey(config.provider, config.model);
    this.configs.set(key, config);
  }

  /**
   * Get all registered configurations
   */
  getConfigs(): TokenCountingConfig[] {
    return Array.from(this.configs.values());
  }

  /**
   * Normalize by applying provider-specific factors
   */
  private normalizeByFactor(usage: UsageRecord, config: TokenCountingConfig): NormalizedUsage {
    const normalizedInputTokens = Math.round(usage.inputTokens * config.inputTokenFactor);
    const normalizedOutputTokens = Math.round(usage.outputTokens * config.outputTokenFactor);

    let normalizedCachedInputTokens = 0;
    if (usage.cachedInputTokens && config.cachedInputTokenFactor) {
      normalizedCachedInputTokens = Math.round(
        usage.cachedInputTokens * config.cachedInputTokenFactor
      );
    }

    const totalNormalizedTokens =
      normalizedInputTokens + normalizedOutputTokens + normalizedCachedInputTokens;

    // Average normalization factor for reporting
    const avgFactor =
      (config.inputTokenFactor + config.outputTokenFactor + (config.cachedInputTokenFactor || 1.0)) /
      3.0;

    return {
      originalUsage: usage,
      normalizedInputTokens,
      normalizedOutputTokens,
      normalizedCachedInputTokens,
      totalNormalizedTokens,
      normalizationFactor: avgFactor,
      normalizationMethod: 'factor',
    };
  }

  /**
   * Normalize by estimating from character count
   * (Fallback when actual token counts are unavailable)
   */
  private normalizeByEstimate(usage: UsageRecord, config: TokenCountingConfig): NormalizedUsage {
    if (!config.averageCharsPerToken) {
      throw new Error(`Cannot estimate tokens: averageCharsPerToken not configured for ${config.provider}`);
    }

    // Use raw token counts as character count approximation
    const estimatedChars = (usage.inputTokens + usage.outputTokens) * config.averageCharsPerToken;
    const baseCharsPerToken = 4.0; // GPT-4 baseline

    const normalizedTotalTokens = Math.round(estimatedChars / baseCharsPerToken);

    // Distribute proportionally
    const totalOriginalTokens = usage.inputTokens + usage.outputTokens;
    const inputRatio = usage.inputTokens / totalOriginalTokens;
    const outputRatio = usage.outputTokens / totalOriginalTokens;

    const normalizedInputTokens = Math.round(normalizedTotalTokens * inputRatio);
    const normalizedOutputTokens = Math.round(normalizedTotalTokens * outputRatio);

    return {
      originalUsage: usage,
      normalizedInputTokens,
      normalizedOutputTokens,
      normalizedCachedInputTokens: 0,
      totalNormalizedTokens: normalizedTotalTokens,
      normalizationFactor: config.averageCharsPerToken / baseCharsPerToken,
      normalizationMethod: 'estimate',
    };
  }

  /**
   * Create normalized usage with identity normalization (no change)
   */
  private createNormalizedUsage(
    usage: UsageRecord,
    factor: number,
    method: string
  ): NormalizedUsage {
    return {
      originalUsage: usage,
      normalizedInputTokens: usage.inputTokens,
      normalizedOutputTokens: usage.outputTokens,
      normalizedCachedInputTokens: usage.cachedInputTokens || 0,
      totalNormalizedTokens: usage.inputTokens + usage.outputTokens + (usage.cachedInputTokens || 0),
      normalizationFactor: factor,
      normalizationMethod: method,
    };
  }

  /**
   * Find configuration for provider and model
   */
  private findConfig(provider: string, model: string): TokenCountingConfig | undefined {
    // Try exact match first (provider + model)
    const exactKey = this.makeConfigKey(provider, model);
    const exactConfig = this.configs.get(exactKey);
    if (exactConfig) return exactConfig;

    // Fall back to provider-level config
    const providerKey = this.makeConfigKey(provider);
    return this.configs.get(providerKey);
  }

  /**
   * Determine which normalization method to use
   */
  private determineNormalizationMethod(config: TokenCountingConfig): string {
    // If we have explicit factors, use them
    if (config.inputTokenFactor && config.outputTokenFactor) {
      return 'factor';
    }

    // Fall back to character estimation
    if (config.averageCharsPerToken) {
      return 'estimate';
    }

    return 'raw';
  }

  /**
   * Create configuration map key
   */
  private makeConfigKey(provider: string, model?: string): string {
    return model ? `${provider}:${model}` : provider;
  }

  /**
   * Calculate normalization variance across providers
   * (Useful for analyzing token count discrepancies)
   */
  calculateNormalizationVariance(usages: NormalizedUsage[]): {
    meanFactor: number;
    stdDeviation: number;
    minFactor: number;
    maxFactor: number;
  } {
    if (usages.length === 0) {
      return { meanFactor: 1.0, stdDeviation: 0, minFactor: 1.0, maxFactor: 1.0 };
    }

    const factors = usages.map(u => u.normalizationFactor);
    const mean = factors.reduce((sum, f) => sum + f, 0) / factors.length;

    const squaredDiffs = factors.map(f => Math.pow(f - mean, 2));
    const variance = squaredDiffs.reduce((sum, d) => sum + d, 0) / factors.length;
    const stdDev = Math.sqrt(variance);

    return {
      meanFactor: mean,
      stdDeviation: stdDev,
      minFactor: Math.min(...factors),
      maxFactor: Math.max(...factors),
    };
  }

  /**
   * Generate normalization report for analysis
   */
  generateNormalizationReport(usages: NormalizedUsage[]): {
    totalOriginalTokens: number;
    totalNormalizedTokens: number;
    overallNormalizationFactor: number;
    methodBreakdown: Record<string, number>;
    providerBreakdown: Record<string, { original: number; normalized: number; factor: number }>;
  } {
    let totalOriginalTokens = 0;
    let totalNormalizedTokens = 0;
    const methodCounts: Record<string, number> = {};
    const providerStats: Record<string, { original: number; normalized: number }> = {};

    for (const usage of usages) {
      const original =
        usage.originalUsage.inputTokens +
        usage.originalUsage.outputTokens +
        (usage.originalUsage.cachedInputTokens || 0);

      totalOriginalTokens += original;
      totalNormalizedTokens += usage.totalNormalizedTokens;

      // Method breakdown
      methodCounts[usage.normalizationMethod] = (methodCounts[usage.normalizationMethod] || 0) + 1;

      // Provider breakdown
      const provider = usage.originalUsage.provider;
      if (!providerStats[provider]) {
        providerStats[provider] = { original: 0, normalized: 0 };
      }
      providerStats[provider].original += original;
      providerStats[provider].normalized += usage.totalNormalizedTokens;
    }

    // Calculate provider-level factors
    const providerBreakdown: Record<string, { original: number; normalized: number; factor: number }> = {};
    for (const [provider, stats] of Object.entries(providerStats)) {
      providerBreakdown[provider] = {
        original: stats.original,
        normalized: stats.normalized,
        factor: stats.normalized / stats.original,
      };
    }

    return {
      totalOriginalTokens,
      totalNormalizedTokens,
      overallNormalizationFactor: totalNormalizedTokens / totalOriginalTokens,
      methodBreakdown: methodCounts,
      providerBreakdown,
    };
  }
}
