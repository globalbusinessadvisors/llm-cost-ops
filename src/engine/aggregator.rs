use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;

use crate::domain::{CostRecord, Result};

#[derive(Debug, Clone)]
pub struct CostSummary {
    pub total_cost: Decimal,
    pub total_requests: u64,
    pub avg_cost_per_request: Decimal,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub by_provider: HashMap<String, Decimal>,
    pub by_model: HashMap<String, Decimal>,
    pub by_project: HashMap<String, Decimal>,
}

pub struct CostAggregator;

impl CostAggregator {
    pub fn new() -> Self {
        Self
    }

    /// Aggregate cost records into a summary
    pub fn aggregate(
        &self,
        records: &[CostRecord],
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> Result<CostSummary> {
        let mut total_cost = Decimal::ZERO;
        let mut by_provider: HashMap<String, Decimal> = HashMap::new();
        let mut by_model: HashMap<String, Decimal> = HashMap::new();
        let mut by_project: HashMap<String, Decimal> = HashMap::new();

        for record in records {
            total_cost += record.total_cost;

            // Aggregate by provider
            *by_provider
                .entry(record.provider.to_string())
                .or_insert(Decimal::ZERO) += record.total_cost;

            // Aggregate by model
            *by_model
                .entry(record.model.clone())
                .or_insert(Decimal::ZERO) += record.total_cost;

            // Aggregate by project
            if let Some(project_id) = &record.project_id {
                *by_project
                    .entry(project_id.clone())
                    .or_insert(Decimal::ZERO) += record.total_cost;
            }
        }

        let total_requests = records.len() as u64;
        let avg_cost_per_request = if total_requests > 0 {
            total_cost / Decimal::from(total_requests)
        } else {
            Decimal::ZERO
        };

        Ok(CostSummary {
            total_cost,
            total_requests,
            avg_cost_per_request,
            period_start,
            period_end,
            by_provider,
            by_model,
            by_project,
        })
    }

    /// Group costs by time window (hourly, daily, etc.)
    pub fn group_by_time_window(
        &self,
        records: &[CostRecord],
        window_hours: i64,
    ) -> Result<HashMap<DateTime<Utc>, Decimal>> {
        let mut grouped: HashMap<DateTime<Utc>, Decimal> = HashMap::new();

        for record in records {
            let window_start = self.round_to_window(record.timestamp, window_hours);
            *grouped.entry(window_start).or_insert(Decimal::ZERO) += record.total_cost;
        }

        Ok(grouped)
    }

    fn round_to_window(&self, timestamp: DateTime<Utc>, window_hours: i64) -> DateTime<Utc> {
        let hours_since_epoch = timestamp.timestamp() / 3600;
        let window_number = hours_since_epoch / window_hours;
        let window_start_hours = window_number * window_hours;

        DateTime::from_timestamp(window_start_hours * 3600, 0)
            .unwrap_or(timestamp)
    }
}

impl Default for CostAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{CostCalculation, Currency, PricingStructure};
    use rust_decimal_macros::dec;

    fn create_test_record(provider: Provider, model: &str, cost: Decimal) -> CostRecord {
        let calc = CostCalculation::new(
            cost / dec!(2),
            cost / dec!(2),
            Currency::USD,
            uuid::Uuid::new_v4(),
        );

        CostRecord::new(
            uuid::Uuid::new_v4(),
            provider,
            model.to_string(),
            "org-test".to_string(),
            calc,
            PricingStructure::simple_per_token(dec!(10.0), dec!(30.0)),
        )
    }

    #[test]
    fn test_aggregate() {
        let aggregator = CostAggregator::new();
        let records = vec![
            create_test_record(Provider::OpenAI, "gpt-4", dec!(0.10)),
            create_test_record(Provider::OpenAI, "gpt-3.5", dec!(0.05)),
            create_test_record(Provider::Anthropic, "claude-3", dec!(0.15)),
        ];

        let period_start = Utc::now() - chrono::Duration::days(1);
        let period_end = Utc::now();

        let summary = aggregator.aggregate(&records, period_start, period_end).unwrap();

        assert_eq!(summary.total_cost, dec!(0.30));
        assert_eq!(summary.total_requests, 3);
        assert_eq!(summary.avg_cost_per_request, dec!(0.10));
        assert_eq!(summary.by_provider.len(), 2);
        assert_eq!(summary.by_model.len(), 3);
    }

    #[test]
    fn test_group_by_time_window() {
        let aggregator = CostAggregator::new();
        let records = vec![
            create_test_record(Provider::OpenAI, "gpt-4", dec!(0.10)),
            create_test_record(Provider::OpenAI, "gpt-4", dec!(0.20)),
        ];

        let grouped = aggregator.group_by_time_window(&records, 24).unwrap();
        assert!(!grouped.is_empty());
    }
}
