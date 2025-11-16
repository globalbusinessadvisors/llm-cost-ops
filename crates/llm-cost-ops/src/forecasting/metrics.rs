// Forecast accuracy metrics

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::{ForecastError, ForecastResult};

/// Forecast accuracy metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastMetrics {
    /// Mean Absolute Error
    pub mae: f64,

    /// Root Mean Square Error
    pub rmse: f64,

    /// Mean Absolute Percentage Error
    pub mape: f64,

    /// R-squared (coefficient of determination)
    pub r_squared: f64,

    /// Number of data points used
    pub n: usize,
}

/// Calculate Mean Absolute Error
pub fn calculate_mae(actual: &[Decimal], predicted: &[Decimal]) -> ForecastResult<f64> {
    if actual.len() != predicted.len() {
        return Err(ForecastError::CalculationError(
            "Actual and predicted arrays must have the same length".to_string(),
        ));
    }

    if actual.is_empty() {
        return Err(ForecastError::InsufficientData(
            "Cannot calculate MAE with empty arrays".to_string(),
        ));
    }

    let sum: f64 = actual
        .iter()
        .zip(predicted.iter())
        .map(|(a, p)| {
            let a_f64 = a.to_string().parse::<f64>().unwrap_or(0.0);
            let p_f64 = p.to_string().parse::<f64>().unwrap_or(0.0);
            (a_f64 - p_f64).abs()
        })
        .sum();

    Ok(sum / actual.len() as f64)
}

/// Calculate Root Mean Square Error
pub fn calculate_rmse(actual: &[Decimal], predicted: &[Decimal]) -> ForecastResult<f64> {
    if actual.len() != predicted.len() {
        return Err(ForecastError::CalculationError(
            "Actual and predicted arrays must have the same length".to_string(),
        ));
    }

    if actual.is_empty() {
        return Err(ForecastError::InsufficientData(
            "Cannot calculate RMSE with empty arrays".to_string(),
        ));
    }

    let sum_squared: f64 = actual
        .iter()
        .zip(predicted.iter())
        .map(|(a, p)| {
            let a_f64 = a.to_string().parse::<f64>().unwrap_or(0.0);
            let p_f64 = p.to_string().parse::<f64>().unwrap_or(0.0);
            (a_f64 - p_f64).powi(2)
        })
        .sum();

    Ok((sum_squared / actual.len() as f64).sqrt())
}

/// Calculate Mean Absolute Percentage Error
pub fn calculate_mape(actual: &[Decimal], predicted: &[Decimal]) -> ForecastResult<f64> {
    if actual.len() != predicted.len() {
        return Err(ForecastError::CalculationError(
            "Actual and predicted arrays must have the same length".to_string(),
        ));
    }

    if actual.is_empty() {
        return Err(ForecastError::InsufficientData(
            "Cannot calculate MAPE with empty arrays".to_string(),
        ));
    }

    let mut sum_percentage_error = 0.0;
    let mut valid_count = 0;

    for (a, p) in actual.iter().zip(predicted.iter()) {
        let a_f64 = a.to_string().parse::<f64>().unwrap_or(0.0);
        let p_f64 = p.to_string().parse::<f64>().unwrap_or(0.0);

        // Skip zero actual values to avoid division by zero
        if a_f64.abs() > f64::EPSILON {
            sum_percentage_error += ((a_f64 - p_f64).abs() / a_f64.abs()) * 100.0;
            valid_count += 1;
        }
    }

    if valid_count == 0 {
        return Err(ForecastError::CalculationError(
            "Cannot calculate MAPE: all actual values are zero".to_string(),
        ));
    }

    Ok(sum_percentage_error / valid_count as f64)
}

/// Calculate R-squared (coefficient of determination)
pub fn calculate_r_squared(actual: &[Decimal], predicted: &[Decimal]) -> ForecastResult<f64> {
    if actual.len() != predicted.len() {
        return Err(ForecastError::CalculationError(
            "Actual and predicted arrays must have the same length".to_string(),
        ));
    }

    if actual.is_empty() {
        return Err(ForecastError::InsufficientData(
            "Cannot calculate R² with empty arrays".to_string(),
        ));
    }

    // Convert to f64
    let actual_f64: Vec<f64> = actual
        .iter()
        .map(|v| v.to_string().parse::<f64>().unwrap_or(0.0))
        .collect();

    let predicted_f64: Vec<f64> = predicted
        .iter()
        .map(|v| v.to_string().parse::<f64>().unwrap_or(0.0))
        .collect();

    // Calculate mean of actual values
    let mean_actual = actual_f64.iter().sum::<f64>() / actual_f64.len() as f64;

    // Calculate total sum of squares
    let ss_tot: f64 = actual_f64.iter().map(|a| (a - mean_actual).powi(2)).sum();

    // Calculate residual sum of squares
    let ss_res: f64 = actual_f64
        .iter()
        .zip(predicted_f64.iter())
        .map(|(a, p)| (a - p).powi(2))
        .sum();

    if ss_tot.abs() < f64::EPSILON {
        return Ok(1.0); // Perfect fit if no variance in actual values
    }

    Ok(1.0 - (ss_res / ss_tot))
}

