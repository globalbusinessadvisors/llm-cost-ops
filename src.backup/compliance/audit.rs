//! Audit log domain model
//!
//! This module defines the comprehensive audit log structure for compliance
//! and security tracking across the LLM-CostOps platform.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::net::IpAddr;
use uuid::Uuid;

/// Comprehensive audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    /// Unique identifier for this audit log entry
    pub id: Uuid,

    /// Type of event being audited
    pub event_type: AuditEventType,

    /// Actor who performed the action
    pub actor: Actor,

    /// Resource that was accessed or modified
    pub resource: Option<ResourceInfo>,

    /// Action performed
    pub action: ActionType,

    /// Outcome of the action
    pub outcome: AuditOutcome,

    /// Timestamp when the event occurred (millisecond precision)
    pub timestamp: DateTime<Utc>,

    /// Duration of the operation in milliseconds
    pub duration_ms: Option<i64>,

    /// IP address of the request originator
    pub ip_address: Option<IpAddr>,

    /// User agent string from the request
    pub user_agent: Option<String>,

    /// Correlation ID for tracing related requests
    pub correlation_id: Option<String>,

    /// Session ID
    pub session_id: Option<String>,

    /// Request ID
    pub request_id: Option<String>,

    /// Organization/tenant context
    pub organization_id: Option<String>,

    /// Additional structured metadata
    pub metadata: AuditMetadata,

    /// Error message if outcome is failure
    pub error_message: Option<String>,

    /// Error code if outcome is failure
    pub error_code: Option<String>,

    /// Security labels/classification
    pub security_labels: Vec<String>,

    /// Compliance tags (e.g., GDPR, SOC2, HIPAA)
    pub compliance_tags: Vec<String>,
}

/// Actor who performed the action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Actor {
    /// Unique identifier of the actor
    pub id: String,

    /// Type of actor
    pub actor_type: ActorType,

    /// Display name or email
    pub name: Option<String>,

    /// Additional actor attributes
    pub attributes: HashMap<String, String>,
}

/// Type of actor
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorType {
    /// Human user
    User,

    /// Service account
    Service,

    /// System-generated action
    System,

    /// API client
    ApiClient,

    /// Anonymous/unauthenticated
    Anonymous,
}

/// Information about the resource being accessed or modified
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceInfo {
    /// Type of resource
    pub resource_type: String,

    /// Unique identifier of the resource
    pub resource_id: String,

    /// Display name of the resource
    pub resource_name: Option<String>,

    /// Parent resource (for hierarchical resources)
    pub parent_resource: Option<Box<ResourceInfo>>,

    /// Additional resource attributes
    pub attributes: HashMap<String, String>,
}

/// Type of action performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    // CRUD operations
    Create,
    Read,
    Update,
    Delete,
    List,

    // Authentication & Authorization
    Login,
    Logout,
    Authorize,
    Deny,

    // Configuration
    Configure,
    Reconfigure,

    // Data operations
    Export,
    Import,
    Backup,
    Restore,
    Purge,

    // Administrative
    Grant,
    Revoke,
    Enable,
    Disable,

    // Other/Unknown action
    Other,
}

/// Outcome of the audited action
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// Action completed successfully
    Success,

    /// Action failed
    Failure,

    /// Action partially succeeded
    Partial,

    /// Action was denied (authorization failure)
    Denied,

    /// Action is still in progress
    Pending,
}

