use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::provider::Provider;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTable {
    pub id: Uuid,
    pub provider: Provider,
    pub model: String,

    /// When this pricing becomes effective
    pub effective_date: DateTime<Utc>,

    /// When this pricing expires (None = no expiration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<DateTime<Utc>>,

    /// Pricing structure
    pub pricing: PricingStructure,

    /// Currency
    pub currency: Currency,

    /// Geographic region this pricing applies to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Metadata about this pricing table
    #[serde(default)]
    pub metadata: serde_json::Value,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PricingStructure {
    /// Per-token pricing (most common)
    PerToken {
        /// Price per 1 million input tokens
        input_price_per_million: Decimal,

        /// Price per 1 million output tokens
        output_price_per_million: Decimal,

        /// Optional cached input discount (0.0 to 1.0)
        #[serde(skip_serializing_if = "Option::is_none")]
        cached_input_discount: Option<Decimal>,
    },

    /// Per-request pricing with included tokens
    PerRequest {
        price_per_request: Decimal,
        included_tokens: u64,
        overage_price_per_million: Decimal,
    },

    /// Tiered pricing based on volume
    Tiered {
        tiers: Vec<PricingTier>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    pub min_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>, // None = unlimited
    pub input_price_per_million: Decimal,
    pub output_price_per_million: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    Custom(String),
}

impl Currency {
    pub fn as_str(&self) -> &str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::GBP => "GBP",
            Currency::JPY => "JPY",
            Currency::Custom(s) => s,
        }
    }
}

impl PricingTable {
    pub fn new(
        provider: Provider,
        model: String,
        pricing: PricingStructure,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            provider,
            model,
            effective_date: now,
            end_date: None,
            pricing,
            currency: Currency::USD,
            region: None,
            metadata: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_active_at(&self, date: &DateTime<Utc>) -> bool {
        if date < &self.effective_date {
            return false;
        }

        if let Some(end_date) = &self.end_date {
            date <= end_date
        } else {
            true
        }
    }

    pub fn with_region(mut self, region: String) -> Self {
        self.region = Some(region);
        self
    }

    pub fn with_end_date(mut self, end_date: DateTime<Utc>) -> Self {
        self.end_date = Some(end_date);
        self
    }
}

impl PricingStructure {
    pub fn simple_per_token(input_price: Decimal, output_price: Decimal) -> Self {
        PricingStructure::PerToken {
            input_price_per_million: input_price,
            output_price_per_million: output_price,
            cached_input_discount: None,
        }
    }

    pub fn per_token_with_cache(
        input_price: Decimal,
        output_price: Decimal,
        cache_discount: Decimal,
    ) -> Self {
        PricingStructure::PerToken {
            input_price_per_million: input_price,
            output_price_per_million: output_price,
            cached_input_discount: Some(cache_discount),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_pricing_table_creation() {
        let pricing = PricingStructure::simple_per_token(dec!(10.0), dec!(30.0));
        let table = PricingTable::new(
            Provider::OpenAI,
            "gpt-4".to_string(),
            pricing,
        );

        assert_eq!(table.provider, Provider::OpenAI);
        assert_eq!(table.model, "gpt-4");
        assert_eq!(table.currency, Currency::USD);
    }

    #[test]
    fn test_pricing_active_date() {
        let pricing = PricingStructure::simple_per_token(dec!(10.0), dec!(30.0));
        let mut table = PricingTable::new(
            Provider::OpenAI,
            "gpt-4".to_string(),
            pricing,
        );

        let now = Utc::now();
        assert!(table.is_active_at(&now));

        // Set end date in the past
        table.end_date = Some(now - chrono::Duration::days(1));
        assert!(!table.is_active_at(&now));
    }

    #[test]
    fn test_pricing_serialization() {
        let pricing = PricingStructure::simple_per_token(dec!(10.0), dec!(30.0));
        let json = serde_json::to_string(&pricing).unwrap();
        let deserialized: PricingStructure = serde_json::from_str(&json).unwrap();

        match deserialized {
            PricingStructure::PerToken { input_price_per_million, .. } => {
                assert_eq!(input_price_per_million, dec!(10.0));
            }
            _ => panic!("Wrong pricing structure type"),
        }
    }
}
