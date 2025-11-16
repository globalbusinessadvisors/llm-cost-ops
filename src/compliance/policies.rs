//! Compliance policy management
//!
//! Defines and enforces compliance policies including retention, access control,
//! and data classification policies.

use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Policy error types
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    #[error("Policy not found: {0}")]
    NotFound(String),

    #[error("Policy validation failed: {0}")]
    ValidationFailed(String),

    #[error("Policy conflict: {0}")]
    Conflict(String),

    #[error("Policy version mismatch: {0}")]
    VersionMismatch(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type PolicyResult<T> = Result<T, PolicyError>;

/// Retention period specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RetentionPeriod {
    Days { days: u32 },
    Months { months: u32 },
    Years { years: u32 },
    Indefinite,
}

impl RetentionPeriod {
    pub fn to_duration(&self) -> Option<Duration> {
        match self {
            RetentionPeriod::Days { days } => Some(Duration::days(*days as i64)),
            RetentionPeriod::Months { months } => Some(Duration::days((*months as i64) * 30)),
            RetentionPeriod::Years { years } => Some(Duration::days((*years as i64) * 365)),
            RetentionPeriod::Indefinite => None,
        }
    }

    pub fn is_expired(&self, created_at: DateTime<Utc>) -> bool {
        match self.to_duration() {
            Some(duration) => Utc::now() > created_at + duration,
            None => false,
        }
    }
}

/// Data classification levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
    Pii,
    Pci,
    Phi,
}

/// Policy types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyType {
    Retention,
    Access,
    Encryption,
    Audit,
    Gdpr,
    Soc2,
    Hipaa,
    PciDss,
    Custom,
}

/// Policy status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    Draft,
    Active,
    Deprecated,
    Archived,
}

/// Policy rule types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PolicyRuleType {
    RetentionRule {
        data_type: String,
        classification: DataClassification,
        period: RetentionPeriod,
        auto_delete: bool,
    },
    AccessRule {
        resource_type: String,
        allowed_roles: Vec<String>,
        required_permissions: Vec<String>,
        mfa_required: bool,
    },
    EncryptionRule {
        data_type: String,
        encryption_required: bool,
        algorithm: String,
        key_rotation_days: Option<u32>,
    },
    AuditRule {
        event_types: Vec<String>,
        retention_days: u32,
        alert_on_failure: bool,
    },
}

/// Individual policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub rule_type: PolicyRuleType,
    pub priority: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Retention policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub data_types: Vec<String>,
    pub classification: DataClassification,
    pub period: RetentionPeriod,
    pub auto_delete: bool,
    pub legal_hold_override: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Access policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub resource_patterns: Vec<String>,
    pub allowed_roles: Vec<String>,
    pub denied_roles: Vec<String>,
    pub ip_whitelist: Vec<String>,
    pub ip_blacklist: Vec<String>,
    pub time_restrictions: Option<TimeRestriction>,
    pub mfa_required: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRestriction {
    pub allowed_days: Vec<u32>,
    pub allowed_hours_start: u32,
    pub allowed_hours_end: u32,
    pub timezone: String,
}

/// Policy version tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyVersion {
    pub version: u32,
    pub policy_id: Uuid,
    pub changes: String,
    pub changed_by: String,
    pub created_at: DateTime<Utc>,
}

/// Compliance policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    pub framework: String,
    pub version: String,
    pub effective_date: DateTime<Utc>,
    pub review_interval_days: u32,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

/// Main compliance policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompliancePolicy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub policy_type: PolicyType,
    pub status: PolicyStatus,
    pub version: u32,
    pub rules: Vec<PolicyRule>,
    pub config: PolicyConfig,
    pub tags: Vec<String>,
    pub owner: String,
    pub approvers: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub next_review_date: DateTime<Utc>,
}

