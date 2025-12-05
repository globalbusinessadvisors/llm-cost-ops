//! LLM-Analytics-Hub Integration
//!
//! Thin adapter for consuming aggregated usage baselines, historical curves,
//! and forecasting clusters from LLM-Analytics-Hub.
//!
//! This module provides a "consumes-from" integration that receives analytics
//! data and converts it into CostOps forecasting types for enhanced predictions.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// Import from llm-analytics-hub
use llm_analytics_hub as analytics;

use crate::forecasting::{
    DataPoint, TimeSeriesData, ForecastConfig, ForecastHorizon,
    TrendDirection, SeasonalityPattern,
};

/// Errors that can occur during Analytics Hub integration
#[derive(Debug, Error)]
pub enum AnalyticsHubError {
    #[error("Failed to connect to Analytics Hub: {0}")]
    ConnectionError(String),

    #[error("Failed to parse analytics data: {0}")]
    ParseError(String),

    #[error("Invalid baseline format: {0}")]
    InvalidBaseline(String),

    #[error("Invalid curve data: {0}")]
    InvalidCurve(String),

    #[error("Cluster not found: {0}")]
    ClusterNotFound(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),

    #[error("Insufficient data for operation: {0}")]
    InsufficientData(String),
}

/// Configuration for Analytics Hub integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsHubConfig {
    /// Whether the integration is enabled
    pub enabled: bool,

    /// Analytics Hub endpoint URL (if remote)
    pub endpoint: Option<String>,

    /// Default time window for baselines (in days)
    pub default_baseline_window_days: u64,

    /// Minimum data points required for forecasting
    pub min_data_points: usize,

    /// Whether to apply seasonal adjustments
    pub apply_seasonal_adjustments: bool,

    /// Cache TTL for baselines (in seconds)
    pub cache_ttl_seconds: u64,
}

impl Default for AnalyticsHubConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            endpoint: None,
            default_baseline_window_days: 30,
            min_data_points: 7,
            apply_seasonal_adjustments: true,
            cache_ttl_seconds: 3600,
        }
    }
}

/// Usage baseline from Analytics Hub
///
/// Represents aggregated historical usage patterns for a specific
/// organization, provider, or model combination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageBaseline {
    /// Baseline identifier
    pub baseline_id: String,

    /// Time range covered by this baseline
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,

    /// Granularity of the baseline (e.g., "hourly", "daily", "weekly")
    pub granularity: String,

    /// Organization filter (None = all orgs)
    pub organization_id: Option<String>,

    /// Provider filter (None = all providers)
    pub provider: Option<String>,

    /// Model filter (None = all models)
    pub model: Option<String>,

    /// Baseline statistics
    pub mean_tokens_per_period: f64,
    pub std_dev_tokens: f64,
    pub mean_cost_per_period: Decimal,
    pub std_dev_cost: f64,

    /// Percentile values
    pub p50_tokens: u64,
    pub p90_tokens: u64,
    pub p99_tokens: u64,
    pub p50_cost: Decimal,
    pub p90_cost: Decimal,
    pub p99_cost: Decimal,

    /// Detected patterns
    pub trend_direction: String,
    pub seasonality_detected: bool,
    pub seasonality_period: Option<String>,

    /// When this baseline was computed
    pub computed_at: DateTime<Utc>,
}

/// Historical curve from Analytics Hub
///
/// Time-series data representing historical usage or cost patterns
/// with optional confidence intervals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalCurve {
    /// Curve identifier
    pub curve_id: String,

    /// Metric type (e.g., "token_usage", "cost", "latency")
    pub metric_type: String,

    /// Data points
    pub data_points: Vec<CurveDataPoint>,

    /// Optional upper confidence bound
    pub upper_bound: Option<Vec<CurveDataPoint>>,

    /// Optional lower confidence bound
    pub lower_bound: Option<Vec<CurveDataPoint>>,

    /// Confidence level (e.g., 0.95 for 95%)
    pub confidence_level: Option<f64>,

    /// Curve metadata
    pub organization_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,

    /// Aggregation window (in seconds)
    pub window_seconds: u64,
}

/// Individual data point in a historical curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: Decimal,
    pub count: Option<u64>,
}

