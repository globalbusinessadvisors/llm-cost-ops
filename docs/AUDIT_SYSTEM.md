# Comprehensive Audit Logging System

## Overview

The LLM-CostOps platform includes a comprehensive audit logging system designed for enterprise-grade compliance, security, and operational tracking. This system provides immutable, queryable audit trails for all system activities.

## Features

### Core Capabilities

1. **Comprehensive Event Tracking**
   - Authentication and authorization events
   - Data access and modifications (CRUD operations)
   - API key management
   - Configuration changes
   - Security incidents
   - Compliance events

2. **Rich Context Capture**
   - Actor information (user, service, system, API client)
   - Resource details with hierarchical relationships
   - HTTP request/response metadata
   - IP addresses and user agents
   - Correlation and request IDs for distributed tracing
   - Organization/tenant context

3. **High-Performance Storage**
   - PostgreSQL backend with optimized indexes
   - Batch insert support for high-volume scenarios
   - Efficient querying with flexible filters
   - Retention policy enforcement

4. **Automatic Middleware**
   - Axum middleware for automatic HTTP request auditing
   - Configurable path exclusions
   - Asynchronous logging (non-blocking)
   - Context extraction from headers

5. **Export and Compliance**
   - Multiple export formats (JSON, NDJSON, CSV, Excel)
   - Retention policies with archival support
   - Security labels and compliance tags
   - Audit statistics and reporting

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────┐
│                    Axum Application                      │
│                                                          │
│  ┌────────────────────────────────────────────────┐    │
│  │         AuditMiddleware Layer                  │    │
│  │  (Automatic HTTP Request Auditing)             │    │
│  └────────────────────────────────────────────────┘    │
│                         │                               │
│                         ▼                               │
│  ┌────────────────────────────────────────────────┐    │
│  │            Business Logic                       │    │
│  │  (Manual audit calls for critical operations)  │    │
│  └────────────────────────────────────────────────┘    │
│                         │                               │
└─────────────────────────│───────────────────────────────┘
                          │
                          ▼
        ┌────────────────────────────────────┐
        │     AuditRepository                │
        │  (PostgreSQL/SQLite backend)       │
        └────────────────────────────────────┘
                          │
                          ▼
        ┌────────────────────────────────────┐
        │     PostgreSQL Database            │
        │  - audit_logs table                │
        │  - Optimized indexes               │
        │  - Retention policies              │
        └────────────────────────────────────┘
```

### Domain Model

#### AuditLog
The main audit log entry containing:
- Unique ID (UUID)
- Event type (comprehensive enum of all trackable events)
- Actor (who performed the action)
- Resource (what was accessed/modified)
- Action type (CRUD, authentication, configuration, etc.)
- Outcome (success, failure, denied, partial, pending)
- Timestamp with millisecond precision
- Duration in milliseconds
- Network context (IP, user agent)
- Correlation IDs for tracing
- Organization/tenant context
- Rich metadata
- Security labels and compliance tags

#### Actor
Represents the entity performing an action:
- User: Human user with ID and email
- Service: Service account
- System: System-generated action
- API Client: External API consumer
- Anonymous: Unauthenticated request

#### ResourceInfo
Information about the accessed/modified resource:
- Resource type and ID
- Display name
- Parent resource (for hierarchical structures)
- Custom attributes

## Usage

### 1. Setup Database

First, initialize the audit log table:

```rust
use llm_cost_ops::compliance::{PostgresAuditRepository, AuditRepository};
use sqlx::PgPool;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create database connection pool
    let pool = PgPool::connect("postgresql://user:pass@localhost/db").await?;
    let pool = Arc::new(pool);

    // Create repository
    let repo = PostgresAuditRepository::new(pool);

    // Initialize table
    repo.init_table().await?;

    Ok(())
}
```

### 2. Add Audit Middleware to Axum

```rust
use axum::{Router, routing::get};
use llm_cost_ops::compliance::{
    PostgresAuditRepository, create_audit_layer,
};