/// Calculate all forecast metrics
pub fn calculate_all_metrics(
    actual: &[Decimal],
    predicted: &[Decimal],
) -> ForecastResult<ForecastMetrics> {
    Ok(ForecastMetrics {
        mae: calculate_mae(actual, predicted)?,
        rmse: calculate_rmse(actual, predicted)?,
        mape: calculate_mape(actual, predicted)?,
        r_squared: calculate_r_squared(actual, predicted)?,
        n: actual.len(),
    })
}

impl ForecastMetrics {
    /// Create metrics from actual and predicted values
    pub fn new(actual: &[Decimal], predicted: &[Decimal]) -> ForecastResult<Self> {
        calculate_all_metrics(actual, predicted)
    }

    /// Check if the forecast is considered accurate (MAPE < 10%)
    pub fn is_accurate(&self) -> bool {
        self.mape < 10.0
    }

    /// Get accuracy grade
    pub fn accuracy_grade(&self) -> &str {
        match self.mape {
            m if m < 10.0 => "Excellent",
            m if m < 20.0 => "Good",
            m if m < 50.0 => "Acceptable",
            _ => "Poor",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_mae() {
        let actual = vec![
            Decimal::from(10),
            Decimal::from(20),
            Decimal::from(30),
            Decimal::from(40),
        ];
        let predicted = vec![
            Decimal::from(12),
            Decimal::from(18),
            Decimal::from(32),
            Decimal::from(38),
        ];

        let mae = calculate_mae(&actual, &predicted).unwrap();
        assert!((mae - 2.0).abs() < 0.01); // MAE = (2 + 2 + 2 + 2) / 4 = 2.0
    }

    #[test]
    fn test_calculate_rmse() {
        let actual = vec![
            Decimal::from(10),
            Decimal::from(20),
            Decimal::from(30),
        ];
        let predicted = vec![
            Decimal::from(12),
            Decimal::from(18),
            Decimal::from(32),
        ];

        let rmse = calculate_rmse(&actual, &predicted).unwrap();
        // RMSE = sqrt((4 + 4 + 4) / 3) = sqrt(4) = 2.0
        assert!((rmse - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_calculate_mape() {
        let actual = vec![
            Decimal::from(100),
            Decimal::from(200),
            Decimal::from(300),
        ];
        let predicted = vec![
            Decimal::from(110),
            Decimal::from(190),
            Decimal::from(330),
        ];

        let mape = calculate_mape(&actual, &predicted).unwrap();
        // MAPE = (10% + 5% + 10%) / 3 = 8.33%
        assert!((mape - 8.33).abs() < 0.1);
    }

    #[test]
    fn test_calculate_r_squared() {
        let actual = vec![
            Decimal::from(2),
            Decimal::from(4),
            Decimal::from(6),
            Decimal::from(8),
        ];
        let predicted = vec![
            Decimal::from(2),
            Decimal::from(4),
            Decimal::from(6),
            Decimal::from(8),
        ];

        let r2 = calculate_r_squared(&actual, &predicted).unwrap();
        assert!((r2 - 1.0).abs() < 0.01); // Perfect prediction
    }

    #[test]
    fn test_calculate_all_metrics() {
        let actual = vec![
            Decimal::from(10),
            Decimal::from(20),
            Decimal::from(30),
        ];
        let predicted = vec![
            Decimal::from(11),
            Decimal::from(19),
            Decimal::from(31),
        ];

        let metrics = calculate_all_metrics(&actual, &predicted).unwrap();

        assert!(metrics.mae > 0.0);
        assert!(metrics.rmse > 0.0);
        assert!(metrics.mape > 0.0);
        assert!(metrics.r_squared >= 0.0 && metrics.r_squared <= 1.0);
        assert_eq!(metrics.n, 3);
    }

    #[test]
    fn test_empty_arrays() {
        let actual: Vec<Decimal> = vec![];
        let predicted: Vec<Decimal> = vec![];

        assert!(calculate_mae(&actual, &predicted).is_err());
        assert!(calculate_rmse(&actual, &predicted).is_err());
        assert!(calculate_mape(&actual, &predicted).is_err());
    }

    #[test]
    fn test_mismatched_lengths() {
        let actual = vec![Decimal::from(10), Decimal::from(20)];
        let predicted = vec![Decimal::from(10)];

        assert!(calculate_mae(&actual, &predicted).is_err());
    }

    #[test]
    fn test_accuracy_grade() {
        let metrics = ForecastMetrics {
            mae: 5.0,
            rmse: 6.0,
            mape: 8.0,
            r_squared: 0.95,
            n: 10,
        };

        assert!(metrics.is_accurate());
        assert_eq!(metrics.accuracy_grade(), "Excellent");

        let poor_metrics = ForecastMetrics {
            mae: 50.0,
            rmse: 60.0,
            mape: 55.0,
            r_squared: 0.5,
            n: 10,
        };

        assert!(!poor_metrics.is_accurate());
        assert_eq!(poor_metrics.accuracy_grade(), "Poor");
    }
}
