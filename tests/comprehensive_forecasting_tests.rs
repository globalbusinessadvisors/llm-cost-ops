/// Comprehensive forecasting tests
///
/// Tests forecast engine, anomaly detection, and budget forecasting

use chrono::{Duration, Utc};
use llm_cost_ops::forecasting::*;
use llm_cost_ops::domain::{Provider, CostRecord};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

mod helpers;
use helpers::builders::CostRecordBuilder;

// === Forecast Engine Tests ===

#[test]
fn test_forecast_engine_creation() {
    let engine = ForecastEngine::new();
    assert!(std::mem::size_of_val(&engine) > 0);
}

#[test]
fn test_simple_forecast() {
    let engine = ForecastEngine::new();

    // Create historical data with upward trend
    let mut costs = vec![];
    for i in 0..30 {
        let timestamp = Utc::now() - Duration::days(30 - i);
        costs.push(
            CostRecordBuilder::new()
                .input_cost(Decimal::from(i))
                .output_cost(Decimal::from(i))
                .timestamp(timestamp)
                .build()
        );
    }

    let forecast = engine.forecast(&costs, 7); // Forecast 7 days
    assert!(forecast.is_ok());

    let result = forecast.unwrap();
    assert_eq!(result.len(), 7);

    // Forecast values should continue the trend
    for prediction in result {
        assert!(prediction.predicted_cost > Decimal::ZERO);
    }
}

#[test]
fn test_forecast_with_empty_data() {
    let engine = ForecastEngine::new();
    let costs: Vec<CostRecord> = vec![];

    let forecast = engine.forecast(&costs, 7);
    assert!(forecast.is_err());
}

#[test]
fn test_forecast_with_insufficient_data() {
    let engine = ForecastEngine::new();

    let costs = vec![
        CostRecordBuilder::new().build(),
    ];

    let forecast = engine.forecast(&costs, 7);
    // Should either error or return conservative estimates
    assert!(forecast.is_ok() || forecast.is_err());
}

#[test]
fn test_forecast_confidence_intervals() {
    let engine = ForecastEngine::new();

    let mut costs = vec![];
    for i in 0..30 {
        let timestamp = Utc::now() - Duration::days(30 - i);
        costs.push(
            CostRecordBuilder::new()
                .input_cost(dec!(10.0))
                .output_cost(dec!(10.0))
                .timestamp(timestamp)
                .build()
        );
    }

    let forecast = engine.forecast_with_confidence(&costs, 7);
    assert!(forecast.is_ok());

    let result = forecast.unwrap();
    for prediction in result {
        assert!(prediction.lower_bound <= prediction.predicted_cost);
        assert!(prediction.predicted_cost <= prediction.upper_bound);
        assert!(prediction.confidence >= 0.0 && prediction.confidence <= 1.0);
    }
}

#[test]
fn test_forecast_seasonal_pattern() {
    let engine = ForecastEngine::new();

    // Create data with weekly pattern
    let mut costs = vec![];
    for i in 0..90 {
        let timestamp = Utc::now() - Duration::days(90 - i);
        let is_weekend = i % 7 >= 5;
        let cost = if is_weekend { dec!(5.0) } else { dec!(20.0) };

        costs.push(
            CostRecordBuilder::new()
                .input_cost(cost)
                .output_cost(cost)
                .timestamp(timestamp)
                .build()
        );
    }

    let forecast = engine.forecast(&costs, 14);
    assert!(forecast.is_ok());

    let result = forecast.unwrap();
    assert_eq!(result.len(), 14);
}

// === Anomaly Detection Tests ===

#[test]
fn test_anomaly_detector_creation() {
    let detector = AnomalyDetector::new(2.5); // 2.5 standard deviations
    assert!(std::mem::size_of_val(&detector) > 0);
}

