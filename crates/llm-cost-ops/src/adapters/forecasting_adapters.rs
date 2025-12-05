/// Benchmark adapters for forecasting operations
///
/// Provides benchmark targets for time series forecasting, anomaly detection, and budget forecasting.

use super::{BenchTarget, calculate_stats, run_iterations};
use crate::benchmarks::result::BenchmarkResult;
use crate::forecasting::{
    DataPoint, TimeSeriesData,
    LinearTrendModel, MovingAverageModel, ExponentialSmoothingModel,
    AnomalyDetector, AnomalyMethod, BudgetForecaster,
};
use crate::forecasting::anomaly::AnomalyConfig;
use crate::forecasting::budget::BudgetConfig;
use crate::forecasting::models::ForecastModel;
use chrono::{Duration as ChronoDuration, Utc};
use rust_decimal::Decimal;

/// Helper to generate test time series data
fn generate_test_data(points: usize) -> TimeSeriesData {
    let base_time = Utc::now() - ChronoDuration::days(points as i64);

    let data_points: Vec<DataPoint> = (0..points)
        .map(|i| {
            let value = 100.0 + (i as f64 * 2.0) + (i as f64 * 0.1 * ((i as f64) / 10.0).sin());
            DataPoint::new(
                base_time + ChronoDuration::days(i as i64),
                Decimal::from_f64_retain(value).unwrap_or(Decimal::from(100)),
            )
        })
        .collect();

    TimeSeriesData::with_auto_interval(data_points)
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

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let mut model = LinearTrendModel::new();
            let _ = model.train(&data);
            let _ = model.forecast(7);
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
        let window_size = self.window_size;

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let mut model = MovingAverageModel::new(window_size);
            let _ = model.train(&data);
            let _ = model.forecast(7);
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

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let mut model = ExponentialSmoothingModel::with_default_alpha();
            let _ = model.train(&data);
            let _ = model.forecast(7);
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
pub struct AnomalyDetectionBench {
    data_points: usize,
    method: AnomalyMethod,
}

impl AnomalyDetectionBench {
    pub fn new(data_points: usize, method: AnomalyMethod) -> Self {
        Self { data_points, method }
    }
}

impl BenchTarget for AnomalyDetectionBench {
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
        let config = AnomalyConfig {
            method: self.method,
            sensitivity: 2.0,
            min_data_points: 10,
            window_size: 7,
        };
        let detector = AnomalyDetector::new(config);

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = detector.detect(&data);
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
pub struct BudgetForecastBench {
    data_points: usize,
}

impl BudgetForecastBench {
    pub fn new(data_points: usize) -> Self {
        Self { data_points }
    }
}

impl BenchTarget for BudgetForecastBench {
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
        let config = BudgetConfig {
            limit: Decimal::from(10000),
            period_days: 30,
            warning_threshold: 0.80,
            critical_threshold: 0.95,
            enable_forecasting: true,
        };
        let forecaster = BudgetForecaster::new(config);
        let period_start = Utc::now() - ChronoDuration::days(self.data_points as i64);
        let period_end = Utc::now() + ChronoDuration::days(30);

        let iterations = 1000;
        let (total_duration, timings) = run_iterations(iterations, || {
            let _ = forecaster.forecast(&data, period_start, period_end);
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
        Box::new(AnomalyDetectionBench::new(30, AnomalyMethod::ZScore)),
        Box::new(AnomalyDetectionBench::new(90, AnomalyMethod::ZScore)),
        Box::new(AnomalyDetectionBench::new(30, AnomalyMethod::Iqr)),
        Box::new(AnomalyDetectionBench::new(90, AnomalyMethod::Iqr)),

        // Budget forecasting
        Box::new(BudgetForecastBench::new(30)),
        Box::new(BudgetForecastBench::new(90)),
    ]
}
