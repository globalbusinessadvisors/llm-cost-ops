# LLM Cost Ops - Compliance

[![Crates.io](https://img.shields.io/crates/v/llm-cost-ops-compliance.svg)](https://crates.io/crates/llm-cost-ops-compliance)
[![Documentation](https://docs.rs/llm-cost-ops-compliance/badge.svg)](https://docs.rs/llm-cost-ops-compliance)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Enterprise-grade compliance, security, and governance for LLM Cost Ops**

This crate provides comprehensive compliance, authentication, authorization, and governance features for the LLM Cost Ops platform.

## Features

### GDPR Compliance
- **Data Subject Access Requests (DSAR)** - Retrieve all data for a specific user
- **Right to Erasure** - Delete or anonymize user data with verification
- **Consent Management** - Track and manage user consent preferences
- **Breach Notifications** - Automated 72-hour breach notification workflow
- **Data Portability** - Export user data in standard formats (JSON, CSV, XML)

### Policy Management
- **Retention Policies** - Automated data lifecycle management
- **Access Policies** - Fine-grained data access controls
- **Data Classification** - Classify data by sensitivity level
- **Policy Versioning** - Track policy changes over time
- **Compliance Checks** - Automated policy compliance validation

### Authentication & Authorization
- **JWT Authentication** - Secure token-based authentication with refresh tokens
- **API Key Management** - Secure key generation, rotation, and revocation
- **Role-Based Access Control (RBAC)** - Granular permission system
- **Scoped Permissions** - Resource-level access control
- **Multi-factor Authentication** - Enhanced security for sensitive operations

### Audit Logging
- **Comprehensive Audit Trail** - Track all system actions
- **Tamper Detection** - Cryptographic verification of audit logs
- **Query Interface** - Search and filter audit events
- **Retention Management** - Automated audit log archival
- **Real-time Monitoring** - Live audit event streaming

### Dead Letter Queue (DLQ)
- **Failed Request Handling** - Capture and retry failed operations
- **Retry Policies** - Configurable backoff and retry strategies
- **Manual Processing** - Human-in-the-loop for complex failures
- **Metrics & Monitoring** - DLQ size and processing metrics

### Compliance Reports
- **SOC2 Evidence Collection** - Automated compliance evidence gathering
- **Audit Log Summaries** - Comprehensive audit activity reports
- **Access Control Reports** - User and permission audits
- **Retention Compliance** - Data retention policy adherence
- **Security Incident Reports** - Breach and incident documentation

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-cost-ops-compliance = "0.1"
llm-cost-ops = "0.1"
tokio = { version = "1", features = ["full"] }
```

### JWT Authentication

```rust
use llm_cost_ops_compliance::{JwtManager, JwtClaims};
use chrono::{Utc, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create JWT manager
    let jwt_manager = JwtManager::new("your-secret-key".to_string());

    // Create claims
    let claims = JwtClaims {
        sub: "user-123".to_string(),
        org: "org-456".to_string(),
        roles: vec!["admin".to_string()],
        exp: (Utc::now() + Duration::hours(1)).timestamp() as usize,
        iat: Utc::now().timestamp() as usize,
    };

    // Generate token pair
    let tokens = jwt_manager.generate_token_pair(&claims)?;

    println!("Access token: {}", tokens.access_token);
    println!("Refresh token: {}", tokens.refresh_token);

    // Verify token
    let verified_claims = jwt_manager.verify_token(&tokens.access_token)?;
    println!("User: {}", verified_claims.sub);

    Ok(())
}
```

### Role-Based Access Control (RBAC)

```rust
use llm_cost_ops_compliance::{RbacManager, Role, RoleType, Permission, Action, Resource};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rbac = RbacManager::new();

    // Define a role
    let admin_role = Role {
        name: "admin".to_string(),
        role_type: RoleType::Admin,
        permissions: vec![
            Permission::new(Resource::Cost, Action::Read),
            Permission::new(Resource::Cost, Action::Write),
            Permission::new(Resource::Usage, Action::Read),
        ],
        description: Some("Administrator role".to_string()),
    };

    // Assign role to user
    rbac.assign_role("user-123", "org-456", admin_role.clone()).await?;

    // Check permission
    let has_access = rbac.check_permission(
        "user-123",
        "org-456",
        &Permission::new(Resource::Cost, Action::Read),
    ).await?;

    println!("User has access: {}", has_access);

    Ok(())
}
```

### Audit Logging

```rust
use llm_cost_ops_compliance::{AuditLogger, AuditEvent, AuditEventType, AuditSeverity};
use chrono::Utc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let audit_logger = AuditLogger::new(/* store */);

    // Log an event
    let event = AuditEvent {
        id: uuid::Uuid::new_v4(),
        timestamp: Utc::now(),
        event_type: AuditEventType::Authentication,
        user_id: Some("user-123".to_string()),
        organization_id: "org-456".to_string(),
        resource: Some("api-endpoint".to_string()),
        action: "login".to_string(),
        outcome: "success".to_string(),
        severity: AuditSeverity::Info,
        ip_address: Some("192.168.1.1".to_string()),
        user_agent: Some("Mozilla/5.0".to_string()),
        details: serde_json::json!({
            "method": "password",
            "mfa_enabled": true
        }),
        correlation_id: None,
    };

    audit_logger.log(event).await?;

    // Query events
    let events = audit_logger.query()
        .event_type(AuditEventType::Authentication)
        .organization_id("org-456")
        .execute()
        .await?;

    println!("Found {} audit events", events.len());

    Ok(())
}
```

### GDPR Compliance

```rust
use llm_cost_ops_compliance::compliance::{
    GdprCompliance, DataSubjectRequest, ConsentRecord, ConsentType
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gdpr = GdprCompliance::new(/* dependencies */);

    // Handle data subject access request
    let data = gdpr.handle_access_request("user-123").await?;
    println!("User data: {:?}", data);

    // Record consent
    let consent = ConsentRecord {
        user_id: "user-123".to_string(),
        consent_type: ConsentType::Marketing,
        granted: true,
        timestamp: Utc::now(),
        version: "1.0".to_string(),
    };
    gdpr.record_consent(consent).await?;

    // Delete user data (right to erasure)
    gdpr.delete_user_data("user-123").await?;

    Ok(())
}
```

### Dead Letter Queue

```rust
use llm_cost_ops_compliance::{DlqProcessor, DlqConfig, DlqItemHandler};
use async_trait::async_trait;

struct MyHandler;

#[async_trait]
impl DlqItemHandler for MyHandler {
    async fn handle(&self, data: &[u8]) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Process the failed item
        println!("Processing DLQ item: {} bytes", data.len());
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = DlqConfig {
        max_retries: 3,
        retry_delay_ms: 1000,
        exponential_backoff: true,
        max_backoff_ms: 60000,
    };

    let processor = DlqProcessor::new(config, /* store */);
    let handler = MyHandler;

    // Process DLQ items
    processor.process_with_handler(&handler).await?;

    Ok(())
}
```

## Architecture

The compliance module integrates with the core LLM Cost Ops platform to provide:

- **Authentication Layer** - JWT and API key authentication
- **Authorization Layer** - RBAC with fine-grained permissions
- **Audit Layer** - Comprehensive audit trail for all operations
- **Compliance Layer** - GDPR, SOC2, and policy management
- **Error Handling** - DLQ for failed operations

## Security

- **Secure Token Storage** - API keys hashed with bcrypt
- **JWT Best Practices** - Short-lived access tokens with refresh tokens
- **Audit Trail Integrity** - Cryptographic verification of logs
- **Rate Limiting** - Prevent brute force attacks
- **Input Validation** - Comprehensive validation of all inputs

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Links

- **Documentation**: [https://docs.rs/llm-cost-ops-compliance](https://docs.rs/llm-cost-ops-compliance)
- **Core Library**: [https://crates.io/crates/llm-cost-ops](https://crates.io/crates/llm-cost-ops)
- **Repository**: [https://github.com/globalbusinessadvisors/llm-cost-ops](https://github.com/globalbusinessadvisors/llm-cost-ops)
