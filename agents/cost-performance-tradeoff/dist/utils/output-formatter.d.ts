/**
 * Output Formatter Utility
 *
 * Formats analysis results for human-readable or machine-readable output.
 */
import type { TradeoffAnalysisOutput } from '../contracts/index.js';
export type OutputFormat = 'json' | 'table';
export declare class OutputFormatter {
    /**
     * Format analysis output
     */
    format(output: TradeoffAnalysisOutput, format: OutputFormat): string;
    /**
     * Format as human-readable table
     */
    private formatTable;
    private formatResultsHeader;
    private formatResultRow;
    /**
     * Format summary only
     */
    formatSummary(output: TradeoffAnalysisOutput): string;
}
//# sourceMappingURL=output-formatter.d.ts.map