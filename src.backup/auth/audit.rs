// Audit logging system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::rbac::{Action, Resource};

/// Audit event type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    /// Authentication events
    AuthLogin,
    AuthLogout,
    AuthFailed,
    AuthTokenRefresh,

    /// Authorization events
    AccessGranted,
    AccessDenied,

    /// Resource operations
    ResourceCreate,
    ResourceRead,
    ResourceUpdate,
    ResourceDelete,

    /// User management
    UserCreated,
    UserUpdated,
    UserDeleted,
    UserRoleAssigned,
    UserRoleRevoked,

    /// Role management
    RoleCreated,
    RoleUpdated,
    RoleDeleted,
    PermissionGranted,
    PermissionRevoked,

    /// API key management
    ApiKeyCreated,
    ApiKeyRevoked,
    ApiKeyUsed,

    /// Data operations
    DataExport,
    DataImport,
    DataPurge,

    /// System events
    SystemConfigChanged,
    SystemStarted,
    SystemStopped,

    /// Security events
    SecurityIncident,
    SuspiciousActivity,
}

/// Audit event severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditSeverity {
    /// Informational
    Info,

    /// Warning
    Warning,

    /// Error
    Error,

    /// Critical security event
    Critical,
}

/// Audit event status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditStatus {
    /// Event succeeded
    Success,

    /// Event failed
    Failure,

    /// Event in progress
    Pending,
}

/// Audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier
    pub id: String,

    /// Event type
    pub event_type: AuditEventType,

    /// Event severity
    pub severity: AuditSeverity,

    /// Event status
    pub status: AuditStatus,

    /// Timestamp when event occurred
    pub timestamp: DateTime<Utc>,

    /// User who triggered the event
    pub user_id: Option<String>,

    /// User's email or identifier
    pub user_email: Option<String>,

    /// Organization context
    pub organization_id: Option<String>,

    /// Resource type involved
    pub resource_type: Option<Resource>,

    /// Resource identifier
    pub resource_id: Option<String>,

    /// Action performed
    pub action: Option<Action>,

    /// IP address of the requestor
    pub ip_address: Option<IpAddr>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Request ID for correlation
    pub request_id: Option<String>,

    /// Session ID
    pub session_id: Option<String>,

    /// Event description
    pub description: String,

    /// Additional metadata
    pub metadata: HashMap<String, JsonValue>,

    /// Error message if status is Failure
    pub error: Option<String>,

    /// Changes made (before/after for updates)
    pub changes: Option<AuditChanges>,
}

/// Audit changes tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChanges {
    /// Old values
    pub before: HashMap<String, JsonValue>,

    /// New values
    pub after: HashMap<String, JsonValue>,
}

impl AuditEvent {
    /// Create a new audit event
    pub fn new(event_type: AuditEventType, description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            severity: AuditSeverity::Info,
            status: AuditStatus::Success,
            timestamp: Utc::now(),
            user_id: None,
            user_email: None,
            organization_id: None,
            resource_type: None,
            resource_id: None,
            action: None,
            ip_address: None,
            user_agent: None,
            request_id: None,
            session_id: None,
            description,
            metadata: HashMap::new(),
            error: None,
            changes: None,
        }
    }

    /// Set user context
    pub fn with_user(mut self, user_id: String, user_email: Option<String>) -> Self {
        self.user_id = Some(user_id);
        self.user_email = user_email;
        self
    }

    /// Set organization context
    pub fn with_organization(mut self, org_id: String) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    /// Set resource context
    pub fn with_resource(mut self, resource_type: Resource, resource_id: String) -> Self {
        self.resource_type = Some(resource_type);
        self.resource_id = Some(resource_id);
        self
    }

    /// Set action
    pub fn with_action(mut self, action: Action) -> Self {
        self.action = Some(action);
        self
    }

    /// Set request context
    pub fn with_request(
        mut self,
        ip: IpAddr,
        user_agent: Option<String>,
        request_id: Option<String>,
    ) -> Self {
        self.ip_address = Some(ip);
        self.user_agent = user_agent;
        self.request_id = request_id;
        self
    }

    /// Set session
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set severity
    pub fn with_severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set status
    pub fn with_status(mut self, status: AuditStatus) -> Self {
        self.status = status;
        self
    }

    /// Set error
    pub fn with_error(mut self, error: String) -> Self {
        self.error = Some(error);
        self.status = AuditStatus::Failure;
        self
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: String, value: JsonValue) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set changes
    pub fn with_changes(mut self, before: HashMap<String, JsonValue>, after: HashMap<String, JsonValue>) -> Self {
        self.changes = Some(AuditChanges { before, after });
        self
    }
}

