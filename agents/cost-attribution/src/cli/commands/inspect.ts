import { Command } from 'commander';
import { DecisionEventEmitter } from '../../types/decision-event.js';
import { OutputFormatter } from '../../utils/output.js';
import { CostRecord } from '../../engine/calculator.js';

/**
 * Inspect command - Inspect a specific cost record
 *
 * Queries ruvector-service for decision events related to a cost record
 */
export function createInspectCommand(): Command {
  const cmd = new Command('inspect');

  cmd
    .description('Inspect a specific cost record and its decision history')
    .requiredOption('-i, --id <id>', 'Cost record ID (execution ID)')
    .option('-f, --format <format>', 'Output format: json or table', 'table')
    .action(async (options) => {
      const startTime = Date.now();

      try {
        // In production, would query ruvector-service for decision events
        // For now, simulate inspection of a cost record
        const costRecord = await fetchCostRecord(options.id);

        if (!costRecord) {
          throw new Error(`Cost record not found: ${options.id}`);
        }

        // Format output
        const formatter = new OutputFormatter();
        const output = formatter.formatCostRecord(costRecord, options.format);
        console.log(output);

        // Show decision history
        console.log('\nDecision History:');
        const decisionHistory = await fetchDecisionHistory(options.id);
        console.log(JSON.stringify(decisionHistory, null, 2));

        // Emit decision event
        const executionTime = Date.now() - startTime;
        const emitter = new DecisionEventEmitter();

        const event = emitter.createEvent(
          'cost-inspection',
          'inspect',
          {
            id: options.id,
            format: options.format,
          },
          {
            success: true,
            recordsProcessed: 1,
          },
          {
            executionTimeMs: executionTime,
          }
        );

        await emitter.emit(event);

        process.exit(0);
      } catch (error) {
        const executionTime = Date.now() - startTime;
        const emitter = new DecisionEventEmitter();

        const event = emitter.createEvent(
          'cost-inspection',
          'inspect',
          {
            id: options.id,
            format: options.format,
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
 * Fetch a cost record by ID
 * In production, would query from database or cache
 */
async function fetchCostRecord(executionId: string): Promise<CostRecord | null> {
  // Simulate fetching from database
  // In production, would query actual storage

  // For demo, return null to show "not found" flow
  // In real implementation, would fetch from persistent storage
  return null;
}

/**
 * Fetch decision history for a cost record
 * In production, would query ruvector-service
 */
async function fetchDecisionHistory(executionId: string): Promise<any[]> {
  // Simulate fetching decision history
  // In production, would query ruvector-service API:
  // GET /api/v1/decisions?execution_id={executionId}

  return [
    {
      timestamp: new Date().toISOString(),
      type: 'cost-attribution',
      action: 'calculated',
      details: {
        executionId,
        status: 'completed',
      },
    },
  ];
}