/// Forecasting cluster from Analytics Hub
///
/// Represents a cluster of similar usage patterns that can be used
/// to improve forecasting accuracy through pattern matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastingCluster {
    /// Cluster identifier
    pub cluster_id: String,

    /// Cluster name/label
    pub cluster_name: String,

    /// Number of members in this cluster
    pub member_count: u64,

    /// Cluster centroid (representative pattern)
    pub centroid: Vec<Decimal>,

    /// Cluster characteristics
    pub mean_daily_tokens: f64,
    pub mean_daily_cost: Decimal,
    pub growth_rate_percent: f64,

    /// Dominant patterns
    pub dominant_trend: String,
    pub dominant_seasonality: Option<String>,

    /// Time-of-day distribution (24 values, one per hour)
    pub hourly_distribution: Option<Vec<f64>>,

    /// Day-of-week distribution (7 values)
    pub daily_distribution: Option<Vec<f64>>,

    /// Cluster quality metrics
    pub silhouette_score: f64,
    pub cohesion: f64,

    /// When this cluster was computed
    pub computed_at: DateTime<Utc>,
}

/// Consumer for Analytics Hub data
pub struct AnalyticsHubConsumer {
    config: AnalyticsHubConfig,
}

impl AnalyticsHubConsumer {
    /// Create a new Analytics Hub consumer with the given configuration
    pub fn new(config: AnalyticsHubConfig) -> Self {
        Self { config }
    }

    /// Create with default configuration
    pub fn with_defaults() -> Self {
        Self::new(AnalyticsHubConfig::default())
    }

    /// Check if the integration is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Convert a UsageBaseline into ForecastConfig
    ///
    /// Uses baseline statistics to configure appropriate forecasting parameters.
    pub fn baseline_to_forecast_config(
        &self,
        baseline: &UsageBaseline,
    ) -> Result<ForecastConfig, AnalyticsHubError> {
        // Determine appropriate horizon based on baseline granularity
        let horizon = match baseline.granularity.as_str() {
            "hourly" => ForecastHorizon::Days(7),
            "daily" => ForecastHorizon::Days(30),
            "weekly" => ForecastHorizon::Days(90),
            _ => ForecastHorizon::Days(self.config.default_baseline_window_days),
        };

        // Determine appropriate minimum data points
        let min_data_points = if baseline.seasonality_detected {
            self.config.min_data_points.max(14) // Need more data for seasonality
        } else {
            self.config.min_data_points
        };

        Ok(ForecastConfig {
            horizon,
            confidence_level: 0.95,
            include_trend: true,
            detect_seasonality: baseline.seasonality_detected,
            min_data_points,
        })
    }

    /// Convert a HistoricalCurve into TimeSeriesData
    pub fn curve_to_time_series(
        &self,
        curve: &HistoricalCurve,
    ) -> Result<TimeSeriesData, AnalyticsHubError> {
        if curve.data_points.is_empty() {
            return Err(AnalyticsHubError::InsufficientData(
                "Historical curve contains no data points".to_string(),
            ));
        }

        let data_points: Vec<DataPoint> = curve
            .data_points
            .iter()
            .map(|p| DataPoint::new(p.timestamp, p.value))
            .collect();

        Ok(TimeSeriesData::with_auto_interval(data_points))
    }

    /// Convert HistoricalCurve bounds into forecast bounds
    pub fn curve_bounds_to_forecast_bounds(
        &self,
        curve: &HistoricalCurve,
    ) -> Result<(Option<TimeSeriesData>, Option<TimeSeriesData>), AnalyticsHubError> {
        let upper = curve.upper_bound.as_ref().map(|points| {
            let data_points: Vec<DataPoint> = points
                .iter()
                .map(|p| DataPoint::new(p.timestamp, p.value))
                .collect();
            TimeSeriesData::with_auto_interval(data_points)
        });

        let lower = curve.lower_bound.as_ref().map(|points| {
            let data_points: Vec<DataPoint> = points
                .iter()
                .map(|p| DataPoint::new(p.timestamp, p.value))
                .collect();
            TimeSeriesData::with_auto_interval(data_points)
        });

        Ok((lower, upper))
    }

