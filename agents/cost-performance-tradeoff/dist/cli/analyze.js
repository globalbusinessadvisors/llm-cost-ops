/**
 * Analyze Command
 *
 * Run cost-performance tradeoff analysis on performance records.
 *
 * Usage:
 *   cost-performance-tradeoff analyze [options]
 *   cat records.json | cost-performance-tradeoff analyze --scope model
 */
import { Command } from 'commander';
import { v4 as uuidv4 } from 'uuid';
import { TradeoffAnalyzer } from '../engine/index.js';
import { DecisionEventEmitter } from '../types/index.js';
import { TelemetryEmitter } from '../services/index.js';
import { InputReader, OutputFormatter } from '../utils/index.js';
export function createAnalyzeCommand() {
    const cmd = new Command('analyze');
    cmd
        .description('Run cost-performance tradeoff analysis on performance records')
        .option('-i, --input <file>', 'Input file or stdin (-)', '-')
        .option('-f, --format <format>', 'Output format: json or table', 'table')
        .option('-s, --scope <scope>', 'Analysis scope: model, provider, tier, execution', 'model')
        .option('-w, --weights <weights>', 'Weights as JSON: {"cost":0.33,"latency":0.33,"quality":0.34}')
        .option('--max-cost <cost>', 'Maximum cost per request constraint (USD)', parseFloat)
        .option('--max-latency <latency>', 'Maximum P95 latency constraint (ms)', parseInt)
        .option('--min-quality <quality>', 'Minimum quality score constraint (0-1)', parseFloat)
        .option('--summary', 'Show summary only', false)
        .option('--no-pareto', 'Disable Pareto frontier analysis')
        .option('--no-diminishing-returns', 'Disable diminishing returns detection')
        .option('--no-recommendations', 'Disable recommendations')
        .option('--trace-id <traceId>', 'Distributed tracing ID')
        .action(async (options) => {
        const startTime = Date.now();
        const traceId = options.traceId ?? uuidv4();
        // Initialize components
        const inputReader = new InputReader();
        const analyzer = new TradeoffAnalyzer();
        const formatter = new OutputFormatter();
        const decisionEmitter = new DecisionEventEmitter();
        const telemetry = new TelemetryEmitter();
        try {
            // Read and validate input
            const input = await inputReader.readInput(options.input);
            // Parse weights if provided
            let weights = undefined;
            if (options.weights) {
                try {
                    weights = JSON.parse(options.weights);
                }
                catch {
                    throw new Error('Invalid weights JSON format');
                }
            }
            // Build analysis input
            const analysisInput = {
                records: input.records,
                analysis_scope: options.scope,
                weights,
                constraints: {
                    max_cost_per_request_usd: options.maxCost,
                    max_latency_p95_ms: options.maxLatency,
                    min_quality_score: options.minQuality
                },
                options: {
                    include_pareto_frontier: options.pareto,
                    include_diminishing_returns: options.diminishingReturns,
                    include_recommendations: options.recommendations,
                    normalize_metrics: true
                }
            };
            // Run analysis
            const output = analyzer.analyze(analysisInput);
            // Emit telemetry
            await telemetry.emitTradeoffAnalysis('cost-performance-tradeoff-agent', output.analysis_id, output.summary.total_records_analyzed, Date.now() - startTime, traceId);
            // Emit decision event (REQUIRED - exactly ONE per invocation)
            await decisionEmitter.createAndEmit({
                decisionType: 'cost_performance_tradeoff',
                input: analysisInput,
                output,
                executionRef: traceId
            });
            // Format and output results
            if (options.summary) {
                console.log(formatter.formatSummary(output));
            }
            else {
                console.log(formatter.format(output, options.format));
            }
            // Cleanup
            await telemetry.flush();
            telemetry.stop();
            process.exit(0);
        }
        catch (error) {
            // Emit error telemetry
            await telemetry.emitError('cost-performance-tradeoff-agent', error, { command: 'analyze', options }, traceId);
            await telemetry.flush();
            telemetry.stop();
            // Output error
            if (options.format === 'json') {
                console.error(JSON.stringify({
                    error: {
                        code: 'ANALYSIS_FAILED',
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
//# sourceMappingURL=analyze.js.map