/// Audit log query filter
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditQuery {
    /// Filter by event types
    pub event_types: Option<Vec<AuditEventType>>,

    /// Filter by user ID
    pub user_id: Option<String>,

    /// Filter by organization ID
    pub organization_id: Option<String>,

    /// Filter by resource type
    pub resource_type: Option<Resource>,

    /// Filter by resource ID
    pub resource_id: Option<String>,

    /// Filter by action
    pub action: Option<Action>,

    /// Filter by severity
    pub severity: Option<AuditSeverity>,

    /// Filter by status
    pub status: Option<AuditStatus>,

    /// Filter by time range (start)
    pub from_time: Option<DateTime<Utc>>,

    /// Filter by time range (end)
    pub to_time: Option<DateTime<Utc>>,

    /// Filter by IP address
    pub ip_address: Option<IpAddr>,

    /// Limit number of results
    pub limit: Option<usize>,

    /// Offset for pagination
    pub offset: Option<usize>,
}

impl AuditQuery {
    /// Create a new empty query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by event type
    pub fn with_event_type(mut self, event_type: AuditEventType) -> Self {
        self.event_types = Some(vec![event_type]);
        self
    }

    /// Filter by user
    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Filter by organization
    pub fn with_organization(mut self, org_id: String) -> Self {
        self.organization_id = Some(org_id);
        self
    }

    /// Filter by time range
    pub fn with_time_range(mut self, from: DateTime<Utc>, to: DateTime<Utc>) -> Self {
        self.from_time = Some(from);
        self.to_time = Some(to);
        self
    }

    /// Set limit
    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Set offset
    pub fn with_offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Audit log storage trait
#[async_trait::async_trait]
pub trait AuditStore: Send + Sync {
    /// Log an audit event
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError>;

    /// Query audit events
    async fn query(&self, query: AuditQuery) -> Result<Vec<AuditEvent>, AuditError>;

    /// Get a specific audit event
    async fn get(&self, event_id: &str) -> Result<Option<AuditEvent>, AuditError>;

    /// Purge old audit logs (for retention policy)
    async fn purge_before(&self, before: DateTime<Utc>) -> Result<usize, AuditError>;

    /// Count events matching query
    async fn count(&self, query: AuditQuery) -> Result<usize, AuditError>;
}

/// In-memory audit store (for development/testing)
pub struct InMemoryAuditStore {
    events: Arc<RwLock<Vec<AuditEvent>>>,
}

impl InMemoryAuditStore {
    /// Create a new in-memory audit store
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if event matches query
    fn matches_query(event: &AuditEvent, query: &AuditQuery) -> bool {
        if let Some(ref types) = query.event_types {
            if !types.contains(&event.event_type) {
                return false;
            }
        }

        if let Some(ref user_id) = query.user_id {
            if event.user_id.as_ref() != Some(user_id) {
                return false;
            }
        }

        if let Some(ref org_id) = query.organization_id {
            if event.organization_id.as_ref() != Some(org_id) {
                return false;
            }
        }

        if let Some(resource_type) = query.resource_type {
            if event.resource_type != Some(resource_type) {
                return false;
            }
        }

        if let Some(ref resource_id) = query.resource_id {
            if event.resource_id.as_ref() != Some(resource_id) {
                return false;
            }
        }

        if let Some(action) = query.action {
            if event.action != Some(action) {
                return false;
            }
        }

        if let Some(severity) = query.severity {
            if event.severity != severity {
                return false;
            }
        }

        if let Some(status) = query.status {
            if event.status != status {
                return false;
            }
        }

        if let Some(from_time) = query.from_time {
            if event.timestamp < from_time {
                return false;
            }
        }

        if let Some(to_time) = query.to_time {
            if event.timestamp > to_time {
                return false;
            }
        }

        if let Some(ip) = query.ip_address {
            if event.ip_address != Some(ip) {
                return false;
            }
        }

        true
    }
}

impl Default for InMemoryAuditStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AuditStore for InMemoryAuditStore {
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError> {
        let mut events = self.events.write().await;
        events.push(event);
        Ok(())
    }

