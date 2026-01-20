/**
 * ROI Estimation Agent - ROI Calculator Engine
 *
 * Core ROI calculation logic using Decimal.js for precision
 * Classification: ROI ANALYSIS
 */

import Decimal from 'decimal.js';
import {
  type ROIMetric,
  type ROICalculationInput,
  type EfficiencyMetric,
  type ROIRecommendation,
  type CorrelationResult,
  type CostRecord,
  type CostAggregation,
  type OutcomeMetric,
  type OutcomeAggregation
} from '../contracts/index.js';

// Configure Decimal.js for financial precision
Decimal.set({
  precision: 20,
  rounding: Decimal.ROUND_HALF_UP
});

export class ROICalculatorEngine {
  /**
   * Calculate ROI metrics from cost and gain data
   *
   * ROI = (Gain - Cost) / Cost × 100
   */
  calculateROI(input: ROICalculationInput): ROIMetric {
    const totalCost = new Decimal(input.totalCost);
    const totalGain = new Decimal(input.totalGain);

    // Prevent division by zero
    if (totalCost.isZero()) {
      return {
        roi_percentage: totalGain.isPositive() ? Infinity : 0,
        roi_ratio: totalGain.isPositive() ? Infinity : 0,
        net_value: totalGain.toString(),
        total_cost: '0',
        total_gain: totalGain.toString(),
        cost_per_outcome_unit: '0',
        gain_per_cost_unit: totalGain.isPositive() ? 'Infinity' : '0',
        payback_period_days: totalGain.isPositive() ? 0 : undefined,
        break_even_point: undefined
      };
    }

    // Calculate core metrics
    const netValue = totalGain.minus(totalCost);
    const roiRatio = totalGain.dividedBy(totalCost);
    const roiPercentage = netValue.dividedBy(totalCost).times(100);
    const costPerOutcomeUnit = totalGain.isZero()
      ? new Decimal(0)
      : totalCost.dividedBy(totalGain);
    const gainPerCostUnit = roiRatio;

    // Calculate payback period if we have period info
    let paybackPeriodDays: number | undefined;
    if (netValue.isPositive() && input.periodDays > 0) {
      const dailyNetGain = netValue.dividedBy(input.periodDays);
      if (dailyNetGain.isPositive()) {
        paybackPeriodDays = totalCost.dividedBy(dailyNetGain).toNumber();
      }
    }

    // Calculate break-even point (cost level where ROI = 0)
    const breakEvenPoint = totalGain.toString();

    return {
      roi_percentage: roiPercentage.toNumber(),
      roi_ratio: roiRatio.toNumber(),
      net_value: netValue.toFixed(6),
      total_cost: totalCost.toFixed(6),
      total_gain: totalGain.toFixed(6),
      cost_per_outcome_unit: costPerOutcomeUnit.toFixed(6),
      gain_per_cost_unit: gainPerCostUnit.toFixed(6),
      payback_period_days: paybackPeriodDays,
      break_even_point: breakEvenPoint
    };
  }

  /**
   * Calculate efficiency metrics from costs and outcomes
   */
  calculateEfficiency(
    costs: CostRecord[] | CostAggregation,
    outcomes: OutcomeMetric[] | OutcomeAggregation[]
  ): EfficiencyMetric[] {
    const efficiencyMetrics: EfficiencyMetric[] = [];

    // Extract total cost
    const totalCost = Array.isArray(costs)
      ? this.aggregateCostRecords(costs)
      : new Decimal(costs.total_cost);

    // Process each outcome metric type
    const outcomeGroups = this.groupOutcomesByType(outcomes);

    for (const [metricName, outcomeData] of outcomeGroups) {
      const totalOutcome = new Decimal(outcomeData.totalValue);
      const unit = outcomeData.unit;

      if (totalCost.isZero() || totalOutcome.isZero()) {
        efficiencyMetrics.push({
          metric_name: metricName,
          current_efficiency: 0,
          baseline_efficiency: undefined,
          efficiency_change_percentage: undefined,
          unit: `${unit} per $`,
          interpretation: 'Insufficient data for efficiency calculation'
        });
        continue;
      }

      // Efficiency = outcome / cost
      const currentEfficiency = totalOutcome.dividedBy(totalCost);

      efficiencyMetrics.push({
        metric_name: metricName,
        current_efficiency: currentEfficiency.toNumber(),
        baseline_efficiency: undefined,
        efficiency_change_percentage: undefined,
        unit: `${unit} per $`,
        interpretation: this.interpretEfficiency(metricName, currentEfficiency.toNumber())
      });
    }

    // Add cost per token efficiency
    if (Array.isArray(costs) && costs.length > 0) {
      const totalTokens = costs.reduce((sum, c) => sum + c.total_tokens, 0);
      if (totalTokens > 0) {
        const costPerThousandTokens = totalCost.dividedBy(totalTokens).times(1000);
        efficiencyMetrics.push({
          metric_name: 'cost_per_1k_tokens',
          current_efficiency: costPerThousandTokens.toNumber(),
          baseline_efficiency: undefined,
          efficiency_change_percentage: undefined,
          unit: '$/1k tokens',
          interpretation: `Current cost is $${costPerThousandTokens.toFixed(4)} per 1,000 tokens`
        });
      }
    }

    return efficiencyMetrics;
  }