async fn create_app(pool: Arc<PgPool>) -> Router {
    let repo = Arc::new(PostgresAuditRepository::new(pool));
    let audit_layer = create_audit_layer(repo);

    Router::new()
        .route("/api/users", get(list_users))
        .route("/api/costs", get(get_costs))
        .layer(audit_layer)
}
```

### 3. Manual Audit Logging

For critical operations or custom events:

```rust
use llm_cost_ops::compliance::{
    AuditLog, ComplianceAuditEventType, AuditOutcome,
    Actor, ActionType, ResourceInfo, AuditRepository,
};

async fn delete_sensitive_data(
    user_id: String,
    resource_id: String,
    repo: Arc<dyn AuditRepository>,
) -> Result<(), Error> {
    // Perform the deletion
    // ... business logic ...

    // Create audit log
    let actor = Actor::user(user_id.clone(), None);
    let resource = ResourceInfo::new("sensitive_data".to_string(), resource_id);

    let audit_log = AuditLog::new(
        ComplianceAuditEventType::DataDelete,
        actor,
        ActionType::Delete,
        AuditOutcome::Success,
    )
    .with_resource(resource)
    .add_security_label("pii".to_string())
    .add_compliance_tag("GDPR".to_string());

    // Store the audit log
    repo.store(&audit_log).await?;

    Ok(())
}
```

### 4. Track Changes

For update operations, track before/after state:

```rust
use std::collections::HashMap;
use llm_cost_ops::compliance::AuditChanges;

async fn update_budget(
    user_id: String,
    project_id: String,
    old_budget: f64,
    new_budget: f64,
    repo: Arc<dyn AuditRepository>,
) -> Result<(), Error> {
    // Create before/after state
    let mut before = HashMap::new();
    before.insert("budget".to_string(), serde_json::json!(old_budget));

    let mut after = HashMap::new();
    after.insert("budget".to_string(), serde_json::json!(new_budget));

    let changes = AuditChanges::new(before, after);

    let actor = Actor::user(user_id, None);
    let resource = ResourceInfo::new("project".to_string(), project_id);

    let audit_log = AuditLog::new(
        ComplianceAuditEventType::DataUpdate,
        actor,
        ActionType::Update,
        AuditOutcome::Success,
    )
    .with_resource(resource)
    .with_changes(changes);

    repo.store(&audit_log).await?;

    Ok(())
}
```

### 5. Querying Audit Logs

```rust
use llm_cost_ops::compliance::AuditFilter;
use chrono::{Utc, Duration};

async fn get_user_activity(
    user_id: String,
    repo: Arc<dyn AuditRepository>,
) -> Result<Vec<AuditLog>, Error> {
    let filter = AuditFilter {
        actor_id: Some(user_id),
        from_time: Some(Utc::now() - Duration::days(30)),
        to_time: Some(Utc::now()),
        limit: Some(100),
        ..Default::default()
    };

    let logs = repo.query(&filter).await?;
    Ok(logs)
}

async fn get_security_incidents(
    repo: Arc<dyn AuditRepository>,
) -> Result<Vec<AuditLog>, Error> {
    let filter = AuditFilter {
        event_types: Some(vec![
            ComplianceAuditEventType::SecurityIncident,
            ComplianceAuditEventType::SecurityThreatDetected,
            ComplianceAuditEventType::SuspiciousActivity,
        ]),
        from_time: Some(Utc::now() - Duration::days(7)),
        ..Default::default()
    };

    let logs = repo.query(&filter).await?;
    Ok(logs)
}
```

### 6. Export Audit Logs

```rust
use llm_cost_ops::compliance::AuditExportFormat;

