//! Agent Registry
//!
//! Centralized registry for all LLM-CostOps agents. This module provides
//! a single source of truth for available agents and their metadata.

use std::collections::HashMap;

use super::contracts::{AgentClassification, AgentId, AgentVersion, DecisionType};

/// Agent metadata for registry
#[derive(Debug, Clone)]
pub struct AgentMetadata {
    /// Agent identifier
    pub id: AgentId,
    /// Agent version
    pub version: AgentVersion,
    /// Agent classification
    pub classification: AgentClassification,
    /// Decision types this agent can produce
    pub decision_types: Vec<DecisionType>,
    /// Human-readable description
    pub description: String,
    /// Whether the agent is enabled
    pub enabled: bool,
}

/// Agent registry
pub struct AgentRegistry {
    agents: HashMap<String, AgentMetadata>,
}

impl AgentRegistry {
    /// Create a new registry with all registered agents
    pub fn new() -> Self {
        let mut registry = Self {
            agents: HashMap::new(),
        };
        registry.register_default_agents();
        registry
    }

    /// Register default agents
    fn register_default_agents(&mut self) {
        // Budget Enforcement Agent
        self.register(AgentMetadata {
            id: AgentId::budget_enforcement(),
            version: AgentVersion::v1_0_0(),
            classification: AgentClassification::FinancialGovernance,
            decision_types: vec![DecisionType::BudgetConstraintEvaluation],
            description: "Evaluate budget thresholds and emit advisory or gating signals".to_string(),
            enabled: true,
        });

        // Future agents can be registered here:
        // - Cost Attribution Agent
        // - Spend Forecaster Agent
        // - ROI Analyzer Agent
    }

    /// Register an agent
    pub fn register(&mut self, metadata: AgentMetadata) {
        self.agents.insert(metadata.id.to_string(), metadata);
    }

    /// Get agent metadata by ID
    pub fn get(&self, id: &str) -> Option<&AgentMetadata> {
        self.agents.get(id)
    }

    /// List all agents
    pub fn list(&self) -> Vec<&AgentMetadata> {
        self.agents.values().collect()
    }

    /// List enabled agents
    pub fn list_enabled(&self) -> Vec<&AgentMetadata> {
        self.agents.values().filter(|a| a.enabled).collect()
    }

    /// List agents by classification
    pub fn list_by_classification(&self, classification: AgentClassification) -> Vec<&AgentMetadata> {
        self.agents
            .values()
            .filter(|a| a.classification == classification)
            .collect()
    }

    /// Check if an agent is registered
    pub fn is_registered(&self, id: &str) -> bool {
        self.agents.contains_key(id)
    }
}

impl Default for AgentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = AgentRegistry::new();
        assert!(registry.is_registered("llm-costops.budget-enforcement"));
    }

    #[test]
    fn test_list_agents() {
        let registry = AgentRegistry::new();
        let agents = registry.list();
        assert!(!agents.is_empty());
    }

    #[test]
    fn test_list_by_classification() {
        let registry = AgentRegistry::new();
        let financial_agents = registry.list_by_classification(AgentClassification::FinancialGovernance);
        assert!(!financial_agents.is_empty());
    }
}
