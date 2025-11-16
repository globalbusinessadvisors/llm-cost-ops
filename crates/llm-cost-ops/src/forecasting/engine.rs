// Forecasting engine - orchestrates models and generates forecasts

use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{
    models::{ExponentialSmoothingModel, ForecastModel, LinearTrendModel, MovingAverageModel},
    types::{
        DataPoint, ForecastConfig, ForecastHorizon, ForecastResult as TypesForecastResult,
        SeasonalityPattern, TimeSeriesData, TrendDirection,
    },
    ForecastError, ForecastResult,
};

/// Forecast request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastRequest {
    /// Historical time series data
    pub data: TimeSeriesData,

    /// Forecast configuration
    pub config: ForecastConfig,

    /// Preferred model (if None, best model will be selected)
    pub preferred_model: Option<ModelType>,
}

/// Available forecasting models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelType {
    LinearTrend,
    MovingAverage,
    ExponentialSmoothing,
    Auto, // Automatically select best model
}

/// Forecast engine
pub struct ForecastEngine {
    config: ForecastConfig,
}

impl ForecastEngine {
    /// Create a new forecast engine
    pub fn new(config: ForecastConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn new_with_defaults() -> Self {
        Self {
            config: ForecastConfig::default(),
        }
    }

    /// Generate forecast from request
    pub fn forecast(&self, request: ForecastRequest) -> ForecastResult<TypesForecastResult> {
        // Validate input data
        self.validate_data(&request.data)?;

        // Determine model to use
        let model_type = request.preferred_model.unwrap_or(ModelType::Auto);
        let model_type = if model_type == ModelType::Auto {
            self.select_best_model(&request.data)?
        } else {
            model_type
        };

        // Create and train model
        let mut model = self.create_model(model_type)?;
        model.train(&request.data)?;

        // Calculate number of periods to forecast
        let n_periods = self.calculate_periods(&request)?;

        // Generate forecast values
        let forecast_values = model.forecast(n_periods)?;

        // Generate forecast data points with timestamps
        let last_timestamp = request
            .data
            .last()
            .ok_or_else(|| ForecastError::InsufficientData("No data points".to_string()))?
            .timestamp;

        let interval_secs = request.data.interval_secs.unwrap_or(3600);

        let forecast_points = self.generate_forecast_points(
            last_timestamp,
            interval_secs,
            forecast_values.clone(),
        );

        // Calculate prediction intervals
        let std_dev = request.data.std_dev().unwrap_or(0.0);
        let z_score = self.calculate_z_score(request.config.confidence_level);

        let (lower_bound, upper_bound) =
            self.calculate_prediction_intervals(&forecast_points, std_dev, z_score);

        // Detect trend
        let trend = if request.config.include_trend {
            model.detect_trend()
        } else {
            TrendDirection::Unknown
        };

        // Detect seasonality
        let seasonality = if request.config.detect_seasonality {
            self.detect_seasonality(&request.data)?
        } else {
            SeasonalityPattern {
                detected: false,
                period: None,
                strength: 0.0,
            }
        };

        // Calculate metrics if we have enough validation data
        let metrics = self.calculate_validation_metrics(&request.data, &model)?;

        Ok(TypesForecastResult {
            forecast: forecast_points,
            lower_bound,
            upper_bound,
            trend,
            seasonality,
            model_name: model.name().to_string(),
            confidence_level: request.config.confidence_level,
            metrics,
        })
    }

    /// Validate input data
    fn validate_data(&self, data: &TimeSeriesData) -> ForecastResult<()> {
        if data.is_empty() {
            return Err(ForecastError::InsufficientData(
                "Time series data is empty".to_string(),
            ));
        }

        if data.len() < self.config.min_data_points {
            return Err(ForecastError::InsufficientData(format!(
                "Insufficient data points: {} (minimum required: {})",
                data.len(),
                self.config.min_data_points
            )));
        }

        Ok(())
    }

    /// Select the best model based on data characteristics
    fn select_best_model(&self, data: &TimeSeriesData) -> ForecastResult<ModelType> {
        // For now, use a simple heuristic:
        // - Linear trend for data with clear trends
        // - Moving average for stable data
        // - Exponential smoothing as fallback

        let values = data.values_f64();
        if values.len() < 2 {
            return Ok(ModelType::ExponentialSmoothing);
        }

        // Calculate simple trend indicator
        let first_half_mean = values[..values.len() / 2].iter().sum::<f64>()
            / (values.len() / 2) as f64;
        let second_half_mean =
            values[values.len() / 2..].iter().sum::<f64>() / (values.len() - values.len() / 2) as f64;

        let trend_ratio = if first_half_mean.abs() > f64::EPSILON {
            second_half_mean / first_half_mean
        } else {
            1.0
        };

        // If strong trend (>5% change), use linear trend
        if !(0.95..=1.05).contains(&trend_ratio) {
            Ok(ModelType::LinearTrend)
        } else if data.len() >= 10 {
            // Use moving average for stable data with enough points
            Ok(ModelType::MovingAverage)
        } else {
            // Default to exponential smoothing
            Ok(ModelType::ExponentialSmoothing)
        }
    }

    /// Create a model instance
    fn create_model(&self, model_type: ModelType) -> ForecastResult<Box<dyn ForecastModel>> {
        match model_type {
            ModelType::LinearTrend => Ok(Box::new(LinearTrendModel::new())),
            ModelType::MovingAverage => {
                let window_size = (self.config.min_data_points / 2).max(3);
                Ok(Box::new(MovingAverageModel::new(window_size)))
            }
            ModelType::ExponentialSmoothing => Ok(Box::new(
                ExponentialSmoothingModel::with_default_alpha(),
            )),
            ModelType::Auto => Err(ForecastError::InvalidConfig(
                "Auto model type should have been resolved".to_string(),
            )),
        }
    }

    /// Calculate number of periods to forecast
    fn calculate_periods(&self, request: &ForecastRequest) -> ForecastResult<usize> {
        match request.config.horizon {
            ForecastHorizon::Periods(n) => Ok(n),
            ForecastHorizon::Days(days) => {
                let interval_secs = request.data.interval_secs.unwrap_or(3600);
                let periods_per_day = 86400 / interval_secs;
                Ok((days as i64 * periods_per_day) as usize)
            }
            ForecastHorizon::UntilDate(target_date) => {
                let last_timestamp = request
                    .data
                    .last()
                    .ok_or_else(|| {
                        ForecastError::InsufficientData("No data points".to_string())
                    })?
                    .timestamp;

                let duration = target_date.signed_duration_since(last_timestamp);
                let interval_secs = request.data.interval_secs.unwrap_or(3600);

                let periods = duration.num_seconds() / interval_secs;
                if periods <= 0 {
                    return Err(ForecastError::InvalidConfig(
                        "Target date must be in the future".to_string(),
                    ));
                }

                Ok(periods as usize)
            }
        }
    }

    /// Generate forecast data points with timestamps
    fn generate_forecast_points(
        &self,
        last_timestamp: DateTime<Utc>,
        interval_secs: i64,
        values: Vec<Decimal>,
    ) -> Vec<DataPoint> {
        values
            .into_iter()
            .enumerate()
            .map(|(i, value)| {
                DataPoint::new(
                    last_timestamp + Duration::seconds((i as i64 + 1) * interval_secs),
                    value,
                )
            })
            .collect()
    }

    /// Calculate z-score for confidence level
    fn calculate_z_score(&self, confidence_level: f64) -> f64 {
        // Common z-scores for confidence levels
        match (confidence_level * 100.0) as i32 {
            90 => 1.645,
            95 => 1.96,
            99 => 2.576,
            _ => 1.96, // Default to 95%
        }
    }

    /// Calculate prediction intervals
    fn calculate_prediction_intervals(
        &self,
        forecast: &[DataPoint],
        std_dev: f64,
        z_score: f64,
    ) -> (Vec<DataPoint>, Vec<DataPoint>) {
        let margin = std_dev * z_score;

        let lower_bound: Vec<DataPoint> = forecast
            .iter()
            .map(|point| {
                let lower_value = point.value
                    - Decimal::try_from(margin).unwrap_or(Decimal::ZERO);
                let lower_value = lower_value.max(Decimal::ZERO); // Ensure non-negative
                DataPoint::new(point.timestamp, lower_value)
            })
            .collect();

        let upper_bound: Vec<DataPoint> = forecast
            .iter()
            .map(|point| {
                let upper_value = point.value
                    + Decimal::try_from(margin).unwrap_or(Decimal::ZERO);
                DataPoint::new(point.timestamp, upper_value)
            })
            .collect();

        (lower_bound, upper_bound)
    }

    /// Detect seasonality in time series
    fn detect_seasonality(&self, data: &TimeSeriesData) -> ForecastResult<SeasonalityPattern> {
        // Simple autocorrelation-based seasonality detection
        if data.len() < 14 {
            return Ok(SeasonalityPattern {
                detected: false,
                period: None,
                strength: 0.0,
            });
        }

        let values = data.values_f64();
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        // Test common periods: daily (24h), weekly (7d)
        let test_periods = vec![24, 168]; // hours
        let interval_hours = data.interval_secs.unwrap_or(3600) / 3600;

        let mut best_period = None;
        let mut best_correlation = 0.0;

        for &period_hours in &test_periods {
            let lag = period_hours / interval_hours;
            if lag as usize >= values.len() {
                continue;
            }

            let correlation = self.calculate_autocorrelation(&values, lag as usize, mean);
            if correlation > best_correlation {
                best_correlation = correlation;
                best_period = Some(lag as usize);
            }
        }

        let detected = best_correlation > 0.3; // Threshold for significance

        Ok(SeasonalityPattern {
            detected,
            period: if detected { best_period } else { None },
            strength: if detected { best_correlation } else { 0.0 },
        })
    }

    /// Calculate autocorrelation at a given lag
    fn calculate_autocorrelation(&self, values: &[f64], lag: usize, mean: f64) -> f64 {
        if lag >= values.len() {
            return 0.0;
        }

        let n = values.len() - lag;
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for i in 0..n {
            numerator += (values[i] - mean) * (values[i + lag] - mean);
        }

        for &value in values {
            denominator += (value - mean).powi(2);
        }

        if denominator.abs() < f64::EPSILON {
            return 0.0;
        }

        numerator / denominator
    }

    /// Calculate validation metrics using holdout set
    fn calculate_validation_metrics(
        &self,
        data: &TimeSeriesData,
        model: &Box<dyn ForecastModel>,
    ) -> ForecastResult<Option<super::metrics::ForecastMetrics>> {
        // Use 20% of data as holdout for validation
        let holdout_size = (data.len() as f64 * 0.2).ceil() as usize;
        if holdout_size < 2 || data.len() - holdout_size < self.config.min_data_points {
            return Ok(None); // Not enough data for validation
        }

        // Split data
        let train_size = data.len() - holdout_size;
        let train_data = data.subset(0, train_size);
        let holdout_data = data.subset(train_size, data.len());

        // Train on training set
        let mut validation_model = self.create_model(
            match model.name() {
                "Linear Trend" => ModelType::LinearTrend,
                "Moving Average" => ModelType::MovingAverage,
                "Exponential Smoothing" => ModelType::ExponentialSmoothing,
                _ => ModelType::ExponentialSmoothing,
            }
        )?;

        validation_model.train(&train_data)?;

        // Forecast holdout period
        let predictions = validation_model.forecast(holdout_size)?;
        let actuals = holdout_data.values();

        // Calculate metrics
        match super::metrics::ForecastMetrics::new(&actuals, &predictions) {
            Ok(metrics) => Ok(Some(metrics)),
            Err(_) => Ok(None), // If metrics calculation fails, return None
        }
    }
}

impl Default for ForecastEngine {
    fn default() -> Self {
        Self::new_with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_series(values: Vec<i32>) -> TimeSeriesData {
        let start = Utc::now();
        let points: Vec<DataPoint> = values
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                DataPoint::new(start + Duration::hours(i as i64), Decimal::from(v))
            })
            .collect();

