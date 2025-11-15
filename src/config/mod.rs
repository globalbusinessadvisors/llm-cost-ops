use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::domain::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub api: Option<ApiConfig>,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub bind: String,
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub json: bool,
}

fn default_pool_size() -> u32 {
    10
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn default_sqlite() -> Self {
        Self {
            database: DatabaseConfig {
                url: "sqlite:cost-ops.db".to_string(),
                pool_size: 10,
            },
            api: None,
            logging: LoggingConfig {
                level: "info".to_string(),
                json: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default_sqlite();
        assert_eq!(config.database.pool_size, 10);
        assert_eq!(config.logging.level, "info");
    }
}
