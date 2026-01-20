//! Cost Forecasting Agent Implementation
//!
//! The core agent logic for cost forecasting.

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::agents::{
    Agent, AgentClassification, AgentError,
    contracts::{
        AgentId, AgentVersion, ConstraintApplied, ConstraintType,
        DecisionEvent, DecisionType, InputsHash,
    },
    ruvector_client::RuVectorClient,
    telemetry::{AgentTelemetry, TelemetryEmitter, TelemetryMetrics},
};
use crate::forecasting::{
    ForecastConfig, ForecastEngine, ForecastHorizon, ForecastRequest,
    DataPoint, TimeSeriesData, TrendDirection,
};

use super::{
    AGENT_ID, AGENT_VERSION,
    types::{
        CostForecastInput, CostForecastOutput, ConstraintResult, ConstraintsEvaluation,
        ForecastProjection, GrowthPattern, HistoricalSummary, RiskIndicator, RiskLevel,
    },
};

/// Cost Forecasting Agent
///
/// # Classification: FORECASTING
///
/// Forecasts future LLM spend based on historical usage patterns.
///
/// # Execution Flow
/// 1. Validate input against contracts
/// 2. Convert historical data to time series
/// 3. Execute forecasting engine
/// 4. Evaluate constraints
/// 5. Generate risk indicators
/// 6. Emit DecisionEvent to ruvector-service
/// 7. Emit telemetry to LLM-Observatory
/// 8. Return deterministic output
pub struct CostForecastingAgent {
    agent_id: AgentId,
    agent_version: AgentVersion,
    forecast_engine: ForecastEngine,
    ruvector_client: Arc<RuVectorClient>,
    telemetry_emitter: TelemetryEmitter,
}

impl CostForecastingAgent {
    /// Create a new Cost Forecasting Agent
    pub fn new(
        ruvector_client: Arc<RuVectorClient>,
        telemetry_emitter: TelemetryEmitter,
    ) -> Self {
        Self {
            agent_id: AgentId::new(AGENT_ID),
            agent_version: AgentVersion::new(AGENT_VERSION),
            forecast_engine: ForecastEngine::new_with_defaults(),
            ruvector_client,
            telemetry_emitter,
        }
    }

    /// Create with default clients (for testing/development)
    pub fn with_defaults() -> Result<Self, AgentError> {
        let ruvector_client = Arc::new(
            RuVectorClient::with_defaults()
                .map_err(|e| AgentError::ConfigError(e.to_string()))?
        );
        let telemetry_emitter = TelemetryEmitter::from_env();

        Ok(Self::new(ruvector_client, telemetry_emitter))
    }

    /// Execute the agent and persist the decision
    pub async fn run(&self, input: CostForecastInput) -> Result<CostForecastOutput, AgentError> {
        // Create telemetry collector
        let mut telemetry = AgentTelemetry::new(
            self.agent_id.clone(),
            self.agent_version.clone(),
            self.decision_type(),
        );

        if let Some(ref exec_ref) = input.metadata.execution_ref {
            telemetry = telemetry.with_execution_ref(exec_ref.clone());
        }

        telemetry.record_start();

        // Execute the agent
        let result = self.execute_internal(&input, &mut telemetry).await;

        // Handle result and emit telemetry
        match &result {
            Ok((output, confidence)) => {
                // Create and persist DecisionEvent
                let constraints = self.collect_constraints(&input, output);
                let decision_event = self.create_decision_event(
                    &input,
                    output,
                    *confidence,
                    constraints,
                    input.metadata.execution_ref.clone(),
                );

                // Persist to ruvector-service (REQUIRED per Constitution)
                if let Err(e) = self.ruvector_client.persist_with_retries(&decision_event).await {
                    tracing::error!(error = %e, "Failed to persist DecisionEvent");
                    // Don't fail the agent, but log the error
                }

                // Record completion telemetry
                let metrics = TelemetryMetrics {
                    input_size_bytes: Some(serde_json::to_vec(&input).map(|v| v.len() as u64).unwrap_or(0)),
                    output_size_bytes: Some(serde_json::to_vec(&output).map(|v| v.len() as u64).unwrap_or(0)),
                    data_points_processed: Some(input.historical_data.len() as u64),
                    confidence: Some(*confidence),
                    model_name: Some(output.model_used.clone()),
                    constraints_evaluated: Some(self.count_constraints(&input.constraints)),
                    memory_bytes: None,
                };
                telemetry.record_completion(metrics);
            }
            Err(e) => {
                telemetry.record_failure(e.to_string());
            }
        }

        // Emit telemetry (best-effort, don't fail if emission fails)
        let events = telemetry.into_events();
        if let Err(e) = self.telemetry_emitter.emit_batch(&events).await {
            tracing::warn!(error = %e, "Failed to emit telemetry");
        }

        result.map(|(output, _)| output)
    }

