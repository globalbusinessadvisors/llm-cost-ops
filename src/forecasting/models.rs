// Forecasting models and algorithms

use chrono::Duration;
use rust_decimal::Decimal;

use super::{
    types::{DataPoint, TimeSeriesData, TrendDirection},
    ForecastError, ForecastResult,
};

/// Trait for forecasting models
pub trait ForecastModel: Send + Sync {
    /// Get the name of the model
    fn name(&self) -> &str;

    /// Train the model on historical data
    fn train(&mut self, data: &TimeSeriesData) -> ForecastResult<()>;

    /// Generate forecast for n periods ahead
    fn forecast(&self, n_periods: usize) -> ForecastResult<Vec<Decimal>>;

    /// Detect trend direction
    fn detect_trend(&self) -> TrendDirection;
}

/// Linear Trend Model
pub struct LinearTrendModel {
    slope: f64,
    intercept: f64,
    last_value: Option<Decimal>,
    interval_secs: i64,
    trained: bool,
}

impl LinearTrendModel {
    /// Create a new linear trend model
    pub fn new() -> Self {
        Self {
            slope: 0.0,
            intercept: 0.0,
            last_value: None,
            interval_secs: 3600, // Default 1 hour
            trained: false,
        }
    }

    /// Calculate linear regression parameters
    fn calculate_regression(values: &[f64]) -> (f64, f64) {
        let n = values.len() as f64;
        let x: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();

        let sum_x: f64 = x.iter().sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = x.iter().zip(values.iter()).map(|(a, b)| a * b).sum();
        let sum_x2: f64 = x.iter().map(|a| a * a).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        let intercept = (sum_y - slope * sum_x) / n;

        (slope, intercept)
    }
}

impl Default for LinearTrendModel {
    fn default() -> Self {
        Self::new()
    }
}

impl ForecastModel for LinearTrendModel {
    fn name(&self) -> &str {
        "Linear Trend"
    }

    fn train(&mut self, data: &TimeSeriesData) -> ForecastResult<()> {
        if data.len() < 2 {
            return Err(ForecastError::InsufficientData(
                "Linear trend requires at least 2 data points".to_string(),
            ));
        }

        let values = data.values_f64();
        let (slope, intercept) = Self::calculate_regression(&values);

        self.slope = slope;
        self.intercept = intercept;
        self.last_value = data.last().map(|p| p.value);
        self.interval_secs = data.interval_secs.unwrap_or(3600);
        self.trained = true;

        Ok(())
    }

    fn forecast(&self, n_periods: usize) -> ForecastResult<Vec<Decimal>> {
        if !self.trained {
            return Err(ForecastError::ModelError(
                "Model must be trained before forecasting".to_string(),
            ));
        }

        let mut forecasts = Vec::with_capacity(n_periods);
        let last_idx = self.intercept.abs() + self.slope.abs();

        for i in 1..=n_periods {
            let forecast_value = self.slope * (last_idx + i as f64) + self.intercept;
            let forecast_value = forecast_value.max(0.0); // Ensure non-negative

            forecasts.push(
                Decimal::try_from(forecast_value)
                    .unwrap_or(Decimal::ZERO),
            );
        }

        Ok(forecasts)
    }

    fn detect_trend(&self) -> TrendDirection {
        if !self.trained {
            return TrendDirection::Unknown;
        }

        if self.slope > 0.01 {
            TrendDirection::Increasing
        } else if self.slope < -0.01 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }
}

/// Moving Average Model
pub struct MovingAverageModel {
    window_size: usize,
    values: Vec<Decimal>,
    interval_secs: i64,
}

impl MovingAverageModel {
    /// Create a new moving average model
    pub fn new(window_size: usize) -> Self {
        Self {
            window_size,
            values: Vec::new(),
            interval_secs: 3600,
        }
    }
}

impl ForecastModel for MovingAverageModel {
    fn name(&self) -> &str {
        "Moving Average"
    }

    fn train(&mut self, data: &TimeSeriesData) -> ForecastResult<()> {
        if data.len() < self.window_size {
            return Err(ForecastError::InsufficientData(format!(
                "Moving average requires at least {} data points",
                self.window_size
            )));
        }

        self.values = data.values();
        self.interval_secs = data.interval_secs.unwrap_or(3600);

        Ok(())
    }

    fn forecast(&self, n_periods: usize) -> ForecastResult<Vec<Decimal>> {
        if self.values.is_empty() {
            return Err(ForecastError::ModelError(
                "Model must be trained before forecasting".to_string(),
            ));
        }

        let mut forecasts = Vec::with_capacity(n_periods);
        let mut extended_values = self.values.clone();

        for _ in 0..n_periods {
            // Calculate moving average of last window_size values
            let start_idx = extended_values.len().saturating_sub(self.window_size);
            let window = &extended_values[start_idx..];

            let sum: Decimal = window.iter().sum();
            let avg = sum / Decimal::from(window.len());

            forecasts.push(avg);
            extended_values.push(avg);
        }

        Ok(forecasts)
    }