/// Comprehensive event types for auditing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    // Authentication Events
    AuthLogin,
    AuthLogout,
    AuthLoginFailed,
    AuthTokenRefresh,
    AuthTokenRevoke,
    AuthPasswordChange,
    AuthPasswordReset,
    AuthMfaEnable,
    AuthMfaDisable,
    AuthMfaChallenge,

    // Authorization Events
    AuthzAccessGranted,
    AuthzAccessDenied,
    AuthzPermissionCheck,
    AuthzRoleAssigned,
    AuthzRoleRevoked,
    AuthzPolicyUpdate,

    // API Key Management
    ApiKeyCreated,
    ApiKeyRevoked,
    ApiKeyRotated,
    ApiKeyUsed,
    ApiKeyExposed,

    // Data Access
    DataRead,
    DataQuery,
    DataSearch,
    DataDownload,

    // Data Modification
    DataCreate,
    DataUpdate,
    DataDelete,
    DataBulkUpdate,
    DataBulkDelete,

    // Data Operations
    DataExport,
    DataImport,
    DataBackup,
    DataRestore,
    DataPurge,
    DataAnonymize,

    // User Management
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserSuspended,
    UserReactivated,
    UserInvited,

    // Organization/Tenant Management
    OrgCreated,
    OrgUpdated,
    OrgDeleted,
    OrgMemberAdded,
    OrgMemberRemoved,

    // Configuration Changes
    ConfigUpdate,
    ConfigReset,
    SettingChanged,
    FeatureFlagChanged,

    // Cost Operations
    CostCalculated,
    BudgetCreated,
    BudgetExceeded,
    AlertTriggered,
    ReportGenerated,
    ForecastGenerated,

    // Integration Events
    IntegrationEnabled,
    IntegrationDisabled,
    IntegrationConfigured,
    WebhookReceived,
    WebhookSent,

    // Security Events
    SecurityIncident,
    SecurityThreatDetected,
    SuspiciousActivity,
    RateLimitExceeded,
    InvalidRequest,
    DataBreach,

    // Compliance Events
    ComplianceCheckPassed,
    ComplianceCheckFailed,
    AuditLogExported,
    RetentionPolicyApplied,
    DataRetentionExpired,

    // System Events
    SystemStarted,
    SystemStopped,
    SystemConfigured,
    MaintenanceModeEnabled,
    MaintenanceModeDisabled,
    HealthCheckFailed,

    // Admin Actions
    AdminAccess,
    AdminOverride,
    EmergencyAccess,
    PrivilegedOperation,
}

/// Additional metadata for audit logs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditMetadata {
    /// Changes made (for update operations)
    pub changes: Option<AuditChanges>,

    /// Custom key-value metadata
    pub custom: HashMap<String, JsonValue>,

    /// HTTP request details
    pub http_request: Option<HttpRequestInfo>,

    /// Geographic location
    pub geo_location: Option<GeoLocation>,
}

/// Changes tracking for update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChanges {
    /// Previous values (before the change)
    pub before: HashMap<String, JsonValue>,

    /// New values (after the change)
    pub after: HashMap<String, JsonValue>,

    /// Fields that were added
    pub added: Vec<String>,

    /// Fields that were removed
    pub removed: Vec<String>,

    /// Fields that were modified
    pub modified: Vec<String>,
}

/// HTTP request information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequestInfo {
    /// HTTP method
    pub method: String,

    /// Request path
    pub path: String,

    /// Query parameters
    pub query_params: HashMap<String, String>,

    /// Request headers (sanitized - no auth tokens)
    pub headers: HashMap<String, String>,

    /// HTTP status code
    pub status_code: Option<u16>,

    /// Request body size in bytes
    pub body_size: Option<usize>,

    /// Response body size in bytes
    pub response_size: Option<usize>,
}

/// Geographic location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    /// Country code (ISO 3166-1 alpha-2)
    pub country: Option<String>,

    /// Region/state
    pub region: Option<String>,

    /// City
    pub city: Option<String>,

    /// Latitude
    pub latitude: Option<f64>,

    /// Longitude
    pub longitude: Option<f64>,
}

