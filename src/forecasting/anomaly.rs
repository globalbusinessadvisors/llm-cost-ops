// Anomaly detection for cost forecasting

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{
    types::{DataPoint, TimeSeriesData},
    ForecastError, ForecastResult,
};

/// Anomaly detection method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnomalyMethod {
    /// Z-Score method (standard deviations from mean)
    ZScore,

    /// Interquartile Range (IQR) method
    Iqr,

    /// Moving Average method
    MovingAverage,

    /// Modified Z-Score (using median absolute deviation)
    ModifiedZScore,
}

/// Anomaly severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Detected anomaly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// Index of the anomalous data point
    pub index: usize,

    /// The anomalous data point
    pub point: DataPoint,

    /// Anomaly score (how anomalous it is)
    pub score: f64,

    /// Severity level
    pub severity: AnomalySeverity,

    /// Method used for detection
    pub method: AnomalyMethod,

    /// Additional context
    pub context: HashMap<String, String>,
}

/// Anomaly detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyResult {
    /// Detected anomalies
    pub anomalies: Vec<Anomaly>,

    /// Total number of data points analyzed
    pub total_points: usize,

    /// Anomaly rate (percentage)
    pub anomaly_rate: f64,

    /// Detection method used
    pub method: AnomalyMethod,

    /// Threshold used for detection
    pub threshold: f64,
}

/// Anomaly detector configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyConfig {
    /// Detection method
    pub method: AnomalyMethod,

    /// Sensitivity threshold (higher = more sensitive)
    pub sensitivity: f64,

    /// Minimum number of data points required
    pub min_data_points: usize,

    /// Window size for moving average method
    pub window_size: usize,
}

impl Default for AnomalyConfig {
    fn default() -> Self {
        Self {
            method: AnomalyMethod::ZScore,
            sensitivity: 3.0, // 3 standard deviations
            min_data_points: 10,
            window_size: 7,
        }
    }
}

/// Anomaly detector
pub struct AnomalyDetector {
    config: AnomalyConfig,
}

impl AnomalyDetector {
    /// Create a new anomaly detector
    pub fn new(config: AnomalyConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self {
            config: AnomalyConfig::default(),
        }
    }

    /// Detect anomalies in time series data
    pub fn detect(&self, data: &TimeSeriesData) -> ForecastResult<AnomalyResult> {
        if data.len() < self.config.min_data_points {
            return Err(ForecastError::InsufficientData(format!(
                "Anomaly detection requires at least {} data points",
                self.config.min_data_points
            )));
        }

        let anomalies = match self.config.method {
            AnomalyMethod::ZScore => self.detect_zscore(data)?,
            AnomalyMethod::Iqr => self.detect_iqr(data)?,
            AnomalyMethod::MovingAverage => self.detect_moving_average(data)?,
            AnomalyMethod::ModifiedZScore => self.detect_modified_zscore(data)?,
        };

        let anomaly_rate = if data.len() > 0 {
            (anomalies.len() as f64 / data.len() as f64) * 100.0
        } else {
            0.0
        };

        Ok(AnomalyResult {
            anomalies,
            total_points: data.len(),
            anomaly_rate,
            method: self.config.method,
            threshold: self.config.sensitivity,
        })
    }

    /// Z-Score anomaly detection
    fn detect_zscore(&self, data: &TimeSeriesData) -> ForecastResult<Vec<Anomaly>> {
        let values = data.values_f64();
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        let variance = values
            .iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;

        let std_dev = variance.sqrt();

        if std_dev < f64::EPSILON {
            return Ok(Vec::new()); // No variation, no anomalies
        }

        let mut anomalies = Vec::new();

        for (i, point) in data.points.iter().enumerate() {
            let value = values[i];
            let z_score = ((value - mean) / std_dev).abs();

            if z_score > self.config.sensitivity {
                let severity = self.calculate_severity(z_score, self.config.sensitivity);

                let mut context = HashMap::new();
                context.insert("mean".to_string(), format!("{:.2}", mean));
                context.insert("std_dev".to_string(), format!("{:.2}", std_dev));
                context.insert("z_score".to_string(), format!("{:.2}", z_score));

                anomalies.push(Anomaly {
                    index: i,
                    point: point.clone(),
                    score: z_score,
                    severity,
                    method: AnomalyMethod::ZScore,
                    context,
                });
            }
        }

        Ok(anomalies)
    }

