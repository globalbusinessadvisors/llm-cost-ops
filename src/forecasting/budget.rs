// Budget forecasting and alerting

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    engine::{ForecastEngine, ForecastRequest, ModelType},
    types::{ForecastConfig, ForecastHorizon, TimeSeriesData},
    ForecastResult,
};

/// Budget alert severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Budget alert type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    /// Budget threshold exceeded
    ThresholdExceeded,

    /// Projected to exceed budget
    ProjectedExceedance,

    /// Unusual spending pattern
    UnusualSpending,

    /// Approaching budget limit
    ApproachingLimit,
}

/// Budget alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    /// Alert type
    pub alert_type: AlertType,

    /// Severity level
    pub severity: AlertSeverity,

    /// Alert message
    pub message: String,

    /// Current spend
    pub current_spend: Decimal,

    /// Budget limit
    pub budget_limit: Decimal,

    /// Utilization percentage
    pub utilization_percent: f64,

    /// Projected spend (if applicable)
    pub projected_spend: Option<Decimal>,

    /// Timestamp when alert was generated
    pub timestamp: DateTime<Utc>,
}

/// Budget configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetConfig {
    /// Total budget limit
    pub limit: Decimal,

    /// Budget period in days
    pub period_days: u64,

    /// Warning threshold (percentage of budget)
    pub warning_threshold: f64,

    /// Critical threshold (percentage of budget)
    pub critical_threshold: f64,

    /// Enable forecasting-based alerts
    pub enable_forecasting: bool,
}

impl Default for BudgetConfig {
    fn default() -> Self {
        Self {
            limit: Decimal::from(1000),
            period_days: 30,
            warning_threshold: 0.80, // 80%
            critical_threshold: 0.95, // 95%
            enable_forecasting: true,
        }
    }
}

/// Budget forecast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetForecast {
    /// Current spend
    pub current_spend: Decimal,

    /// Budget limit
    pub budget_limit: Decimal,

    /// Remaining budget
    pub remaining_budget: Decimal,

    /// Utilization percentage
    pub utilization_percent: f64,

    /// Projected end-of-period spend
    pub projected_spend: Option<Decimal>,

    /// Projected utilization at end of period
    pub projected_utilization: Option<f64>,

    /// Days remaining in period
    pub days_remaining: i64,

    /// Current daily average spend
    pub daily_average: Decimal,

    /// Projected daily spend (based on forecast)
    pub projected_daily: Option<Decimal>,

    /// Active alerts
    pub alerts: Vec<BudgetAlert>,

    /// Forecast timestamp
    pub timestamp: DateTime<Utc>,
}

/// Budget forecaster
pub struct BudgetForecaster {
    config: BudgetConfig,
    forecast_engine: ForecastEngine,
}

impl BudgetForecaster {
    /// Create a new budget forecaster
    pub fn new(config: BudgetConfig) -> Self {
        let forecast_config = ForecastConfig {
            horizon: ForecastHorizon::Days(config.period_days),
            confidence_level: 0.95,
            include_trend: true,
            detect_seasonality: true,
            min_data_points: 7,
        };

        Self {
            config,
            forecast_engine: ForecastEngine::new(forecast_config),
        }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(BudgetConfig::default())
    }

