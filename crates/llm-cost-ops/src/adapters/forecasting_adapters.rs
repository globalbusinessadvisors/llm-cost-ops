/// Benchmark adapters for forecasting operations
///
/// Provides benchmark targets for time series forecasting, anomaly detection, and budget forecasting.

use super::{BenchTarget, calculate_stats, run_iterations};
use crate::benchmarks::result::BenchmarkResult;
use crate::forecasting::{
    DataPoint, ForecastConfig, ForecastEngine, ForecastHorizon,
    LinearTrendModel, MovingAverageModel, ExponentialSmoothingModel,
    AnomalyDetector, AnomalyMethod, BudgetForecaster,
};
use chrono::{Duration as ChronoDuration, Utc};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// Helper to generate test time series data
fn generate_test_data(points: usize) -> Vec<DataPoint> {
    let base_time = Utc::now() - ChronoDuration::days(points as i64);

    (0..points)
        .map(|i| {
            let value = 100.0 + (i as f64 * 2.0) + (i as f64 * 0.1 * ((i as f64) / 10.0).sin());
            DataPoint {
                timestamp: base_time + ChronoDuration::days(i as i64),
                value,
            }
        })
        .collect()
}

/// Benchmark: Linear trend forecasting
pub struct LinearTrendForecasting {
    data_points: usize,
}

impl LinearTrendForecasting {
    pub fn new(data_points: usize) -> Self {
        Self { data_points }
    }
}

impl BenchTarget for LinearTrendForecasting {
    fn id(&self) -> String {
        format!("forecasting/linear_trend_{}", self.data_points)
    }

    fn name(&self) -> String {
        format!("Linear Trend Forecasting ({} points)", self.data_points)
    }

    fn category(&self) -> String {
        "forecasting".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let data = generate_test_data(self.data_points);
        let model = LinearTrendModel::new();
        let config = ForecastConfig {
            horizon: ForecastHorizon::Days(7),
            confidence_level: 0.95,
        };

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = model.forecast(&data, &config);
        });

        let (min, max, std_dev) = calculate_stats(&timings);

        let metadata = serde_json::json!({
            "model": "linear_trend",
            "data_points": self.data_points,
            "horizon": "7 days",
        });

        BenchmarkResult::success(
            self.id(),
            self.name(),
            self.category(),
            total_duration,
            iterations,
        )
        .with_stats(min, max, std_dev)
        .with_metadata(metadata)
    }
}

/// Benchmark: Moving average forecasting
pub struct MovingAverageForecasting {
    data_points: usize,
    window_size: usize,
}

impl MovingAverageForecasting {
    pub fn new(data_points: usize, window_size: usize) -> Self {
        Self { data_points, window_size }
    }
}

impl BenchTarget for MovingAverageForecasting {
    fn id(&self) -> String {
        format!("forecasting/moving_average_{}_{}", self.data_points, self.window_size)
    }

    fn name(&self) -> String {
        format!("Moving Average Forecasting ({} points, window {})",
            self.data_points, self.window_size)
    }

    fn category(&self) -> String {
        "forecasting".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let data = generate_test_data(self.data_points);
        let model = MovingAverageModel::new(self.window_size);
        let config = ForecastConfig {
            horizon: ForecastHorizon::Days(7),
            confidence_level: 0.95,
        };

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = model.forecast(&data, &config);
        });

        let (min, max, std_dev) = calculate_stats(&timings);

        let metadata = serde_json::json!({
            "model": "moving_average",
            "data_points": self.data_points,
            "window_size": self.window_size,
            "horizon": "7 days",
        });

        BenchmarkResult::success(
            self.id(),
            self.name(),
            self.category(),
            total_duration,
            iterations,
        )
        .with_stats(min, max, std_dev)
        .with_metadata(metadata)
    }
}

/// Benchmark: Exponential smoothing forecasting
pub struct ExponentialSmoothingForecasting {
    data_points: usize,
}

impl ExponentialSmoothingForecasting {
    pub fn new(data_points: usize) -> Self {
        Self { data_points }
    }
}

impl BenchTarget for ExponentialSmoothingForecasting {
    fn id(&self) -> String {
        format!("forecasting/exponential_smoothing_{}", self.data_points)
    }

    fn name(&self) -> String {
        format!("Exponential Smoothing Forecasting ({} points)", self.data_points)
    }