    /// Internal execution logic
    async fn execute_internal(
        &self,
        input: &CostForecastInput,
        _telemetry: &mut AgentTelemetry,
    ) -> Result<(CostForecastOutput, f64), AgentError> {
        // Convert historical data to time series
        let time_series = self.convert_to_time_series(&input.historical_data)?;

        // Configure forecast
        let config = ForecastConfig {
            horizon: ForecastHorizon::Days(input.forecast_horizon_days),
            confidence_level: input.confidence_level,
            include_trend: true,
            detect_seasonality: true,
            min_data_points: super::MIN_DATA_POINTS,
        };

        // Create forecast request
        let request = ForecastRequest {
            data: time_series.clone(),
            config,
            preferred_model: input.preferred_model.as_ref().and_then(|m| {
                match m.to_lowercase().as_str() {
                    "linear" | "linear_trend" => Some(crate::forecasting::engine::ModelType::LinearTrend),
                    "moving_average" => Some(crate::forecasting::engine::ModelType::MovingAverage),
                    "exponential" | "exponential_smoothing" => Some(crate::forecasting::engine::ModelType::ExponentialSmoothing),
                    _ => None,
                }
            }),
        };

        // Execute forecast
        let forecast_result = self.forecast_engine.forecast(request)
            .map_err(|e| AgentError::ModelError(e.to_string()))?;

        // Convert to output format
        let projections = self.convert_projections(&forecast_result.forecast, &forecast_result.lower_bound, &forecast_result.upper_bound)?;

        // Calculate summary statistics
        let total_forecasted_cost: Decimal = projections.iter().map(|p| p.projected_cost).sum();
        let average_daily_cost = if !projections.is_empty() {
            total_forecasted_cost / Decimal::from(projections.len())
        } else {
            Decimal::ZERO
        };
        let peak_daily_cost = projections.iter().map(|p| p.projected_cost).max().unwrap_or(Decimal::ZERO);

        // Detect growth pattern
        let growth_pattern = self.detect_growth_pattern(&forecast_result.trend, &time_series);

        // Calculate average growth rate
        let average_growth_rate = self.calculate_average_growth_rate(&time_series);

        // Generate risk indicators
        let risk_indicators = self.generate_risk_indicators(
            input,
            &projections,
            total_forecasted_cost,
            average_growth_rate,
            &growth_pattern,
        );

        // Evaluate constraints
        let constraints_evaluation = self.evaluate_constraints(
            input,
            &projections,
            total_forecasted_cost,
            average_growth_rate,
        );

        // Calculate historical summary
        let historical_summary = self.calculate_historical_summary(&input.historical_data);

        // Calculate confidence based on data quality and model metrics
        let confidence = self.calculate_confidence(&forecast_result, &time_series);

        let output = CostForecastOutput {
            projections,
            total_forecasted_cost,
            average_daily_cost,
            peak_daily_cost,
            risk_indicators,
            growth_pattern,
            average_growth_rate,
            model_used: forecast_result.model_name,
            confidence,
            confidence_level: input.confidence_level,
            constraints_evaluation,
            generated_at: Utc::now(),
            historical_summary,
        };

        Ok((output, confidence))
    }

    /// Convert historical data to time series format
    fn convert_to_time_series(
        &self,
        data: &[super::types::HistoricalDataPoint],
    ) -> Result<TimeSeriesData, AgentError> {
        if data.is_empty() {
            return Err(AgentError::InsufficientData("No historical data provided".to_string()));
        }

        let points: Vec<DataPoint> = data
            .iter()
            .map(|d| DataPoint::new(d.timestamp, d.total_cost))
            .collect();

        Ok(TimeSeriesData::with_auto_interval(points))
    }

