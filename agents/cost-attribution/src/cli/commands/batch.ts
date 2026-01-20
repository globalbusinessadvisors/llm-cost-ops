import { Command } from 'commander';
import * as fs from 'fs';
import { CostCalculator, UsageRecord } from '../../engine/calculator.js';
import { DecisionEventEmitter } from '../../types/decision-event.js';
import { InputReader } from '../../utils/input.js';
import { OutputFormatter } from '../../utils/output.js';

/**
 * Batch command - Process batch of usage records with parallel execution
 *
 * Reads usage records from file, processes in parallel batches, writes results
 */
export function createBatchCommand(): Command {
  const cmd = new Command('batch');

  cmd
    .description('Process batch of usage records in parallel')
    .requiredOption('-i, --input <file>', 'Input file (JSON or JSONL)')
    .requiredOption('-o, --output <file>', 'Output file for results')
    .option('-p, --parallel <number>', 'Number of parallel workers', '4')
    .option('-f, --format <format>', 'Output format: json or table', 'json')
    .action(async (options) => {
      const startTime = Date.now();

      try {
        // Read input
        const inputReader = new InputReader();
        const usageRecords = await inputReader.readUsageRecords(options.input);

        if (usageRecords.length === 0) {
          throw new Error('No usage records found');
        }

        const parallelWorkers = parseInt(options.parallel, 10);
        if (isNaN(parallelWorkers) || parallelWorkers < 1) {
          throw new Error('Invalid parallel workers count');
        }

        console.error(`Processing ${usageRecords.length} records with ${parallelWorkers} workers...`);

        // Process in batches
        const calculator = new CostCalculator();
        const costRecords = await processBatchParallel(
          usageRecords,
          calculator,
          parallelWorkers
        );

        // Format output
        const formatter = new OutputFormatter();
        const output = formatter.formatCostRecords(costRecords, options.format);

        // Write to output file
        fs.writeFileSync(options.output, output, 'utf-8');

        console.error(`✓ Processed ${costRecords.length} records`);
        console.error(`✓ Results written to ${options.output}`);

        // Emit decision event
        const executionTime = Date.now() - startTime;
        const emitter = new DecisionEventEmitter();

        const totalCost = costRecords.reduce(
          (sum, r) => sum + parseFloat(r.totalCost),
          0
        );

        const event = emitter.createEvent(
          'batch-processing',
          'batch',
          {
            input: options.input,
            output: options.output,
            parallel: parallelWorkers,
            format: options.format,
          },
          {
            success: true,
            recordsProcessed: costRecords.length,
            totalCost: totalCost.toFixed(6),
            currency: costRecords[0]?.currency || 'USD',
          },
          {
            executionTimeMs: executionTime,
            recordsPerSecond: Math.round(costRecords.length / (executionTime / 1000)),
          }
        );

        await emitter.emit(event);

        process.exit(0);
      } catch (error) {
        const executionTime = Date.now() - startTime;
        const emitter = new DecisionEventEmitter();

        const event = emitter.createEvent(
          'batch-processing',
          'batch',
          {
            input: options.input,
            output: options.output,
            parallel: parseInt(options.parallel, 10),
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
 * Process usage records in parallel batches
 * Simulates parallel processing by chunking the array
 */
async function processBatchParallel(
  usageRecords: UsageRecord[],
  calculator: CostCalculator,
  parallelWorkers: number
) {
  // Get default pricing (in production, would handle multiple providers/models)
  const { getDefaultPricingTable } = await import('./analyze.js');
  const pricingTable = (getDefaultPricingTable as any)(
    usageRecords[0].provider,
    usageRecords[0].model
  );

  // Split into chunks for parallel processing
  const chunkSize = Math.ceil(usageRecords.length / parallelWorkers);
  const chunks: UsageRecord[][] = [];

  for (let i = 0; i < usageRecords.length; i += chunkSize) {
    chunks.push(usageRecords.slice(i, i + chunkSize));
  }

  // Process chunks in parallel
  const results = await Promise.all(
    chunks.map(async (chunk) => {
      return calculator.calculateBatchCosts(chunk, pricingTable);
    })
  );

  // Flatten results
  return results.flat();
}
