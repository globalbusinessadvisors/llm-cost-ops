use rust_decimal::Decimal;
use tracing::{info, warn};

use crate::domain::{
    CostCalculation, CostOpsError, CostRecord, Currency, PricingStructure,
    PricingTable, Result, UsageRecord,
};

pub struct CostCalculator;

impl CostCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculate cost for a usage record using the provided pricing table
    pub fn calculate(
        &self,
        usage: &UsageRecord,
        pricing: &PricingTable,
    ) -> Result<CostRecord> {
        info!(
            "Calculating cost for usage_id={} provider={} model={}",
            usage.id,
            usage.provider,
            usage.model.name
        );

        // Validate pricing applies to this usage
        if usage.provider != pricing.provider {
            return Err(CostOpsError::InvalidPricingStructure(format!(
                "Provider mismatch: usage={}, pricing={}",
                usage.provider, pricing.provider
            )));
        }

        if !pricing.is_active_at(&usage.timestamp) {
            warn!(
                "Pricing not active at timestamp {} for model {}",
                usage.timestamp, pricing.model
            );
        }

        let calculation = self.calculate_cost_internal(usage, pricing)?;

        let record = CostRecord::new(
            usage.id,
            usage.provider.clone(),
            usage.model.name.clone(),
            usage.organization_id.clone(),
            calculation,
            pricing.pricing.clone(),
        )
        .with_tags(usage.tags.clone());

        if let Some(project_id) = &usage.project_id {
            Ok(record.with_project(project_id.clone()))
        } else {
            Ok(record)
        }
    }

    fn calculate_cost_internal(
        &self,
        usage: &UsageRecord,
        pricing: &PricingTable,
    ) -> Result<CostCalculation> {
        match &pricing.pricing {
            PricingStructure::PerToken {
                input_price_per_million,
                output_price_per_million,
                cached_input_discount,
            } => self.calculate_per_token(
                usage,
                *input_price_per_million,
                *output_price_per_million,
                *cached_input_discount,
                &pricing.currency,
                pricing.id,
            ),

            PricingStructure::PerRequest {
                price_per_request,
                included_tokens,
                overage_price_per_million,
            } => self.calculate_per_request(
                usage,
                *price_per_request,
                *included_tokens,
                *overage_price_per_million,
                &pricing.currency,
                pricing.id,
            ),

            PricingStructure::Tiered { tiers } => {
                self.calculate_tiered(usage, tiers, &pricing.currency, pricing.id)
            }
        }
    }

    fn calculate_per_token(
        &self,
        usage: &UsageRecord,
        input_price_per_million: Decimal,
        output_price_per_million: Decimal,
        cached_input_discount: Option<Decimal>,
        currency: &Currency,
        pricing_id: uuid::Uuid,
    ) -> Result<CostCalculation> {
        let million = Decimal::from(1_000_000);

        // Calculate base input cost
        let mut input_cost =
            Decimal::from(usage.prompt_tokens) * input_price_per_million / million;

        // Apply cached token discount if applicable
        if let Some(cached_tokens) = usage.cached_tokens {
            if let Some(discount) = cached_input_discount {
                let cached_cost = Decimal::from(cached_tokens) * input_price_per_million / million;
                let discount_amount = cached_cost * discount;
                input_cost -= discount_amount;

                info!(
                    "Applied cache discount: cached_tokens={} discount={} saved={}",
                    cached_tokens, discount, discount_amount
                );
            }
        }

        // Calculate output cost
        let output_cost =
            Decimal::from(usage.completion_tokens) * output_price_per_million / million;

        // Round to 10 decimal places for precision
        let input_cost = input_cost.round_dp(10);
        let output_cost = output_cost.round_dp(10);

        Ok(CostCalculation::new(
            input_cost,
            output_cost,
            currency.clone(),
            pricing_id,
        ))
    }

    fn calculate_per_request(
        &self,
        usage: &UsageRecord,
        price_per_request: Decimal,
        included_tokens: u64,
        overage_price_per_million: Decimal,
        currency: &Currency,
        pricing_id: uuid::Uuid,
    ) -> Result<CostCalculation> {
        let mut total_cost = price_per_request;

        // Check for overage
        if usage.total_tokens > included_tokens {
            let overage_tokens = usage.total_tokens - included_tokens;
            let million = Decimal::from(1_000_000);
            let overage_cost = Decimal::from(overage_tokens) * overage_price_per_million / million;
            total_cost += overage_cost;

            info!(
                "Overage detected: included={} total={} overage={} cost={}",
                included_tokens, usage.total_tokens, overage_tokens, overage_cost
            );
        }

        // Split cost arbitrarily between input/output
        let input_ratio = Decimal::from(usage.prompt_tokens) / Decimal::from(usage.total_tokens);
        let input_cost = (total_cost * input_ratio).round_dp(10);
        let output_cost = (total_cost - input_cost).round_dp(10);

        Ok(CostCalculation::new(
            input_cost,
            output_cost,
            currency.clone(),
            pricing_id,
        ))
    }

    fn calculate_tiered(
        &self,
        usage: &UsageRecord,
        tiers: &[crate::domain::pricing::PricingTier],
        currency: &Currency,
        pricing_id: uuid::Uuid,
    ) -> Result<CostCalculation> {
        // Find applicable tier
        let tier = tiers
            .iter()
            .find(|t| {
                usage.total_tokens >= t.min_tokens
                    && t.max_tokens.is_none_or(|max| usage.total_tokens <= max)
            })
            .ok_or_else(|| {
                CostOpsError::InvalidPricingStructure(format!(
                    "No tier found for {} tokens",
                    usage.total_tokens
                ))
            })?;

        let million = Decimal::from(1_000_000);
        let input_cost =
            Decimal::from(usage.prompt_tokens) * tier.input_price_per_million / million;
        let output_cost =
            Decimal::from(usage.completion_tokens) * tier.output_price_per_million / million;

        Ok(CostCalculation::new(
            input_cost.round_dp(10),
            output_cost.round_dp(10),
            currency.clone(),
            pricing_id,
        ))
    }
}