    /// Convert forecast result to output projections
    fn convert_projections(
        &self,
        forecast: &[DataPoint],
        lower_bound: &[DataPoint],
        upper_bound: &[DataPoint],
    ) -> Result<Vec<ForecastProjection>, AgentError> {
        let mut cumulative = Decimal::ZERO;
        let mut prev_cost: Option<Decimal> = None;

        let projections: Vec<ForecastProjection> = forecast
            .iter()
            .zip(lower_bound.iter())
            .zip(upper_bound.iter())
            .map(|((f, l), u)| {
                cumulative += f.value;
                let growth_rate = prev_cost.map(|p| {
                    if p > Decimal::ZERO {
                        ((f.value - p) / p * Decimal::from(100))
                            .to_string()
                            .parse::<f64>()
                            .unwrap_or(0.0)
                    } else {
                        0.0
                    }
                });
                prev_cost = Some(f.value);

                ForecastProjection {
                    timestamp: f.timestamp,
                    projected_cost: f.value,
                    lower_bound: l.value,
                    upper_bound: u.value,
                    cumulative_cost: cumulative,
                    growth_rate,
                }
            })
            .collect();

        Ok(projections)
    }

    /// Detect growth pattern from trend direction
    fn detect_growth_pattern(&self, trend: &TrendDirection, time_series: &TimeSeriesData) -> GrowthPattern {
        let std_dev = time_series.std_dev().unwrap_or(0.0);
        let mean = time_series.mean().map(|m| m.to_string().parse::<f64>().unwrap_or(0.0)).unwrap_or(0.0);

        // Check for high volatility
        if mean > 0.0 && (std_dev / mean) > 0.3 {
            return GrowthPattern::Volatile;
        }

        match trend {
            TrendDirection::Increasing => GrowthPattern::Linear, // Simplified; could detect exponential
            TrendDirection::Decreasing => GrowthPattern::Declining,
            TrendDirection::Stable => GrowthPattern::Stable,
            TrendDirection::Unknown => GrowthPattern::Volatile,
        }
    }

    /// Calculate average growth rate from time series
    fn calculate_average_growth_rate(&self, time_series: &TimeSeriesData) -> f64 {
        let values = time_series.values_f64();
        if values.len() < 2 {
            return 0.0;
        }

        let growth_rates: Vec<f64> = values
            .windows(2)
            .filter_map(|w| {
                if w[0] > 0.0 {
                    Some((w[1] - w[0]) / w[0] * 100.0)
                } else {
                    None
                }
            })
            .collect();

        if growth_rates.is_empty() {
            return 0.0;
        }

        growth_rates.iter().sum::<f64>() / growth_rates.len() as f64
    }

    /// Generate risk indicators based on forecast
    fn generate_risk_indicators(
        &self,
        input: &CostForecastInput,
        projections: &[ForecastProjection],
        total_cost: Decimal,
        growth_rate: f64,
        growth_pattern: &GrowthPattern,
    ) -> Vec<RiskIndicator> {
        let mut indicators = Vec::new();

        // Check budget cap risk
        if let Some(budget_cap) = input.constraints.budget_cap {
            if total_cost > budget_cap {
                let overage = ((total_cost - budget_cap) / budget_cap * Decimal::from(100))
                    .to_string()
                    .parse::<f64>()
                    .unwrap_or(0.0);

                let level = if overage > 50.0 {
                    RiskLevel::Critical
                } else if overage > 25.0 {
                    RiskLevel::High
                } else if overage > 10.0 {
                    RiskLevel::Medium
                } else {
                    RiskLevel::Low
                };

                indicators.push(RiskIndicator::budget_exceedance(
                    level,
                    format!("Projected to exceed budget by {:.1}%", overage),
                    0.95, // High confidence based on projection
                    Some(total_cost - budget_cap),
                ));
            }
        }

        // Check growth rate risk
        if let Some(max_growth) = input.constraints.max_growth_rate {
            if growth_rate > max_growth {
                let level = if growth_rate > max_growth * 2.0 {
                    RiskLevel::High
                } else {
                    RiskLevel::Medium
                };

                indicators.push(RiskIndicator::high_growth_rate(
                    level,
                    format!("Growth rate {:.1}% exceeds threshold {:.1}%", growth_rate, max_growth),
                    0.85,
                ));
            }
        } else if growth_rate > 20.0 {
            // Default high growth warning
            indicators.push(RiskIndicator::high_growth_rate(
                RiskLevel::Medium,
                format!("High growth rate detected: {:.1}%", growth_rate),
                0.85,
            ));
        }

        // Check volatility risk
        if matches!(growth_pattern, GrowthPattern::Volatile) {
            indicators.push(RiskIndicator::high_volatility(
                RiskLevel::Medium,
                "Cost patterns show high variability, reducing forecast confidence",
            ));
        }

        // Check for cost spikes in projections
        if let Some(peak) = projections.iter().map(|p| p.projected_cost).max() {
            let avg = if !projections.is_empty() {
                projections.iter().map(|p| p.projected_cost).sum::<Decimal>() / Decimal::from(projections.len())
            } else {
                Decimal::ZERO
            };

            if avg > Decimal::ZERO && peak > avg * Decimal::from(2) {
                indicators.push(RiskIndicator {
                    risk_type: "cost_spike".to_string(),
                    level: RiskLevel::Medium,
                    description: "Significant cost spike predicted in forecast period".to_string(),
                    probability: 0.7,
                    potential_impact: Some(peak - avg),
                    recommendation: Some("Investigate potential causes of spike".to_string()),
                });
            }
        }

        indicators
    }

