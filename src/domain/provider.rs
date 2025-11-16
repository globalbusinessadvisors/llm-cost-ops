use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    #[serde(alias = "OpenAI", alias = "openai")]
    OpenAI,

    #[serde(alias = "Anthropic", alias = "anthropic")]
    Anthropic,

    #[serde(alias = "Google", alias = "google", alias = "vertex")]
    GoogleVertexAI,

    #[serde(alias = "Azure", alias = "azure")]
    AzureOpenAI,

    #[serde(alias = "AWS", alias = "aws", alias = "bedrock")]
    AWSBedrock,

    #[serde(alias = "Cohere", alias = "cohere")]
    Cohere,

    #[serde(alias = "Mistral", alias = "mistral")]
    Mistral,

    Custom(String),
}

impl Provider {
    pub fn as_str(&self) -> &str {
        match self {
            Provider::OpenAI => "openai",
            Provider::Anthropic => "anthropic",
            Provider::GoogleVertexAI => "google",
            Provider::AzureOpenAI => "azure",
            Provider::AWSBedrock => "aws",
            Provider::Cohere => "cohere",
            Provider::Mistral => "mistral",
            Provider::Custom(name) => name,
        }
    }

    pub fn parse(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "openai" => Provider::OpenAI,
            "anthropic" => Provider::Anthropic,
            "google" | "vertex" => Provider::GoogleVertexAI,
            "azure" => Provider::AzureOpenAI,
            "aws" | "bedrock" => Provider::AWSBedrock,
            "cohere" => Provider::Cohere,
            "mistral" => Provider::Mistral,
            other => Provider::Custom(other.to_string()),
        }
    }

    pub fn supports_token_validation(&self) -> bool {
        matches!(
            self,
            Provider::OpenAI | Provider::Anthropic | Provider::GoogleVertexAI
        )
    }

    pub fn default_context_window(&self, model: &str) -> u64 {
        match (self, model) {
            (Provider::OpenAI, m) if m.contains("gpt-4") => 8192,
            (Provider::OpenAI, m) if m.contains("gpt-3.5") => 4096,
            (Provider::Anthropic, m) if m.contains("claude-3") => 200000,
            (Provider::GoogleVertexAI, _) => 32768,
            _ => 4096,
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Provider {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "openai" => Provider::OpenAI,
            "anthropic" => Provider::Anthropic,
            "google" | "vertex" => Provider::GoogleVertexAI,
            "azure" => Provider::AzureOpenAI,
            "aws" | "bedrock" => Provider::AWSBedrock,
            "cohere" => Provider::Cohere,
            "mistral" => Provider::Mistral,
            other => Provider::Custom(other.to_string()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_parsing() {
        assert_eq!(Provider::from_str("OpenAI"), Ok(Provider::OpenAI));
        assert_eq!(Provider::from_str("openai"), Ok(Provider::OpenAI));
        assert_eq!(Provider::from_str("anthropic"), Ok(Provider::Anthropic));
        assert_eq!(Provider::from_str("custom-provider"), Ok(Provider::Custom("custom-provider".to_string())));
    }

    #[test]
    fn test_provider_serialization() {
        let provider = Provider::OpenAI;
        let json = serde_json::to_string(&provider).unwrap();
        assert_eq!(json, "\"openai\"");

        let deserialized: Provider = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Provider::OpenAI);
    }
}