async fn export_compliance_report(
    org_id: String,
    repo: Arc<dyn AuditRepository>,
) -> Result<Vec<u8>, Error> {
    let filter = AuditFilter {
        organization_id: Some(org_id),
        compliance_tags: Some(vec!["SOC2".to_string()]),
        from_time: Some(Utc::now() - Duration::days(90)),
        ..Default::default()
    };

    // Export as CSV for compliance reporting
    let csv_data = repo.export(&filter, AuditExportFormat::Csv).await?;
    Ok(csv_data)
}
```

### 7. Retention Policy

```rust
use llm_cost_ops::compliance::AuditRetentionPolicy;

async fn apply_retention(
    repo: Arc<dyn AuditRepository>,
) -> Result<(), Error> {
    // Standard retention: 90 days
    let standard_policy = AuditRetentionPolicy {
        retention_days: 90,
        event_types: None,
        archive_before_delete: true,
        archive_location: Some("s3://audit-archive/standard/".to_string()),
    };

    let deleted = repo.apply_retention_policy(&standard_policy).await?;
    println!("Deleted {} standard audit logs", deleted);

    // Extended retention for security events: 365 days
    let security_policy = AuditRetentionPolicy {
        retention_days: 365,
        event_types: Some(vec![
            ComplianceAuditEventType::SecurityIncident,
            ComplianceAuditEventType::DataBreach,
        ]),
        archive_before_delete: true,
        archive_location: Some("s3://audit-archive/security/".to_string()),
    };

    let deleted = repo.apply_retention_policy(&security_policy).await?;
    println!("Deleted {} security audit logs", deleted);

    Ok(())
}
```

## Event Types Reference

### Authentication Events
- `AuthLogin` - Successful login
- `AuthLogout` - User logout
- `AuthLoginFailed` - Failed login attempt
- `AuthTokenRefresh` - Token refresh
- `AuthTokenRevoke` - Token revocation
- `AuthPasswordChange` - Password change
- `AuthPasswordReset` - Password reset
- `AuthMfaEnable` - MFA enabled
- `AuthMfaDisable` - MFA disabled

### Authorization Events
- `AuthzAccessGranted` - Access granted
- `AuthzAccessDenied` - Access denied
- `AuthzPermissionCheck` - Permission check
- `AuthzRoleAssigned` - Role assigned to user
- `AuthzRoleRevoked` - Role revoked from user

### Data Events
- `DataRead` - Data read/query
- `DataCreate` - Data creation
- `DataUpdate` - Data update
- `DataDelete` - Data deletion
- `DataExport` - Data export
- `DataImport` - Data import
- `DataBackup` - Backup operation
- `DataRestore` - Restore operation
- `DataPurge` - Data purge
- `DataAnonymize` - Data anonymization

### API Key Events
- `ApiKeyCreated` - API key created
- `ApiKeyRevoked` - API key revoked
- `ApiKeyRotated` - API key rotated
- `ApiKeyUsed` - API key used
- `ApiKeyExposed` - API key exposure detected

### Security Events
- `SecurityIncident` - Security incident
- `SecurityThreatDetected` - Threat detected
- `SuspiciousActivity` - Suspicious activity
- `RateLimitExceeded` - Rate limit exceeded
- `DataBreach` - Data breach

### Configuration Events
- `ConfigUpdate` - Configuration update
- `SettingChanged` - Setting change
- `FeatureFlagChanged` - Feature flag change

### System Events
- `SystemStarted` - System started
- `SystemStopped` - System stopped
- `HealthCheckFailed` - Health check failure

## Best Practices

### 1. Security Labels

Use security labels to categorize sensitive data:

```rust
audit_log
    .add_security_label("pii".to_string())
    .add_security_label("financial".to_string())
    .add_security_label("confidential".to_string())
```

### 2. Compliance Tags

Tag logs for compliance frameworks:

```rust
audit_log
    .add_compliance_tag("SOC2".to_string())
    .add_compliance_tag("GDPR".to_string())
    .add_compliance_tag("HIPAA".to_string())
```

### 3. Correlation IDs

Use correlation IDs to track related operations:

```rust
let correlation_id = Uuid::new_v4().to_string();