    /// IQR (Interquartile Range) anomaly detection
    fn detect_iqr(&self, data: &TimeSeriesData) -> ForecastResult<Vec<Anomaly>> {
        let mut values = data.values_f64();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let q1_idx = values.len() / 4;
        let q3_idx = (values.len() * 3) / 4;

        let q1 = values[q1_idx];
        let q3 = values[q3_idx];
        let iqr = q3 - q1;

        if iqr < f64::EPSILON {
            return Ok(Vec::new()); // No variation
        }

        let lower_bound = q1 - self.config.sensitivity * iqr;
        let upper_bound = q3 + self.config.sensitivity * iqr;

        let mut anomalies = Vec::new();
        let original_values = data.values_f64();

        for (i, point) in data.points.iter().enumerate() {
            let value = original_values[i];

            if value < lower_bound || value > upper_bound {
                let distance_from_bound = if value < lower_bound {
                    lower_bound - value
                } else {
                    value - upper_bound
                };

                let score = distance_from_bound / iqr;
                let severity = self.calculate_severity(score, self.config.sensitivity);

                let mut context = HashMap::new();
                context.insert("q1".to_string(), format!("{:.2}", q1));
                context.insert("q3".to_string(), format!("{:.2}", q3));
                context.insert("iqr".to_string(), format!("{:.2}", iqr));
                context.insert("lower_bound".to_string(), format!("{:.2}", lower_bound));
                context.insert("upper_bound".to_string(), format!("{:.2}", upper_bound));

                anomalies.push(Anomaly {
                    index: i,
                    point: point.clone(),
                    score,
                    severity,
                    method: AnomalyMethod::Iqr,
                    context,
                });
            }
        }

        Ok(anomalies)
    }

    /// Moving Average anomaly detection
    fn detect_moving_average(&self, data: &TimeSeriesData) -> ForecastResult<Vec<Anomaly>> {
        let values = data.values_f64();
        let window_size = self.config.window_size.min(data.len() / 2);

        if window_size < 2 {
            return Err(ForecastError::InvalidConfig(
                "Window size too small for moving average".to_string(),
            ));
        }

        let mut anomalies = Vec::new();

        for (i, point) in data.points.iter().enumerate() {
            // Skip first window_size points
            if i < window_size {
                continue;
            }

            // Calculate moving average and std dev for window
            let window_start = i.saturating_sub(window_size);
            let window = &values[window_start..i];

            let window_mean = window.iter().sum::<f64>() / window.len() as f64;
            let window_variance = window
                .iter()
                .map(|v| (v - window_mean).powi(2))
                .sum::<f64>() / window.len() as f64;
            let window_std = window_variance.sqrt();

            if window_std < f64::EPSILON {
                continue; // No variation in window
            }

            let current_value = values[i];
            let deviation = ((current_value - window_mean) / window_std).abs();

            if deviation > self.config.sensitivity {
                let severity = self.calculate_severity(deviation, self.config.sensitivity);

                let mut context = HashMap::new();
                context.insert("window_mean".to_string(), format!("{:.2}", window_mean));
                context.insert("window_std".to_string(), format!("{:.2}", window_std));
                context.insert("deviation".to_string(), format!("{:.2}", deviation));

                anomalies.push(Anomaly {
                    index: i,
                    point: point.clone(),
                    score: deviation,
                    severity,
                    method: AnomalyMethod::MovingAverage,
                    context,
                });
            }
        }

        Ok(anomalies)
    }

    /// Modified Z-Score using Median Absolute Deviation
    fn detect_modified_zscore(&self, data: &TimeSeriesData) -> ForecastResult<Vec<Anomaly>> {
        let mut values = data.values_f64();
        let original_values = values.clone();

        // Calculate median
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if values.len() % 2 == 0 {
            (values[values.len() / 2 - 1] + values[values.len() / 2]) / 2.0
        } else {
            values[values.len() / 2]
        };

        // Calculate MAD (Median Absolute Deviation)
        let mut deviations: Vec<f64> = original_values
            .iter()
            .map(|v| (v - median).abs())
            .collect();

        deviations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mad = if deviations.len() % 2 == 0 {
            (deviations[deviations.len() / 2 - 1] + deviations[deviations.len() / 2]) / 2.0
        } else {
            deviations[deviations.len() / 2]
        };

        if mad < f64::EPSILON {
            return Ok(Vec::new()); // No variation
        }

        let mut anomalies = Vec::new();

        for (i, point) in data.points.iter().enumerate() {
            let value = original_values[i];
            // Modified Z-score = 0.6745 * (x - median) / MAD
            let modified_z = 0.6745 * ((value - median) / mad).abs();

            if modified_z > self.config.sensitivity {
                let severity = self.calculate_severity(modified_z, self.config.sensitivity);

                let mut context = HashMap::new();
                context.insert("median".to_string(), format!("{:.2}", median));
                context.insert("mad".to_string(), format!("{:.2}", mad));
                context.insert("modified_z".to_string(), format!("{:.2}", modified_z));

                anomalies.push(Anomaly {
                    index: i,
                    point: point.clone(),
                    score: modified_z,
                    severity,
                    method: AnomalyMethod::ModifiedZScore,
                    context,
                });
            }
        }

        Ok(anomalies)
    }

