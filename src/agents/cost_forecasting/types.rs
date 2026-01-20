//! Cost Forecasting Agent Types
//!
//! Input/output schemas for the Cost Forecasting Agent.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::agents::contracts::{
    AgentInput, AgentOutput, ValidationError,
    validation::validators,
};

/// Granularity for forecast projections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForecastGranularity {
    /// Hourly projections
    Hourly,
    /// Daily projections
    Daily,
    /// Weekly projections
    Weekly,
    /// Monthly projections
    Monthly,
}

impl Default for ForecastGranularity {
    fn default() -> Self {
        Self::Daily
    }
}

/// Growth pattern detected in historical data
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GrowthPattern {
    /// Linear growth
    Linear,
    /// Exponential growth
    Exponential,
    /// Stable/flat
    Stable,
    /// Declining trend
    Declining,
    /// Seasonal pattern
    Seasonal,
    /// Volatile/unpredictable
    Volatile,
}

/// Risk level for forecast indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Low risk - costs within normal parameters
    Low,
    /// Medium risk - costs trending upward
    Medium,
    /// High risk - costs exceeding thresholds
    High,
    /// Critical - immediate attention needed
    Critical,
}

/// Historical data point for cost analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalDataPoint {
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,

    /// Total cost for this period
    pub total_cost: Decimal,

    /// Cost breakdown by provider (optional)
    #[serde(default)]
    pub by_provider: std::collections::HashMap<String, Decimal>,

    /// Cost breakdown by model (optional)
    #[serde(default)]
    pub by_model: std::collections::HashMap<String, Decimal>,

    /// Token usage for this period
    pub total_tokens: Option<u64>,

    /// Request count for this period
    pub request_count: Option<u64>,
}

/// Constraints for forecast generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastConstraints {
    /// Budget cap to evaluate against
    pub budget_cap: Option<Decimal>,

    /// Target ROI threshold
    pub roi_threshold: Option<f64>,

    /// Maximum cost per period
    pub max_cost_per_period: Option<Decimal>,

    /// Maximum acceptable growth rate (percentage)
    pub max_growth_rate: Option<f64>,

    /// Minimum confidence level required
    pub min_confidence: Option<f64>,
}

/// Metadata for the forecast request
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ForecastMetadata {
    /// Organization ID
    pub organization_id: Option<String>,

    /// Project ID
    pub project_id: Option<String>,

    /// Execution reference (correlation ID)
    pub execution_ref: Option<String>,

    /// Request source
    pub source: Option<String>,

    /// Additional tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Input schema for Cost Forecasting Agent
///
/// # Required Fields
/// - historical_data: At least 7 data points for analysis
/// - forecast_horizon_days: Number of days to forecast (1-365)
///
/// # Optional Fields
/// - granularity: Forecast output granularity (default: daily)
/// - confidence_level: Prediction interval confidence (default: 0.95)
/// - constraints: Budget and ROI constraints to evaluate
/// - metadata: Request context and tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostForecastInput {
    /// Historical cost data (minimum 7 data points)
    pub historical_data: Vec<HistoricalDataPoint>,

    /// Number of days to forecast (1-365)
    pub forecast_horizon_days: u64,

    /// Output granularity
    #[serde(default)]
    pub granularity: ForecastGranularity,

    /// Confidence level for prediction intervals (0.0-1.0)
    #[serde(default = "default_confidence_level")]
    pub confidence_level: f64,

    /// Optional constraints to evaluate
    #[serde(default)]
    pub constraints: ForecastConstraints,

    /// Request metadata
    #[serde(default)]
    pub metadata: ForecastMetadata,

    /// Preferred forecast model (auto if not specified)
    pub preferred_model: Option<String>,
}

fn default_confidence_level() -> f64 {
    super::DEFAULT_CONFIDENCE_LEVEL
}

impl AgentInput for CostForecastInput {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate historical data length
        validators::min_length(
            "historical_data",
            &self.historical_data,
            super::MIN_DATA_POINTS,
        )?;

        // Validate forecast horizon
        validators::in_range(
            "forecast_horizon_days",
            self.forecast_horizon_days,
            1,
            super::MAX_FORECAST_DAYS,
        )?;

        // Validate confidence level
        validators::confidence("confidence_level", self.confidence_level)?;

        // Validate constraints if provided
        if let Some(min_conf) = self.constraints.min_confidence {
            validators::confidence("constraints.min_confidence", min_conf)?;
        }

        if let Some(max_growth) = self.constraints.max_growth_rate {
            validators::percentage("constraints.max_growth_rate", max_growth)?;
        }

        // Validate data points have costs
        for (i, point) in self.historical_data.iter().enumerate() {
            if point.total_cost < Decimal::ZERO {
                return Err(ValidationError::InvalidField {
                    field: format!("historical_data[{}].total_cost", i),
                    reason: "must be non-negative".to_string(),
                });
            }
        }

