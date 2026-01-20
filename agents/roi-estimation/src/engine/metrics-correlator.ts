/**
 * ROI Estimation Agent - Metrics Correlator
 *
 * Statistical correlation analysis between cost and outcome metrics
 * Supports Pearson, Spearman, and Kendall correlation methods
 */

import {
  type CorrelationResult,
  type CorrelationInput,
  type CorrelationConfig,
  type CostRecord,
  type OutcomeMetric,
  type OutcomeAggregation
} from '../contracts/index.js';

export class MetricsCorrelator {
  /**
   * Calculate correlation between cost and outcome values
   */
  calculateCorrelation(
    input: CorrelationInput,
    config: CorrelationConfig,
    metricName: string,
    metricType: OutcomeMetric['metric_type']
  ): CorrelationResult {
    const { costValues, outcomeValues } = input;
    const n = Math.min(costValues.length, outcomeValues.length);

    if (n < 3) {
      return this.createInsufficientDataResult(metricName, metricType, n);
    }

    // Trim to equal lengths
    const costs = costValues.slice(0, n);
    const outcomes = outcomeValues.slice(0, n);

    // Calculate correlation coefficient based on method
    let coefficient: number;
    switch (config.method) {
      case 'spearman':
        coefficient = this.spearmanCorrelation(costs, outcomes);
        break;
      case 'kendall':
        coefficient = this.kendallCorrelation(costs, outcomes);
        break;
      case 'pearson':
      default:
        coefficient = this.pearsonCorrelation(costs, outcomes);
        break;
    }

    // Calculate p-value (simplified approximation)
    const pValue = this.calculatePValue(coefficient, n);

    // Determine significance
    const isSignificant = pValue < config.significanceLevel;

    // Determine correlation strength
    const correlationStrength = this.classifyCorrelationStrength(coefficient);

    return {
      metric_name: metricName,
      metric_type: metricType,
      correlation_coefficient: Number(coefficient.toFixed(6)),
      correlation_strength: correlationStrength,
      p_value: Number(pValue.toFixed(6)),
      sample_size: n,
      is_significant: isSignificant
    };
  }

  /**
   * Batch calculate correlations for multiple outcome metrics
   */
  calculateMultipleCorrelations(
    costTimeSeries: { timestamp: string; value: number }[],
    outcomeTimeSeries: {
      metricName: string;
      metricType: OutcomeMetric['metric_type'];
      data: { timestamp: string; value: number }[];
    }[],
    config: CorrelationConfig
  ): CorrelationResult[] {
    const results: CorrelationResult[] = [];

    for (const outcome of outcomeTimeSeries) {
      // Align time series data
      const aligned = this.alignTimeSeries(costTimeSeries, outcome.data);

      if (aligned.costValues.length >= 3) {
        const result = this.calculateCorrelation(
          {
            costValues: aligned.costValues,
            outcomeValues: aligned.outcomeValues,
            timestamps: aligned.timestamps
          },
          config,
          outcome.metricName,
          outcome.metricType
        );
        results.push(result);
      } else {
        results.push(
          this.createInsufficientDataResult(
            outcome.metricName,
            outcome.metricType,
            aligned.costValues.length
          )
        );
      }
    }

    return results;
  }

  /**
   * Prepare correlation data from cost records and outcome metrics
   */
  prepareCorrelationData(
    costRecords: CostRecord[],
    outcomeMetrics: OutcomeMetric[],
    windowHours: number = 24
  ): {
    costTimeSeries: { timestamp: string; value: number }[];
    outcomeTimeSeries: {
      metricName: string;
      metricType: OutcomeMetric['metric_type'];
      data: { timestamp: string; value: number }[];
    }[];
  } {
    // Aggregate costs by time window
    const costByWindow = this.aggregateByTimeWindow(
      costRecords.map(c => ({
        timestamp: c.timestamp,
        value: parseFloat(c.cost_amount)
      })),
      windowHours
    );

    // Group outcome metrics by name and aggregate by time window
    const outcomeGroups = new Map<
      string,
      {
        metricType: OutcomeMetric['metric_type'];
        data: { timestamp: string; value: number }[];
      }
    >();

    for (const metric of outcomeMetrics) {
      const existing = outcomeGroups.get(metric.metric_name);
      if (existing) {
        existing.data.push({ timestamp: metric.timestamp, value: metric.value });
      } else {
        outcomeGroups.set(metric.metric_name, {
          metricType: metric.metric_type,
          data: [{ timestamp: metric.timestamp, value: metric.value }]
        });
      }
    }

    // Aggregate each outcome group by time window
    const outcomeTimeSeries = Array.from(outcomeGroups.entries()).map(
      ([metricName, { metricType, data }]) => ({
        metricName,
        metricType,
        data: this.aggregateByTimeWindow(data, windowHours)
      })
    );

    return {
      costTimeSeries: costByWindow,
      outcomeTimeSeries
    };
  }

