/**
 * Input Reader Utility
 *
 * Reads and validates input from stdin, files, or direct JSON.
 */
import type { TradeoffAnalysisInput, PerformanceRecord } from '../contracts/index.js';
export declare class InputReader {
    /**
     * Read input from the specified source
     */
    readInput(source: string): Promise<TradeoffAnalysisInput>;
    /**
     * Read performance records from JSONL format
     */
    readRecordsJSONL(source: string): Promise<PerformanceRecord[]>;
    /**
     * Read from stdin
     */
    private readStdin;
}
//# sourceMappingURL=input-reader.d.ts.map