        TimeSeriesData::with_auto_interval(points)
    }

    #[test]
    fn test_engine_creation() {
        let engine = ForecastEngine::default();
        assert_eq!(engine.config.confidence_level, 0.95);
    }

    #[test]
    fn test_validate_data() {
        let engine = ForecastEngine::default();

        // Empty data
        let empty_data = TimeSeriesData::new(vec![]);
        assert!(engine.validate_data(&empty_data).is_err());

        // Insufficient data
        let small_data = create_test_series(vec![1, 2]);
        assert!(engine.validate_data(&small_data).is_err());

        // Valid data
        let valid_data = create_test_series(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        assert!(engine.validate_data(&valid_data).is_ok());
    }

    #[test]
    fn test_select_best_model() {
        let engine = ForecastEngine::default();

        // Trending data should select linear trend
        let trending_data = create_test_series(vec![10, 20, 30, 40, 50, 60, 70, 80]);
        let model_type = engine.select_best_model(&trending_data).unwrap();
        assert_eq!(model_type, ModelType::LinearTrend);

        // Stable data should select moving average
        let stable_data = create_test_series(vec![50, 51, 49, 50, 52, 48, 50, 51, 49, 50]);
        let model_type = engine.select_best_model(&stable_data).unwrap();
        assert_eq!(model_type, ModelType::MovingAverage);
    }

    #[test]
    fn test_calculate_periods() {
        let engine = ForecastEngine::default();

        // Test Periods horizon
        let data = create_test_series(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        let mut config = ForecastConfig::default();
        config.horizon = ForecastHorizon::Periods(10);

        let request = ForecastRequest {
            data: data.clone(),
            config: config.clone(),
            preferred_model: None,
        };

        let periods = engine.calculate_periods(&request).unwrap();
        assert_eq!(periods, 10);

        // Test Days horizon
        config.horizon = ForecastHorizon::Days(7);
        let request = ForecastRequest {
            data: data.clone(),
            config,
            preferred_model: None,
        };

        let periods = engine.calculate_periods(&request).unwrap();
        assert_eq!(periods, 168); // 7 days * 24 hours
    }

    #[test]
    fn test_forecast_generation() {
        let engine = ForecastEngine::default();
        let data = create_test_series(vec![10, 20, 30, 40, 50, 60, 70, 80]);

        let mut config = ForecastConfig::default();
        config.horizon = ForecastHorizon::Periods(5);

        let request = ForecastRequest {
            data,
            config,
            preferred_model: Some(ModelType::LinearTrend),
        };

        let result = engine.forecast(request);
        assert!(result.is_ok());

        let forecast = result.unwrap();
        assert_eq!(forecast.forecast.len(), 5);
        assert_eq!(forecast.lower_bound.len(), 5);
        assert_eq!(forecast.upper_bound.len(), 5);
        assert_eq!(forecast.model_name, "Linear Trend");
        assert_eq!(forecast.trend, TrendDirection::Increasing);
    }

    #[test]
    fn test_z_score_calculation() {
        let engine = ForecastEngine::default();

        assert_eq!(engine.calculate_z_score(0.90), 1.645);
        assert_eq!(engine.calculate_z_score(0.95), 1.96);
        assert_eq!(engine.calculate_z_score(0.99), 2.576);
    }

    #[test]
    fn test_autocorrelation() {
        let engine = ForecastEngine::default();
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        let correlation = engine.calculate_autocorrelation(&values, 1, mean);
        assert!(correlation > 0.0); // Positive autocorrelation for trending data
    }

    #[test]
    fn test_seasonality_detection() {
        let engine = ForecastEngine::default();

        // Not enough data
        let small_data = create_test_series(vec![1, 2, 3, 4, 5]);
        let seasonality = engine.detect_seasonality(&small_data).unwrap();
        assert!(!seasonality.detected);

        // Enough data but no clear seasonality
        let data = create_test_series(vec![
            10, 20, 15, 25, 20, 30, 25, 35, 30, 40, 35, 45, 40, 50,
        ]);
        let seasonality = engine.detect_seasonality(&data).unwrap();
        // Result depends on autocorrelation, might or might not detect
        assert!(seasonality.strength >= 0.0 && seasonality.strength <= 1.0);
    }

    #[test]
    fn test_insufficient_data_error() {
        let engine = ForecastEngine::default();
        let data = create_test_series(vec![1, 2]); // Not enough data

        let config = ForecastConfig::default();
        let request = ForecastRequest {
            data,
            config,
            preferred_model: None,
        };

        let result = engine.forecast(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_different_models() {
        let engine = ForecastEngine::default();
        let data = create_test_series(vec![10, 20, 30, 40, 50, 60, 70, 80]);

        let config = ForecastConfig {
            horizon: ForecastHorizon::Periods(3),
            ..Default::default()
        };

        // Test Linear Trend
        let request = ForecastRequest {
            data: data.clone(),
            config: config.clone(),
            preferred_model: Some(ModelType::LinearTrend),
        };
        assert!(engine.forecast(request).is_ok());

        // Test Moving Average
        let request = ForecastRequest {
            data: data.clone(),
            config: config.clone(),
            preferred_model: Some(ModelType::MovingAverage),
        };
        assert!(engine.forecast(request).is_ok());

        // Test Exponential Smoothing
        let request = ForecastRequest {
            data: data.clone(),
            config: config.clone(),
            preferred_model: Some(ModelType::ExponentialSmoothing),
        };
        assert!(engine.forecast(request).is_ok());

        // Test Auto selection
        let request = ForecastRequest {
            data,
            config,
            preferred_model: Some(ModelType::Auto),
        };
        assert!(engine.forecast(request).is_ok());
    }
}