    /// Extract trend direction from baseline
    pub fn baseline_trend(&self, baseline: &UsageBaseline) -> TrendDirection {
        match baseline.trend_direction.to_lowercase().as_str() {
            "increasing" | "up" | "growing" => TrendDirection::Increasing,
            "decreasing" | "down" | "declining" => TrendDirection::Decreasing,
            "stable" | "flat" | "steady" => TrendDirection::Stable,
            _ => TrendDirection::Unknown,
        }
    }

    /// Extract seasonality pattern from baseline
    pub fn baseline_seasonality(&self, baseline: &UsageBaseline) -> SeasonalityPattern {
        if !baseline.seasonality_detected {
            return SeasonalityPattern {
                detected: false,
                period: None,
                strength: 0.0,
            };
        }

        let period = baseline.seasonality_period.as_ref().and_then(|p| {
            match p.to_lowercase().as_str() {
                "hourly" => Some(24),
                "daily" => Some(7),
                "weekly" => Some(4),
                "monthly" => Some(12),
                _ => None,
            }
        });

        SeasonalityPattern {
            detected: true,
            period,
            strength: 0.5, // Default strength if not provided
        }
    }

    /// Find the best matching cluster for a usage pattern
    pub fn match_cluster<'a>(
        &self,
        pattern: &TimeSeriesData,
        clusters: &'a [ForecastingCluster],
    ) -> Option<&'a ForecastingCluster> {
        if clusters.is_empty() || pattern.is_empty() {
            return None;
        }

        // Calculate pattern mean for simple matching
        let pattern_mean = pattern.mean().map(|d| {
            d.to_string().parse::<f64>().unwrap_or(0.0)
        }).unwrap_or(0.0);

        // Find cluster with closest mean
        clusters.iter().min_by(|a, b| {
            let diff_a = (a.mean_daily_cost.to_string().parse::<f64>().unwrap_or(0.0) - pattern_mean).abs();
            let diff_b = (b.mean_daily_cost.to_string().parse::<f64>().unwrap_or(0.0) - pattern_mean).abs();
            diff_a.partial_cmp(&diff_b).unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Apply cluster adjustments to a forecast configuration
    pub fn apply_cluster_adjustments(
        &self,
        config: &ForecastConfig,
        cluster: &ForecastingCluster,
    ) -> ForecastConfig {
        let mut adjusted = config.clone();

        // Adjust seasonality detection based on cluster characteristics
        if cluster.dominant_seasonality.is_some() {
            adjusted.detect_seasonality = true;
        }

        // Adjust trend inclusion based on growth rate
        if cluster.growth_rate_percent.abs() > 5.0 {
            adjusted.include_trend = true;
        }

        adjusted
    }

    /// Generate seasonal adjustment factors from cluster data
    pub fn cluster_seasonal_factors(
        &self,
        cluster: &ForecastingCluster,
    ) -> Option<HashMap<String, Vec<f64>>> {
        let mut factors = HashMap::new();

        if let Some(hourly) = &cluster.hourly_distribution {
            factors.insert("hourly".to_string(), hourly.clone());
        }

        if let Some(daily) = &cluster.daily_distribution {
            factors.insert("daily".to_string(), daily.clone());
        }

        if factors.is_empty() {
            None
        } else {
            Some(factors)
        }
    }
}

impl Default for AnalyticsHubConsumer {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_baseline() -> UsageBaseline {
        UsageBaseline {
            baseline_id: "baseline-123".to_string(),
            start_time: Utc::now() - chrono::Duration::days(30),
            end_time: Utc::now(),
            granularity: "daily".to_string(),
            organization_id: Some("org-123".to_string()),
            provider: Some("openai".to_string()),
            model: Some("gpt-4".to_string()),
            mean_tokens_per_period: 10000.0,
            std_dev_tokens: 2000.0,
            mean_cost_per_period: Decimal::from(100),
            std_dev_cost: 20.0,
            p50_tokens: 9500,
            p90_tokens: 13000,
            p99_tokens: 16000,
            p50_cost: Decimal::from(95),
            p90_cost: Decimal::from(130),
            p99_cost: Decimal::from(160),
            trend_direction: "increasing".to_string(),
            seasonality_detected: true,
            seasonality_period: Some("weekly".to_string()),
            computed_at: Utc::now(),
        }
    }

    fn create_test_curve() -> HistoricalCurve {
        let now = Utc::now();
        HistoricalCurve {
            curve_id: "curve-123".to_string(),
            metric_type: "cost".to_string(),
            data_points: vec![
                CurveDataPoint {
                    timestamp: now - chrono::Duration::days(2),
                    value: Decimal::from(100),
                    count: Some(10),
                },
                CurveDataPoint {
                    timestamp: now - chrono::Duration::days(1),
                    value: Decimal::from(110),
                    count: Some(12),
                },
                CurveDataPoint {
                    timestamp: now,
                    value: Decimal::from(120),
                    count: Some(15),
                },
            ],
            upper_bound: None,
            lower_bound: None,
            confidence_level: Some(0.95),
            organization_id: Some("org-123".to_string()),
            provider: Some("openai".to_string()),
            model: None,
            window_seconds: 86400,
        }
    }

    #[test]
    fn test_baseline_to_forecast_config() {
        let consumer = AnalyticsHubConsumer::with_defaults();
        let baseline = create_test_baseline();

        let result = consumer.baseline_to_forecast_config(&baseline);
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.detect_seasonality);
        assert!(config.include_trend);
        assert!(config.min_data_points >= 14); // Increased for seasonality
    }

    #[test]
    fn test_curve_to_time_series() {
        let consumer = AnalyticsHubConsumer::with_defaults();
        let curve = create_test_curve();

        let result = consumer.curve_to_time_series(&curve);
        assert!(result.is_ok());

        let time_series = result.unwrap();
        assert_eq!(time_series.len(), 3);
    }

    #[test]
    fn test_baseline_trend() {
        let consumer = AnalyticsHubConsumer::with_defaults();
        let baseline = create_test_baseline();

        let trend = consumer.baseline_trend(&baseline);
        assert_eq!(trend, TrendDirection::Increasing);
    }

    #[test]
    fn test_baseline_seasonality() {
        let consumer = AnalyticsHubConsumer::with_defaults();
        let baseline = create_test_baseline();

        let seasonality = consumer.baseline_seasonality(&baseline);
        assert!(seasonality.detected);
        assert_eq!(seasonality.period, Some(4)); // weekly = 4 periods
    }

    #[test]
    fn test_match_cluster() {
        let consumer = AnalyticsHubConsumer::with_defaults();

        let clusters = vec![
            ForecastingCluster {
                cluster_id: "c1".to_string(),
                cluster_name: "Low Usage".to_string(),
                member_count: 100,
                centroid: vec![Decimal::from(10)],
                mean_daily_tokens: 1000.0,
                mean_daily_cost: Decimal::from(10),
                growth_rate_percent: 2.0,
                dominant_trend: "stable".to_string(),
                dominant_seasonality: None,
                hourly_distribution: None,
                daily_distribution: None,
                silhouette_score: 0.8,
                cohesion: 0.9,
                computed_at: Utc::now(),
            },
            ForecastingCluster {
                cluster_id: "c2".to_string(),
                cluster_name: "High Usage".to_string(),
                member_count: 50,
                centroid: vec![Decimal::from(100)],
                mean_daily_tokens: 10000.0,
                mean_daily_cost: Decimal::from(100),
                growth_rate_percent: 10.0,
                dominant_trend: "increasing".to_string(),
                dominant_seasonality: Some("weekly".to_string()),
                hourly_distribution: None,
                daily_distribution: None,
                silhouette_score: 0.75,
                cohesion: 0.85,
                computed_at: Utc::now(),
            },
        ];

        // Create a pattern closer to the high usage cluster
        let now = Utc::now();
        let points = vec![
            DataPoint::new(now - chrono::Duration::days(2), Decimal::from(95)),
            DataPoint::new(now - chrono::Duration::days(1), Decimal::from(100)),
            DataPoint::new(now, Decimal::from(105)),
        ];
        let pattern = TimeSeriesData::with_auto_interval(points);

        let matched = consumer.match_cluster(&pattern, &clusters);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().cluster_id, "c2");
    }
}