    fn category(&self) -> String {
        "forecasting".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let data = generate_test_data(self.data_points);
        let model = ExponentialSmoothingModel::new(0.3); // alpha = 0.3
        let config = ForecastConfig {
            horizon: ForecastHorizon::Days(7),
            confidence_level: 0.95,
        };

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = model.forecast(&data, &config);
        });

        let (min, max, std_dev) = calculate_stats(&timings);

        let metadata = serde_json::json!({
            "model": "exponential_smoothing",
            "data_points": self.data_points,
            "alpha": 0.3,
            "horizon": "7 days",
        });

        BenchmarkResult::success(
            self.id(),
            self.name(),
            self.category(),
            total_duration,
            iterations,
        )
        .with_stats(min, max, std_dev)
        .with_metadata(metadata)
    }
}

/// Benchmark: Anomaly detection
pub struct AnomalyDetection {
    data_points: usize,
    method: AnomalyMethod,
}

impl AnomalyDetection {
    pub fn new(data_points: usize, method: AnomalyMethod) -> Self {
        Self { data_points, method }
    }
}

impl BenchTarget for AnomalyDetection {
    fn id(&self) -> String {
        format!("forecasting/anomaly_detection_{:?}_{}", self.method, self.data_points)
    }

    fn name(&self) -> String {
        format!("Anomaly Detection ({:?}, {} points)", self.method, self.data_points)
    }

    fn category(&self) -> String {
        "forecasting".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let data = generate_test_data(self.data_points);
        let detector = AnomalyDetector::new(self.method.clone());

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = detector.detect(&data, 2.0); // 2.0 standard deviations threshold
        });

        let (min, max, std_dev) = calculate_stats(&timings);

        let metadata = serde_json::json!({
            "method": format!("{:?}", self.method),
            "data_points": self.data_points,
            "threshold": 2.0,
        });

        BenchmarkResult::success(
            self.id(),
            self.name(),
            self.category(),
            total_duration,
            iterations,
        )
        .with_stats(min, max, std_dev)
        .with_metadata(metadata)
    }
}

/// Benchmark: Budget forecasting
pub struct BudgetForecast {
    data_points: usize,
}

impl BudgetForecast {
    pub fn new(data_points: usize) -> Self {
        Self { data_points }
    }
}

impl BenchTarget for BudgetForecast {
    fn id(&self) -> String {
        format!("forecasting/budget_forecast_{}", self.data_points)
    }

    fn name(&self) -> String {
        format!("Budget Forecasting ({} points)", self.data_points)
    }

    fn category(&self) -> String {
        "forecasting".to_string()
    }

    fn run(&self) -> BenchmarkResult {
        let data = generate_test_data(self.data_points);
        let forecaster = BudgetForecaster::new();
        let monthly_budget = dec!(10000.0);

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = forecaster.forecast_budget(&data, monthly_budget);
        });

        let (min, max, std_dev) = calculate_stats(&timings);

        let metadata = serde_json::json!({
            "data_points": self.data_points,
            "monthly_budget": 10000.0,
        });

        BenchmarkResult::success(
            self.id(),
            self.name(),
            self.category(),
            total_duration,
            iterations,
        )
        .with_stats(min, max, std_dev)
        .with_metadata(metadata)
    }
}

/// Create all forecasting benchmark targets
pub fn create_targets() -> Vec<Box<dyn BenchTarget>> {
    vec![
        // Linear trend forecasting
        Box::new(LinearTrendForecasting::new(30)),
        Box::new(LinearTrendForecasting::new(90)),
        Box::new(LinearTrendForecasting::new(365)),

        // Moving average forecasting
        Box::new(MovingAverageForecasting::new(30, 7)),
        Box::new(MovingAverageForecasting::new(90, 14)),
        Box::new(MovingAverageForecasting::new(365, 30)),

        // Exponential smoothing forecasting
        Box::new(ExponentialSmoothingForecasting::new(30)),
        Box::new(ExponentialSmoothingForecasting::new(90)),
        Box::new(ExponentialSmoothingForecasting::new(365)),

        // Anomaly detection
        Box::new(AnomalyDetection::new(30, AnomalyMethod::ZScore)),
        Box::new(AnomalyDetection::new(90, AnomalyMethod::ZScore)),
        Box::new(AnomalyDetection::new(30, AnomalyMethod::IQR)),
        Box::new(AnomalyDetection::new(90, AnomalyMethod::IQR)),

        // Budget forecasting
        Box::new(BudgetForecast::new(30)),
        Box::new(BudgetForecast::new(90)),
    ]
}
