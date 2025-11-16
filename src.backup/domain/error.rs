use thiserror::Error;

pub type Result<T> = std::result::Result<T, CostOpsError>;

#[derive(Error, Debug)]
pub enum CostOpsError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Invalid token count: {0}")]
    InvalidTokenCount(String),

    #[error("Token count mismatch: calculated={calculated}, reported={reported}")]
    TokenCountMismatch {
        calculated: u64,
        reported: u64,
    },

    #[error("Missing organization ID")]
    MissingOrganizationId,

    #[error("Future timestamp not allowed")]
    FutureTimestamp,

    #[error("Provider not found: {0}")]
    ProviderNotFound(String),

    #[error("Pricing model not found for provider={provider}, model={model}, date={date}")]
    PricingModelNotFound {
        provider: String,
        model: String,
        date: String,
    },

    #[error("Invalid pricing structure: {0}")]
    InvalidPricingStructure(String),

    #[error("Currency conversion error: {0}")]
    CurrencyConversion(String),

    #[error("Integration error: {0}")]
    Integration(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl CostOpsError {
    pub fn config<S: Into<String>>(msg: S) -> Self {
        CostOpsError::Config(msg.into())
    }

    pub fn validation<S: Into<String>>(msg: S) -> Self {
        CostOpsError::Validation(msg.into())
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        CostOpsError::Internal(msg.into())
    }
}
