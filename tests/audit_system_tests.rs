//! Comprehensive tests for the audit logging system
//!
//! These tests demonstrate the usage of the audit logging system and verify
//! its functionality.

#[cfg(test)]
mod audit_tests {
    use llm_cost_ops::compliance::{
        AuditLog, ComplianceAuditEventType, AuditOutcome, Actor, ActorType,
        ActionType, ResourceInfo, AuditChanges, HttpRequestInfo,
        AuditRepository, PostgresAuditRepository, AuditFilter,
        AuditExportFormat, AuditRetentionPolicy,
    };
    use chrono::{Duration, Utc};
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;

    /// Test basic audit log creation
    #[test]
    fn test_audit_log_creation() {
        let actor = Actor::user(
            "user-123".to_string(),
            Some("john.doe@example.com".to_string()),
        );

        let audit_log = AuditLog::new(
            ComplianceAuditEventType::AuthLogin,
            actor,
            ActionType::Login,
            AuditOutcome::Success,
        );

        assert_eq!(audit_log.event_type, ComplianceAuditEventType::AuthLogin);
        assert_eq!(audit_log.action, ActionType::Login);
        assert_eq!(audit_log.outcome, AuditOutcome::Success);
        assert_eq!(audit_log.actor.id, "user-123");
        assert_eq!(audit_log.actor.actor_type, ActorType::User);
    }

