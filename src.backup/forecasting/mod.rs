// Forecasting engine for LLM cost prediction and analysis

pub mod types;
pub mod models;
pub mod engine;
pub mod metrics;
pub mod anomaly;
pub mod budget;

pub use types::{
    DataPoint, ForecastConfig, ForecastHorizon, TimeSeriesData,
    TrendDirection, SeasonalityPattern,
    ForecastResult as TypesForecastResult,
};
pub use models::{
    ForecastModel, LinearTrendModel, ExponentialSmoothingModel, MovingAverageModel,
};
pub use engine::{ForecastEngine, ForecastRequest};
pub use metrics::{ForecastMetrics, calculate_mape, calculate_rmse, calculate_mae};
pub use anomaly::{AnomalyDetector, AnomalyResult, AnomalyMethod};
pub use budget::{BudgetForecaster, BudgetForecast, BudgetAlert, AlertSeverity};

/// Forecasting error types
#[derive(Debug, thiserror::Error)]
pub enum ForecastError {
    #[error("Insufficient data: {0}")]
    InsufficientData(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Model error: {0}")]
    ModelError(String),

    #[error("Calculation error: {0}")]
    CalculationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<ForecastError> for crate::domain::CostOpsError {
    fn from(err: ForecastError) -> Self {
        crate::domain::CostOpsError::internal(err.to_string())
    }
}

pub type ForecastResult<T> = Result<T, ForecastError>;