    /// Evaluate constraints against forecast
    fn evaluate_constraints(
        &self,
        input: &CostForecastInput,
        projections: &[ForecastProjection],
        total_cost: Decimal,
        growth_rate: f64,
    ) -> ConstraintsEvaluation {
        let mut evaluation = ConstraintsEvaluation::default();

        // Budget cap evaluation
        if let Some(budget_cap) = input.constraints.budget_cap {
            let satisfied = total_cost <= budget_cap;
            let margin = budget_cap - total_cost;

            // Find breach date if not satisfied
            let breach_date = if !satisfied {
                let mut cumulative = Decimal::ZERO;
                projections.iter().find_map(|p| {
                    cumulative += p.projected_cost;
                    if cumulative > budget_cap {
                        Some(p.timestamp)
                    } else {
                        None
                    }
                })
            } else {
                None
            };

            evaluation.budget_cap = Some(ConstraintResult {
                satisfied,
                constraint_value: budget_cap.to_string(),
                actual_value: total_cost.to_string(),
                margin: Some(margin.to_string()),
                breach_date,
            });
        }

        // Max cost per period evaluation
        if let Some(max_cost) = input.constraints.max_cost_per_period {
            let max_projected = projections.iter().map(|p| p.projected_cost).max().unwrap_or(Decimal::ZERO);
            let satisfied = max_projected <= max_cost;

            let breach_date = if !satisfied {
                projections.iter().find(|p| p.projected_cost > max_cost).map(|p| p.timestamp)
            } else {
                None
            };

            evaluation.max_cost_per_period = Some(ConstraintResult {
                satisfied,
                constraint_value: max_cost.to_string(),
                actual_value: max_projected.to_string(),
                margin: Some((max_cost - max_projected).to_string()),
                breach_date,
            });
        }

        // Growth rate evaluation
        if let Some(max_growth) = input.constraints.max_growth_rate {
            let satisfied = growth_rate <= max_growth;

            evaluation.growth_rate = Some(ConstraintResult {
                satisfied,
                constraint_value: format!("{:.1}%", max_growth),
                actual_value: format!("{:.1}%", growth_rate),
                margin: Some(format!("{:.1}%", max_growth - growth_rate)),
                breach_date: None,
            });
        }

        evaluation
    }

    /// Calculate historical summary
    fn calculate_historical_summary(
        &self,
        data: &[super::types::HistoricalDataPoint],
    ) -> HistoricalSummary {
        let total_cost: Decimal = data.iter().map(|d| d.total_cost).sum();
        let average_cost = if !data.is_empty() {
            total_cost / Decimal::from(data.len())
        } else {
            Decimal::ZERO
        };

        let values: Vec<f64> = data.iter()
            .map(|d| d.total_cost.to_string().parse::<f64>().unwrap_or(0.0))
            .collect();

        let std_deviation = if values.len() >= 2 {
            let mean = values.iter().sum::<f64>() / values.len() as f64;
            let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
            Some(variance.sqrt())
        } else {
            None
        };

        let (period_start, period_end) = if !data.is_empty() {
            let mut timestamps: Vec<_> = data.iter().map(|d| d.timestamp).collect();
            timestamps.sort();
            (*timestamps.first().unwrap(), *timestamps.last().unwrap())
        } else {
            (Utc::now(), Utc::now())
        };

        HistoricalSummary {
            data_points: data.len(),
            period_start,
            period_end,
            total_cost,
            average_cost,
            std_deviation,
        }
    }

