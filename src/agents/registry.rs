//! Agent Registry
//!
//! Platform registration for LLM-CostOps agents.
//!
//! This module provides:
//! - Agent registration metadata
//! - Discovery endpoints
//! - Version compatibility checking

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use super::{
    AgentClassification,
    contracts::{AgentId, AgentVersion, DecisionType},
    cost_forecasting::{AGENT_ID, AGENT_VERSION},
};

/// Agent registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistryEntry {
    /// Agent identifier
    pub agent_id: String,

    /// Agent version
    pub version: String,

    /// Agent classification
    pub classification: AgentClassification,

    /// Decision type produced
    pub decision_type: DecisionType,

    /// Human-readable description
    pub description: String,

    /// CLI command name
    pub cli_command: String,

    /// API endpoint path
    pub api_endpoint: String,

    /// Input schema version
    pub input_schema_version: String,

    /// Output schema version
    pub output_schema_version: String,

    /// Whether the agent is enabled
    pub enabled: bool,

    /// Agent tags for discovery
    pub tags: Vec<String>,
}

/// Agent registry for LLM-CostOps
#[derive(Debug, Clone, Default)]
pub struct AgentRegistry {
    agents: HashMap<String, AgentRegistryEntry>,
}

impl AgentRegistry {
    /// Create a new registry with default agents
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_default_agents();
        registry
    }

    /// Register the default agents
    fn register_default_agents(&mut self) {
        // Register Cost Forecasting Agent
        self.register(AgentRegistryEntry {
            agent_id: AGENT_ID.to_string(),
            version: AGENT_VERSION.to_string(),
            classification: AgentClassification::Forecasting,
            decision_type: DecisionType::CostForecast,
            description: "Forecasts future LLM spend based on historical usage patterns and growth trends".to_string(),
            cli_command: "forecast".to_string(),
            api_endpoint: "/api/v1/agents/cost-forecasting/forecast".to_string(),
            input_schema_version: super::contracts::CONTRACT_VERSION.to_string(),
            output_schema_version: super::contracts::CONTRACT_VERSION.to_string(),
            enabled: true,
            tags: vec![
                "forecasting".to_string(),
                "cost".to_string(),
                "prediction".to_string(),
                "llm".to_string(),
            ],
        });
    }

    /// Register an agent
    pub fn register(&mut self, entry: AgentRegistryEntry) {
        self.agents.insert(entry.agent_id.clone(), entry);
    }

    /// Get an agent by ID
    pub fn get(&self, agent_id: &str) -> Option<&AgentRegistryEntry> {
        self.agents.get(agent_id)
    }

    /// List all agents
    pub fn list(&self) -> Vec<&AgentRegistryEntry> {
        self.agents.values().collect()
    }

    /// List agents by classification
    pub fn list_by_classification(&self, classification: AgentClassification) -> Vec<&AgentRegistryEntry> {
        self.agents
            .values()
            .filter(|a| a.classification == classification)
            .collect()
    }

    /// List enabled agents
    pub fn list_enabled(&self) -> Vec<&AgentRegistryEntry> {
        self.agents
            .values()
            .filter(|a| a.enabled)
            .collect()
    }

    /// Check if an agent is registered
    pub fn is_registered(&self, agent_id: &str) -> bool {
        self.agents.contains_key(agent_id)
    }

    /// Get agent count
    pub fn count(&self) -> usize {
        self.agents.len()
    }
}

/// Global agent registry instance
lazy_static::lazy_static! {
    static ref GLOBAL_REGISTRY: AgentRegistry = AgentRegistry::new();
}

/// Get the global agent registry
pub fn global_registry() -> &'static AgentRegistry {
    &GLOBAL_REGISTRY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = AgentRegistry::new();
        assert!(registry.count() > 0);
    }

    #[test]
    fn test_cost_forecasting_registered() {
        let registry = AgentRegistry::new();
        let entry = registry.get(AGENT_ID);
        assert!(entry.is_some());

        let entry = entry.unwrap();
        assert_eq!(entry.classification, AgentClassification::Forecasting);
        assert!(entry.enabled);
    }

    #[test]
    fn test_list_by_classification() {
        let registry = AgentRegistry::new();
        let forecasting_agents = registry.list_by_classification(AgentClassification::Forecasting);
        assert!(!forecasting_agents.is_empty());
    }

    #[test]
    fn test_global_registry() {
        let registry = global_registry();
        assert!(registry.is_registered(AGENT_ID));
    }
}