    /// Generate budget forecast
    pub fn forecast(
        &self,
        historical_data: &TimeSeriesData,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> ForecastResult<BudgetForecast> {
        // Calculate current spend
        let current_spend: Decimal = historical_data.values().iter().sum();

        // Calculate remaining budget
        let remaining_budget = self.config.limit - current_spend;
        let utilization_percent = if self.config.limit > Decimal::ZERO {
            (current_spend / self.config.limit * Decimal::from(100))
                .to_string()
                .parse::<f64>()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Calculate days remaining
        let now = Utc::now();
        let days_remaining = (period_end - now).num_days();

        // Calculate daily average
        let period_days_elapsed = (now - period_start).num_days().max(1);
        let daily_average = current_spend / Decimal::from(period_days_elapsed);

        // Generate forecast if enabled and we have enough data
        let (projected_spend, projected_utilization, projected_daily) =
            if self.config.enable_forecasting && historical_data.len() >= 7 {
                match self.generate_projection(historical_data, days_remaining) {
                    Ok((projected, daily)) => {
                        let total_projected = current_spend + projected;
                        let proj_util = if self.config.limit > Decimal::ZERO {
                            (total_projected / self.config.limit * Decimal::from(100))
                                .to_string()
                                .parse::<f64>()
                                .unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        (Some(total_projected), Some(proj_util), Some(daily))
                    }
                    Err(_) => (None, None, None),
                }
            } else {
                (None, None, None)
            };

        // Generate alerts
        let alerts = self.generate_alerts(
            current_spend,
            utilization_percent,
            projected_spend,
            projected_utilization,
        );

        Ok(BudgetForecast {
            current_spend,
            budget_limit: self.config.limit,
            remaining_budget,
            utilization_percent,
            projected_spend,
            projected_utilization,
            days_remaining,
            daily_average,
            projected_daily,
            alerts,
            timestamp: Utc::now(),
        })
    }

    /// Generate spending projection
    fn generate_projection(
        &self,
        data: &TimeSeriesData,
        days_remaining: i64,
    ) -> ForecastResult<(Decimal, Decimal)> {
        if days_remaining <= 0 {
            return Ok((Decimal::ZERO, Decimal::ZERO));
        }

        let mut forecast_config = ForecastConfig::default();
        forecast_config.horizon = ForecastHorizon::Days(days_remaining as u64);

        let request = ForecastRequest {
            data: data.clone(),
            config: forecast_config,
            preferred_model: Some(ModelType::Auto),
        };

        let forecast_result = self.forecast_engine.forecast(request)?;

        // Sum forecasted values
        let projected_total: Decimal = forecast_result
            .forecast
            .iter()
            .map(|p| p.value)
            .sum();

        // Calculate average daily projection
        let projected_daily = if days_remaining > 0 {
            projected_total / Decimal::from(days_remaining)
        } else {
            Decimal::ZERO
        };

        Ok((projected_total, projected_daily))
    }

    /// Generate budget alerts
    fn generate_alerts(
        &self,
        current_spend: Decimal,
        utilization_percent: f64,
        projected_spend: Option<Decimal>,
        projected_utilization: Option<f64>,
    ) -> Vec<BudgetAlert> {
        let mut alerts = Vec::new();
        let now = Utc::now();

        // Check current utilization
        if utilization_percent >= self.config.critical_threshold * 100.0 {
            alerts.push(BudgetAlert {
                alert_type: AlertType::ThresholdExceeded,
                severity: AlertSeverity::Critical,
                message: format!(
                    "Budget critical: {:.1}% utilized (threshold: {:.1}%)",
                    utilization_percent,
                    self.config.critical_threshold * 100.0
                ),
                current_spend,
                budget_limit: self.config.limit,
                utilization_percent,
                projected_spend,
                timestamp: now,
            });
        } else if utilization_percent >= self.config.warning_threshold * 100.0 {
            alerts.push(BudgetAlert {
                alert_type: AlertType::ApproachingLimit,
                severity: AlertSeverity::Warning,
                message: format!(
                    "Budget warning: {:.1}% utilized (threshold: {:.1}%)",
                    utilization_percent,
                    self.config.warning_threshold * 100.0
                ),
                current_spend,
                budget_limit: self.config.limit,
                utilization_percent,
                projected_spend,
                timestamp: now,
            });
        }

        // Check projected utilization
        if let Some(proj_util) = projected_utilization {
            if proj_util >= 100.0 {
                alerts.push(BudgetAlert {
                    alert_type: AlertType::ProjectedExceedance,
                    severity: AlertSeverity::Critical,
                    message: format!(
                        "Projected to exceed budget: {:.1}% utilization expected",
                        proj_util
                    ),
                    current_spend,
                    budget_limit: self.config.limit,
                    utilization_percent,
                    projected_spend,
                    timestamp: now,
                });
            } else if proj_util >= self.config.critical_threshold * 100.0 {
                alerts.push(BudgetAlert {
                    alert_type: AlertType::ProjectedExceedance,
                    severity: AlertSeverity::Warning,
                    message: format!(
                        "Projected to approach budget limit: {:.1}% utilization expected",
                        proj_util
                    ),
                    current_spend,
                    budget_limit: self.config.limit,
                    utilization_percent,
                    projected_spend,
                    timestamp: now,
                });
            }
        }

        // Check for unusual spending patterns
        if let Some(proj_spend) = projected_spend {
            // If projected spend is significantly higher than current trajectory
            let simple_projection = current_spend * Decimal::from(2); // Simple doubling
            if proj_spend > simple_projection * Decimal::new(120, 2) {
                // 20% more than simple projection
                alerts.push(BudgetAlert {
                    alert_type: AlertType::UnusualSpending,
                    severity: AlertSeverity::Warning,
                    message: "Unusual spending pattern detected: costs accelerating faster than historical average".to_string(),
                    current_spend,
                    budget_limit: self.config.limit,
                    utilization_percent,
                    projected_spend: Some(proj_spend),
                    timestamp: now,
                });
            }
        }

        alerts
    }

    /// Check if budget is on track
    pub fn is_on_track(&self, forecast: &BudgetForecast) -> bool {
        if let Some(proj_util) = forecast.projected_utilization {
            proj_util <= 100.0
        } else {
            forecast.utilization_percent <= 100.0
        }
    }

    /// Get recommended budget for next period based on historical data
    pub fn recommend_budget(
        &self,
        historical_data: &TimeSeriesData,
        period_days: u64,
    ) -> ForecastResult<Decimal> {
        if historical_data.is_empty() {
            return Ok(self.config.limit); // Default to current limit
        }

        let forecast_config = ForecastConfig {
            horizon: ForecastHorizon::Days(period_days),
            confidence_level: 0.95,
            include_trend: true,
            detect_seasonality: true,
            min_data_points: 7,
        };

        let request = ForecastRequest {
            data: historical_data.clone(),
            config: forecast_config,
            preferred_model: Some(ModelType::Auto),
        };

        let forecast_result = self.forecast_engine.forecast(request)?;

        // Sum forecasted values and add 20% buffer
        let projected_total: Decimal = forecast_result
            .forecast
            .iter()
            .map(|p| p.value)
            .sum();

        let recommended = projected_total * Decimal::new(120, 2); // Add 20% buffer

        Ok(recommended.max(self.config.limit)) // At least current limit
    }
}

impl Default for BudgetForecaster {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::DataPoint;
    use chrono::Duration;
    use rust_decimal::Decimal;

    fn create_test_series(values: Vec<i32>) -> TimeSeriesData {
        let start = Utc::now() - Duration::days(values.len() as i64);
        let points: Vec<DataPoint> = values
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                DataPoint::new(start + Duration::days(i as i64), Decimal::from(v))
            })
            .collect();

        TimeSeriesData::with_auto_interval(points)
    }

    #[test]
    fn test_budget_forecaster_creation() {
        let forecaster = BudgetForecaster::with_defaults();
        assert_eq!(forecaster.config.limit, Decimal::from(1000));
        assert_eq!(forecaster.config.period_days, 30);
    }

    #[test]
    fn test_budget_forecast_basic() {
        let mut config = BudgetConfig::default();
        config.limit = Decimal::from(1000);
        config.enable_forecasting = false; // Disable for simple test

        let forecaster = BudgetForecaster::new(config);

        let data = create_test_series(vec![10, 12, 15, 13, 14, 16, 18]);
        let period_start = Utc::now() - Duration::days(7);
        let period_end = Utc::now() + Duration::days(23);

        let forecast = forecaster.forecast(&data, period_start, period_end).unwrap();

        assert_eq!(forecast.current_spend, Decimal::from(98)); // Sum of values
        assert_eq!(forecast.budget_limit, Decimal::from(1000));
        assert!(forecast.remaining_budget > Decimal::ZERO);
        assert!(forecast.utilization_percent < 100.0);
    }

    #[test]
    fn test_budget_alerts_warning() {
        let mut config = BudgetConfig::default();
        config.limit = Decimal::from(100);
        config.warning_threshold = 0.80;
        config.critical_threshold = 0.95;
        config.enable_forecasting = false;

        let forecaster = BudgetForecaster::new(config);

        // Spend 85 out of 100 (85% utilization)
        let data = create_test_series(vec![10, 15, 20, 15, 25]);
        let period_start = Utc::now() - Duration::days(5);
        let period_end = Utc::now() + Duration::days(25);

        let forecast = forecaster.forecast(&data, period_start, period_end).unwrap();

        assert!(forecast.alerts.len() > 0);
        assert!(forecast
            .alerts
            .iter()
            .any(|a| a.severity == AlertSeverity::Warning));
    }

    #[test]
    fn test_budget_alerts_critical() {
        let mut config = BudgetConfig::default();
        config.limit = Decimal::from(100);
        config.critical_threshold = 0.95;
        config.enable_forecasting = false;

        let forecaster = BudgetForecaster::new(config);

        // Spend 96 out of 100 (96% utilization)
        let data = create_test_series(vec![20, 24, 26, 26]);
        let period_start = Utc::now() - Duration::days(4);
        let period_end = Utc::now() + Duration::days(26);

        let forecast = forecaster.forecast(&data, period_start, period_end).unwrap();

        assert!(forecast.alerts.len() > 0);
        assert!(forecast
            .alerts
            .iter()
            .any(|a| a.severity == AlertSeverity::Critical));
    }

    #[test]
    fn test_is_on_track() {
        let forecaster = BudgetForecaster::with_defaults();

        let on_track_forecast = BudgetForecast {
            current_spend: Decimal::from(50),
            budget_limit: Decimal::from(100),
            remaining_budget: Decimal::from(50),
            utilization_percent: 50.0,
            projected_spend: Some(Decimal::from(80)),
            projected_utilization: Some(80.0),
            days_remaining: 15,
            daily_average: Decimal::from(3),
            projected_daily: Some(Decimal::from(2)),
            alerts: vec![],
            timestamp: Utc::now(),
        };

        assert!(forecaster.is_on_track(&on_track_forecast));

        let over_budget_forecast = BudgetForecast {
            current_spend: Decimal::from(50),
            budget_limit: Decimal::from(100),
            remaining_budget: Decimal::from(50),
            utilization_percent: 50.0,
            projected_spend: Some(Decimal::from(120)),
            projected_utilization: Some(120.0),
            days_remaining: 15,
            daily_average: Decimal::from(3),
            projected_daily: Some(Decimal::from(5)),
            alerts: vec![],
            timestamp: Utc::now(),
        };

        assert!(!forecaster.is_on_track(&over_budget_forecast));
    }

    #[test]
    fn test_daily_average_calculation() {
        let config = BudgetConfig::default();
        let forecaster = BudgetForecaster::new(config);

        let data = create_test_series(vec![10, 10, 10, 10, 10]); // 5 days, 50 total
        let period_start = Utc::now() - Duration::days(5);
        let period_end = Utc::now() + Duration::days(25);

        let forecast = forecaster.forecast(&data, period_start, period_end).unwrap();

        // Daily average should be approximately 10 (50 total / 5 days)
        assert!(forecast.daily_average >= Decimal::from(9));
        assert!(forecast.daily_average <= Decimal::from(11));
    }

    #[test]
    fn test_budget_recommendation() {
        let forecaster = BudgetForecaster::with_defaults();

        let data = create_test_series(vec![
            10, 12, 11, 13, 14, 15, 16, 17, 18, 19, 20, 21,
        ]);

        let recommended = forecaster.recommend_budget(&data, 30).unwrap();

        // Recommended budget should be at least the current limit
        assert!(recommended >= forecaster.config.limit);
    }

    #[test]
    fn test_no_alerts_when_within_budget() {
        let mut config = BudgetConfig::default();
        config.limit = Decimal::from(1000);
        config.enable_forecasting = false;

        let forecaster = BudgetForecaster::new(config);

        let data = create_test_series(vec![10, 12, 15, 13, 14]);
        let period_start = Utc::now() - Duration::days(5);
        let period_end = Utc::now() + Duration::days(25);

        let forecast = forecaster.forecast(&data, period_start, period_end).unwrap();

        // Should have no alerts when well within budget
        assert_eq!(forecast.alerts.len(), 0);
    }

    #[test]
    fn test_remaining_budget_calculation() {
        let mut config = BudgetConfig::default();
        config.limit = Decimal::from(500);
        config.enable_forecasting = false;

        let forecaster = BudgetForecaster::new(config);

        let data = create_test_series(vec![50, 60, 70]);
        let period_start = Utc::now() - Duration::days(3);
        let period_end = Utc::now() + Duration::days(27);

        let forecast = forecaster.forecast(&data, period_start, period_end).unwrap();

        assert_eq!(forecast.current_spend, Decimal::from(180));
        assert_eq!(forecast.remaining_budget, Decimal::from(320)); // 500 - 180
    }
}