    /// Calculate overall confidence score
    fn calculate_confidence(
        &self,
        forecast_result: &crate::forecasting::TypesForecastResult,
        time_series: &TimeSeriesData,
    ) -> f64 {
        let mut confidence = 0.8; // Base confidence

        // Adjust based on data quality
        let data_points = time_series.len();
        if data_points >= 30 {
            confidence += 0.1;
        } else if data_points >= 14 {
            confidence += 0.05;
        }

        // Adjust based on volatility
        if let Some(std_dev) = time_series.std_dev() {
            let mean = time_series.values_f64().iter().sum::<f64>() / time_series.len() as f64;
            if mean > 0.0 {
                let cv = std_dev / mean; // Coefficient of variation
                if cv < 0.1 {
                    confidence += 0.05;
                } else if cv > 0.3 {
                    confidence -= 0.1;
                }
            }
        }

        // Adjust based on trend clarity
        if !matches!(forecast_result.trend, TrendDirection::Unknown) {
            confidence += 0.05;
        }

        // Adjust based on forecast metrics if available
        if let Some(ref metrics) = forecast_result.metrics {
            // Lower MAPE = higher confidence
            let mape = metrics.mape().unwrap_or(100.0);
            if mape < 5.0 {
                confidence += 0.05;
            } else if mape > 20.0 {
                confidence -= 0.1;
            }
        }

        confidence.clamp(0.0, 1.0)
    }

    /// Collect constraints applied during execution
    fn collect_constraints(
        &self,
        input: &CostForecastInput,
        output: &CostForecastOutput,
    ) -> Vec<ConstraintApplied> {
        let mut constraints = Vec::new();

        if let Some(ref budget_eval) = output.constraints_evaluation.budget_cap {
            constraints.push(ConstraintApplied {
                constraint_type: ConstraintType::BudgetCap,
                name: "budget_cap".to_string(),
                value: serde_json::json!(budget_eval.constraint_value),
                satisfied: budget_eval.satisfied,
                impact: budget_eval.margin.clone(),
            });
        }

        if let Some(ref cost_eval) = output.constraints_evaluation.max_cost_per_period {
            constraints.push(ConstraintApplied {
                constraint_type: ConstraintType::CostCap,
                name: "max_cost_per_period".to_string(),
                value: serde_json::json!(cost_eval.constraint_value),
                satisfied: cost_eval.satisfied,
                impact: cost_eval.margin.clone(),
            });
        }

        if let Some(ref growth_eval) = output.constraints_evaluation.growth_rate {
            constraints.push(ConstraintApplied {
                constraint_type: ConstraintType::GrowthLimit,
                name: "max_growth_rate".to_string(),
                value: serde_json::json!(growth_eval.constraint_value),
                satisfied: growth_eval.satisfied,
                impact: growth_eval.margin.clone(),
            });
        }

        if let Some(min_conf) = input.constraints.min_confidence {
            constraints.push(ConstraintApplied {
                constraint_type: ConstraintType::MinConfidence,
                name: "min_confidence".to_string(),
                value: serde_json::json!(min_conf),
                satisfied: output.confidence >= min_conf,
                impact: Some(format!("{:.2}", output.confidence - min_conf)),
            });
        }

        constraints
    }

    /// Count constraints in input
    fn count_constraints(&self, constraints: &super::types::ForecastConstraints) -> u32 {
        let mut count = 0;
        if constraints.budget_cap.is_some() { count += 1; }
        if constraints.roi_threshold.is_some() { count += 1; }
        if constraints.max_cost_per_period.is_some() { count += 1; }
        if constraints.max_growth_rate.is_some() { count += 1; }
        if constraints.min_confidence.is_some() { count += 1; }
        count
    }
}

#[async_trait::async_trait]
impl Agent for CostForecastingAgent {
    type Input = CostForecastInput;
    type Output = CostForecastOutput;

    fn agent_id(&self) -> &AgentId {
        &self.agent_id
    }

    fn agent_version(&self) -> &AgentVersion {
        &self.agent_version
    }

    fn classification(&self) -> AgentClassification {
        AgentClassification::Forecasting
    }

    fn decision_type(&self) -> DecisionType {
        DecisionType::CostForecast
    }

    fn validate_input(&self, input: &Self::Input) -> Result<(), crate::agents::contracts::ValidationError> {
        use crate::agents::contracts::AgentInput;
        input.validate()
    }

    async fn execute(&self, input: Self::Input) -> Result<(Self::Output, f64), AgentError> {
        // Validate input first
        self.validate_input(&input)?;

        // Create a placeholder telemetry (the real one is in run())
        let mut telemetry = AgentTelemetry::new(
            self.agent_id.clone(),
            self.agent_version.clone(),
            self.decision_type(),
        );

        self.execute_internal(&input, &mut telemetry).await
    }

