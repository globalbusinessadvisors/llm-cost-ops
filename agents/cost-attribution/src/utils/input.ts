import * as fs from 'fs';
import * as readline from 'readline';
import { UsageRecord } from '../engine/calculator.js';

/**
 * Input reader for CLI commands
 * Supports reading from files or stdin
 */
export class InputReader {
  /**
   * Read usage records from file or stdin
   */
  async readUsageRecords(source: string | null): Promise<UsageRecord[]> {
    if (!source || source === '-') {
      return this.readFromStdin();
    }
    return this.readFromFile(source);
  }

  /**
   * Read usage records from stdin (JSONL format)
   */
  private async readFromStdin(): Promise<UsageRecord[]> {
    const records: UsageRecord[] = [];
    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
      terminal: false,
    });

    for await (const line of rl) {
      if (line.trim()) {
        try {
          const record = this.parseUsageRecord(JSON.parse(line));
          records.push(record);
        } catch (error) {
          throw new Error(`Invalid JSON line: ${line}`);
        }
      }
    }

    return records;
  }

  /**
   * Read usage records from file (JSON or JSONL format)
   */
  private async readFromFile(filePath: string): Promise<UsageRecord[]> {
    if (!fs.existsSync(filePath)) {
      throw new Error(`File not found: ${filePath}`);
    }

    const content = fs.readFileSync(filePath, 'utf-8');

    // Try parsing as JSON array first
    try {
      const json = JSON.parse(content);
      if (Array.isArray(json)) {
        return json.map(r => this.parseUsageRecord(r));
      }
      // Single record
      return [this.parseUsageRecord(json)];
    } catch {
      // Try parsing as JSONL
      const lines = content.split('\n').filter(l => l.trim());
      return lines.map(line => this.parseUsageRecord(JSON.parse(line)));
    }
  }

  /**
   * Parse and validate a usage record
   */
  private parseUsageRecord(data: any): UsageRecord {
    // Convert timestamp string to Date if needed
    const timestamp = typeof data.timestamp === 'string'
      ? new Date(data.timestamp)
      : data.timestamp;

    return {
      executionId: data.executionId,
      agentId: data.agentId,
      workflowId: data.workflowId,
      tenantId: data.tenantId,
      provider: data.provider,
      model: data.model,
      inputTokens: data.inputTokens,
      outputTokens: data.outputTokens,
      cachedInputTokens: data.cachedInputTokens,
      requestCount: data.requestCount || 1,
      timestamp,
      metadata: data.metadata,
    };
  }
}
