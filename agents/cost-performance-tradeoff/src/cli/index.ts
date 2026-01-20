#!/usr/bin/env node
/**
 * Cost-Performance Tradeoff Agent CLI
 *
 * CLI entrypoint for the Cost-Performance Tradeoff Agent.
 *
 * Classification: TRADEOFF ANALYSIS
 *
 * Commands:
 *   analyze   - Run cost-performance tradeoff analysis
 *   inspect   - Inspect historical analyses
 *   version   - Show agent version
 *
 * This agent MUST NOT:
 * - Enforce budgets
 * - Modify execution behavior
 * - Apply optimizations automatically
 * - Intercept runtime execution
 */

import { Command } from 'commander';
import { createAnalyzeCommand } from './analyze.js';
import { createInspectCommand } from './inspect.js';

const AGENT_ID = 'cost-performance-tradeoff-agent';
const AGENT_VERSION = '1.0.0';

async function main(): Promise<void> {
  const program = new Command();

  program
    .name('cost-performance-tradeoff')
    .description('Cost-Performance Tradeoff Agent - Evaluate tradeoffs between cost, latency, and quality')
    .version(AGENT_VERSION);

  // Add commands
  program.addCommand(createAnalyzeCommand());
  program.addCommand(createInspectCommand());

  // Version command with detailed info
  program
    .command('version')
    .description('Show detailed version information')
    .action(() => {
      console.log(JSON.stringify({
        agent_id: AGENT_ID,
        agent_version: AGENT_VERSION,
        classification: 'TRADEOFF_ANALYSIS',
        decision_type: 'cost_performance_tradeoff',
        capabilities: [
          'cost_vs_latency_analysis',
          'cost_vs_quality_analysis',
          'diminishing_returns_detection',
          'pareto_frontier_computation',
          'model_recommendations'
        ],
        constraints: {
          max_cost_per_request: true,
          max_latency_p95: true,
          min_quality_score: true
        },
        runtime: 'google_cloud_edge_function',
        persistence: 'ruvector_service_only'
      }, null, 2));
    });

  // Health check command
  program
    .command('health')
    .description('Check agent health and dependencies')
    .action(async () => {
      const { RuvectorServiceClient } = await import('../services/index.js');
      const client = new RuvectorServiceClient();

      try {
        const health = await client.healthCheck();
        console.log(JSON.stringify({
          agent: {
            id: AGENT_ID,
            version: AGENT_VERSION,
            status: 'healthy'
          },
          dependencies: {
            ruvector_service: {
              healthy: health.healthy,
              latency_ms: health.latencyMs
            }
          }
        }, null, 2));

        process.exit(health.healthy ? 0 : 1);
      } catch (error) {
        console.log(JSON.stringify({
          agent: {
            id: AGENT_ID,
            version: AGENT_VERSION,
            status: 'degraded'
          },
          dependencies: {
            ruvector_service: {
              healthy: false,
              error: error instanceof Error ? error.message : String(error)
            }
          }
        }, null, 2));

        process.exit(1);
      }
    });

  await program.parseAsync(process.argv);
}

main().catch((error) => {
  console.error('Fatal error:', error);
  process.exit(1);
});
