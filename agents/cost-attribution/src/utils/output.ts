import Table from 'cli-table3';
import chalk from 'chalk';
import { CostRecord } from '../engine/calculator.js';

/**
 * Output format types
 */
export type OutputFormat = 'json' | 'table';

/**
 * Output formatter for CLI results
 */
export class OutputFormatter {
  /**
   * Format cost records for output
   */
  formatCostRecords(records: CostRecord[], format: OutputFormat): string {
    if (format === 'json') {
      return this.formatAsJson(records);
    }
    return this.formatAsTable(records);
  }

  /**
   * Format single cost record for output
   */
  formatCostRecord(record: CostRecord, format: OutputFormat): string {
    if (format === 'json') {
      return this.formatAsJson([record]);
    }
    return this.formatAsTable([record]);
  }

  /**
   * Format summary statistics
   */
  formatSummary(records: CostRecord[], format: OutputFormat): string {
    const summary = this.calculateSummary(records);

    if (format === 'json') {
      return JSON.stringify(summary, null, 2);
    }

    const table = new Table({
      head: [chalk.cyan('Metric'), chalk.cyan('Value')],
      colWidths: [30, 30],
    });

    table.push(
      ['Total Records', summary.totalRecords.toString()],
      ['Total Cost', `${summary.totalCost} ${summary.currency}`],
      ['Avg Cost per Record', `${summary.avgCostPerRecord} ${summary.currency}`],
      ['Total Input Tokens', summary.totalInputTokens.toLocaleString()],
      ['Total Output Tokens', summary.totalOutputTokens.toLocaleString()],
      ['Total Cached Tokens', summary.totalCachedTokens.toLocaleString()],
      ['Unique Providers', summary.uniqueProviders.toString()],
      ['Unique Models', summary.uniqueModels.toString()]
    );

    return table.toString();
  }

  private formatAsJson(records: CostRecord[]): string {
    return JSON.stringify(records, null, 2);
  }

  private formatAsTable(records: CostRecord[]): string {
    const table = new Table({
      head: [
        chalk.cyan('Execution ID'),
        chalk.cyan('Agent'),
        chalk.cyan('Provider'),
        chalk.cyan('Model'),
        chalk.cyan('Input Tokens'),
        chalk.cyan('Output Tokens'),
        chalk.cyan('Total Cost'),
        chalk.cyan('Currency'),
      ],
      colWidths: [20, 15, 12, 20, 15, 15, 15, 10],
    });

    for (const record of records) {
      table.push([
        record.executionId.substring(0, 18),
        record.agentId.substring(0, 13),
        record.provider,
        record.model.substring(0, 18),
        record.inputTokens.toLocaleString(),
        record.outputTokens.toLocaleString(),
        parseFloat(record.totalCost).toFixed(6),
        record.currency,
      ]);
    }

    return table.toString();
  }

  private calculateSummary(records: CostRecord[]) {
    const totalCost = records.reduce(
      (sum, r) => sum + parseFloat(r.totalCost),
      0
    );

    const totalInputTokens = records.reduce(
      (sum, r) => sum + r.inputTokens,
      0
    );

    const totalOutputTokens = records.reduce(
      (sum, r) => sum + r.outputTokens,
      0
    );

    const totalCachedTokens = records.reduce(
      (sum, r) => sum + r.cachedInputTokens,
      0
    );

    const providers = new Set(records.map(r => r.provider));
    const models = new Set(records.map(r => r.model));

    return {
      totalRecords: records.length,
      totalCost: totalCost.toFixed(6),
      avgCostPerRecord: records.length > 0 ? (totalCost / records.length).toFixed(6) : '0.000000',
      currency: records[0]?.currency || 'USD',
      totalInputTokens,
      totalOutputTokens,
      totalCachedTokens,
      uniqueProviders: providers.size,
      uniqueModels: models.size,
    };
  }
}