impl AuditLog {
    /// Create a new audit log entry
    pub fn new(
        event_type: AuditEventType,
        actor: Actor,
        action: ActionType,
        outcome: AuditOutcome,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type,
            actor,
            resource: None,
            action,
            outcome,
            timestamp: Utc::now(),
            duration_ms: None,
            ip_address: None,
            user_agent: None,
            correlation_id: None,
            session_id: None,
            request_id: None,
            organization_id: None,
            metadata: AuditMetadata::default(),
            error_message: None,
            error_code: None,
            security_labels: Vec::new(),
            compliance_tags: Vec::new(),
        }
    }

    /// Set resource information
    pub fn with_resource(mut self, resource: ResourceInfo) -> Self {
        self.resource = Some(resource);
        self
    }

    /// Set duration in milliseconds
    pub fn with_duration(mut self, duration_ms: i64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Set IP address
    pub fn with_ip_address(mut self, ip: IpAddr) -> Self {
        self.ip_address = Some(ip);
        self
    }

    /// Set user agent
    pub fn with_user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = Some(user_agent);
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set request ID
    pub fn with_request_id(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    /// Set organization ID
    pub fn with_organization_id(mut self, org_id: String) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    /// Set error information
    pub fn with_error(mut self, message: String, code: Option<String>) -> Self {
        self.error_message = Some(message);
        self.error_code = code;
        self.outcome = AuditOutcome::Failure;
        self
    }

    /// Add security label
    pub fn add_security_label(mut self, label: String) -> Self {
        self.security_labels.push(label);
        self
    }

    /// Add compliance tag
    pub fn add_compliance_tag(mut self, tag: String) -> Self {
        self.compliance_tags.push(tag);
        self
    }

    /// Set changes
    pub fn with_changes(mut self, changes: AuditChanges) -> Self {
        self.metadata.changes = Some(changes);
        self
    }

    /// Add custom metadata
    pub fn add_metadata(mut self, key: String, value: JsonValue) -> Self {
        self.metadata.custom.insert(key, value);
        self
    }

    /// Set HTTP request info
    pub fn with_http_request(mut self, http_request: HttpRequestInfo) -> Self {
        self.metadata.http_request = Some(http_request);
        self
    }

    /// Set geo location
    pub fn with_geo_location(mut self, location: GeoLocation) -> Self {
        self.metadata.geo_location = Some(location);
        self
    }
}

impl Actor {
    /// Create a new user actor
    pub fn user(id: String, name: Option<String>) -> Self {
        Self {
            id,
            actor_type: ActorType::User,
            name,
            attributes: HashMap::new(),
        }
    }

    /// Create a new service actor
    pub fn service(id: String, name: Option<String>) -> Self {
        Self {
            id,
            actor_type: ActorType::Service,
            name,
            attributes: HashMap::new(),
        }
    }

    /// Create a new system actor
    pub fn system(name: String) -> Self {
        Self {
            id: "system".to_string(),
            actor_type: ActorType::System,
            name: Some(name),
            attributes: HashMap::new(),
        }
    }

    /// Create a new API client actor
    pub fn api_client(id: String, name: Option<String>) -> Self {
        Self {
            id,
            actor_type: ActorType::ApiClient,
            name,
            attributes: HashMap::new(),
        }
    }

    /// Create an anonymous actor
    pub fn anonymous() -> Self {
        Self {
            id: "anonymous".to_string(),
            actor_type: ActorType::Anonymous,
            name: None,
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}

impl ResourceInfo {
    /// Create a new resource info
    pub fn new(resource_type: String, resource_id: String) -> Self {
        Self {
            resource_type,
            resource_id,
            resource_name: None,
            parent_resource: None,
            attributes: HashMap::new(),
        }
    }

    /// Set resource name
    pub fn with_name(mut self, name: String) -> Self {
        self.resource_name = Some(name);
        self
    }

    /// Set parent resource
    pub fn with_parent(mut self, parent: ResourceInfo) -> Self {
        self.parent_resource = Some(Box::new(parent));
        self
    }

    /// Add an attribute
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}

impl HttpRequestInfo {
    /// Create new HTTP request info
    pub fn new(method: String, path: String, status_code: Option<u16>) -> Self {
        Self {
            method,
            path,
            query_params: HashMap::new(),
            headers: HashMap::new(),
            status_code,
            body_size: None,
            response_size: None,
        }
    }
}

impl AuditChanges {
    /// Create new audit changes
    pub fn new(
        before: HashMap<String, JsonValue>,
        after: HashMap<String, JsonValue>,
    ) -> Self {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut modified = Vec::new();

        // Find added fields
        for key in after.keys() {
            if !before.contains_key(key) {
                added.push(key.clone());
            }
        }

        // Find removed and modified fields
        for key in before.keys() {
            if !after.contains_key(key) {
                removed.push(key.clone());
            } else if before.get(key) != after.get(key) {
                modified.push(key.clone());
            }
        }

        Self {
            before,
            after,
            added,
            removed,
            modified,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_audit_log_creation() {
        let actor = Actor::user("user123".to_string(), Some("test@example.com".to_string()));
        let log = AuditLog::new(
            AuditEventType::AuthLogin,
            actor,
            ActionType::Login,
            AuditOutcome::Success,
        );

        assert_eq!(log.event_type, AuditEventType::AuthLogin);
        assert_eq!(log.action, ActionType::Login);
        assert_eq!(log.outcome, AuditOutcome::Success);
        assert_eq!(log.actor.id, "user123");
    }

    #[test]
    fn test_audit_log_builder() {
        let actor = Actor::user("user123".to_string(), Some("test@example.com".to_string()));
        let resource = ResourceInfo::new("cost_record".to_string(), "rec123".to_string())
            .with_name("Q4 Cost Report".to_string());

        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));

        let log = AuditLog::new(
            AuditEventType::DataRead,
            actor,
            ActionType::Read,
            AuditOutcome::Success,
        )
        .with_resource(resource)
        .with_ip_address(ip)
        .with_user_agent("Mozilla/5.0".to_string())
        .with_correlation_id("corr-123".to_string())
        .add_security_label("confidential".to_string())
        .add_compliance_tag("SOC2".to_string());

        assert!(log.resource.is_some());
        assert_eq!(log.ip_address, Some(ip));
        assert_eq!(log.security_labels.len(), 1);
        assert_eq!(log.compliance_tags.len(), 1);
    }

    #[test]
    fn test_actor_types() {
        let user = Actor::user("u1".to_string(), None);
        assert_eq!(user.actor_type, ActorType::User);

        let service = Actor::service("svc1".to_string(), None);
        assert_eq!(service.actor_type, ActorType::Service);

        let system = Actor::system("background-job".to_string());
        assert_eq!(system.actor_type, ActorType::System);

        let api = Actor::api_client("api1".to_string(), None);
        assert_eq!(api.actor_type, ActorType::ApiClient);

        let anon = Actor::anonymous();
        assert_eq!(anon.actor_type, ActorType::Anonymous);
    }

    #[test]
    fn test_audit_changes() {
        let mut before = HashMap::new();
        before.insert("status".to_string(), serde_json::json!("active"));
        before.insert("name".to_string(), serde_json::json!("old_name"));

        let mut after = HashMap::new();
        after.insert("status".to_string(), serde_json::json!("inactive"));
        after.insert("description".to_string(), serde_json::json!("new description"));

        let changes = AuditChanges::new(before, after);

        assert_eq!(changes.added.len(), 1);
        assert!(changes.added.contains(&"description".to_string()));
        assert_eq!(changes.removed.len(), 1);
        assert!(changes.removed.contains(&"name".to_string()));
        assert_eq!(changes.modified.len(), 1);
        assert!(changes.modified.contains(&"status".to_string()));
    }

    #[test]
    fn test_resource_with_parent() {
        let parent = ResourceInfo::new("organization".to_string(), "org123".to_string());
        let resource = ResourceInfo::new("user".to_string(), "user456".to_string())
            .with_parent(parent);

        assert!(resource.parent_resource.is_some());
        let parent_ref = resource.parent_resource.unwrap();
        assert_eq!(parent_ref.resource_type, "organization");
        assert_eq!(parent_ref.resource_id, "org123");
    }
}