    async fn query(&self, query: AuditQuery) -> Result<Vec<AuditEvent>, AuditError> {
        let events = self.events.read().await;

        let mut matched: Vec<AuditEvent> = events
            .iter()
            .filter(|e| Self::matches_query(e, &query))
            .cloned()
            .collect();

        // Sort by timestamp descending (most recent first)
        matched.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        // Apply pagination
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(usize::MAX);

        Ok(matched.into_iter().skip(offset).take(limit).collect())
    }

    async fn get(&self, event_id: &str) -> Result<Option<AuditEvent>, AuditError> {
        let events = self.events.read().await;
        Ok(events.iter().find(|e| e.id == event_id).cloned())
    }

    async fn purge_before(&self, before: DateTime<Utc>) -> Result<usize, AuditError> {
        let mut events = self.events.write().await;
        let original_len = events.len();
        events.retain(|e| e.timestamp >= before);
        Ok(original_len - events.len())
    }

    async fn count(&self, query: AuditQuery) -> Result<usize, AuditError> {
        let events = self.events.read().await;
        Ok(events.iter().filter(|e| Self::matches_query(e, &query)).count())
    }
}

/// Audit logger
pub struct AuditLogger {
    store: Arc<dyn AuditStore>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(store: Arc<dyn AuditStore>) -> Self {
        Self { store }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) -> Result<(), AuditError> {
        self.store.log(event).await
    }

    /// Log authentication success
    pub async fn log_auth_success(
        &self,
        user_id: String,
        user_email: String,
        ip: IpAddr,
    ) -> Result<(), AuditError> {
        let event = AuditEvent::new(
            AuditEventType::AuthLogin,
            format!("User {} logged in successfully", user_email),
        )
        .with_user(user_id, Some(user_email))
        .with_request(ip, None, None);

        self.log(event).await
    }

    /// Log authentication failure
    pub async fn log_auth_failure(
        &self,
        user_email: String,
        reason: String,
        ip: IpAddr,
    ) -> Result<(), AuditError> {
        let event = AuditEvent::new(
            AuditEventType::AuthFailed,
            format!("Login attempt failed for {}", user_email),
        )
        .with_severity(AuditSeverity::Warning)
        .with_status(AuditStatus::Failure)
        .with_error(reason)
        .with_request(ip, None, None);

        self.log(event).await
    }

    /// Log access denied
    pub async fn log_access_denied(
        &self,
        user_id: String,
        resource_type: Resource,
        resource_id: String,
        action: Action,
    ) -> Result<(), AuditError> {
        let event = AuditEvent::new(
            AuditEventType::AccessDenied,
            format!("Access denied to {:?} {:?} for user {}", action, resource_type, user_id),
        )
        .with_user(user_id, None)
        .with_resource(resource_type, resource_id)
        .with_action(action)
        .with_severity(AuditSeverity::Warning)
        .with_status(AuditStatus::Failure);

        self.log(event).await
    }

    /// Log resource creation
    pub async fn log_resource_create(
        &self,
        user_id: String,
        resource_type: Resource,
        resource_id: String,
        metadata: HashMap<String, JsonValue>,
    ) -> Result<(), AuditError> {
        let mut event = AuditEvent::new(
            AuditEventType::ResourceCreate,
            format!("Created {:?} {}", resource_type, resource_id),
        )
        .with_user(user_id, None)
        .with_resource(resource_type, resource_id)
        .with_action(Action::Create);

        for (key, value) in metadata {
            event = event.add_metadata(key, value);
        }

        self.log(event).await
    }