    /// Test audit log with full context
    #[test]
    fn test_audit_log_with_full_context() {
        let actor = Actor::user(
            "user-456".to_string(),
            Some("jane.smith@example.com".to_string()),
        );

        let resource = ResourceInfo::new(
            "cost_record".to_string(),
            "rec-789".to_string(),
        )
        .with_name("Q4 2024 Cost Report".to_string());

        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));

        let http_request = HttpRequestInfo::new(
            "GET".to_string(),
            "/api/v1/costs/rec-789".to_string(),
            Some(200),
        );

        let audit_log = AuditLog::new(
            ComplianceAuditEventType::DataRead,
            actor,
            ActionType::Read,
            AuditOutcome::Success,
        )
        .with_resource(resource)
        .with_ip_address(ip)
        .with_user_agent("Mozilla/5.0".to_string())
        .with_correlation_id("corr-12345".to_string())
        .with_request_id("req-67890".to_string())
        .with_organization_id("org-abc".to_string())
        .with_duration(45)
        .with_http_request(http_request)
        .add_security_label("confidential".to_string())
        .add_compliance_tag("SOC2".to_string())
        .add_compliance_tag("GDPR".to_string());

        assert!(audit_log.resource.is_some());
        assert_eq!(audit_log.ip_address, Some(ip));
        assert_eq!(audit_log.correlation_id, Some("corr-12345".to_string()));
        assert_eq!(audit_log.duration_ms, Some(45));
        assert_eq!(audit_log.security_labels.len(), 1);
        assert_eq!(audit_log.compliance_tags.len(), 2);
    }

    /// Test actor types
    #[test]
    fn test_different_actor_types() {
        // User actor
        let user = Actor::user("user-1".to_string(), Some("user@example.com".to_string()));
        assert_eq!(user.actor_type, ActorType::User);

        // Service actor
        let service = Actor::service("svc-1".to_string(), Some("billing-service".to_string()));
        assert_eq!(service.actor_type, ActorType::Service);

        // System actor
        let system = Actor::system("cron-job".to_string());
        assert_eq!(system.actor_type, ActorType::System);
        assert_eq!(system.id, "system");

        // API client actor
        let api_client = Actor::api_client("key-abc123".to_string(), None);
        assert_eq!(api_client.actor_type, ActorType::ApiClient);

        // Anonymous actor
        let anon = Actor::anonymous();
        assert_eq!(anon.actor_type, ActorType::Anonymous);
        assert_eq!(anon.id, "anonymous");
    }

    /// Test audit changes tracking
    #[test]
    fn test_audit_changes() {
        let mut before = HashMap::new();
        before.insert("status".to_string(), serde_json::json!("active"));
        before.insert("budget".to_string(), serde_json::json!(1000.0));
        before.insert("name".to_string(), serde_json::json!("Old Project"));

        let mut after = HashMap::new();
        after.insert("status".to_string(), serde_json::json!("inactive"));
        after.insert("budget".to_string(), serde_json::json!(1500.0));
        after.insert("description".to_string(), serde_json::json!("New description"));

        let changes = AuditChanges::new(before, after);

        // Check that we detected the changes correctly
        assert!(changes.added.contains(&"description".to_string()));
        assert!(changes.removed.contains(&"name".to_string()));
        assert!(changes.modified.contains(&"status".to_string()));
        assert!(changes.modified.contains(&"budget".to_string()));

        assert_eq!(changes.added.len(), 1);
        assert_eq!(changes.removed.len(), 1);
        assert_eq!(changes.modified.len(), 2);
    }

    /// Test resource with parent hierarchy
    #[test]
    fn test_resource_hierarchy() {
        let parent = ResourceInfo::new(
            "organization".to_string(),
            "org-123".to_string(),
        )
        .with_name("ACME Corp".to_string());

        let child = ResourceInfo::new(
            "project".to_string(),
            "proj-456".to_string(),
        )
        .with_name("AI Project".to_string())
        .with_parent(parent);

        assert!(child.parent_resource.is_some());
        let parent_ref = child.parent_resource.unwrap();
        assert_eq!(parent_ref.resource_type, "organization");
        assert_eq!(parent_ref.resource_id, "org-123");
        assert_eq!(parent_ref.resource_name, Some("ACME Corp".to_string()));
    }

    /// Test logging different event types
    #[test]
    fn test_event_types() {
        let actor = Actor::user("user-1".to_string(), None);

        // Authentication events
        let login = AuditLog::new(
            ComplianceAuditEventType::AuthLogin,
            actor.clone(),
            ActionType::Login,
            AuditOutcome::Success,
        );
        assert_eq!(login.event_type, ComplianceAuditEventType::AuthLogin);

        // Data operation events
        let export = AuditLog::new(
            ComplianceAuditEventType::DataExport,
            actor.clone(),
            ActionType::Export,
            AuditOutcome::Success,
        );
        assert_eq!(export.event_type, ComplianceAuditEventType::DataExport);

        // API key events
        let api_key_created = AuditLog::new(
            ComplianceAuditEventType::ApiKeyCreated,
            actor.clone(),
            ActionType::Create,
            AuditOutcome::Success,
        );
        assert_eq!(api_key_created.event_type, ComplianceAuditEventType::ApiKeyCreated);

        // Security events
        let security_incident = AuditLog::new(
            ComplianceAuditEventType::SecurityIncident,
            actor.clone(),
            ActionType::Other,
            AuditOutcome::Failure,
        );
        assert_eq!(security_incident.event_type, ComplianceAuditEventType::SecurityIncident);
    }

    /// Test audit filter building
    #[test]
    fn test_audit_filter() {
        let now = Utc::now();
        let one_week_ago = now - Duration::days(7);

        let filter = AuditFilter {
            event_types: Some(vec![
                ComplianceAuditEventType::AuthLogin,
                ComplianceAuditEventType::AuthLoginFailed,
            ]),
            actor_id: Some("user-123".to_string()),
            organization_id: Some("org-abc".to_string()),
            from_time: Some(one_week_ago),
            to_time: Some(now),
            limit: Some(100),
            offset: Some(0),
            ..Default::default()
        };

        assert_eq!(filter.event_types.as_ref().unwrap().len(), 2);
        assert_eq!(filter.limit, Some(100));
        assert!(filter.from_time.is_some());
    }

    /// Test retention policy
    #[test]
    fn test_retention_policy() {
        // Standard retention policy
        let policy = AuditRetentionPolicy {
            retention_days: 90,
            event_types: None,
            archive_before_delete: true,
            archive_location: Some("s3://audit-logs-archive/".to_string()),
        };

        assert_eq!(policy.retention_days, 90);
        assert!(policy.archive_before_delete);

        // Event-specific retention policy
        let sensitive_policy = AuditRetentionPolicy {
            retention_days: 365,
            event_types: Some(vec![
                ComplianceAuditEventType::SecurityIncident,
                ComplianceAuditEventType::DataExport,
            ]),
            archive_before_delete: true,
            archive_location: Some("s3://sensitive-audit-archive/".to_string()),
        };

        assert_eq!(sensitive_policy.retention_days, 365);
        assert_eq!(sensitive_policy.event_types.as_ref().unwrap().len(), 2);
    }

    /// Test error logging
    #[test]
    fn test_error_audit_log() {
        let actor = Actor::user("user-789".to_string(), None);

        let error_log = AuditLog::new(
            ComplianceAuditEventType::AuthLoginFailed,
            actor,
            ActionType::Login,
            AuditOutcome::Failure,
        )
        .with_error(
            "Invalid credentials".to_string(),
            Some("AUTH_001".to_string()),
        );

        assert_eq!(error_log.outcome, AuditOutcome::Failure);
        assert_eq!(error_log.error_message, Some("Invalid credentials".to_string()));
        assert_eq!(error_log.error_code, Some("AUTH_001".to_string()));
    }

    /// Test denied access logging
    #[test]
    fn test_access_denied_log() {
        let actor = Actor::user("user-999".to_string(), None);

        let denied_log = AuditLog::new(
            ComplianceAuditEventType::AuthzAccessDenied,
            actor,
            ActionType::Read,
            AuditOutcome::Denied,
        )
        .with_resource(ResourceInfo::new(
            "sensitive_data".to_string(),
            "data-123".to_string(),
        ))
        .add_security_label("pii".to_string())
        .add_compliance_tag("GDPR".to_string());

        assert_eq!(denied_log.outcome, AuditOutcome::Denied);
        assert!(denied_log.resource.is_some());
        assert!(denied_log.security_labels.contains(&"pii".to_string()));
    }

    /// Test metadata addition
    #[test]
    fn test_custom_metadata() {
        let actor = Actor::service("billing-svc".to_string(), None);

        let log = AuditLog::new(
            ComplianceAuditEventType::CostCalculated,
            actor,
            ActionType::Create,
            AuditOutcome::Success,
        )
        .add_metadata("total_cost".to_string(), serde_json::json!(1234.56))
        .add_metadata("currency".to_string(), serde_json::json!("USD"))
        .add_metadata("provider".to_string(), serde_json::json!("openai"));

        assert_eq!(log.metadata.custom.len(), 3);
        assert!(log.metadata.custom.contains_key("total_cost"));
        assert!(log.metadata.custom.contains_key("currency"));
        assert!(log.metadata.custom.contains_key("provider"));
    }
}

// Integration tests would go here if we had a test database
#[cfg(test)]
mod integration_tests {
    use super::*;

    // Note: These tests would require a test database setup
    // Skipping for now as we can't compile/run in this environment

    #[test]
    #[ignore]
    fn test_postgres_repository_store_and_retrieve() {
        // Would test:
        // - Creating PostgresAuditRepository
        // - Storing audit logs
        // - Retrieving by ID
        // - Querying with filters
    }

    #[test]
    #[ignore]
    fn test_batch_insert_performance() {
        // Would test:
        // - Batch inserting 1000+ audit logs
        // - Measuring performance
        // - Verifying data integrity
    }

    #[test]
    #[ignore]
    fn test_retention_policy_execution() {
        // Would test:
        // - Applying retention policy
        // - Verifying old logs are deleted
        // - Ensuring recent logs are kept
    }

    #[test]
    #[ignore]
    fn test_export_formats() {
        // Would test:
        // - Exporting to JSON
        // - Exporting to CSV
        // - Exporting to NDJSON
        // - Verifying format correctness
    }
}