        Ok(())
    }
}

/// A single forecast projection point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastProjection {
    /// Projected timestamp
    pub timestamp: DateTime<Utc>,

    /// Projected cost (point estimate)
    pub projected_cost: Decimal,

    /// Lower bound of prediction interval
    pub lower_bound: Decimal,

    /// Upper bound of prediction interval
    pub upper_bound: Decimal,

    /// Cumulative cost from forecast start
    pub cumulative_cost: Decimal,

    /// Growth rate from previous period (percentage)
    pub growth_rate: Option<f64>,
}

/// Risk indicator for the forecast
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskIndicator {
    /// Type of risk
    pub risk_type: String,

    /// Risk level
    pub level: RiskLevel,

    /// Description of the risk
    pub description: String,

    /// Probability of occurrence (0.0-1.0)
    pub probability: f64,

    /// Potential impact (monetary)
    pub potential_impact: Option<Decimal>,

    /// Recommended action
    pub recommendation: Option<String>,
}

impl RiskIndicator {
    /// Create a budget exceedance risk
    pub fn budget_exceedance(
        level: RiskLevel,
        description: impl Into<String>,
        probability: f64,
        impact: Option<Decimal>,
    ) -> Self {
        Self {
            risk_type: "budget_exceedance".to_string(),
            level,
            description: description.into(),
            probability,
            potential_impact: impact,
            recommendation: Some("Review usage patterns and consider optimization".to_string()),
        }
    }

    /// Create a growth rate risk
    pub fn high_growth_rate(
        level: RiskLevel,
        description: impl Into<String>,
        probability: f64,
    ) -> Self {
        Self {
            risk_type: "high_growth_rate".to_string(),
            level,
            description: description.into(),
            probability,
            potential_impact: None,
            recommendation: Some("Monitor usage trends and consider rate limiting".to_string()),
        }
    }

    /// Create a volatility risk
    pub fn high_volatility(
        level: RiskLevel,
        description: impl Into<String>,
    ) -> Self {
        Self {
            risk_type: "high_volatility".to_string(),
            level,
            description: description.into(),
            probability: 1.0, // Already observed
            potential_impact: None,
            recommendation: Some("Investigate cause of cost variability".to_string()),
        }
    }
}

/// Output schema for Cost Forecasting Agent
///
/// # Fields
/// - projections: Time-series of forecast points
/// - total_forecasted_cost: Sum of all projected costs
/// - risk_indicators: Risk assessments and advisories
/// - growth_pattern: Detected pattern in historical data
/// - model_used: Forecasting model applied
/// - confidence: Overall forecast confidence score
/// - constraints_evaluation: Results of constraint checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostForecastOutput {
    /// Time-series forecast projections
    pub projections: Vec<ForecastProjection>,

    /// Total forecasted cost over the horizon
    pub total_forecasted_cost: Decimal,

    /// Average daily cost projection
    pub average_daily_cost: Decimal,

    /// Peak projected daily cost
    pub peak_daily_cost: Decimal,

    /// Risk indicators and advisories
    pub risk_indicators: Vec<RiskIndicator>,

    /// Detected growth pattern
    pub growth_pattern: GrowthPattern,

    /// Average growth rate (percentage)
    pub average_growth_rate: f64,

    /// Forecasting model used
    pub model_used: String,

    /// Overall confidence score (0.0-1.0)
    pub confidence: f64,

    /// Prediction interval confidence level
    pub confidence_level: f64,

    /// Results of constraint evaluations
    pub constraints_evaluation: ConstraintsEvaluation,

    /// Forecast generation timestamp
    pub generated_at: DateTime<Utc>,

    /// Historical data summary
    pub historical_summary: HistoricalSummary,
}

/// Summary of historical data used for forecasting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalSummary {
    /// Number of data points analyzed
    pub data_points: usize,

    /// Start of historical period
    pub period_start: DateTime<Utc>,

    /// End of historical period
    pub period_end: DateTime<Utc>,

    /// Total historical cost
    pub total_cost: Decimal,

    /// Average cost per period
    pub average_cost: Decimal,

    /// Standard deviation of costs
    pub std_deviation: Option<f64>,
}

/// Results of constraint evaluations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstraintsEvaluation {
    /// Budget cap evaluation
    pub budget_cap: Option<ConstraintResult>,

    /// ROI threshold evaluation
    pub roi_threshold: Option<ConstraintResult>,

    /// Max cost per period evaluation
    pub max_cost_per_period: Option<ConstraintResult>,

    /// Growth rate evaluation
    pub growth_rate: Option<ConstraintResult>,
}