    /// Log resource update
    pub async fn log_resource_update(
        &self,
        user_id: String,
        resource_type: Resource,
        resource_id: String,
        before: HashMap<String, JsonValue>,
        after: HashMap<String, JsonValue>,
    ) -> Result<(), AuditError> {
        let event = AuditEvent::new(
            AuditEventType::ResourceUpdate,
            format!("Updated {:?} {}", resource_type, resource_id),
        )
        .with_user(user_id, None)
        .with_resource(resource_type, resource_id)
        .with_action(Action::Update)
        .with_changes(before, after);

        self.log(event).await
    }

    /// Log resource deletion
    pub async fn log_resource_delete(
        &self,
        user_id: String,
        resource_type: Resource,
        resource_id: String,
    ) -> Result<(), AuditError> {
        let event = AuditEvent::new(
            AuditEventType::ResourceDelete,
            format!("Deleted {:?} {}", resource_type, resource_id),
        )
        .with_user(user_id, None)
        .with_resource(resource_type, resource_id)
        .with_action(Action::Delete)
        .with_severity(AuditSeverity::Warning);

        self.log(event).await
    }

    /// Query audit logs
    pub async fn query(&self, query: AuditQuery) -> Result<Vec<AuditEvent>, AuditError> {
        self.store.query(query).await
    }

    /// Get a specific audit event
    pub async fn get(&self, event_id: &str) -> Result<Option<AuditEvent>, AuditError> {
        self.store.get(event_id).await
    }

