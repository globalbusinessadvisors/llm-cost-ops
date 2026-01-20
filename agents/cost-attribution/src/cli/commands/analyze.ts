import { Command } from 'commander';
import { CostCalculator, PricingTable, PricingType, Currency } from '../../engine/calculator.js';
import { DecisionEventEmitter } from '../../types/decision-event.js';
import { InputReader } from '../../utils/input.js';
import { OutputFormatter } from '../../utils/output.js';

/**
 * Analyze command - Run cost attribution analysis
 *
 * Reads usage records, runs attribution, outputs results, emits DecisionEvent
 */
export function createAnalyzeCommand(): Command {
  const cmd = new Command('analyze');

  cmd
    .description('Run cost attribution analysis on usage records')
    .option('-i, --input <file>', 'Input file (JSON or JSONL) or "-" for stdin', '-')
    .option('-f, --format <format>', 'Output format: json or table', 'table')
    .option('-s, --scope <scope>', 'Attribution scope: execution, agent, workflow, or tenant', 'execution')
    .option('--summary', 'Show summary statistics', false)
    .action(async (options) => {
      const startTime = Date.now();

      try {
        // Read input
        const inputReader = new InputReader();
        const usageRecords = await inputReader.readUsageRecords(options.input);

        if (usageRecords.length === 0) {
          throw new Error('No usage records found');
        }

        // Get pricing table (in production, would fetch from database/config)
        const pricingTable = getDefaultPricingTable(usageRecords[0].provider, usageRecords[0].model);

        // Calculate costs
        const calculator = new CostCalculator();
        const costRecords = calculator.calculateBatchCosts(usageRecords, pricingTable);

        // Format output
        const formatter = new OutputFormatter();

        if (options.summary) {
          const summaryOutput = formatter.formatSummary(costRecords, options.format);
          console.log(summaryOutput);
        } else {
          const output = formatter.formatCostRecords(costRecords, options.format);
          console.log(output);
        }

        // Emit decision event
        const executionTime = Date.now() - startTime;
        const emitter = new DecisionEventEmitter();

        const totalCost = costRecords.reduce(
          (sum, r) => sum + parseFloat(r.totalCost),
          0
        );

        const event = emitter.createEvent(
          'cost-attribution',
          'analyze',
          {
            input: options.input,
            format: options.format,
            scope: options.scope,
            summary: options.summary,
          },
          {
            success: true,
            recordsProcessed: costRecords.length,
            totalCost: totalCost.toFixed(6),
            currency: costRecords[0]?.currency || 'USD',
          },
          {
            executionTimeMs: executionTime,
            inputRecords: usageRecords.length,
            outputRecords: costRecords.length,
          }
        );

        await emitter.emit(event);

        process.exit(0);
      } catch (error) {
        const executionTime = Date.now() - startTime;
        const emitter = new DecisionEventEmitter();

        const event = emitter.createEvent(
          'cost-attribution',
          'analyze',
          {
            input: options.input,
            format: options.format,
            scope: options.scope,
          },
          {
            success: false,
            errors: [error instanceof Error ? error.message : String(error)],
          },
          {
            executionTimeMs: executionTime,
          }
        );

        await emitter.emit(event);

        console.error(`Error: ${error instanceof Error ? error.message : String(error)}`);
        process.exit(1);
      }
    });

  return cmd;
}

/**
 * Get default pricing table for a provider/model
 * In production, this would fetch from a database or config service
 */
export function getDefaultPricingTable(provider: string, model: string): PricingTable {
  // Default pricing for common providers
  // These are example prices - would be loaded from config in production
  const pricingTables: Record<string, PricingTable> = {
    'anthropic/claude-3-opus': {
      provider: 'anthropic',
      model: 'claude-3-opus',
      type: PricingType.PER_TOKEN,
      currency: Currency.USD,
      inputTokenPrice: '15.00', // $15 per 1M tokens
      outputTokenPrice: '75.00', // $75 per 1M tokens
      cachedInputTokenPrice: '1.50', // $1.50 per 1M cached tokens
      effectiveDate: new Date('2024-01-01'),
    },
    'anthropic/claude-3-sonnet': {
      provider: 'anthropic',
      model: 'claude-3-sonnet',
      type: PricingType.PER_TOKEN,
      currency: Currency.USD,
      inputTokenPrice: '3.00',
      outputTokenPrice: '15.00',
      cachedInputTokenPrice: '0.30',
      effectiveDate: new Date('2024-01-01'),
    },
    'openai/gpt-4': {
      provider: 'openai',
      model: 'gpt-4',
      type: PricingType.PER_TOKEN,
      currency: Currency.USD,
      inputTokenPrice: '30.00',
      outputTokenPrice: '60.00',
      effectiveDate: new Date('2024-01-01'),
    },
  };

  const key = `${provider}/${model}`;
  const pricing = pricingTables[key];

  if (!pricing) {
    // Return a default pricing table if not found
    return {
      provider,
      model,
      type: PricingType.PER_TOKEN,
      currency: Currency.USD,
      inputTokenPrice: '1.00',
      outputTokenPrice: '2.00',
      effectiveDate: new Date(),
    };
  }

  return pricing;
}