/// Result of a single constraint evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintResult {
    /// Whether the constraint is satisfied
    pub satisfied: bool,

    /// Constraint value
    pub constraint_value: String,

    /// Actual/projected value
    pub actual_value: String,

    /// Margin (positive = within limit, negative = exceeded)
    pub margin: Option<String>,

    /// When the constraint will be breached (if applicable)
    pub breach_date: Option<DateTime<Utc>>,
}

impl AgentOutput for CostForecastOutput {
    fn validate(&self) -> Result<(), ValidationError> {
        // Validate projections are present
        if self.projections.is_empty() {
            return Err(ValidationError::InvalidField {
                field: "projections".to_string(),
                reason: "must have at least one projection".to_string(),
            });
        }

        // Validate confidence
        validators::confidence("confidence", self.confidence)?;
        validators::confidence("confidence_level", self.confidence_level)?;

        // Validate growth rate
        if self.average_growth_rate < -100.0 {
            return Err(ValidationError::InvalidField {
                field: "average_growth_rate".to_string(),
                reason: "cannot be less than -100%".to_string(),
            });
        }

        // Validate costs are non-negative
        if self.total_forecasted_cost < Decimal::ZERO {
            return Err(ValidationError::InvalidField {
                field: "total_forecasted_cost".to_string(),
                reason: "must be non-negative".to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_data() -> Vec<HistoricalDataPoint> {
        let now = Utc::now();
        (0..10)
            .map(|i| HistoricalDataPoint {
                timestamp: now - chrono::Duration::days(10 - i),
                total_cost: Decimal::from(100 + i * 10),
                by_provider: Default::default(),
                by_model: Default::default(),
                total_tokens: Some(1000000),
                request_count: Some(1000),
            })
            .collect()
    }

    #[test]
    fn test_input_validation() {
        let valid_input = CostForecastInput {
            historical_data: create_test_data(),
            forecast_horizon_days: 30,
            granularity: ForecastGranularity::Daily,
            confidence_level: 0.95,
            constraints: ForecastConstraints::default(),
            metadata: ForecastMetadata::default(),
            preferred_model: None,
        };

        assert!(valid_input.validate().is_ok());
    }

    #[test]
    fn test_input_validation_insufficient_data() {
        let invalid_input = CostForecastInput {
            historical_data: vec![], // No data
            forecast_horizon_days: 30,
            granularity: ForecastGranularity::Daily,
            confidence_level: 0.95,
            constraints: ForecastConstraints::default(),
            metadata: ForecastMetadata::default(),
            preferred_model: None,
        };

        assert!(invalid_input.validate().is_err());
    }

    #[test]
    fn test_input_validation_invalid_horizon() {
        let mut input = CostForecastInput {
            historical_data: create_test_data(),
            forecast_horizon_days: 0, // Invalid
            granularity: ForecastGranularity::Daily,
            confidence_level: 0.95,
            constraints: ForecastConstraints::default(),
            metadata: ForecastMetadata::default(),
            preferred_model: None,
        };

        assert!(input.validate().is_err());

        input.forecast_horizon_days = 366; // Too long
        assert!(input.validate().is_err());
    }

    #[test]
    fn test_risk_indicator_creation() {
        let risk = RiskIndicator::budget_exceedance(
            RiskLevel::High,
            "Projected to exceed budget by 20%",
            0.85,
            Some(dec!(5000.00)),
        );

        assert_eq!(risk.risk_type, "budget_exceedance");
        assert_eq!(risk.level, RiskLevel::High);
        assert_eq!(risk.probability, 0.85);
    }

    #[test]
    fn test_output_validation() {
        let output = CostForecastOutput {
            projections: vec![ForecastProjection {
                timestamp: Utc::now(),
                projected_cost: dec!(100),
                lower_bound: dec!(80),
                upper_bound: dec!(120),
                cumulative_cost: dec!(100),
                growth_rate: Some(5.0),
            }],
            total_forecasted_cost: dec!(3000),
            average_daily_cost: dec!(100),
            peak_daily_cost: dec!(150),
            risk_indicators: vec![],
            growth_pattern: GrowthPattern::Linear,
            average_growth_rate: 5.0,
            model_used: "Linear Trend".to_string(),
            confidence: 0.92,
            confidence_level: 0.95,
            constraints_evaluation: ConstraintsEvaluation::default(),
            generated_at: Utc::now(),
            historical_summary: HistoricalSummary {
                data_points: 10,
                period_start: Utc::now() - chrono::Duration::days(10),
                period_end: Utc::now(),
                total_cost: dec!(1450),
                average_cost: dec!(145),
                std_deviation: Some(30.0),
            },
        };

        assert!(output.validate().is_ok());
    }

    #[test]
    fn test_granularity_default() {
        assert_eq!(ForecastGranularity::default(), ForecastGranularity::Daily);
    }
}