    /// Calculate severity based on score
    fn calculate_severity(&self, score: f64, threshold: f64) -> AnomalySeverity {
        let ratio = score / threshold;

        if ratio > 2.0 {
            AnomalySeverity::Critical
        } else if ratio > 1.5 {
            AnomalySeverity::High
        } else if ratio > 1.2 {
            AnomalySeverity::Medium
        } else {
            AnomalySeverity::Low
        }
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use rust_decimal::Decimal;

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
    fn test_detector_creation() {
        let detector = AnomalyDetector::with_defaults();
        assert_eq!(detector.config.method, AnomalyMethod::ZScore);
        assert_eq!(detector.config.sensitivity, 3.0);
    }

    #[test]
    fn test_zscore_detection() {
        let detector = AnomalyDetector::with_defaults();

        // Data with one clear outlier
        let data = create_test_series(vec![10, 12, 11, 13, 100, 12, 11, 10, 12, 11]);

        let result = detector.detect(&data).unwrap();

        assert!(result.anomalies.len() > 0);
        assert!(result.anomaly_rate > 0.0);
        assert_eq!(result.method, AnomalyMethod::ZScore);

        // The outlier (100) should be detected
        let outlier_detected = result.anomalies.iter().any(|a| a.point.value == Decimal::from(100));
        assert!(outlier_detected);
    }

    #[test]
    fn test_iqr_detection() {
        let mut config = AnomalyConfig::default();
        config.method = AnomalyMethod::Iqr;
        config.sensitivity = 1.5;

        let detector = AnomalyDetector::new(config);
        let data = create_test_series(vec![10, 12, 11, 13, 12, 150, 11, 10, 12, 11]);

        let result = detector.detect(&data).unwrap();

        assert!(result.anomalies.len() > 0);
        assert_eq!(result.method, AnomalyMethod::Iqr);

        // The outlier (150) should be detected
        let outlier_detected = result.anomalies.iter().any(|a| a.point.value == Decimal::from(150));
        assert!(outlier_detected);
    }

    #[test]
    fn test_moving_average_detection() {
        let mut config = AnomalyConfig::default();
        config.method = AnomalyMethod::MovingAverage;
        config.window_size = 3;
        config.sensitivity = 3.0;

        let detector = AnomalyDetector::new(config);
        let data = create_test_series(vec![10, 12, 11, 13, 12, 11, 100, 10, 12, 11, 13]);

        let result = detector.detect(&data).unwrap();

        assert!(result.anomalies.len() > 0);
        assert_eq!(result.method, AnomalyMethod::MovingAverage);
    }

    #[test]
    fn test_modified_zscore_detection() {
        let mut config = AnomalyConfig::default();
        config.method = AnomalyMethod::ModifiedZScore;
        config.sensitivity = 3.5;

        let detector = AnomalyDetector::new(config);
        let data = create_test_series(vec![10, 12, 11, 13, 12, 200, 11, 10, 12, 11]);

        let result = detector.detect(&data).unwrap();

        assert!(result.anomalies.len() > 0);
        assert_eq!(result.method, AnomalyMethod::ModifiedZScore);

        // The outlier (200) should be detected
        let outlier_detected = result.anomalies.iter().any(|a| a.point.value == Decimal::from(200));
        assert!(outlier_detected);
    }

    #[test]
    fn test_no_anomalies() {
        let detector = AnomalyDetector::with_defaults();

        // Stable data with no outliers
        let data = create_test_series(vec![10, 11, 12, 11, 10, 12, 11, 10, 11, 12]);

        let result = detector.detect(&data).unwrap();

        assert_eq!(result.anomalies.len(), 0);
        assert_eq!(result.anomaly_rate, 0.0);
    }

    #[test]
    fn test_insufficient_data() {
        let detector = AnomalyDetector::with_defaults();
        let data = create_test_series(vec![10, 12, 11]); // Less than min_data_points

        let result = detector.detect(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_severity_calculation() {
        let detector = AnomalyDetector::with_defaults();

        assert_eq!(
            detector.calculate_severity(6.0, 3.0),
            AnomalySeverity::Critical
        );
        assert_eq!(
            detector.calculate_severity(5.0, 3.0),
            AnomalySeverity::High
        );
        assert_eq!(
            detector.calculate_severity(4.0, 3.0),
            AnomalySeverity::Medium
        );
        assert_eq!(
            detector.calculate_severity(3.5, 3.0),
            AnomalySeverity::Low
        );
    }

    #[test]
    fn test_anomaly_context() {
        let detector = AnomalyDetector::with_defaults();
        let data = create_test_series(vec![10, 12, 11, 13, 100, 12, 11, 10, 12, 11]);

        let result = detector.detect(&data).unwrap();

        if let Some(anomaly) = result.anomalies.first() {
            assert!(anomaly.context.contains_key("mean"));
            assert!(anomaly.context.contains_key("std_dev"));
            assert!(anomaly.context.contains_key("z_score"));
        }
    }

    #[test]
    fn test_different_sensitivities() {
        let mut config = AnomalyConfig::default();
        let data = create_test_series(vec![10, 12, 11, 13, 25, 12, 11, 10, 12, 11]);

        // High sensitivity (lower threshold)
        config.sensitivity = 2.0;
        let detector = AnomalyDetector::new(config.clone());
        let result_high = detector.detect(&data).unwrap();

        // Low sensitivity (higher threshold)
        config.sensitivity = 4.0;
        let detector = AnomalyDetector::new(config);
        let result_low = detector.detect(&data).unwrap();

        // High sensitivity should detect more anomalies
        assert!(result_high.anomalies.len() >= result_low.anomalies.len());
    }
}