  /**
   * Generate recommendations based on ROI analysis
   */
  generateRecommendations(
    roiMetrics: ROIMetric,
    correlations: CorrelationResult[],
    efficiencyMetrics: EfficiencyMetric[]
  ): ROIRecommendation[] {
    const recommendations: ROIRecommendation[] = [];

    // ROI-based recommendations
    if (roiMetrics.roi_percentage < 0) {
      recommendations.push({
        category: 'cost_optimization',
        priority: 'high',
        recommendation: 'Current LLM investment shows negative ROI. Review model selection and optimize prompt engineering to reduce token usage.',
        estimated_impact: `Potential to recover ${Math.abs(roiMetrics.roi_percentage).toFixed(1)}% of costs`,
        confidence: 0.8
      });
    } else if (roiMetrics.roi_percentage < 50) {
      recommendations.push({
        category: 'outcome_improvement',
        priority: 'medium',
        recommendation: 'ROI is positive but below optimal threshold. Consider improving outcome metrics through better model selection or workflow optimization.',
        estimated_impact: 'Potential 20-50% ROI improvement',
        confidence: 0.7
      });
    } else if (roiMetrics.roi_percentage > 200) {
      recommendations.push({
        category: 'scaling_decision',
        priority: 'medium',
        recommendation: 'Excellent ROI indicates opportunity for scaling. Consider increasing LLM investment in high-performing areas.',
        estimated_impact: 'Maintain or improve current returns at higher scale',
        confidence: 0.75
      });
    }

    // Correlation-based recommendations
    const strongPositiveCorrelations = correlations.filter(
      c => c.correlation_strength === 'strong_positive' && c.is_significant
    );
    const strongNegativeCorrelations = correlations.filter(
      c => c.correlation_strength === 'strong_negative' && c.is_significant
    );

    if (strongPositiveCorrelations.length > 0) {
      recommendations.push({
        category: 'process_improvement',
        priority: 'low',
        recommendation: `Strong positive correlation found between cost and: ${strongPositiveCorrelations.map(c => c.metric_name).join(', ')}. Higher investment correlates with better outcomes.`,
        estimated_impact: 'Data-driven basis for investment decisions',
        confidence: Math.max(...strongPositiveCorrelations.map(c => Math.abs(c.correlation_coefficient)))
      });
    }

    if (strongNegativeCorrelations.length > 0) {
      recommendations.push({
        category: 'cost_optimization',
        priority: 'high',
        recommendation: `Negative correlation detected: ${strongNegativeCorrelations.map(c => c.metric_name).join(', ')}. Review these areas for potential cost reduction without outcome degradation.`,
        estimated_impact: 'Potential cost savings identified',
        confidence: Math.max(...strongNegativeCorrelations.map(c => Math.abs(c.correlation_coefficient)))
      });
    }

    // Efficiency-based recommendations
    const tokenCostMetric = efficiencyMetrics.find(e => e.metric_name === 'cost_per_1k_tokens');
    if (tokenCostMetric && tokenCostMetric.current_efficiency > 0.01) {
      recommendations.push({
        category: 'model_selection',
        priority: 'medium',
        recommendation: `Current cost per 1K tokens ($${tokenCostMetric.current_efficiency.toFixed(4)}) may be optimizable. Consider evaluating smaller models or cached responses for appropriate use cases.`,
        estimated_impact: '10-40% potential cost reduction',
        confidence: 0.65
      });
    }

    // Budget recommendations
    if (roiMetrics.payback_period_days !== undefined) {
      if (roiMetrics.payback_period_days > 90) {
        recommendations.push({
          category: 'budget_adjustment',
          priority: 'medium',
          recommendation: `Payback period of ${Math.round(roiMetrics.payback_period_days)} days exceeds 90-day threshold. Review budget allocation and expected returns.`,
          estimated_impact: 'Better budget utilization',
          confidence: 0.7
        });
      } else if (roiMetrics.payback_period_days < 7) {
        recommendations.push({
          category: 'scaling_decision',
          priority: 'low',
          recommendation: `Rapid payback period of ${Math.round(roiMetrics.payback_period_days)} days indicates high efficiency. Consider scaling this investment.`,
          estimated_impact: 'Accelerated returns through scaling',
          confidence: 0.75
        });
      }
    }

    // Sort by priority
    const priorityOrder = { high: 0, medium: 1, low: 2 };
    return recommendations.sort((a, b) => priorityOrder[a.priority] - priorityOrder[b.priority]);
  }