impl Default for CostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ModelIdentifier, IngestionSource, Provider};
    use chrono::Utc;
    use rust_decimal_macros::dec;

    fn create_test_usage(prompt_tokens: u64, completion_tokens: u64) -> UsageRecord {
        UsageRecord {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            provider: Provider::OpenAI,
            model: ModelIdentifier::new("gpt-4".to_string(), 8192),
            organization_id: "org-test".to_string(),
            project_id: None,
            user_id: None,
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
            cached_tokens: None,
            reasoning_tokens: None,
            latency_ms: None,
            time_to_first_token_ms: None,
            tags: vec![],
            metadata: serde_json::Value::Null,
            ingested_at: Utc::now(),
            source: IngestionSource::Api {
                endpoint: "test".to_string(),
            },
        }
    }

    fn create_test_pricing(pricing_structure: PricingStructure) -> PricingTable {
        PricingTable::new(Provider::OpenAI, "gpt-4".to_string(), pricing_structure)
    }

    #[test]
    fn test_per_token_calculation() {
        let calculator = CostCalculator::new();
        let usage = create_test_usage(1000, 500);
        let pricing = create_test_pricing(PricingStructure::simple_per_token(
            dec!(10.0),  // $10 per million input tokens
            dec!(30.0),  // $30 per million output tokens
        ));

        let result = calculator.calculate(&usage, &pricing).unwrap();

        // 1000 tokens * $10/million = $0.01
        assert_eq!(result.input_cost, dec!(0.01));
        // 500 tokens * $30/million = $0.015
        assert_eq!(result.output_cost, dec!(0.015));
        assert_eq!(result.total_cost, dec!(0.025));
    }

    #[test]
    fn test_per_token_with_cache() {
        let calculator = CostCalculator::new();
        let mut usage = create_test_usage(1000, 500);
        usage.cached_tokens = Some(500); // 50% cache hit

        let pricing = create_test_pricing(PricingStructure::per_token_with_cache(
            dec!(10.0),  // $10 per million input tokens
            dec!(30.0),  // $30 per million output tokens
            dec!(0.9),   // 90% discount on cached tokens
        ));

        let result = calculator.calculate(&usage, &pricing).unwrap();

        // Non-cached: 500 tokens * $10/million = $0.005
        // Cached: 500 tokens * $10/million * 0.1 (10% cost) = $0.0005
        // Total input: $0.0055
        assert_eq!(result.input_cost, dec!(0.0055));
    }

    #[test]
    fn test_provider_mismatch() {
        let calculator = CostCalculator::new();
        let mut usage = create_test_usage(1000, 500);
        usage.provider = Provider::Anthropic;

        let pricing = create_test_pricing(PricingStructure::simple_per_token(
            dec!(10.0),
            dec!(30.0),
        ));

        let result = calculator.calculate(&usage, &pricing);
        assert!(result.is_err());
    }

    #[test]
    fn test_per_request_calculation() {
        let calculator = CostCalculator::new();
        let usage = create_test_usage(1000, 500);

        let pricing = create_test_pricing(PricingStructure::PerRequest {
            price_per_request: dec!(0.01),
            included_tokens: 2000,
            overage_price_per_million: dec!(5.0),
        });

        let result = calculator.calculate(&usage, &pricing).unwrap();

        // Within included tokens, so just base price
        assert_eq!(result.total_cost, dec!(0.01));
    }

    #[test]
    fn test_per_request_with_overage() {
        let calculator = CostCalculator::new();
        let usage = create_test_usage(2000, 1000); // 3000 total tokens

        let pricing = create_test_pricing(PricingStructure::PerRequest {
            price_per_request: dec!(0.01),
            included_tokens: 2000,
            overage_price_per_million: dec!(5.0),
        });

        let result = calculator.calculate(&usage, &pricing).unwrap();

        // Base: $0.01
        // Overage: 1000 tokens * $5/million = $0.005
        // Total: $0.015
        assert_eq!(result.total_cost, dec!(0.015));
    }
}