  // ============================================================================
  // CORRELATION METHODS
  // ============================================================================

  /**
   * Pearson correlation coefficient
   * Measures linear relationship between two variables
   */
  private pearsonCorrelation(x: number[], y: number[]): number {
    const n = x.length;
    if (n === 0) return 0;

    const sumX = x.reduce((a, b) => a + b, 0);
    const sumY = y.reduce((a, b) => a + b, 0);
    const sumXY = x.reduce((total, xi, i) => total + xi * y[i], 0);
    const sumX2 = x.reduce((total, xi) => total + xi * xi, 0);
    const sumY2 = y.reduce((total, yi) => total + yi * yi, 0);

    const numerator = n * sumXY - sumX * sumY;
    const denominator = Math.sqrt(
      (n * sumX2 - sumX * sumX) * (n * sumY2 - sumY * sumY)
    );

    if (denominator === 0) return 0;
    return numerator / denominator;
  }

  /**
   * Spearman rank correlation coefficient
   * Measures monotonic relationship using ranks
   */
  private spearmanCorrelation(x: number[], y: number[]): number {
    const n = x.length;
    if (n === 0) return 0;

    // Convert to ranks
    const rankX = this.toRanks(x);
    const rankY = this.toRanks(y);

    // Calculate Pearson correlation on ranks
    return this.pearsonCorrelation(rankX, rankY);
  }

  /**
   * Kendall tau correlation coefficient
   * Measures ordinal association based on concordant/discordant pairs
   */
  private kendallCorrelation(x: number[], y: number[]): number {
    const n = x.length;
    if (n < 2) return 0;

    let concordant = 0;
    let discordant = 0;

    for (let i = 0; i < n - 1; i++) {
      for (let j = i + 1; j < n; j++) {
        const xDiff = x[j] - x[i];
        const yDiff = y[j] - y[i];
        const product = xDiff * yDiff;

        if (product > 0) {
          concordant++;
        } else if (product < 0) {
          discordant++;
        }
        // Ties (product === 0) are neither concordant nor discordant
      }
    }

    const totalPairs = (n * (n - 1)) / 2;
    if (totalPairs === 0) return 0;

    return (concordant - discordant) / totalPairs;
  }

  /**
   * Convert values to ranks (for Spearman correlation)
   */
  private toRanks(values: number[]): number[] {
    const indexed = values.map((value, index) => ({ value, index }));
    indexed.sort((a, b) => a.value - b.value);

    const ranks = new Array(values.length);
    let rank = 1;

    for (let i = 0; i < indexed.length; i++) {
      // Handle ties by averaging ranks
      let j = i;
      while (j < indexed.length - 1 && indexed[j].value === indexed[j + 1].value) {
        j++;
      }

      const avgRank = (rank + rank + (j - i)) / 2;
      for (let k = i; k <= j; k++) {
        ranks[indexed[k].index] = avgRank;
      }

      rank += j - i + 1;
      i = j;
    }

    return ranks;
  }

  // ============================================================================
  // STATISTICAL HELPERS
  // ============================================================================

  /**
   * Calculate approximate p-value for correlation coefficient
   * Using t-distribution approximation
   */
  private calculatePValue(r: number, n: number): number {
    if (n < 3) return 1;
    if (Math.abs(r) === 1) return 0;

    // t-statistic
    const t = r * Math.sqrt((n - 2) / (1 - r * r));

    // Approximate p-value using normal distribution for large n
    // For more accurate results, use a proper t-distribution CDF
    const df = n - 2;

    if (df >= 30) {
      // Use normal approximation
      return 2 * (1 - this.normalCDF(Math.abs(t)));
    }

    // Simplified approximation for smaller samples
    const x = df / (df + t * t);
    const pValue = this.incompleteBeta(df / 2, 0.5, x);
    return Math.min(1, Math.max(0, pValue));
  }

  /**
   * Standard normal CDF approximation
   */
  private normalCDF(x: number): number {
    const a1 = 0.254829592;
    const a2 = -0.284496736;
    const a3 = 1.421413741;
    const a4 = -1.453152027;
    const a5 = 1.061405429;
    const p = 0.3275911;

    const sign = x < 0 ? -1 : 1;
    x = Math.abs(x) / Math.sqrt(2);

    const t = 1.0 / (1.0 + p * x);
    const y = 1.0 - ((((a5 * t + a4) * t + a3) * t + a2) * t + a1) * t * Math.exp(-x * x);

    return 0.5 * (1.0 + sign * y);
  }

