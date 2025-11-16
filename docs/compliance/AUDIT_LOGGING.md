# Audit Logging Guide

## Overview

The LLM Cost Ops platform provides comprehensive audit logging capabilities to track all system activities, user actions, and security events. This guide explains the audit logging system, event types, retention policies, query capabilities, and integration options.

## Table of Contents

1. [Event Types Tracked](#event-types-tracked)
2. [Audit Log Structure](#audit-log-structure)
3. [Log Retention](#log-retention)
4. [Query Examples](#query-examples)
5. [Integration Guide](#integration-guide)
6. [Export and Reporting](#export-and-reporting)
7. [Security and Integrity](#security-and-integrity)
8. [Compliance Mapping](#compliance-mapping)

## Event Types Tracked

### Authentication Events

| Event Type | Description | Severity | Typical Data Captured |
|------------|-------------|----------|----------------------|
| `auth_login` | Successful user login | Info | User ID, IP address, timestamp, user agent |
| `auth_logout` | User logout | Info | User ID, session duration |
| `auth_failed` | Failed login attempt | Warning | Username/email, IP address, failure reason |
| `auth_token_refresh` | JWT token refreshed | Info | User ID, token ID |
| `auth_mfa_success` | MFA verification succeeded | Info | User ID, MFA method |
| `auth_mfa_failed` | MFA verification failed | Warning | User ID, attempt count |
| `auth_password_reset` | Password reset initiated | Warning | User ID, reset method |
| `auth_session_timeout` | Session expired | Info | User ID, last activity |

### Authorization Events

| Event Type | Description | Severity | Typical Data Captured |
|------------|-------------|----------|----------------------|
| `access_granted` | Access to resource granted | Info | User ID, resource type, resource ID, action |
| `access_denied` | Access to resource denied | Warning | User ID, resource type, resource ID, action, reason |
| `permission_check` | Permission validation performed | Info | User ID, permission checked |

### Resource Operations

| Event Type | Description | Severity | Typical Data Captured |
|------------|-------------|----------|----------------------|
| `resource_create` | Resource created | Info | Resource type, resource ID, creator, metadata |
| `resource_read` | Resource accessed/viewed | Info | Resource type, resource ID, user ID |
| `resource_update` | Resource modified | Info | Resource type, resource ID, changes (before/after) |
| `resource_delete` | Resource deleted | Warning | Resource type, resource ID, deletion reason |
| `resource_export` | Resource data exported | Warning | Resource type, export format, user ID |

### User Management

| Event Type | Description | Severity | Typical Data Captured |
|------------|-------------|----------|----------------------|
| `user_created` | New user account created | Info | User ID, created by, role |
| `user_updated` | User account modified | Info | User ID, changes made |
| `user_deleted` | User account deleted | Warning | User ID, deleted by, reason |
| `user_role_assigned` | Role assigned to user | Warning | User ID, role ID, assigned by |
| `user_role_revoked` | Role removed from user | Warning | User ID, role ID, revoked by |
| `user_suspended` | User account suspended | Critical | User ID, reason, suspended by |
| `user_reactivated` | User account reactivated | Warning | User ID, reactivated by |

### Role Management

| Event Type | Description | Severity | Typical Data Captured |
|------------|-------------|----------|----------------------|
| `role_created` | New role created | Warning | Role ID, role name, created by |
| `role_updated` | Role modified | Warning | Role ID, changes made |
| `role_deleted` | Role deleted | Warning | Role ID, deleted by |
| `permission_granted` | Permission added to role | Warning | Role ID, permission, granted by |
| `permission_revoked` | Permission removed from role | Warning | Role ID, permission, revoked by |

### API Key Management

| Event Type | Description | Severity | Typical Data Captured |
|------------|-------------|----------|----------------------|
| `api_key_created` | New API key generated | Warning | Key ID, user ID, scopes |
| `api_key_revoked` | API key revoked | Warning | Key ID, revoked by, reason |
| `api_key_used` | API key used for authentication | Info | Key ID, endpoint, IP address |
| `api_key_expired` | API key expired | Info | Key ID, expiration date |
| `api_key_rotated` | API key rotated | Warning | Old key ID, new key ID |

### Data Operations

| Event Type | Description | Severity | Typical Data Captured |
|------------|-------------|----------|----------------------|
| `data_export` | Data exported | Warning | Data type, format, user ID, record count |
| `data_import` | Data imported | Warning | Data type, source, user ID, record count |
| `data_purge` | Data purged/deleted | Critical | Data type, purge criteria, user ID |
| `data_backup` | Backup created | Info | Backup ID, data size, location |
| `data_restore` | Data restored from backup | Critical | Backup ID, restored by |

### System Events

| Event Type | Description | Severity | Typical Data Captured |
|------------|-------------|----------|----------------------|
| `system_config_changed` | System configuration modified | Warning | Config key, old value, new value, changed by |
| `system_started` | System/service started | Info | Service name, version, start time |
| `system_stopped` | System/service stopped | Warning | Service name, stop reason |
| `system_error` | System error occurred | Error | Error type, error message, stack trace |
| `system_health_check` | Health check performed | Info | Check type, status, metrics |

### Security Events

| Event Type | Description | Severity | Typical Data Captured |
|------------|-------------|----------|----------------------|
| `security_incident` | Security incident detected | Critical | Incident type, severity, affected resources |
| `suspicious_activity` | Suspicious behavior detected | Critical | Activity type, user ID, IP address, details |
| `brute_force_detected` | Brute force attack detected | Critical | Target user, IP address, attempt count |
| `privilege_escalation` | Privilege escalation attempt | Critical | User ID, attempted permission, denied reason |
| `data_breach_suspected` | Potential data breach | Critical | Data type, scope, detection method |
| `malware_detected` | Malware detected | Critical | Malware type, location, action taken |

## Audit Log Structure

### Event Schema

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier (UUID)
    pub id: String,

    /// Event type (from enum AuditEventType)
    pub event_type: AuditEventType,

    /// Event severity (Info, Warning, Error, Critical)
    pub severity: AuditSeverity,

    /// Event status (Success, Failure, Pending)
    pub status: AuditStatus,

    /// Timestamp when event occurred (UTC)
    pub timestamp: DateTime<Utc>,

    /// User who triggered the event
    pub user_id: Option<String>,

    /// User's email or identifier
    pub user_email: Option<String>,

    /// Organization context
    pub organization_id: Option<String>,

    /// Resource type involved (Usage, Cost, User, etc.)
    pub resource_type: Option<Resource>,

    /// Resource identifier
    pub resource_id: Option<String>,

    /// Action performed (Read, Create, Update, Delete, etc.)
    pub action: Option<Action>,

    /// IP address of the requestor
    pub ip_address: Option<IpAddr>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Request ID for correlation
    pub request_id: Option<String>,

    /// Session ID
    pub session_id: Option<String>,

    /// Event description (human-readable)
    pub description: String,

    /// Additional metadata (flexible JSON)
    pub metadata: HashMap<String, JsonValue>,

    /// Error message if status is Failure
    pub error: Option<String>,

    /// Changes made (before/after for updates)
    pub changes: Option<AuditChanges>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditChanges {
    /// Old values (before change)
    pub before: HashMap<String, JsonValue>,

    /// New values (after change)
    pub after: HashMap<String, JsonValue>,
}
```

### Example Audit Events

#### Successful Login
```json
{
  "id": "evt_login_123456",
  "event_type": "auth_login",
  "severity": "info",
  "status": "success",
  "timestamp": "2024-11-16T10:30:45Z",
  "user_id": "user_12345",
  "user_email": "john.doe@example.com",
  "organization_id": "org_98765",
  "ip_address": "203.0.113.42",
  "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)...",
  "request_id": "req_abc123",
  "session_id": "sess_xyz789",
  "description": "User john.doe@example.com logged in successfully",
  "metadata": {
    "login_method": "password",
    "mfa_used": true,
    "device_type": "desktop"
  }
}
```

#### Failed Authorization
```json
{
  "id": "evt_authz_789012",
  "event_type": "access_denied",
  "severity": "warning",
  "status": "failure",
  "timestamp": "2024-11-16T10:35:12Z",
  "user_id": "user_12345",
  "user_email": "john.doe@example.com",
  "organization_id": "org_98765",
  "resource_type": "cost",
  "resource_id": "cost_record_456",
  "action": "delete",
  "ip_address": "203.0.113.42",
  "request_id": "req_def456",
  "description": "Access denied to delete cost_record_456 for user john.doe@example.com",
  "error": "Insufficient permissions: user does not have Delete permission on Cost resource",
  "metadata": {
    "required_permission": "cost:delete",
    "user_roles": ["read_only"]
  }
}
```

#### Resource Update with Changes
```json
{
  "id": "evt_update_345678",
  "event_type": "resource_update",
  "severity": "info",
  "status": "success",
  "timestamp": "2024-11-16T10:40:00Z",
  "user_id": "user_12345",
  "organization_id": "org_98765",
  "resource_type": "user",
  "resource_id": "user_67890",
  "action": "update",
  "description": "Updated user profile for user_67890",
  "changes": {
    "before": {
      "email": "old@example.com",
      "role": "member"
    },
    "after": {
      "email": "new@example.com",
      "role": "admin"
    }
  },
  "metadata": {
    "updated_fields": ["email", "role"],
    "change_reason": "User promotion and email change"
  }
}
```

#### Security Incident
```json
{
  "id": "evt_security_901234",
  "event_type": "suspicious_activity",
  "severity": "critical",
  "status": "success",
  "timestamp": "2024-11-16T10:45:30Z",
  "user_id": "user_12345",
  "ip_address": "198.51.100.99",
  "description": "Suspicious activity detected: multiple failed login attempts from unusual location",
  "metadata": {
    "failed_attempts": 10,
    "time_window_minutes": 5,
    "usual_locations": ["US", "CA"],
    "current_location": "RU",
    "action_taken": "account_locked",
    "notification_sent": true
  }
}
```

## Log Retention

### Default Retention Periods

| Event Category | Retention Period | Reason |
|----------------|------------------|--------|
| Authentication Events | 90 days | Security analysis |
| Authorization Events | 90 days | Access pattern analysis |
| Resource Operations | 1 year | Operational history |
| User Management | 7 years | Compliance (GDPR, SOC 2) |
| Role Management | 7 years | Compliance |
| API Key Management | 7 years | Security audit trail |
| Data Operations | 7 years | Compliance (GDPR Art. 30) |
| System Events | 90 days | Troubleshooting |
| Security Events | 7 years | Security investigations |

### Configuration

```toml
# config/audit.toml

[audit.retention]
# Retention periods in days
authentication_events = 90
authorization_events = 90
resource_operations = 365
user_management = 2555  # 7 years
role_management = 2555
api_key_management = 2555
data_operations = 2555
system_events = 90
security_events = 2555

# Purge settings
auto_purge_enabled = true
purge_schedule = "0 3 * * 0"  # Every Sunday at 3 AM
purge_batch_size = 10000

# Archive settings
archive_before_purge = true
archive_location = "s3://audit-archives/"
archive_encryption = true
```

### Retention Policy Implementation

```rust
use llm_cost_ops::auth::audit::{AuditLogger, AuditQuery};
use chrono::{Utc, Duration};

async fn apply_retention_policy(
    logger: &AuditLogger,
) -> Result<(), Box<dyn std::error::Error>> {
    let now = Utc::now();

    // Purge authentication events older than 90 days
    let auth_cutoff = now - Duration::days(90);
    let auth_purged = logger.purge_events_before(
        auth_cutoff,
        vec![
            AuditEventType::AuthLogin,
            AuditEventType::AuthLogout,
            AuditEventType::AuthFailed,
        ]
    ).await?;

    println!("Purged {} authentication events", auth_purged);

    // Archive and purge resource operations older than 1 year
    let resource_cutoff = now - Duration::days(365);

    // First, archive
    let archive_query = AuditQuery::new()
        .with_event_types(vec![
            AuditEventType::ResourceCreate,
            AuditEventType::ResourceUpdate,
            AuditEventType::ResourceDelete,
        ])
        .with_time_range(Utc::now() - Duration::days(3650), resource_cutoff);

    let events_to_archive = logger.query(archive_query).await?;
    archive_to_s3(events_to_archive).await?;

    // Then purge
    let resource_purged = logger.purge_events_before(
        resource_cutoff,
        vec![
            AuditEventType::ResourceCreate,
            AuditEventType::ResourceUpdate,
            AuditEventType::ResourceDelete,
        ]
    ).await?;

    println!("Archived and purged {} resource operation events", resource_purged);

    Ok(())
}
```

## Query Examples

### Basic Queries

#### Query by Event Type
```rust
use llm_cost_ops::auth::audit::{AuditLogger, AuditQuery, AuditEventType};

let logger = AuditLogger::new(store);

// Query all login events
let query = AuditQuery::new()
    .with_event_type(AuditEventType::AuthLogin)
    .with_limit(100);

let login_events = logger.query(query).await?;

for event in login_events {
    println!("{}: {} from {}",
        event.timestamp,
        event.user_email.unwrap_or_default(),
        event.ip_address.unwrap_or_default()
    );
}
```

#### Query by User
```rust
// Query all events for a specific user
let query = AuditQuery::new()
    .with_user("user_12345".to_string())
    .with_limit(50);

let user_events = logger.query(query).await?;
```

#### Query by Time Range
```rust
use chrono::{Utc, Duration};

// Query events from the last 24 hours
let now = Utc::now();
let yesterday = now - Duration::days(1);

let query = AuditQuery::new()
    .with_time_range(yesterday, now)
    .with_limit(1000);

let recent_events = logger.query(query).await?;
```

### Advanced Queries

#### Query Failed Login Attempts
```rust
// Find all failed login attempts in the last 7 days
let query = AuditQuery::new()
    .with_event_type(AuditEventType::AuthFailed)
    .with_time_range(Utc::now() - Duration::days(7), Utc::now())
    .with_status(AuditStatus::Failure);

let failed_logins = logger.query(query).await?;

// Group by user to detect potential brute force
let mut attempts_by_user = HashMap::new();
for event in failed_logins {
    if let Some(email) = event.user_email {
        *attempts_by_user.entry(email).or_insert(0) += 1;
    }
}

// Alert on users with >5 failed attempts
for (email, count) in attempts_by_user {
    if count > 5 {
        println!("ALERT: {} has {} failed login attempts", email, count);
    }
}
```

#### Query Access Denied Events
```rust
// Find all access denied events for sensitive resources
let query = AuditQuery::new()
    .with_event_type(AuditEventType::AccessDenied)
    .with_resource_type(Resource::Cost)
    .with_time_range(Utc::now() - Duration::days(30), Utc::now());

let denied_access = logger.query(query).await?;

// Analyze patterns
for event in denied_access {
    println!("User {} attempted {} on {} - DENIED",
        event.user_id.unwrap_or_default(),
        event.action.unwrap_or_default(),
        event.resource_id.unwrap_or_default()
    );
}
```

#### Query Resource Changes
```rust
// Track all changes to a specific resource
let query = AuditQuery::new()
    .with_event_type(AuditEventType::ResourceUpdate)
    .with_resource_id("user_12345".to_string())
    .with_limit(100);

let changes = logger.query(query).await?;

for event in changes {
    if let Some(changes) = event.changes {
        println!("Changes at {}: {:?} -> {:?}",
            event.timestamp,
            changes.before,
            changes.after
        );
    }
}
```

#### Query by Organization
```rust
// Query all events for an organization
let query = AuditQuery::new()
    .with_organization("org_98765".to_string())
    .with_time_range(Utc::now() - Duration::days(30), Utc::now())
    .with_limit(10000);

let org_events = logger.query(query).await?;

// Generate activity report
let mut event_counts = HashMap::new();
for event in org_events {
    *event_counts.entry(event.event_type).or_insert(0) += 1;
}

println!("Organization Activity Report:");
for (event_type, count) in event_counts {
    println!("{:?}: {}", event_type, count);
}
```

### HTTP API Queries

#### Query Endpoint
```http
GET /api/v1/audit/events
Authorization: Bearer {token}
```

**Query Parameters**:
- `event_type`: Filter by event type (e.g., `auth_login`)
- `user_id`: Filter by user ID
- `organization_id`: Filter by organization ID
- `resource_type`: Filter by resource type (e.g., `cost`)
- `resource_id`: Filter by specific resource ID
- `action`: Filter by action (e.g., `create`, `delete`)
- `severity`: Filter by severity (`info`, `warning`, `error`, `critical`)
- `status`: Filter by status (`success`, `failure`)
- `from_time`: Start of time range (ISO 8601)
- `to_time`: End of time range (ISO 8601)
- `ip_address`: Filter by IP address
- `limit`: Maximum results (default: 100, max: 1000)
- `offset`: Pagination offset

**Examples**:

```bash
# Get all login events from the last day
curl "https://api.example.com/api/v1/audit/events?event_type=auth_login&from_time=2024-11-15T00:00:00Z" \
  -H "Authorization: Bearer $TOKEN"

# Get failed authentication attempts
curl "https://api.example.com/api/v1/audit/events?event_type=auth_failed&status=failure&limit=50" \
  -H "Authorization: Bearer $TOKEN"

# Get all events for a user
curl "https://api.example.com/api/v1/audit/events?user_id=user_12345&limit=100" \
  -H "Authorization: Bearer $TOKEN"

# Get critical security events
curl "https://api.example.com/api/v1/audit/events?severity=critical&limit=100" \
  -H "Authorization: Bearer $TOKEN"
```

## Integration Guide

### Logging Events

#### Basic Event Logging
```rust
use llm_cost_ops::auth::audit::{AuditLogger, AuditEvent, AuditEventType};

let logger = AuditLogger::new(store);

// Log a simple event
let event = AuditEvent::new(
    AuditEventType::AuthLogin,
    "User logged in successfully".to_string()
)
.with_user("user_12345".to_string(), Some("user@example.com".to_string()))
.with_request(ip_address, Some(user_agent), Some(request_id));

logger.log(event).await?;
```

#### Logging with Context
```rust
// Log with full context
let event = AuditEvent::new(
    AuditEventType::ResourceUpdate,
    "Updated pricing table".to_string()
)
.with_user(user_id, Some(user_email))
.with_organization(org_id)
.with_resource(Resource::Pricing, pricing_id)
.with_action(Action::Update)
.with_request(ip_address, Some(user_agent), Some(request_id))
.with_session(session_id)
.add_metadata("table_name".to_string(), json!("openai_pricing"))
.add_metadata("rows_affected".to_string(), json!(15));

logger.log(event).await?;
```

#### Logging Changes
```rust
// Log resource update with before/after changes
let mut before = HashMap::new();
before.insert("status".to_string(), json!("active"));
before.insert("role".to_string(), json!("member"));

let mut after = HashMap::new();
after.insert("status".to_string(), json!("active"));
after.insert("role".to_string(), json!("admin"));

logger.log_resource_update(
    user_id,
    Resource::User,
    target_user_id,
    before,
    after
).await?;
```

### Middleware Integration

#### Axum Middleware
```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

pub async fn audit_middleware(
    audit_logger: Arc<AuditLogger>,
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let ip_address = extract_ip(&request);

    // Process request
    let response = next.run(request).await;

    // Log the request
    let event = AuditEvent::new(
        AuditEventType::ResourceRead,
        format!("{} {}", method, uri)
    )
    .with_request(ip_address, None, None)
    .with_status(if response.status().is_success() {
        AuditStatus::Success
    } else {
        AuditStatus::Failure
    });

    let _ = audit_logger.log(event).await;

    response
}
```

### SIEM Integration

#### Syslog Export
```rust
use llm_cost_ops::integrations::syslog::SyslogExporter;

let syslog = SyslogExporter::new("syslog.example.com:514");

// Export audit events to syslog
let events = logger.query(query).await?;
for event in events {
    syslog.send(&event).await?;
}
```

#### Splunk Integration
```rust
use llm_cost_ops::integrations::splunk::SplunkExporter;

let splunk = SplunkExporter::new(
    "https://splunk.example.com:8088",
    "your-hec-token"
);

// Send events to Splunk
splunk.send_batch(events).await?;
```

#### Elasticsearch Integration
```rust
use llm_cost_ops::integrations::elasticsearch::ElasticsearchExporter;

let es = ElasticsearchExporter::new("https://elasticsearch.example.com:9200");

// Index events in Elasticsearch
for event in events {
    es.index("audit-logs", &event).await?;
}
```

## Export and Reporting

### Export Formats

#### CSV Export
```rust
use llm_cost_ops::export::{ExportFormat, Exporter};

let exporter = Exporter::new(ExportFormat::Csv);

// Query events
let events = logger.query(query).await?;

// Export to CSV
let csv_data = exporter.export_audit_events(events)?;

// Save to file
std::fs::write("audit_logs.csv", csv_data)?;
```

#### JSON Export
```rust
let exporter = Exporter::new(ExportFormat::Json);
let json_data = exporter.export_audit_events(events)?;
std::fs::write("audit_logs.json", json_data)?;
```

#### Excel Export
```rust
let exporter = Exporter::new(ExportFormat::Excel);
let excel_data = exporter.export_audit_events(events)?;
std::fs::write("audit_logs.xlsx", excel_data)?;
```

### Automated Reports

#### Daily Summary Report
```rust
use llm_cost_ops::export::scheduler::{ReportScheduler, ReportSchedule};

let scheduler = ReportScheduler::new(db_pool);

// Schedule daily audit summary
let schedule = ReportSchedule {
    name: "Daily Audit Summary".to_string(),
    cron: "0 9 * * *".to_string(),  // Every day at 9 AM
    report_type: ReportType::AuditSummary,
    filters: ReportFilters::default(),
    delivery: DeliveryMethod::Email {
        recipients: vec!["security@example.com".to_string()],
        subject: "Daily Audit Summary".to_string(),
    },
};

scheduler.add_schedule(schedule).await?;
```

#### Security Event Alert
```rust
// Alert on critical security events
let scheduler = ReportScheduler::new(db_pool);

let schedule = ReportSchedule {
    name: "Security Alert".to_string(),
    cron: "*/5 * * * *".to_string(),  // Every 5 minutes
    report_type: ReportType::SecurityAlert,
    filters: ReportFilters {
        severity: Some(AuditSeverity::Critical),
        ..Default::default()
    },
    delivery: DeliveryMethod::Webhook {
        url: "https://alerts.example.com/webhook".to_string(),
    },
};

scheduler.add_schedule(schedule).await?;
```

## Security and Integrity

### Log Integrity

#### Cryptographic Hashing
```rust
use sha2::{Sha256, Digest};

// Generate hash chain for log integrity
fn generate_log_hash(
    event: &AuditEvent,
    previous_hash: &str
) -> String {
    let mut hasher = Sha256::new();

    // Hash event data + previous hash
    hasher.update(serde_json::to_string(event).unwrap());
    hasher.update(previous_hash);

    format!("{:x}", hasher.finalize())
}

// Verify log chain integrity
fn verify_log_chain(events: &[AuditEvent]) -> bool {
    let mut previous_hash = "genesis_hash";

    for event in events {
        let expected_hash = generate_log_hash(event, previous_hash);
        if event.hash != expected_hash {
            return false;
        }
        previous_hash = &event.hash;
    }

    true
}
```

### Tamper Detection

#### Write-Once Storage
```rust
// Audit logs are immutable
impl AuditStore {
    // Logs can only be written, never updated or deleted
    async fn log(&self, event: AuditEvent) -> Result<(), AuditError> {
        // Insert only - no UPDATE or DELETE allowed
        self.db.execute(
            "INSERT INTO audit_logs (...) VALUES (...)",
            &event
        ).await?;

        Ok(())
    }
}
```

#### Separate Audit Database
```toml
# config/database.toml

[database.primary]
url = "postgresql://localhost/llm_cost_ops"

[database.audit]
# Separate database for audit logs
url = "postgresql://localhost/audit_logs"
# Optional: different server for additional security
# url = "postgresql://audit-server/audit_logs"
read_only_users = ["app_user"]
admin_users = ["audit_admin"]
```

## Compliance Mapping

### GDPR Article 30 - Records of Processing

Audit logs provide evidence of:
- Who accessed personal data
- What operations were performed
- When data was accessed or modified
- Purpose of data processing

### SOC 2 Controls

| Control | Audit Events | Purpose |
|---------|--------------|---------|
| CC6.1 - Logical Access | auth_login, auth_failed, access_denied | Access control effectiveness |
| CC6.3 - Access Provisioning | user_created, user_role_assigned | User lifecycle management |
| CC6.4 - Access Removal | user_deleted, api_key_revoked | Deprovisioning verification |
| CC7.2 - Security Events | security_incident, suspicious_activity | Security monitoring |
| CC8.1 - Change Management | system_config_changed | Change tracking |

### HIPAA Audit Controls (45 CFR § 164.312(b))

- Hardware, software, and procedural mechanisms to record and examine activity
- Audit logs retention for 6 years minimum
- Regular review of information system activity

## Best Practices

### 1. Log Everything Important
```rust
// Always log authentication attempts
logger.log_auth_attempt(user_email, success, ip_address).await?;

// Always log permission changes
logger.log_permission_change(user_id, role_id, granted).await?;

// Always log data exports
logger.log_data_export(user_id, data_type, record_count).await?;
```

### 2. Include Context
```rust
// Good: includes full context
let event = AuditEvent::new(event_type, description)
    .with_user(user_id, Some(user_email))
    .with_organization(org_id)
    .with_request(ip_address, Some(user_agent), Some(request_id))
    .with_session(session_id);

// Bad: minimal context
let event = AuditEvent::new(event_type, description);
```

### 3. Use Appropriate Severity
```rust
// Critical: Security incidents, data breaches
event.with_severity(AuditSeverity::Critical)

// Warning: Permission changes, deletions, failed access
event.with_severity(AuditSeverity::Warning)

// Info: Normal operations
event.with_severity(AuditSeverity::Info)
```

### 4. Regular Review
```bash
# Weekly security review
cargo run --bin audit-review -- --last-week --critical-only

# Monthly access review
cargo run --bin audit-review -- --last-month --event-type access_denied
```

### 5. Monitor for Patterns
```rust
// Detect unusual patterns
let detector = AnomalyDetector::new();

detector.monitor(vec![
    Pattern::MultipleFailedLogins { threshold: 5, window_minutes: 15 },
    Pattern::UnusualAccessTime,
    Pattern::AccessFromNewLocation,
    Pattern::BulkDataExport { threshold_records: 10000 },
]);
```

## Troubleshooting

### High Log Volume
```toml
# config/audit.toml
[audit.sampling]
# Sample low-severity events
enabled = true
sample_rate_info = 0.1  # Log 10% of info events
sample_rate_warning = 1.0  # Log all warnings
sample_rate_critical = 1.0  # Log all critical events
```

### Slow Queries
```sql
-- Create indexes for common queries
CREATE INDEX idx_audit_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_org_id ON audit_logs(organization_id);
CREATE INDEX idx_audit_severity ON audit_logs(severity);
```

### Storage Management
```bash
# Archive old logs
cargo run --bin archive-audit-logs -- --older-than 90d --destination s3://archives/

# Purge archived logs
cargo run --bin purge-audit-logs -- --archived-only --older-than 1y
```

## Resources

### Tools
- Audit log viewer: Web UI at `/audit/logs`
- Export tool: `cargo run --bin export-audit-logs`
- Analytics tool: `cargo run --bin audit-analytics`

### Documentation
- API Reference: `/docs/api/audit-api.md`
- Event Schema: `/docs/schemas/audit-event.json`
- Integration Guide: `/docs/integrations/audit-logging.md`

### Support
- **Email**: audit@llm-cost-ops.io
- **Documentation**: https://docs.llm-cost-ops.io/audit-logging
- **Issue Tracker**: https://github.com/llm-cost-ops/issues

---

**Last Updated**: November 2024
**Version**: 1.0.0
**Reviewed By**: Security Team
