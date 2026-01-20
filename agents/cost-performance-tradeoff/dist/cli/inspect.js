/**
 * Inspect Command
 *
 * Inspect historical tradeoff analyses and decision events.
 *
 * Usage:
 *   cost-performance-tradeoff inspect [options]
 *   cost-performance-tradeoff inspect --model gpt-4o
 */
import { Command } from 'commander';
import { RuvectorServiceClient } from '../services/index.js';
export function createInspectCommand() {
    const cmd = new Command('inspect');
    cmd
        .description('Inspect historical tradeoff analyses and decision events')
        .option('-m, --model <model>', 'Filter by model name')
        .option('-p, --provider <provider>', 'Filter by provider')
        .option('-f, --format <format>', 'Output format: json or table', 'table')
        .option('-l, --limit <limit>', 'Maximum number of records', parseInt, 10)
        .option('--start-time <time>', 'Start time filter (ISO 8601)')
        .option('--end-time <time>', 'End time filter (ISO 8601)')
        .action(async (options) => {
        const client = new RuvectorServiceClient();
        try {
            // Query decision events
            const events = await client.queryDecisionEvents({
                agentId: 'cost-performance-tradeoff-agent',
                decisionType: 'cost_performance_tradeoff',
                startTime: options.startTime,
                endTime: options.endTime,
                limit: options.limit
            });
            // Filter by model/provider if specified
            let filtered = events;
            if (options.model) {
                filtered = filtered.filter(e => JSON.stringify(e.outputs).includes(options.model));
            }
            if (options.provider) {
                filtered = filtered.filter(e => JSON.stringify(e.outputs).includes(options.provider));
            }
            // Format output
            if (options.format === 'json') {
                console.log(JSON.stringify(filtered, null, 2));
            }
            else {
                console.log(formatEventsTable(filtered));
            }
            process.exit(0);
        }
        catch (error) {
            if (options.format === 'json') {
                console.error(JSON.stringify({
                    error: {
                        code: 'INSPECT_FAILED',
                        message: error instanceof Error ? error.message : String(error)
                    }
                }, null, 2));
            }
            else {
                console.error(`Error: ${error instanceof Error ? error.message : String(error)}`);
            }
            process.exit(1);
        }
    });
    return cmd;
}
function formatEventsTable(events) {
    const lines = [];
    lines.push('');
    lines.push('HISTORICAL TRADEOFF ANALYSES');
    lines.push('='.repeat(80));
    lines.push('');
    if (events.length === 0) {
        lines.push('No decision events found.');
        return lines.join('\n');
    }
    // Header
    lines.push([
        'Timestamp'.padEnd(24),
        'Decision Type'.padEnd(30),
        'Confidence'.padEnd(12),
        'Records'
    ].join(' '));
    lines.push('-'.repeat(80));
    // Rows
    for (const event of events) {
        const outputs = event.outputs;
        lines.push([
            event.timestamp.substring(0, 23).padEnd(24),
            event.decision_type.padEnd(30),
            `${(event.confidence * 100).toFixed(0)}%`.padEnd(12),
            (outputs.total_records ?? 'N/A').toString()
        ].join(' '));
        if (outputs.best_overall) {
            lines.push(`  Best Overall: ${outputs.best_overall}`);
        }
    }
    lines.push('');
    lines.push(`Total: ${events.length} events`);
    return lines.join('\n');
}
//# sourceMappingURL=inspect.js.map