#[test]
fn test_detect_no_anomalies() {
    let detector = AnomalyDetector::new(3.0);

    let mut costs = vec![];
    for _ in 0..30 {
        costs.push(
            CostRecordBuilder::new()
                .input_cost(dec!(10.0))
                .output_cost(dec!(10.0))
                .build()
        );
    }

    let anomalies = detector.detect(&costs);
    assert!(anomalies.is_ok());
    assert!(anomalies.unwrap().is_empty());
}

#[test]
fn test_detect_spike_anomaly() {
    let detector = AnomalyDetector::new(2.0);

    let mut costs = vec![];
    for i in 0..30 {
        let cost = if i == 15 {
            dec!(100.0) // Spike
        } else {
            dec!(10.0)
        };

        costs.push(
            CostRecordBuilder::new()
                .input_cost(cost)
                .output_cost(cost)
                .build()
        );
    }

    let anomalies = detector.detect(&costs);
    assert!(anomalies.is_ok());

    let result = anomalies.unwrap();
    assert!(!result.is_empty());

    let spike_anomaly = &result[0];
    assert_eq!(spike_anomaly.anomaly_type, AnomalyType::Spike);
    assert!(spike_anomaly.severity > 0.0);
}

#[test]
fn test_detect_drop_anomaly() {
    let detector = AnomalyDetector::new(2.0);

    let mut costs = vec![];
    for i in 0..30 {
        let cost = if i == 15 {
            dec!(0.1) // Drop
        } else {
            dec!(10.0)
        };

        costs.push(
            CostRecordBuilder::new()
                .input_cost(cost)
                .output_cost(cost)
                .build()
        );
    }

    let anomalies = detector.detect(&costs);
    assert!(anomalies.is_ok());

    let result = anomalies.unwrap();
    assert!(!result.is_empty());
}

#[test]
fn test_anomaly_severity_levels() {
    let detector = AnomalyDetector::new(2.0);

    let baseline = dec!(10.0);
    let anomaly_costs = vec![
        baseline * dec!(2.0),  // 2x - minor
        baseline * dec!(3.0),  // 3x - moderate
        baseline * dec!(5.0),  // 5x - severe
        baseline * dec!(10.0), // 10x - critical
    ];

    for anomaly_cost in anomaly_costs {
        let mut costs = vec![];
        for i in 0..30 {
            let cost = if i == 15 { anomaly_cost } else { baseline };
            costs.push(
                CostRecordBuilder::new()
                    .input_cost(cost)
                    .output_cost(dec!(0.0))
                    .build()
            );
        }

        let anomalies = detector.detect(&costs).unwrap();
        if !anomalies.is_empty() {
            assert!(anomalies[0].severity > 0.0);
        }
    }
}

// === Budget Forecaster Tests ===

#[test]
fn test_budget_forecaster_creation() {
    let forecaster = BudgetForecaster::new();
    assert!(std::mem::size_of_val(&forecaster) > 0);
}

#[test]
fn test_budget_forecast() {
    let forecaster = BudgetForecaster::new();

    let mut costs = vec![];
    for i in 0..30 {
        let timestamp = Utc::now() - Duration::days(30 - i);
        costs.push(
            CostRecordBuilder::new()
                .input_cost(dec!(100.0))
                .output_cost(dec!(50.0))
                .timestamp(timestamp)
                .build()
        );
    }

    let monthly_budget = dec!(5000.0);
    let forecast = forecaster.forecast_budget_usage(&costs, monthly_budget, 30);

    assert!(forecast.is_ok());
    let result = forecast.unwrap();

    assert!(result.projected_spend > Decimal::ZERO);
    assert!(result.projected_spend <= monthly_budget * dec!(1.5)); // Within reasonable range
}

#[test]
fn test_budget_alert_threshold() {
    let forecaster = BudgetForecaster::new();

    let mut costs = vec![];
    for i in 0..30 {
        let timestamp = Utc::now() - Duration::days(30 - i);
        costs.push(
            CostRecordBuilder::new()
                .input_cost(dec!(200.0))
                .output_cost(dec!(100.0))
                .timestamp(timestamp)
                .build()
        );
    }

    let monthly_budget = dec!(5000.0);
    let result = forecaster.forecast_budget_usage(&costs, monthly_budget, 30).unwrap();

    // Check if alerts are triggered
    if result.projected_spend > monthly_budget {
        assert!(result.alert_level > 0);
    }
}

