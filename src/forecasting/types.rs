// Forecasting data types and structures

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A single data point in a time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// Timestamp of the data point
    pub timestamp: DateTime<Utc>,

    /// Value at this point
    pub value: Decimal,

    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Time series data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesData {
    /// Data points in chronological order
    pub points: Vec<DataPoint>,

    /// Interval between data points (in seconds)
    pub interval_secs: Option<i64>,

    /// Time series metadata
    pub metadata: Option<serde_json::Value>,
}

/// Forecast horizon configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ForecastHorizon {
    /// Number of periods to forecast
    Periods(usize),

    /// Forecast until a specific date
    UntilDate(DateTime<Utc>),

    /// Forecast for a duration (in days)
    Days(u64),
}

/// Trend direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrendDirection {
    /// Upward trend
    Increasing,

    /// Downward trend
    Decreasing,

    /// No significant trend
    Stable,

    /// Trend cannot be determined
    Unknown,
}

/// Seasonality pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalityPattern {
    /// Whether seasonality was detected
    pub detected: bool,

    /// Period of seasonality (in number of data points)
    pub period: Option<usize>,

    /// Strength of seasonality (0.0 to 1.0)
    pub strength: f64,
}

/// Forecast configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastConfig {
    /// Forecast horizon
    pub horizon: ForecastHorizon,

    /// Confidence level for prediction intervals (e.g., 0.95 for 95%)
    pub confidence_level: f64,

    /// Whether to include trend analysis
    pub include_trend: bool,

    /// Whether to detect and account for seasonality
    pub detect_seasonality: bool,

    /// Minimum number of data points required
    pub min_data_points: usize,
}

/// Forecast result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastResult {
    /// Forecasted data points
    pub forecast: Vec<DataPoint>,

    /// Lower bound of prediction interval
    pub lower_bound: Vec<DataPoint>,

    /// Upper bound of prediction interval
    pub upper_bound: Vec<DataPoint>,

    /// Detected trend
    pub trend: TrendDirection,

    /// Detected seasonality
    pub seasonality: SeasonalityPattern,

    /// Model used for forecasting
    pub model_name: String,

    /// Confidence level
    pub confidence_level: f64,

    /// Forecast accuracy metrics (if historical data available for validation)
    pub metrics: Option<super::metrics::ForecastMetrics>,
}

impl Default for ForecastConfig {
    fn default() -> Self {
        Self {
            horizon: ForecastHorizon::Days(30),
            confidence_level: 0.95,
            include_trend: true,
            detect_seasonality: true,
            min_data_points: 7,
        }
    }
}

impl TimeSeriesData {
    /// Create a new time series from data points
    pub fn new(points: Vec<DataPoint>) -> Self {
        Self {
            points,
            interval_secs: None,
            metadata: None,
        }
    }

    /// Create with automatic interval detection
    pub fn with_auto_interval(mut points: Vec<DataPoint>) -> Self {
        // Sort by timestamp
        points.sort_by_key(|p| p.timestamp);

        let interval = if points.len() >= 2 {
            // Calculate average interval
            let total_secs: i64 = points
                .windows(2)
                .map(|w| (w[1].timestamp - w[0].timestamp).num_seconds())
                .sum();
            Some(total_secs / (points.len() - 1) as i64)
        } else {
            None
        };

        Self {
            points,
            interval_secs: interval,
            metadata: None,
        }
    }

    /// Get the length of the time series
    pub fn len(&self) -> usize {
        self.points.len()
    }

    /// Check if the time series is empty
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }

    /// Get values as a vector of Decimal
    pub fn values(&self) -> Vec<Decimal> {
        self.points.iter().map(|p| p.value).collect()
    }

    /// Get values as a vector of f64
    pub fn values_f64(&self) -> Vec<f64> {
        self.points
            .iter()
            .map(|p| p.value.to_string().parse::<f64>().unwrap_or(0.0))
            .collect()
    }

    /// Get the last data point
    pub fn last(&self) -> Option<&DataPoint> {
        self.points.last()
    }

    /// Get the first data point
    pub fn first(&self) -> Option<&DataPoint> {
        self.points.first()
    }

    /// Calculate mean value
    pub fn mean(&self) -> Option<Decimal> {
        if self.is_empty() {
            return None;
        }

        let sum: Decimal = self.points.iter().map(|p| p.value).sum();
        Some(sum / Decimal::from(self.points.len()))
    }

    /// Calculate standard deviation
    pub fn std_dev(&self) -> Option<f64> {
        if self.points.len() < 2 {
            return None;
        }

        let values = self.values_f64();
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        let variance = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / (values.len() - 1) as f64;

        Some(variance.sqrt())
    }

    /// Get a subset of the time series
    pub fn subset(&self, start_idx: usize, end_idx: usize) -> Self {
        let points = self.points[start_idx..end_idx].to_vec();
        Self {
            points,
            interval_secs: self.interval_secs,
            metadata: self.metadata.clone(),
        }
    }
}

impl DataPoint {
    /// Create a new data point
    pub fn new(timestamp: DateTime<Utc>, value: Decimal) -> Self {
        Self {
            timestamp,
            value,
            metadata: None,
        }
    }

    /// Create with metadata
    pub fn with_metadata(
        timestamp: DateTime<Utc>,
        value: Decimal,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            timestamp,
            value,
            metadata: Some(metadata),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn create_test_series(count: usize) -> TimeSeriesData {
        let start = Utc::now();
        let points: Vec<DataPoint> = (0..count)
            .map(|i| DataPoint::new(
                start + Duration::hours(i as i64),
                Decimal::from(i),
            ))
            .collect();

        TimeSeriesData::with_auto_interval(points)
    }

    #[test]
    fn test_time_series_creation() {
        let series = create_test_series(10);
        assert_eq!(series.len(), 10);
        assert!(!series.is_empty());
        assert!(series.interval_secs.is_some());
    }

    #[test]
    fn test_time_series_values() {
        let series = create_test_series(5);
        let values = series.values();

        assert_eq!(values.len(), 5);
        assert_eq!(values[0], Decimal::from(0));
        assert_eq!(values[4], Decimal::from(4));
    }

    #[test]
    fn test_time_series_mean() {
        let series = create_test_series(5);
        let mean = series.mean().unwrap();

        // Mean of [0, 1, 2, 3, 4] = 2
        assert_eq!(mean, Decimal::from(2));
    }

    #[test]
    fn test_time_series_std_dev() {
        let series = create_test_series(5);
        let std_dev = series.std_dev().unwrap();

        // Should be approximately 1.58 for [0, 1, 2, 3, 4]
        assert!(std_dev > 1.4 && std_dev < 1.6);
    }

    #[test]
    fn test_time_series_subset() {
        let series = create_test_series(10);
        let subset = series.subset(2, 7);

        assert_eq!(subset.len(), 5);
        assert_eq!(subset.values()[0], Decimal::from(2));
        assert_eq!(subset.values()[4], Decimal::from(6));
    }

    #[test]
    fn test_empty_time_series() {
        let series = TimeSeriesData::new(vec![]);
        assert!(series.is_empty());
        assert_eq!(series.len(), 0);
        assert!(series.mean().is_none());
        assert!(series.std_dev().is_none());
    }
}