impl CompliancePolicy {
    pub fn new(
        name: String,
        description: String,
        policy_type: PolicyType,
        config: PolicyConfig,
        owner: String,
    ) -> Self {
        let now = Utc::now();
        let next_review_date = now + Duration::days(config.review_interval_days as i64);

        Self {
            id: Uuid::new_v4(),
            name,
            description,
            policy_type,
            status: PolicyStatus::Draft,
            version: 1,
            rules: Vec::new(),
            config,
            tags: Vec::new(),
            owner,
            approvers: Vec::new(),
            created_at: now,
            updated_at: now,
            approved_at: None,
            next_review_date,
        }
    }

    pub fn add_rule(&mut self, rule: PolicyRule) {
        self.rules.push(rule);
        self.updated_at = Utc::now();
    }

    pub fn activate(&mut self) -> PolicyResult<()> {
        if self.rules.is_empty() {
            return Err(PolicyError::ValidationFailed(
                "Cannot activate policy without rules".to_string(),
            ));
        }
        self.status = PolicyStatus::Active;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn approve(&mut self, approver: String) {
        self.approvers.push(approver);
        self.approved_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn deprecate(&mut self) {
        self.status = PolicyStatus::Deprecated;
        self.updated_at = Utc::now();
    }

    pub fn needs_review(&self) -> bool {
        Utc::now() > self.next_review_date
    }
}

/// Policy manager for creating and managing compliance policies
pub struct PolicyManager {
    policies: Arc<RwLock<HashMap<Uuid, CompliancePolicy>>>,
    retention_policies: Arc<RwLock<HashMap<Uuid, RetentionPolicy>>>,
    access_policies: Arc<RwLock<HashMap<Uuid, AccessPolicy>>>,
    versions: Arc<RwLock<HashMap<Uuid, Vec<PolicyVersion>>>>,
}

impl PolicyManager {
    pub fn new() -> Self {
        Self {
            policies: Arc::new(RwLock::new(HashMap::new())),
            retention_policies: Arc::new(RwLock::new(HashMap::new())),
            access_policies: Arc::new(RwLock::new(HashMap::new())),
            versions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new compliance policy
    pub async fn create_policy(&self, policy: CompliancePolicy) -> PolicyResult<Uuid> {
        let id = policy.id;
        let mut policies = self.policies.write().await;

        if policies.contains_key(&id) {
            return Err(PolicyError::Conflict(format!("Policy {} already exists", id)));
        }

        policies.insert(id, policy);
        Ok(id)
    }

    /// Get a policy by ID
    pub async fn get_policy(&self, id: Uuid) -> PolicyResult<CompliancePolicy> {
        let policies = self.policies.read().await;
        policies
            .get(&id)
            .cloned()
            .ok_or_else(|| PolicyError::NotFound(id.to_string()))
    }

    /// Update an existing policy
    pub async fn update_policy(
        &self,
        id: Uuid,
        mut policy: CompliancePolicy,
        changed_by: String,
    ) -> PolicyResult<()> {
        let mut policies = self.policies.write().await;

        let existing = policies
            .get(&id)
            .ok_or_else(|| PolicyError::NotFound(id.to_string()))?;

        // Create version record
        let version = PolicyVersion {
            version: existing.version + 1,
            policy_id: id,
            changes: format!("Updated policy from version {}", existing.version),
            changed_by,
            created_at: Utc::now(),
        };

        policy.version = version.version;
        policy.updated_at = Utc::now();
        policies.insert(id, policy);

        // Store version history
        let mut versions = self.versions.write().await;
        versions.entry(id).or_insert_with(Vec::new).push(version);

        Ok(())
    }

    /// List all policies with optional filtering
    pub async fn list_policies(
        &self,
        policy_type: Option<PolicyType>,
        status: Option<PolicyStatus>,
    ) -> Vec<CompliancePolicy> {
        let policies = self.policies.read().await;

        policies
            .values()
            .filter(|p| {
                policy_type.is_none_or(|t| p.policy_type == t)
                    && status.is_none_or(|s| p.status == s)
            })
            .cloned()
            .collect()
    }

    /// Create a retention policy
    pub async fn create_retention_policy(&self, policy: RetentionPolicy) -> PolicyResult<Uuid> {
        let id = policy.id;
        let mut policies = self.retention_policies.write().await;
        policies.insert(id, policy);
        Ok(id)
    }

    /// Get retention policy for a data type
    pub async fn get_retention_policy_for_type(
        &self,
        data_type: &str,
        classification: DataClassification,
    ) -> Option<RetentionPolicy> {
        let policies = self.retention_policies.read().await;

        policies
            .values()
            .find(|p| {
                p.data_types.contains(&data_type.to_string())
                    && p.classification == classification
            })
            .cloned()
    }

    /// Create an access policy
    pub async fn create_access_policy(&self, policy: AccessPolicy) -> PolicyResult<Uuid> {
        let id = policy.id;
        let mut policies = self.access_policies.write().await;
        policies.insert(id, policy);
        Ok(id)
    }

    /// Check if access is allowed by policies
    pub async fn check_access(
        &self,
        resource: &str,
        role: &str,
        ip_address: Option<&str>,
    ) -> bool {
        let policies = self.access_policies.read().await;

        for policy in policies.values() {
            // Check if resource matches pattern
            let matches_resource = policy.resource_patterns.iter().any(|pattern| {
                // Simple glob matching
                resource.contains(pattern)
            });

            if !matches_resource {
                continue;
            }

            // Check denied roles first
            if policy.denied_roles.contains(&role.to_string()) {
                return false;
            }

            // Check allowed roles
            if !policy.allowed_roles.is_empty() && !policy.allowed_roles.contains(&role.to_string()) {
                return false;
            }

            // Check IP whitelist/blacklist
            if let Some(ip) = ip_address {
                if !policy.ip_blacklist.is_empty() && policy.ip_blacklist.contains(&ip.to_string()) {
                    return false;
                }
                if !policy.ip_whitelist.is_empty() && !policy.ip_whitelist.contains(&ip.to_string()) {
                    return false;
                }
            }
        }

        true
    }

    /// Get policies that need review
    pub async fn get_policies_needing_review(&self) -> Vec<CompliancePolicy> {
        let policies = self.policies.read().await;

        policies
            .values()
            .filter(|p| p.needs_review() && p.status == PolicyStatus::Active)
            .cloned()
            .collect()
    }

    /// Get policy version history
    pub async fn get_policy_versions(&self, policy_id: Uuid) -> Vec<PolicyVersion> {
        let versions = self.versions.read().await;
        versions.get(&policy_id).cloned().unwrap_or_default()
    }
}

impl Default for PolicyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retention_period_days() {
        let period = RetentionPeriod::Days { days: 30 };
        let duration = period.to_duration().unwrap();
        assert_eq!(duration.num_days(), 30);
    }

    #[test]
    fn test_retention_period_expired() {
        let period = RetentionPeriod::Days { days: 1 };
        let created_at = Utc::now() - Duration::days(2);
        assert!(period.is_expired(created_at));
    }

    #[tokio::test]
    async fn test_policy_creation() {
        let manager = PolicyManager::new();

        let config = PolicyConfig {
            framework: "SOC2".to_string(),
            version: "2.0".to_string(),
            effective_date: Utc::now(),
            review_interval_days: 365,
            custom_fields: HashMap::new(),
        };

        let policy = CompliancePolicy::new(
            "Test Policy".to_string(),
            "Test Description".to_string(),
            PolicyType::Soc2,
            config,
            "admin@example.com".to_string(),
        );

        let id = manager.create_policy(policy).await.unwrap();
        let retrieved = manager.get_policy(id).await.unwrap();

        assert_eq!(retrieved.name, "Test Policy");
        assert_eq!(retrieved.status, PolicyStatus::Draft);
    }

    #[tokio::test]
    async fn test_retention_policy_lookup() {
        let manager = PolicyManager::new();

        let policy = RetentionPolicy {
            id: Uuid::new_v4(),
            name: "PII Retention".to_string(),
            description: "7 year retention for PII".to_string(),
            data_types: vec!["user_data".to_string(), "customer_info".to_string()],
            classification: DataClassification::Pii,
            period: RetentionPeriod::Years { years: 7 },
            auto_delete: true,
            legal_hold_override: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        manager.create_retention_policy(policy).await.unwrap();

        let found = manager
            .get_retention_policy_for_type("user_data", DataClassification::Pii)
            .await;

        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "PII Retention");
    }
}