  /**
   * Incomplete beta function approximation
   */
  private incompleteBeta(a: number, b: number, x: number): number {
    // Simple approximation - for production use a proper implementation
    if (x === 0) return 0;
    if (x === 1) return 1;

    // Use continued fraction approximation
    const maxIterations = 100;
    const epsilon = 1e-10;

    let result = Math.exp(
      a * Math.log(x) + b * Math.log(1 - x) -
      Math.log(a) - this.logBeta(a, b)
    );

    let sum = 0;
    let term = 1;

    for (let n = 0; n < maxIterations; n++) {
      term *= (a + n) * x / (a + b + n);
      sum += term / (a + n + 1);
      if (Math.abs(term) < epsilon) break;
    }

    return result * (1 + a * sum);
  }

  /**
   * Log beta function
   */
  private logBeta(a: number, b: number): number {
    return this.logGamma(a) + this.logGamma(b) - this.logGamma(a + b);
  }

  /**
   * Log gamma function approximation (Lanczos)
   */
  private logGamma(x: number): number {
    const g = 7;
    const coefficients = [
      0.99999999999980993,
      676.5203681218851,
      -1259.1392167224028,
      771.32342877765313,
      -176.61502916214059,
      12.507343278686905,
      -0.13857109526572012,
      9.9843695780195716e-6,
      1.5056327351493116e-7
    ];

    if (x < 0.5) {
      return Math.log(Math.PI / Math.sin(Math.PI * x)) - this.logGamma(1 - x);
    }

    x -= 1;
    let a = coefficients[0];
    const t = x + g + 0.5;

    for (let i = 1; i < g + 2; i++) {
      a += coefficients[i] / (x + i);
    }

    return 0.5 * Math.log(2 * Math.PI) + (x + 0.5) * Math.log(t) - t + Math.log(a);
  }

  /**
   * Classify correlation strength
   */
  private classifyCorrelationStrength(
    coefficient: number
  ): CorrelationResult['correlation_strength'] {
    const abs = Math.abs(coefficient);

    if (abs >= 0.7) {
      return coefficient > 0 ? 'strong_positive' : 'strong_negative';
    }
    if (abs >= 0.4) {
      return coefficient > 0 ? 'moderate_positive' : 'moderate_negative';
    }
    if (abs >= 0.2) {
      return coefficient > 0 ? 'weak_positive' : 'weak_negative';
    }
    return 'none';
  }

  // ============================================================================
  // TIME SERIES HELPERS
  // ============================================================================

  /**
   * Aggregate time series data by time window
   */
  private aggregateByTimeWindow(
    data: { timestamp: string; value: number }[],
    windowHours: number
  ): { timestamp: string; value: number }[] {
    if (data.length === 0) return [];

    const windowMs = windowHours * 60 * 60 * 1000;
    const buckets = new Map<number, { sum: number; count: number }>();

    for (const point of data) {
      const ts = new Date(point.timestamp).getTime();
      const bucket = Math.floor(ts / windowMs) * windowMs;

      const existing = buckets.get(bucket);
      if (existing) {
        existing.sum += point.value;
        existing.count++;
      } else {
        buckets.set(bucket, { sum: point.value, count: 1 });
      }
    }

    return Array.from(buckets.entries())
      .sort((a, b) => a[0] - b[0])
      .map(([ts, { sum, count }]) => ({
        timestamp: new Date(ts).toISOString(),
        value: sum / count // Average within window
      }));
  }

  /**
   * Align two time series by matching timestamps
   */
  private alignTimeSeries(
    series1: { timestamp: string; value: number }[],
    series2: { timestamp: string; value: number }[]
  ): { costValues: number[]; outcomeValues: number[]; timestamps: string[] } {
    const map1 = new Map(series1.map(s => [s.timestamp, s.value]));
    const map2 = new Map(series2.map(s => [s.timestamp, s.value]));

    const commonTimestamps = series1
      .filter(s => map2.has(s.timestamp))
      .map(s => s.timestamp)
      .sort();

    return {
      costValues: commonTimestamps.map(t => map1.get(t)!),
      outcomeValues: commonTimestamps.map(t => map2.get(t)!),
      timestamps: commonTimestamps
    };
  }

  /**
   * Create result for insufficient data
   */
  private createInsufficientDataResult(
    metricName: string,
    metricType: OutcomeMetric['metric_type'],
    sampleSize: number
  ): CorrelationResult {
    return {
      metric_name: metricName,
      metric_type: metricType,
      correlation_coefficient: 0,
      correlation_strength: 'none',
      p_value: 1,
      sample_size: sampleSize,
      is_significant: false
    };
  }
}

// Export singleton instance
export const metricsCorrelator = new MetricsCorrelator();
