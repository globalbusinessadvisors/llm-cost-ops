use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::provider::Provider;
use super::pricing::{Currency, PricingStructure};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostRecord {
    pub id: Uuid,
    pub usage_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub provider: Provider,
    pub model: String,

    // Cost breakdown
    pub input_cost: Decimal,
    pub output_cost: Decimal,
    pub total_cost: Decimal,
    pub currency: Currency,

    // Cost model reference
    pub cost_model_id: Uuid,
    pub pricing_structure: PricingStructure,

    // Aggregation dimensions
    pub organization_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    pub tags: Vec<String>,

    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CostCalculation {
    pub input_cost: Decimal,
    pub output_cost: Decimal,
    pub total_cost: Decimal,
    pub currency: Currency,
    pub pricing_model_id: Uuid,
}

impl CostRecord {
    pub fn new(
        usage_id: Uuid,
        provider: Provider,
        model: String,
        organization_id: String,
        calculation: CostCalculation,
        pricing_structure: PricingStructure,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            usage_id,
            timestamp: Utc::now(),
            provider,
            model,
            input_cost: calculation.input_cost,
            output_cost: calculation.output_cost,
            total_cost: calculation.total_cost,
            currency: calculation.currency,
            cost_model_id: calculation.pricing_model_id,
            pricing_structure,
            organization_id,
            project_id: None,
            tags: Vec::new(),
            calculated_at: Utc::now(),
        }
    }

    pub fn with_project(mut self, project_id: String) -> Self {
        self.project_id = Some(project_id);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn cost_per_token(&self, total_tokens: u64) -> Decimal {
        if total_tokens == 0 {
            Decimal::ZERO
        } else {
            self.total_cost / Decimal::from(total_tokens)
        }
    }
}

impl CostCalculation {
    pub fn new(
        input_cost: Decimal,
        output_cost: Decimal,
        currency: Currency,
        pricing_model_id: Uuid,
    ) -> Self {
        Self {
            input_cost,
            output_cost,
            total_cost: input_cost + output_cost,
            currency,
            pricing_model_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Provider;
    use rust_decimal_macros::dec;

    #[test]
    fn test_cost_calculation() {
        let calc = CostCalculation::new(
            dec!(0.05),
            dec!(0.15),
            Currency::USD,
            Uuid::new_v4(),
        );

        assert_eq!(calc.total_cost, dec!(0.20));
    }

    #[test]
    fn test_cost_per_token() {
        let calc = CostCalculation::new(
            dec!(0.10),
            dec!(0.30),
            Currency::USD,
            Uuid::new_v4(),
        );

        let record = CostRecord::new(
            Uuid::new_v4(),
            Provider::OpenAI,
            "gpt-4".to_string(),
            "org-123".to_string(),
            calc,
            crate::domain::pricing::PricingStructure::simple_per_token(
                dec!(10.0),
                dec!(30.0),
            ),
        );

        let cost_per_token = record.cost_per_token(1000);
        assert_eq!(cost_per_token, dec!(0.0004));
    }
}
