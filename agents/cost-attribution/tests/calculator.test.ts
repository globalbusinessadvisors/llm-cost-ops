/**
 * Cost Calculator Tests
 */

import { CostCalculator } from '../src/calculator';
import type { UsageData, PricingContext } from '../src/types';

describe('CostCalculator', () => {
  let calculator: CostCalculator;

  beforeEach(() => {
    calculator = new CostCalculator();
  });

  describe('calculate', () => {
    it('should calculate costs for Anthropic Claude Sonnet 4', () => {
      const usage: UsageData = {
        provider: 'anthropic',
        model: 'claude-sonnet-4',
        inputTokens: 1000,
        outputTokens: 500,
        cachedTokens: 200,
      };

      const result = calculator.calculate(usage);

      expect(result.currency).toBe('USD');
      expect(result.inputCost).toBe(0.003); // 1000 * 3.0 / 1M
      expect(result.outputCost).toBe(0.0075); // 500 * 15.0 / 1M
      expect(result.cachedCost).toBe(0.00006); // 200 * 0.3 / 1M
      expect(result.totalCost).toBe(0.01056);
    });

    it('should calculate costs for OpenAI GPT-4', () => {
      const usage: UsageData = {
        provider: 'openai',
        model: 'gpt-4',
        inputTokens: 2000,
        outputTokens: 1000,
      };

      const result = calculator.calculate(usage);

      expect(result.currency).toBe('USD');
      expect(result.inputCost).toBe(0.06); // 2000 * 30.0 / 1M
      expect(result.outputCost).toBe(0.06); // 1000 * 60.0 / 1M
      expect(result.totalCost).toBe(0.12);
      expect(result.cachedCost).toBeUndefined();
    });

    it('should calculate costs with custom pricing', () => {
      const usage: UsageData = {
        provider: 'anthropic',
        model: 'claude-sonnet-4',
        inputTokens: 1000,
        outputTokens: 500,
      };

      const pricingContext: PricingContext = {
        currency: 'EUR',
        customPricing: {
          inputPricePerToken: 0.000005, // 5.0 per million
          outputPricePerToken: 0.00002, // 20.0 per million
        },
      };

      const result = calculator.calculate(usage, pricingContext);

      expect(result.currency).toBe('EUR');
      expect(result.inputCost).toBe(0.005);
      expect(result.outputCost).toBe(0.01);
      expect(result.totalCost).toBe(0.015);
    });

    it('should throw error for unknown provider', () => {
      const usage: UsageData = {
        provider: 'unknown-provider',
        model: 'some-model',
        inputTokens: 1000,
        outputTokens: 500,
      };

      expect(() => calculator.calculate(usage)).toThrow('Unknown provider');
    });

    it('should throw error for unknown model', () => {
      const usage: UsageData = {
        provider: 'anthropic',
        model: 'unknown-model',
        inputTokens: 1000,
        outputTokens: 500,
      };

      expect(() => calculator.calculate(usage)).toThrow('Unknown model');
    });

    it('should calculate costPer1kTokens correctly', () => {
      const usage: UsageData = {
        provider: 'anthropic',
        model: 'claude-haiku-4',
        inputTokens: 10000,
        outputTokens: 5000,
      };

      const result = calculator.calculate(usage);

      const totalTokens = 15000;
      const expectedCostPer1k = (result.totalCost / totalTokens) * 1000;

      expect(result.costPer1kTokens).toBeCloseTo(expectedCostPer1k, 6);
    });
  });
});