    /// Purge old logs (retention policy)
    pub async fn purge_before(&self, before: DateTime<Utc>) -> Result<usize, AuditError> {
        self.store.purge_before(before).await
    }
}

/// Audit errors
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Event not found: {0}")]
    EventNotFound(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_audit_event_builder() {
        let event = AuditEvent::new(
            AuditEventType::AuthLogin,
            "User logged in".to_string(),
        )
        .with_user("user1".to_string(), Some("user@example.com".to_string()))
        .with_organization("org1".to_string())
        .with_severity(AuditSeverity::Info);

        assert_eq!(event.event_type, AuditEventType::AuthLogin);
        assert_eq!(event.user_id, Some("user1".to_string()));
        assert_eq!(event.severity, AuditSeverity::Info);
    }

    #[tokio::test]
    async fn test_in_memory_store_log_and_query() {
        let store = InMemoryAuditStore::new();

        let event = AuditEvent::new(
            AuditEventType::AuthLogin,
            "User logged in".to_string(),
        )
        .with_user("user1".to_string(), Some("user@example.com".to_string()));

        assert!(store.log(event).await.is_ok());

        let query = AuditQuery::new().with_user("user1".to_string());
        let results = store.query(query).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].user_id, Some("user1".to_string()));
    }

    #[tokio::test]
    async fn test_in_memory_store_multiple_events() {
        let store = InMemoryAuditStore::new();

        // Log multiple events
        for i in 0..5 {
            let event = AuditEvent::new(
                AuditEventType::AuthLogin,
                format!("User {} logged in", i),
            )
            .with_user(format!("user{}", i), Some(format!("user{}@example.com", i)));

            store.log(event).await.unwrap();
        }

        let query = AuditQuery::new();
        let results = store.query(query).await.unwrap();

        assert_eq!(results.len(), 5);
    }

    #[tokio::test]
    async fn test_query_filtering() {
        let store = InMemoryAuditStore::new();

        // Log different event types
        store.log(
            AuditEvent::new(AuditEventType::AuthLogin, "Login".to_string())
                .with_user("user1".to_string(), None),
        ).await.unwrap();

        store.log(
            AuditEvent::new(AuditEventType::AuthLogout, "Logout".to_string())
                .with_user("user1".to_string(), None),
        ).await.unwrap();

        store.log(
            AuditEvent::new(AuditEventType::AuthLogin, "Login".to_string())
                .with_user("user2".to_string(), None),
        ).await.unwrap();

        // Query by event type
        let query = AuditQuery::new().with_event_type(AuditEventType::AuthLogin);
        let results = store.query(query).await.unwrap();
        assert_eq!(results.len(), 2);

        // Query by user
        let query = AuditQuery::new().with_user("user1".to_string());
        let results = store.query(query).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_query_pagination() {
        let store = InMemoryAuditStore::new();

        // Log 10 events
        for i in 0..10 {
            store.log(
                AuditEvent::new(AuditEventType::AuthLogin, format!("Event {}", i))
                    .with_user("user1".to_string(), None),
            ).await.unwrap();
        }

        // Query with limit
        let query = AuditQuery::new().with_limit(5);
        let results = store.query(query).await.unwrap();
        assert_eq!(results.len(), 5);

        // Query with offset and limit
        let query = AuditQuery::new().with_offset(5).with_limit(3);
        let results = store.query(query).await.unwrap();
        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_purge_before() {
        let store = InMemoryAuditStore::new();

        let now = Utc::now();
        let old_time = now - chrono::Duration::days(10);

        // Log old event
        let mut old_event = AuditEvent::new(AuditEventType::AuthLogin, "Old event".to_string());
        old_event.timestamp = old_time;
        store.log(old_event).await.unwrap();

        // Log recent event
        store.log(
            AuditEvent::new(AuditEventType::AuthLogin, "Recent event".to_string())
        ).await.unwrap();

        // Purge events older than 5 days
        let purge_time = now - chrono::Duration::days(5);
        let purged = store.purge_before(purge_time).await.unwrap();

        assert_eq!(purged, 1);

        let query = AuditQuery::new();
        let results = store.query(query).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_audit_logger() {
        let store = Arc::new(InMemoryAuditStore::new());
        let logger = AuditLogger::new(store.clone());

        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Log auth success
        logger.log_auth_success(
            "user1".to_string(),
            "user@example.com".to_string(),
            ip,
        ).await.unwrap();

        // Log auth failure
        logger.log_auth_failure(
            "user@example.com".to_string(),
            "Invalid password".to_string(),
            ip,
        ).await.unwrap();

        // Query events
        let query = AuditQuery::new();
        let results = logger.query(query).await.unwrap();

        assert_eq!(results.len(), 2);

        // Check that one is success and one is failure
        let success = results.iter().any(|e| e.status == AuditStatus::Success);
        let failure = results.iter().any(|e| e.status == AuditStatus::Failure);
        assert!(success && failure);
    }

    #[tokio::test]
    async fn test_resource_operations_logging() {
        let store = Arc::new(InMemoryAuditStore::new());
        let logger = AuditLogger::new(store.clone());

        // Log resource creation
        logger.log_resource_create(
            "user1".to_string(),
            Resource::Usage,
            "usage123".to_string(),
            HashMap::new(),
        ).await.unwrap();

        // Log resource update
        let mut before = HashMap::new();
        before.insert("status".to_string(), serde_json::json!("active"));
        let mut after = HashMap::new();
        after.insert("status".to_string(), serde_json::json!("inactive"));

        logger.log_resource_update(
            "user1".to_string(),
            Resource::Usage,
            "usage123".to_string(),
            before,
            after,
        ).await.unwrap();

        // Log resource deletion
        logger.log_resource_delete(
            "user1".to_string(),
            Resource::Usage,
            "usage123".to_string(),
        ).await.unwrap();

        // Query all resource events
        let query = AuditQuery::new();
        let results = logger.query(query).await.unwrap();

        assert_eq!(results.len(), 3);
    }

    #[tokio::test]
    async fn test_count() {
        let store = InMemoryAuditStore::new();

        for i in 0..15 {
            store.log(
                AuditEvent::new(AuditEventType::AuthLogin, format!("Event {}", i))
                    .with_user("user1".to_string(), None),
            ).await.unwrap();
        }

        let query = AuditQuery::new().with_user("user1".to_string());
        let count = store.count(query).await.unwrap();

        assert_eq!(count, 15);
    }
}