    fn create_decision_event(
        &self,
        input: &Self::Input,
        output: &Self::Output,
        confidence: f64,
        constraints: Vec<ConstraintApplied>,
        execution_ref: Option<String>,
    ) -> DecisionEvent {
        let inputs_hash = InputsHash::compute(input);
        let outputs = serde_json::to_value(output).unwrap_or(serde_json::Value::Null);

        let mut event = DecisionEvent::new(
            self.agent_id.clone(),
            self.agent_version.clone(),
            self.decision_type(),
            inputs_hash,
            outputs,
            confidence,
        )
        .with_constraints(constraints);

        if let Some(exec_ref) = execution_ref {
            event = event.with_execution_ref(exec_ref);
        }

        if let Some(ref org_id) = input.metadata.organization_id {
            event = event.with_organization(org_id.clone());
        }

        if let Some(ref project_id) = input.metadata.project_id {
            event = event.with_project(project_id.clone());
        }

        event
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_input() -> CostForecastInput {
        let now = Utc::now();
        let historical_data: Vec<super::super::types::HistoricalDataPoint> = (0..14)
            .map(|i| super::super::types::HistoricalDataPoint {
                timestamp: now - Duration::days(14 - i),
                total_cost: Decimal::from(100 + i * 5), // Linear growth
                by_provider: Default::default(),
                by_model: Default::default(),
                total_tokens: Some(1_000_000),
                request_count: Some(1000),
            })
            .collect();

        CostForecastInput {
            historical_data,
            forecast_horizon_days: 30,
            granularity: super::super::types::ForecastGranularity::Daily,
            confidence_level: 0.95,
            constraints: super::super::types::ForecastConstraints {
                budget_cap: Some(dec!(5000)),
                max_growth_rate: Some(10.0),
                ..Default::default()
            },
            metadata: super::super::types::ForecastMetadata {
                organization_id: Some("org-123".to_string()),
                project_id: Some("project-456".to_string()),
                execution_ref: Some("exec-789".to_string()),
                ..Default::default()
            },
            preferred_model: None,
        }
    }

    #[tokio::test]
    async fn test_agent_creation() {
        let agent = CostForecastingAgent::with_defaults();
        assert!(agent.is_ok());
    }

    #[tokio::test]
    async fn test_agent_execution() {
        let agent = CostForecastingAgent::with_defaults().unwrap();
        let input = create_test_input();

        let result = agent.run(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(!output.projections.is_empty());
        assert!(output.confidence > 0.0 && output.confidence <= 1.0);
        assert!(output.total_forecasted_cost > Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_input_validation() {
        let agent = CostForecastingAgent::with_defaults().unwrap();

        // Invalid input (no data)
        let invalid_input = CostForecastInput {
            historical_data: vec![],
            forecast_horizon_days: 30,
            granularity: super::super::types::ForecastGranularity::Daily,
            confidence_level: 0.95,
            constraints: Default::default(),
            metadata: Default::default(),
            preferred_model: None,
        };

        let result = agent.validate_input(&invalid_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_agent_classification() {
        let agent = CostForecastingAgent::with_defaults().unwrap();
        assert_eq!(agent.classification(), AgentClassification::Forecasting);
        assert_eq!(agent.decision_type(), DecisionType::CostForecast);
    }

    #[test]
    fn test_agent_id() {
        let agent = CostForecastingAgent::with_defaults().unwrap();
        assert_eq!(agent.agent_id().as_str(), AGENT_ID);
        assert_eq!(agent.agent_version().as_str(), AGENT_VERSION);
    }

    #[tokio::test]
    async fn test_risk_indicator_generation() {
        let agent = CostForecastingAgent::with_defaults().unwrap();
        let mut input = create_test_input();

        // Set a low budget cap to trigger risk indicator
        input.constraints.budget_cap = Some(dec!(1000));

        let result = agent.run(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();

        // Should have budget exceedance risk
        assert!(output.risk_indicators.iter().any(|r| r.risk_type == "budget_exceedance"));
    }

    #[tokio::test]
    async fn test_constraint_evaluation() {
        let agent = CostForecastingAgent::with_defaults().unwrap();
        let input = create_test_input();

        let result = agent.run(input).await;
        assert!(result.is_ok());

        let output = result.unwrap();

        // Should have budget cap evaluation
        assert!(output.constraints_evaluation.budget_cap.is_some());
        assert!(output.constraints_evaluation.growth_rate.is_some());
    }
}
