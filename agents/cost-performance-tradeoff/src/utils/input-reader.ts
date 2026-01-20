/**
 * Input Reader Utility
 *
 * Reads and validates input from stdin, files, or direct JSON.
 */

import * as fs from 'fs/promises';
import * as readline from 'readline';
import { TradeoffAnalysisInputSchema } from '../contracts/index.js';
import type { TradeoffAnalysisInput, PerformanceRecord } from '../contracts/index.js';

export class InputReader {
  /**
   * Read input from the specified source
   */
  async readInput(source: string): Promise<TradeoffAnalysisInput> {
    let rawData: string;

    if (source === '-') {
      // Read from stdin
      rawData = await this.readStdin();
    } else if (source.startsWith('{')) {
      // Direct JSON string
      rawData = source;
    } else {
      // File path
      rawData = await fs.readFile(source, 'utf-8');
    }

    // Parse and validate
    const parsed = JSON.parse(rawData);
    return TradeoffAnalysisInputSchema.parse(parsed);
  }

  /**
   * Read performance records from JSONL format
   */
  async readRecordsJSONL(source: string): Promise<PerformanceRecord[]> {
    let lines: string[];

    if (source === '-') {
      const data = await this.readStdin();
      lines = data.trim().split('\n');
    } else {
      const data = await fs.readFile(source, 'utf-8');
      lines = data.trim().split('\n');
    }

    const records: PerformanceRecord[] = [];
    for (const line of lines) {
      if (line.trim()) {
        records.push(JSON.parse(line));
      }
    }

    return records;
  }

  /**
   * Read from stdin
   */
  private async readStdin(): Promise<string> {
    return new Promise((resolve, reject) => {
      const chunks: string[] = [];

      const rl = readline.createInterface({
        input: process.stdin,
        terminal: false
      });

      rl.on('line', (line) => {
        chunks.push(line);
      });

      rl.on('close', () => {
        resolve(chunks.join('\n'));
      });

      rl.on('error', reject);

      // Set a timeout for stdin read
      setTimeout(() => {
        if (chunks.length === 0 && !process.stdin.isTTY) {
          reject(new Error('Timeout waiting for stdin input'));
        }
      }, 30000);
    });
  }
}