  /**
   * Calculate overall assessment from metrics
   */
  calculateOverallAssessment(
    roiMetrics: ROIMetric,
    correlations: CorrelationResult[],
    dataQualityScore: number
  ): {
    overall_assessment: 'highly_positive' | 'positive' | 'neutral' | 'negative' | 'highly_negative' | 'insufficient_data';
    key_insight: string;
    confidence_level: 'high' | 'medium' | 'low';
  } {
    // Check data quality first
    if (dataQualityScore < 0.3) {
      return {
        overall_assessment: 'insufficient_data',
        key_insight: 'Insufficient data quality for reliable ROI assessment. Collect more data points.',
        confidence_level: 'low'
      };
    }

    // Determine confidence based on data quality and correlation significance
    const significantCorrelations = correlations.filter(c => c.is_significant);
    let confidenceLevel: 'high' | 'medium' | 'low';
    if (dataQualityScore > 0.8 && significantCorrelations.length > 0) {
      confidenceLevel = 'high';
    } else if (dataQualityScore > 0.5) {
      confidenceLevel = 'medium';
    } else {
      confidenceLevel = 'low';
    }

    // Determine assessment based on ROI
    let overallAssessment: 'highly_positive' | 'positive' | 'neutral' | 'negative' | 'highly_negative';
    let keyInsight: string;

    if (roiMetrics.roi_percentage >= 100) {
      overallAssessment = 'highly_positive';
      keyInsight = `Excellent ROI of ${roiMetrics.roi_percentage.toFixed(1)}%. LLM investment is generating strong returns.`;
    } else if (roiMetrics.roi_percentage >= 25) {
      overallAssessment = 'positive';
      keyInsight = `Good ROI of ${roiMetrics.roi_percentage.toFixed(1)}%. LLM investment is profitable.`;
    } else if (roiMetrics.roi_percentage >= -10) {
      overallAssessment = 'neutral';
      keyInsight = `ROI of ${roiMetrics.roi_percentage.toFixed(1)}% is near break-even. Monitor closely and optimize where possible.`;
    } else if (roiMetrics.roi_percentage >= -50) {
      overallAssessment = 'negative';
      keyInsight = `Negative ROI of ${roiMetrics.roi_percentage.toFixed(1)}%. Review cost structure and outcome alignment.`;
    } else {
      overallAssessment = 'highly_negative';
      keyInsight = `Severely negative ROI of ${roiMetrics.roi_percentage.toFixed(1)}%. Immediate action required to address cost-outcome imbalance.`;
    }

    return {
      overall_assessment: overallAssessment,
      key_insight: keyInsight,
      confidence_level: confidenceLevel
    };
  }

  /**
   * Calculate data quality score
   */
  calculateDataQualityScore(
    costRecordCount: number,
    outcomeMetricCount: number,
    correlations: CorrelationResult[]
  ): number {
    let score = 0;

    // Data volume scoring (0-0.4)
    const volumeScore = Math.min(0.4, (costRecordCount + outcomeMetricCount) / 250);
    score += volumeScore;

    // Correlation quality scoring (0-0.3)
    const significantCorrelations = correlations.filter(c => c.is_significant).length;
    const correlationScore = Math.min(0.3, significantCorrelations * 0.1);
    score += correlationScore;

    // Sample size scoring (0-0.3)
    const avgSampleSize = correlations.length > 0
      ? correlations.reduce((sum, c) => sum + c.sample_size, 0) / correlations.length
      : 0;
    const sampleSizeScore = Math.min(0.3, avgSampleSize / 100);
    score += sampleSizeScore;

    return Math.min(1, score);
  }

  // ============================================================================
  // PRIVATE HELPERS
  // ============================================================================

  private aggregateCostRecords(costs: CostRecord[]): Decimal {
    return costs.reduce(
      (sum, cost) => sum.plus(new Decimal(cost.cost_amount)),
      new Decimal(0)
    );
  }

  private groupOutcomesByType(
    outcomes: OutcomeMetric[] | OutcomeAggregation[]
  ): Map<string, { totalValue: number; unit: string }> {
    const groups = new Map<string, { totalValue: number; unit: string }>();

    for (const outcome of outcomes) {
      const metricName = outcome.metric_name;
      const existing = groups.get(metricName);

      const value = 'aggregate_value' in outcome
        ? outcome.aggregate_value
        : outcome.value;

      const unit = 'unit' in outcome ? outcome.unit : 'units';

      if (existing) {
        existing.totalValue += value;
      } else {
        groups.set(metricName, { totalValue: value, unit });
      }
    }

    return groups;
  }

  private interpretEfficiency(metricName: string, efficiency: number): string {
    const formatted = efficiency.toFixed(4);

    // Common interpretations based on metric type
    if (metricName.includes('success') || metricName.includes('completion')) {
      return `Achieving ${formatted} successful outcomes per dollar spent`;
    }
    if (metricName.includes('throughput')) {
      return `Processing ${formatted} operations per dollar spent`;
    }
    if (metricName.includes('quality') || metricName.includes('score')) {
      return `Quality score of ${formatted} achieved per dollar spent`;
    }
    if (metricName.includes('revenue')) {
      return `Generating $${formatted} revenue per dollar of LLM cost`;
    }

    return `Efficiency rate of ${formatted} ${metricName} per dollar spent`;
  }
}

// Export singleton instance
export const roiCalculator = new ROICalculatorEngine();
