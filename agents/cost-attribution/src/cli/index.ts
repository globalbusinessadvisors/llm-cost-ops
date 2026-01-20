#!/usr/bin/env node

import { Command } from 'commander';
import { createAnalyzeCommand } from './commands/analyze.js';
import { createInspectCommand } from './commands/inspect.js';
import { createBatchCommand } from './commands/batch.js';
import { fileURLToPath } from 'url';

/**
 * Cost Attribution Agent CLI
 *
 * Main entry point for all CLI commands
 * Deterministic, machine-readable, with DecisionEvent emission
 */
export function createCLI(): Command {
  const program = new Command();

  program
    .name('cost-attribution')
    .description('Cost Attribution Agent - Deterministic LLM cost tracking and attribution')
    .version('1.0.0');

  // Add commands
  program.addCommand(createAnalyzeCommand());
  program.addCommand(createInspectCommand());
  program.addCommand(createBatchCommand());

  // Version command
  program
    .command('version')
    .description('Show agent version and build info')
    .action(() => {
      const versionInfo = {
        version: '1.0.0',
        name: '@llm-costops/cost-attribution-agent',
        node: process.version,
        platform: process.platform,
        arch: process.arch,
      };
      console.log(JSON.stringify(versionInfo, null, 2));
    });

  return program;
}

// Run CLI if invoked directly
const isMainModule = process.argv[1] === fileURLToPath(import.meta.url);
if (isMainModule) {
  const program = createCLI();
  program.parse(process.argv);
}