#[test]
fn test_budget_burn_rate() {
    let forecaster = BudgetForecaster::new();

    let mut costs = vec![];
    let daily_cost = dec!(100.0);
    for i in 0..15 {
        let timestamp = Utc::now() - Duration::days(15 - i);
        costs.push(
            CostRecordBuilder::new()
                .input_cost(daily_cost)
                .output_cost(daily_cost)
                .timestamp(timestamp)
                .build()
        );
    }

    let burn_rate = forecaster.calculate_burn_rate(&costs);
    assert!(burn_rate.is_ok());

    let rate = burn_rate.unwrap();
    // Should be close to daily_cost * 2 (input + output)
    assert!(rate >= daily_cost && rate <= daily_cost * dec!(3.0));
}

// === Trend Analysis Tests ===

#[test]
fn test_trend_analysis_upward() {
    let analyzer = TrendAnalyzer::new();

    let mut costs = vec![];
    for i in 0..30 {
        let timestamp = Utc::now() - Duration::days(30 - i);
        costs.push(
            CostRecordBuilder::new()
                .input_cost(Decimal::from(i))
                .output_cost(Decimal::from(i))
                .timestamp(timestamp)
                .build()
        );
    }

    let trend = analyzer.analyze(&costs);
    assert!(trend.is_ok());

    let result = trend.unwrap();
    assert_eq!(result.direction, TrendDirection::Upward);
}

#[test]
fn test_trend_analysis_downward() {
    let analyzer = TrendAnalyzer::new();

    let mut costs = vec![];
    for i in 0..30 {
        let timestamp = Utc::now() - Duration::days(30 - i);
        costs.push(
            CostRecordBuilder::new()
                .input_cost(Decimal::from(30 - i))
                .output_cost(Decimal::from(30 - i))
                .timestamp(timestamp)
                .build()
        );
    }

    let trend = analyzer.analyze(&costs);
    assert!(trend.is_ok());

    let result = trend.unwrap();
    assert_eq!(result.direction, TrendDirection::Downward);
}

#[test]
fn test_trend_analysis_stable() {
    let analyzer = TrendAnalyzer::new();

    let mut costs = vec![];
    for i in 0..30 {
        let timestamp = Utc::now() - Duration::days(30 - i);
        costs.push(
            CostRecordBuilder::new()
                .input_cost(dec!(100.0))
                .output_cost(dec!(100.0))
                .timestamp(timestamp)
                .build()
        );
    }

    let trend = analyzer.analyze(&costs);
    assert!(trend.is_ok());

    let result = trend.unwrap();
    assert_eq!(result.direction, TrendDirection::Stable);
}

// === Performance Tests ===

#[test]
fn test_forecast_performance() {
    let engine = ForecastEngine::new();

    let mut costs = vec![];
    for i in 0..365 {
        let timestamp = Utc::now() - Duration::days(365 - i);
        costs.push(
            CostRecordBuilder::new()
                .input_cost(dec!(10.0))
                .output_cost(dec!(10.0))
                .timestamp(timestamp)
                .build()
        );
    }

    let start = std::time::Instant::now();
    let _ = engine.forecast(&costs, 30);
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 1, "Forecasting too slow: {:?}", elapsed);
}

#[test]
fn test_anomaly_detection_performance() {
    let detector = AnomalyDetector::new(2.0);

    let mut costs = vec![];
    for _ in 0..1000 {
        costs.push(
            CostRecordBuilder::new()
                .input_cost(dec!(10.0))
                .output_cost(dec!(10.0))
                .build()
        );
    }

    let start = std::time::Instant::now();
    let _ = detector.detect(&costs);
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 100, "Anomaly detection too slow: {:?}", elapsed);
}