    fn detect_trend(&self) -> TrendDirection {
        if self.values.len() < 2 {
            return TrendDirection::Unknown;
        }

        let mid_point = self.values.len() / 2;
        let first_half: Decimal = self.values[..mid_point].iter().sum::<Decimal>()
            / Decimal::from(mid_point);
        let second_half: Decimal = self.values[mid_point..].iter().sum::<Decimal>()
            / Decimal::from(self.values.len() - mid_point);

        if second_half > first_half * Decimal::new(101, 2) {
            // 1% increase
            TrendDirection::Increasing
        } else if second_half < first_half * Decimal::new(99, 2) {
            // 1% decrease
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }
}

/// Exponential Smoothing Model
pub struct ExponentialSmoothingModel {
    alpha: f64, // Smoothing factor (0 < alpha < 1)
    last_smoothed: Option<f64>,
    interval_secs: i64,
    trained: bool,
}

impl ExponentialSmoothingModel {
    /// Create a new exponential smoothing model
    pub fn new(alpha: f64) -> ForecastResult<Self> {
        if !(0.0..=1.0).contains(&alpha) {
            return Err(ForecastError::InvalidConfig(
                "Alpha must be between 0 and 1".to_string(),
            ));
        }

        Ok(Self {
            alpha,
            last_smoothed: None,
            interval_secs: 3600,
            trained: false,
        })
    }

    /// Create with default alpha (0.3)
    pub fn with_default_alpha() -> Self {
        Self {
            alpha: 0.3,
            last_smoothed: None,
            interval_secs: 3600,
            trained: false,
        }
    }
}

impl ForecastModel for ExponentialSmoothingModel {
    fn name(&self) -> &str {
        "Exponential Smoothing"
    }

    fn train(&mut self, data: &TimeSeriesData) -> ForecastResult<()> {
        if data.is_empty() {
            return Err(ForecastError::InsufficientData(
                "Exponential smoothing requires at least 1 data point".to_string(),
            ));
        }

        let values = data.values_f64();
        let mut smoothed = values[0];

        for &value in &values[1..] {
            smoothed = self.alpha * value + (1.0 - self.alpha) * smoothed;
        }

        self.last_smoothed = Some(smoothed);
        self.interval_secs = data.interval_secs.unwrap_or(3600);
        self.trained = true;

        Ok(())
    }

    fn forecast(&self, n_periods: usize) -> ForecastResult<Vec<Decimal>> {
        if !self.trained {
            return Err(ForecastError::ModelError(
                "Model must be trained before forecasting".to_string(),
            ));
        }

        let forecast_value = self.last_smoothed.unwrap_or(0.0).max(0.0);
        let decimal_value = Decimal::try_from(forecast_value)
            .unwrap_or(Decimal::ZERO);

        // Exponential smoothing produces constant forecast
        Ok(vec![decimal_value; n_periods])
    }

    fn detect_trend(&self) -> TrendDirection {
        // Exponential smoothing doesn't directly detect trends
        TrendDirection::Stable
    }
}

/// Generate forecast data points with timestamps
pub fn generate_forecast_points(
    last_timestamp: chrono::DateTime<chrono::Utc>,
    interval_secs: i64,
    values: Vec<Decimal>,
) -> Vec<DataPoint> {
    values
        .into_iter()
        .enumerate()
        .map(|(i, value)| DataPoint::new(
            last_timestamp + Duration::seconds((i as i64 + 1) * interval_secs),
            value,
        ))
        .collect()
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
    fn test_linear_trend_increasing() {
        let data = create_test_series(vec![10, 20, 30, 40, 50]);
        let mut model = LinearTrendModel::new();

        assert!(model.train(&data).is_ok());
        assert_eq!(model.detect_trend(), TrendDirection::Increasing);

        let forecast = model.forecast(3).unwrap();
        assert_eq!(forecast.len(), 3);
        // Should predict continuation of upward trend
        assert!(forecast[0] > Decimal::from(50));
    }

    #[test]
    fn test_linear_trend_decreasing() {
        let data = create_test_series(vec![50, 40, 30, 20, 10]);
        let mut model = LinearTrendModel::new();

        assert!(model.train(&data).is_ok());
        assert_eq!(model.detect_trend(), TrendDirection::Decreasing);
    }

    #[test]
    fn test_moving_average() {
        let data = create_test_series(vec![10, 20, 15, 25, 20]);
        let mut model = MovingAverageModel::new(3);

        assert!(model.train(&data).is_ok());

        let forecast = model.forecast(2).unwrap();
        assert_eq!(forecast.len(), 2);
    }

    #[test]
    fn test_exponential_smoothing() {
        let data = create_test_series(vec![10, 12, 11, 13, 12]);
        let mut model = ExponentialSmoothingModel::with_default_alpha();

        assert!(model.train(&data).is_ok());

        let forecast = model.forecast(3).unwrap();
        assert_eq!(forecast.len(), 3);
        // All forecasts should be the same (constant forecast)
        assert_eq!(forecast[0], forecast[1]);
        assert_eq!(forecast[1], forecast[2]);
    }

    #[test]
    fn test_insufficient_data() {
        let data = create_test_series(vec![10]);
        let mut model = LinearTrendModel::new();

        assert!(model.train(&data).is_err());
    }

    #[test]
    fn test_untrained_forecast() {
        let model = LinearTrendModel::new();
        assert!(model.forecast(5).is_err());
    }

    #[test]
    fn test_invalid_alpha() {
        assert!(ExponentialSmoothingModel::new(1.5).is_err());
        assert!(ExponentialSmoothingModel::new(-0.1).is_err());
        assert!(ExponentialSmoothingModel::new(0.5).is_ok());
    }
}
