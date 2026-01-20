//! Agent Type Definitions
//!
//! Core types for agent identification, versioning, and I/O contracts.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Agent identifier (e.g., "cost-forecasting-agent")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentId(pub String);

impl AgentId {
    /// Create a new AgentId
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AgentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Agent semantic version following semver
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AgentVersion(pub String);

impl AgentVersion {
    /// Create a new AgentVersion
    pub fn new(version: impl Into<String>) -> Self {
        Self(version.into())
    }

    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Parse version components (major, minor, patch)
    pub fn parse(&self) -> Option<(u32, u32, u32)> {
        let parts: Vec<&str> = self.0.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let major = parts[0].parse().ok()?;
        let minor = parts[1].parse().ok()?;
        let patch = parts[2].parse().ok()?;

        Some((major, minor, patch))
    }

    /// Check if this version is compatible with another (same major version)
    pub fn is_compatible_with(&self, other: &AgentVersion) -> bool {
        match (self.parse(), other.parse()) {
            (Some((m1, _, _)), Some((m2, _, _))) => m1 == m2,
            _ => false,
        }
    }
}

impl std::fmt::Display for AgentVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// SHA-256 hash of inputs for deduplication and audit
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct InputsHash(pub String);

impl InputsHash {
    /// Create a new InputsHash from a hex string
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Compute hash from serializable input
    pub fn compute<T: Serialize>(input: &T) -> Self {
        let serialized = serde_json::to_vec(input).unwrap_or_default();
        Self::compute_from_bytes(&serialized)
    }

    /// Compute hash from bytes
    pub fn compute_from_bytes(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        Self(hex::encode(result))
    }

    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for InputsHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// SHA-256 hash of outputs for verification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OutputsHash(pub String);

impl OutputsHash {
    /// Create a new OutputsHash from a hex string
    pub fn new(hash: impl Into<String>) -> Self {
        Self(hash.into())
    }

    /// Compute hash from serializable output
    pub fn compute<T: Serialize>(output: &T) -> Self {
        let serialized = serde_json::to_vec(output).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&serialized);
        let result = hasher.finalize();
        Self(hex::encode(result))
    }

    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for OutputsHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Trait for agent input types
pub trait AgentInput: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + Clone {
    /// Validate the input
    fn validate(&self) -> Result<(), super::ValidationError>;

    /// Compute the inputs hash
    fn inputs_hash(&self) -> InputsHash {
        InputsHash::compute(self)
    }
}

/// Trait for agent output types
pub trait AgentOutput: serde::de::DeserializeOwned + serde::Serialize + Send + Sync + Clone {
    /// Validate the output
    fn validate(&self) -> Result<(), super::ValidationError>;

    /// Compute the outputs hash
    fn outputs_hash(&self) -> OutputsHash {
        OutputsHash::compute(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_id() {
        let id = AgentId::new("cost-forecasting-agent");
        assert_eq!(id.as_str(), "cost-forecasting-agent");
        assert_eq!(id.to_string(), "cost-forecasting-agent");
    }

    #[test]
    fn test_agent_version_parsing() {
        let version = AgentVersion::new("1.2.3");
        assert_eq!(version.parse(), Some((1, 2, 3)));

        let invalid = AgentVersion::new("not-a-version");
        assert_eq!(invalid.parse(), None);
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = AgentVersion::new("1.0.0");
        let v1_2_3 = AgentVersion::new("1.2.3");
        let v2_0_0 = AgentVersion::new("2.0.0");

        assert!(v1_0_0.is_compatible_with(&v1_2_3));
        assert!(!v1_0_0.is_compatible_with(&v2_0_0));
    }

    #[test]
    fn test_inputs_hash() {
        let data = serde_json::json!({"key": "value"});
        let hash1 = InputsHash::compute(&data);
        let hash2 = InputsHash::compute(&data);

        // Same input should produce same hash
        assert_eq!(hash1, hash2);

        // Different input should produce different hash
        let data2 = serde_json::json!({"key": "different"});
        let hash3 = InputsHash::compute(&data2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_format() {
        let hash = InputsHash::compute_from_bytes(b"test");
        // SHA-256 produces 64 hex characters
        assert_eq!(hash.as_str().len(), 64);
        assert!(hash.as_str().chars().all(|c| c.is_ascii_hexdigit()));
    }
}
