/**
 * Output Formatter Utility
 *
 * Formats analysis results for human-readable or machine-readable output.
 */
export class OutputFormatter {
    /**
     * Format analysis output
     */
    format(output, format) {
        switch (format) {
            case 'json':
                return JSON.stringify(output, null, 2);
            case 'table':
                return this.formatTable(output);
            default:
                return JSON.stringify(output, null, 2);
        }
    }
    /**
     * Format as human-readable table
     */
    formatTable(output) {
        const lines = [];
        // Header
        lines.push('');
        lines.push('='.repeat(80));
        lines.push('COST-PERFORMANCE TRADEOFF ANALYSIS');
        lines.push('='.repeat(80));
        lines.push('');
        // Summary
        lines.push('SUMMARY');
        lines.push('-'.repeat(40));
        lines.push(`Analysis ID:        ${output.analysis_id}`);
        lines.push(`Analyzed At:        ${output.analyzed_at}`);
        lines.push(`Scope:              ${output.analysis_scope}`);
        lines.push(`Records Analyzed:   ${output.summary.total_records_analyzed}`);
        lines.push(`Unique Models:      ${output.summary.unique_models}`);
        lines.push(`Unique Providers:   ${output.summary.unique_providers}`);
        lines.push('');
        // Best performers
        lines.push('BEST PERFORMERS');
        lines.push('-'.repeat(40));
        if (output.summary.best_overall) {
            lines.push(`Best Overall:       ${output.summary.best_overall}`);
        }
        if (output.summary.best_cost_efficiency) {
            lines.push(`Best Cost Efficiency: ${output.summary.best_cost_efficiency}`);
        }
        if (output.summary.best_latency) {
            lines.push(`Best Latency:       ${output.summary.best_latency}`);
        }
        if (output.summary.best_quality) {
            lines.push(`Best Quality:       ${output.summary.best_quality}`);
        }
        lines.push('');
        // Results table
        lines.push('TRADEOFF SCORES');
        lines.push('-'.repeat(80));
        lines.push(this.formatResultsHeader());
        lines.push('-'.repeat(80));
        for (const result of output.results) {
            lines.push(this.formatResultRow(result));
        }
        lines.push('');
        // Pareto frontier
        if (output.pareto_frontier && output.pareto_frontier.length > 0) {
            lines.push('PARETO OPTIMAL (EFFICIENT FRONTIER)');
            lines.push('-'.repeat(40));
            for (const point of output.pareto_frontier) {
                lines.push(`  - ${point.model} (${point.provider})`);
                lines.push(`    Cost: $${point.cost.toFixed(4)} | Latency: ${point.latency.toFixed(0)}ms` +
                    (point.quality !== undefined ? ` | Quality: ${(point.quality * 100).toFixed(1)}%` : ''));
            }
            lines.push('');
        }
        // Diminishing returns
        if (output.diminishing_returns) {
            lines.push('DIMINISHING RETURNS ANALYSIS');
            lines.push('-'.repeat(40));
            lines.push(`Detected: ${output.diminishing_returns.detected ? 'Yes' : 'No'}`);
            if (output.diminishing_returns.threshold_cost_usd) {
                lines.push(`Threshold Cost: $${output.diminishing_returns.threshold_cost_usd.toFixed(4)}`);
            }
            lines.push(`Recommendation: ${output.diminishing_returns.recommendation}`);
            lines.push('');
        }
        // Recommendations
        if (output.recommendations && output.recommendations.length > 0) {
            lines.push('RECOMMENDATIONS');
            lines.push('-'.repeat(40));
            for (const rec of output.recommendations) {
                lines.push(`[${rec.recommendation_type.toUpperCase()}]`);
                lines.push(`  Model: ${rec.recommended_model} (${rec.recommended_provider})`);
                lines.push(`  Rationale: ${rec.rationale}`);
                lines.push(`  Impact: Cost ${rec.estimated_impact.cost_change_percent > 0 ? '+' : ''}${rec.estimated_impact.cost_change_percent.toFixed(1)}%` +
                    ` | Latency ${rec.estimated_impact.latency_change_percent > 0 ? '+' : ''}${rec.estimated_impact.latency_change_percent.toFixed(1)}%` +
                    (rec.estimated_impact.quality_change_percent !== undefined
                        ? ` | Quality ${rec.estimated_impact.quality_change_percent > 0 ? '+' : ''}${rec.estimated_impact.quality_change_percent.toFixed(1)}%`
                        : ''));
                lines.push(`  Confidence: ${(rec.confidence * 100).toFixed(0)}%`);
                lines.push('');
            }
        }
        // Constraints applied
        if (output.constraints_applied) {
            lines.push('CONSTRAINTS APPLIED');
            lines.push('-'.repeat(40));
            if (output.constraints_applied.max_cost_per_request_usd) {
                lines.push(`  Max Cost/Request: $${output.constraints_applied.max_cost_per_request_usd}`);
            }
            if (output.constraints_applied.max_latency_p95_ms) {
                lines.push(`  Max Latency P95: ${output.constraints_applied.max_latency_p95_ms}ms`);
            }
            if (output.constraints_applied.min_quality_score) {
                lines.push(`  Min Quality: ${(output.constraints_applied.min_quality_score * 100).toFixed(0)}%`);
            }
            lines.push('');
        }
        // Metadata
        lines.push('METADATA');
        lines.push('-'.repeat(40));
        lines.push(`Weights: Cost=${output.metadata.weights_used.cost.toFixed(2)} ` +
            `Latency=${output.metadata.weights_used.latency.toFixed(2)} ` +
            `Quality=${output.metadata.weights_used.quality.toFixed(2)}`);
        lines.push(`Analysis Duration: ${output.metadata.analysis_duration_ms}ms`);
        lines.push('');
        return lines.join('\n');
    }
    formatResultsHeader() {
        return [
            'Model'.padEnd(25),
            'Provider'.padEnd(12),
            'Cost'.padEnd(10),
            'Latency'.padEnd(10),
            'Quality'.padEnd(10),
            'Overall'.padEnd(10),
            'Records'
        ].join(' ');
    }
    formatResultRow(result) {
        const model = result.model.length > 24
            ? result.model.substring(0, 21) + '...'
            : result.model;
        return [
            model.padEnd(25),
            result.provider.padEnd(12),
            `$${result.avg_cost.cost_per_request_usd.toFixed(4)}`.padEnd(10),
            `${result.avg_latency.p95_ms.toFixed(0)}ms`.padEnd(10),
            (result.avg_quality
                ? `${(result.avg_quality.composite_score * 100).toFixed(0)}%`
                : 'N/A').padEnd(10),
            `${(result.tradeoff_score.overall_score * 100).toFixed(0)}%`.padEnd(10),
            result.record_count.toString()
        ].join(' ');
    }
    /**
     * Format summary only
     */
    formatSummary(output) {
        const lines = [];
        lines.push('');
        lines.push('TRADEOFF ANALYSIS SUMMARY');
        lines.push('='.repeat(40));
        lines.push(`Records: ${output.summary.total_records_analyzed}`);
        lines.push(`Models: ${output.summary.unique_models}`);
        lines.push(`Providers: ${output.summary.unique_providers}`);
        lines.push('');
        lines.push(`Best Overall: ${output.summary.best_overall ?? 'N/A'}`);
        lines.push(`Best Value: ${output.summary.best_cost_efficiency ?? 'N/A'}`);
        lines.push(`Fastest: ${output.summary.best_latency ?? 'N/A'}`);
        lines.push(`Highest Quality: ${output.summary.best_quality ?? 'N/A'}`);
        lines.push('');
        if (output.diminishing_returns?.detected) {
            lines.push(`Diminishing returns detected above $${output.diminishing_returns.threshold_cost_usd?.toFixed(4)}`);
        }
        return lines.join('\n');
    }
}
//# sourceMappingURL=output-formatter.js.map