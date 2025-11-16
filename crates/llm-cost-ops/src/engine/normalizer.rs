use crate::domain::{Provider, Result, UsageRecord};

/// Token normalizer for handling provider-specific token counting
pub struct TokenNormalizer;

impl TokenNormalizer {
    pub fn new() -> Self {
        Self
    }

    /// Normalize token counts across providers
    pub fn normalize(&self, record: &UsageRecord) -> Result<UsageRecord> {
        let mut normalized = record.clone();

        // Apply provider-specific normalization factors
        let normalization_factor = self.get_normalization_factor(&record.provider);

        if normalization_factor != 1.0 {
            normalized.prompt_tokens = (record.prompt_tokens as f64 * normalization_factor) as u64;
            normalized.completion_tokens =
                (record.completion_tokens as f64 * normalization_factor) as u64;
            normalized.total_tokens = normalized.prompt_tokens + normalized.completion_tokens;
        }

        Ok(normalized)
    }

    /// Get provider-specific normalization factor
    fn get_normalization_factor(&self, provider: &Provider) -> f64 {
        match provider {
            Provider::OpenAI => 1.0,
            Provider::Anthropic => 1.0,
            Provider::GoogleVertexAI => 1.0,
            Provider::AzureOpenAI => 1.0,
            Provider::AWSBedrock => 1.0,
            Provider::Cohere => 1.0,
            Provider::Mistral => 1.0,
            Provider::Custom(_) => 1.0,
        }
    }

    /// Estimate tokens from text if not provided
    pub fn estimate_tokens(&self, text: &str, provider: &Provider) -> u64 {
        // Conservative estimation: ~4 characters per token
        let char_count = text.len() as f64;
        let estimated_tokens = (char_count / 4.0).ceil() as u64;

        // Apply provider-specific adjustment
        match provider {
            Provider::OpenAI => estimated_tokens,
            Provider::Anthropic => estimated_tokens,
            _ => estimated_tokens,
        }
    }

    /// Validate token consistency
    pub fn validate_consistency(&self, record: &UsageRecord) -> Result<bool> {
        let calculated_total = record.prompt_tokens + record.completion_tokens;
        let tolerance = 1; // Allow 1 token difference for rounding

        Ok(calculated_total.abs_diff(record.total_tokens) <= tolerance)
    }
}

impl Default for TokenNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ModelIdentifier, IngestionSource};
    use chrono::Utc;

    fn create_test_usage() -> UsageRecord {
        UsageRecord {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            provider: Provider::OpenAI,
            model: ModelIdentifier::new("gpt-4".to_string(), 8192),
            organization_id: "org-test".to_string(),
            project_id: None,
            user_id: None,
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
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

    #[test]
    fn test_normalize() {
        let normalizer = TokenNormalizer::new();
        let usage = create_test_usage();

        let normalized = normalizer.normalize(&usage).unwrap();
        assert_eq!(normalized.total_tokens, usage.total_tokens);
    }

    #[test]
    fn test_estimate_tokens() {
        let normalizer = TokenNormalizer::new();
        let text = "This is a test message with approximately twenty words";

        let estimated = normalizer.estimate_tokens(text, &Provider::OpenAI);
        assert!(estimated > 0);
        assert!(estimated < 100); // Reasonable upper bound
    }

    #[test]
    fn test_validate_consistency() {
        let normalizer = TokenNormalizer::new();
        let usage = create_test_usage();

        let is_consistent = normalizer.validate_consistency(&usage).unwrap();
        assert!(is_consistent);
    }

    #[test]
    fn test_validate_inconsistent() {
        let normalizer = TokenNormalizer::new();
        let mut usage = create_test_usage();
        usage.total_tokens = 200; // Intentional mismatch

        let is_consistent = normalizer.validate_consistency(&usage).unwrap();
        assert!(!is_consistent);
    }
}