// First operation
let log1 = AuditLog::new(...)
    .with_correlation_id(correlation_id.clone());

// Related operation
let log2 = AuditLog::new(...)
    .with_correlation_id(correlation_id.clone());
```

### 4. Error Handling

Always log failures with error details:

```rust
match perform_operation() {
    Ok(_) => {
        let log = AuditLog::new(..., AuditOutcome::Success);
        repo.store(&log).await?;
    }
    Err(e) => {
        let log = AuditLog::new(..., AuditOutcome::Failure)
            .with_error(e.to_string(), Some(error_code));
        repo.store(&log).await?;
    }
}
```

### 5. Batch Operations

For high-volume scenarios, use batch inserts:

```rust
let mut logs = Vec::new();

for item in items {
    let log = AuditLog::new(...);
    logs.push(log);
}

// Store all logs in a single transaction
repo.store_batch(&logs).await?;
```

## Database Schema

The `audit_logs` table structure:

```sql
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    event_type TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    actor_type TEXT NOT NULL,
    actor_name TEXT,
    actor_attributes JSONB,
    resource_type TEXT,
    resource_id TEXT,
    resource_name TEXT,
    resource_attributes JSONB,
    action TEXT NOT NULL,
    outcome TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    duration_ms BIGINT,
    ip_address INET,
    user_agent TEXT,
    correlation_id TEXT,
    session_id TEXT,
    request_id TEXT,
    organization_id TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_message TEXT,
    error_code TEXT,
    security_labels TEXT[],
    compliance_tags TEXT[],
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX idx_audit_logs_actor_id ON audit_logs(actor_id);
CREATE INDEX idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
CREATE INDEX idx_audit_logs_organization_id ON audit_logs(organization_id);
CREATE INDEX idx_audit_logs_correlation_id ON audit_logs(correlation_id);
CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
CREATE INDEX idx_audit_logs_outcome ON audit_logs(outcome);
CREATE INDEX idx_audit_logs_security_labels ON audit_logs USING GIN(security_labels);
CREATE INDEX idx_audit_logs_compliance_tags ON audit_logs USING GIN(compliance_tags);
```

## Performance Considerations

1. **Asynchronous Logging**: The middleware uses `tokio::spawn` for non-blocking audit log storage
2. **Batch Inserts**: Use `store_batch` for high-volume scenarios
3. **Index Usage**: Queries automatically use optimized indexes
4. **Retention Policies**: Regular cleanup prevents table bloat
5. **Connection Pooling**: Use SQLx connection pooling for efficient database access

## Compliance and Regulatory Support

The audit system supports compliance with:

- **SOC 2**: Comprehensive logging of all access and changes
- **GDPR**: Data access tracking, consent logging, breach notification
- **HIPAA**: PHI access logging with security labels
- **PCI DSS**: Payment data access tracking
- **Custom Frameworks**: Flexible tagging and labeling system

## Testing

Comprehensive tests are available in `/tests/audit_system_tests.rs`:

```bash
cargo test --test audit_system_tests
```

## Migration Guide

If you're using the older `auth::AuditLogger`, you can migrate to the new system:

**Before:**
```rust
use llm_cost_ops::auth::{AuditLogger, AuditEvent};

let logger = AuditLogger::new(store);
logger.log_auth_success(user_id, email, ip).await?;
```

**After:**
```rust
use llm_cost_ops::compliance::{
    AuditLog, ComplianceAuditEventType, Actor, ActionType, AuditOutcome,
};

let actor = Actor::user(user_id, Some(email));
let log = AuditLog::new(
    ComplianceAuditEventType::AuthLogin,
    actor,
    ActionType::Login,
    AuditOutcome::Success,
)
.with_ip_address(ip);

repo.store(&log).await?;
```

## Future Enhancements

Planned features:
- Elasticsearch integration for full-text search
- Real-time streaming to SIEM systems
- Automated anomaly detection
- GraphQL query interface
- Blockchain-based immutability guarantees
