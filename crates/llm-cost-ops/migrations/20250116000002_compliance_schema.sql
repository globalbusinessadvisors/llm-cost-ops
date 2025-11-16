-- Compliance Schema Migration
-- Supports GDPR, SOC 2, HIPAA, ISO 27001, PCI DSS compliance

-- ============================================================================
-- AUDIT EVENTS (Enhanced)
-- ============================================================================

CREATE TABLE IF NOT EXISTS audit_events (
    id TEXT PRIMARY KEY NOT NULL,
    event_id TEXT UNIQUE NOT NULL,
    event_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    status TEXT NOT NULL,
    timestamp TEXT NOT NULL,

    -- User context
    user_id TEXT,
    user_email TEXT,
    organization_id TEXT,

    -- Resource context
    resource_type TEXT,
    resource_id TEXT,
    action TEXT,

    -- Request context
    ip_address TEXT,
    user_agent TEXT,
    request_id TEXT,
    session_id TEXT,

    -- Event details
    description TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    error TEXT,

    -- Change tracking
    changes_before TEXT,
    changes_after TEXT,

    -- Compliance fields
    compliance_category TEXT,
    retention_period_days INTEGER,
    is_personal_data INTEGER DEFAULT 0,

    -- Integrity (hash chain)
    previous_event_hash TEXT,
    event_hash TEXT NOT NULL,

    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_events(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_events(user_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_org ON audit_events(organization_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_event_type ON audit_events(event_type, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_resource ON audit_events(resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_compliance ON audit_events(compliance_category);
CREATE INDEX IF NOT EXISTS idx_audit_event_hash ON audit_events(event_hash);

-- ============================================================================
-- DATA SUBJECT REQUESTS (GDPR)
-- ============================================================================

CREATE TABLE IF NOT EXISTS data_subject_requests (
    id TEXT PRIMARY KEY NOT NULL,
    request_type TEXT NOT NULL,
    subject_id TEXT NOT NULL,
    subject_email TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    requested_at TEXT NOT NULL,
    verified_at TEXT,
    completed_at TEXT,
    status TEXT NOT NULL,
    verification_token TEXT,
    data_export_url TEXT,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_dsr_subject ON data_subject_requests(subject_id, organization_id);
CREATE INDEX IF NOT EXISTS idx_dsr_status ON data_subject_requests(status, requested_at DESC);
CREATE INDEX IF NOT EXISTS idx_dsr_org ON data_subject_requests(organization_id, requested_at DESC);

-- ============================================================================
-- CONSENT RECORDS (GDPR Art. 7)
-- ============================================================================

CREATE TABLE IF NOT EXISTS consent_records (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    consent_type TEXT NOT NULL,
    purpose TEXT NOT NULL,
    granted INTEGER NOT NULL,
    granted_at TEXT,
    revoked_at TEXT,
    expires_at TEXT,
    consent_method TEXT,
    ip_address TEXT,
    user_agent TEXT,
    consent_text TEXT NOT NULL,
    consent_version TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    CHECK (
        (granted = 1 AND granted_at IS NOT NULL) OR
        (granted = 0 AND revoked_at IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_consent_user ON consent_records(user_id, organization_id);
CREATE INDEX IF NOT EXISTS idx_consent_type ON consent_records(consent_type, granted);
CREATE INDEX IF NOT EXISTS idx_consent_expires ON consent_records(expires_at);

-- ============================================================================
-- DATA CATALOG (GDPR Art. 30)
-- ============================================================================

CREATE TABLE IF NOT EXISTS data_catalog (
    id TEXT PRIMARY KEY NOT NULL,
    table_name TEXT NOT NULL,
    column_name TEXT NOT NULL,
    data_category TEXT NOT NULL,
    contains_pii INTEGER NOT NULL DEFAULT 0,
    contains_sensitive INTEGER NOT NULL DEFAULT 0,
    legal_basis TEXT,
    processing_purpose TEXT,
    retention_period_days INTEGER,
    encryption_required INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE (table_name, column_name)
);

-- Populate data catalog
INSERT OR IGNORE INTO data_catalog VALUES
    (lower(hex(randomblob(16))), 'usage_records', 'user_id', 'identifier', 1, 0,
     'contract', 'Cost tracking and billing', 2555, 1, datetime('now'), datetime('now')),
    (lower(hex(randomblob(16))), 'usage_records', 'organization_id', 'identifier', 1, 0,
     'contract', 'Cost tracking and billing', 2555, 1, datetime('now'), datetime('now')),
    (lower(hex(randomblob(16))), 'audit_events', 'ip_address', 'network_identifier', 1, 0,
     'legitimate_interest', 'Security and audit logging', 2555, 1, datetime('now'), datetime('now')),
    (lower(hex(randomblob(16))), 'audit_events', 'user_email', 'contact', 1, 0,
     'legitimate_interest', 'Security and audit logging', 2555, 1, datetime('now'), datetime('now'));

-- ============================================================================
-- PROCESSING ACTIVITIES (GDPR Art. 30)
-- ============================================================================

CREATE TABLE IF NOT EXISTS processing_activities (
    id TEXT PRIMARY KEY NOT NULL,
    activity_name TEXT NOT NULL,
    purpose TEXT NOT NULL,
    legal_basis TEXT NOT NULL,
    data_categories TEXT NOT NULL,
    data_subjects TEXT NOT NULL,
    recipients TEXT,
    international_transfers INTEGER DEFAULT 0,
    safeguards TEXT,
    retention_period TEXT NOT NULL,
    technical_measures TEXT,
    organizational_measures TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert sample processing activities
INSERT OR IGNORE INTO processing_activities VALUES
    (lower(hex(randomblob(16))), 'Usage Tracking', 'Track LLM usage for billing',
     'contract', '["usage_data", "cost_data"]', '["customers", "end_users"]',
     '["billing_system"]', 0, NULL, '7 years',
     'Encryption at rest and in transit, access controls, audit logging',
     'RBAC, employee training, incident response plan',
     datetime('now'), datetime('now'));

-- ============================================================================
-- COMPLIANCE POLICIES
-- ============================================================================

CREATE TABLE IF NOT EXISTS compliance_policies (
    id TEXT PRIMARY KEY NOT NULL,
    policy_id TEXT UNIQUE NOT NULL,
    policy_name TEXT NOT NULL,
    policy_type TEXT NOT NULL,
    description TEXT NOT NULL,
    standard TEXT NOT NULL,
    policy_document_url TEXT,
    effective_date TEXT NOT NULL,
    review_date TEXT NOT NULL,
    owner TEXT NOT NULL,
    status TEXT NOT NULL,
    version TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_policy_type ON compliance_policies(policy_type, status);
CREATE INDEX IF NOT EXISTS idx_policy_standard ON compliance_policies(standard);
CREATE INDEX IF NOT EXISTS idx_policy_effective ON compliance_policies(effective_date);

-- ============================================================================
-- POLICY RULES
-- ============================================================================

CREATE TABLE IF NOT EXISTS policy_rules (
    id TEXT PRIMARY KEY NOT NULL,
    policy_id TEXT NOT NULL,
    rule_name TEXT NOT NULL,
    rule_type TEXT NOT NULL,
    condition TEXT NOT NULL,
    action TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    severity TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (policy_id) REFERENCES compliance_policies(policy_id)
);

CREATE INDEX IF NOT EXISTS idx_policy_rules_policy ON policy_rules(policy_id, enabled);

-- ============================================================================
-- POLICY VIOLATIONS
-- ============================================================================

CREATE TABLE IF NOT EXISTS policy_violations (
    id TEXT PRIMARY KEY NOT NULL,
    violation_id TEXT UNIQUE NOT NULL,
    policy_id TEXT NOT NULL,
    rule_id TEXT,
    resource_type TEXT,
    resource_id TEXT,
    user_id TEXT,
    organization_id TEXT,
    violation_time TEXT NOT NULL,
    description TEXT NOT NULL,
    severity TEXT NOT NULL,
    status TEXT NOT NULL,
    resolved_at TEXT,
    resolved_by TEXT,
    resolution_notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (policy_id) REFERENCES compliance_policies(policy_id),
    FOREIGN KEY (rule_id) REFERENCES policy_rules(id)
);

CREATE INDEX IF NOT EXISTS idx_violation_policy ON policy_violations(policy_id, violation_time DESC);
CREATE INDEX IF NOT EXISTS idx_violation_status ON policy_violations(status, severity);
CREATE INDEX IF NOT EXISTS idx_violation_org ON policy_violations(organization_id, violation_time DESC);

-- ============================================================================
-- SOC 2 CONTROLS
-- ============================================================================

CREATE TABLE IF NOT EXISTS soc2_controls (
    id TEXT PRIMARY KEY NOT NULL,
    control_id TEXT UNIQUE NOT NULL,
    control_name TEXT NOT NULL,
    trust_service_category TEXT NOT NULL,
    description TEXT NOT NULL,
    implementation_status TEXT NOT NULL,
    automated INTEGER NOT NULL DEFAULT 0,
    evidence_type TEXT,
    test_procedure TEXT,
    test_frequency TEXT,
    last_tested_at TEXT,
    next_test_at TEXT,
    responsible_team TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert key SOC 2 controls
INSERT OR IGNORE INTO soc2_controls VALUES
    (lower(hex(randomblob(16))), 'CC6.1', 'Logical Access Controls', 'Security',
     'The entity implements logical access security software, infrastructure, and architectures',
     'implemented', 1, 'audit_logs', 'Review RBAC enforcement in audit logs', 'quarterly',
     datetime('now'), datetime('now', '+3 months'), 'Engineering', datetime('now'), datetime('now')),

    (lower(hex(randomblob(16))), 'CC6.6', 'Encryption at Rest', 'Security',
     'The entity encrypts sensitive data at rest',
     'implemented', 1, 'configuration_review', 'Verify database encryption settings', 'quarterly',
     datetime('now'), datetime('now', '+3 months'), 'Infrastructure', datetime('now'), datetime('now')),

    (lower(hex(randomblob(16))), 'CC7.2', 'System Monitoring', 'Security',
     'The entity monitors system components and operations',
     'implemented', 1, 'metrics_dashboard', 'Review monitoring coverage', 'monthly',
     datetime('now'), datetime('now', '+1 month'), 'SRE', datetime('now'), datetime('now')),

    (lower(hex(randomblob(16))), 'CC7.3', 'Security Event Detection', 'Security',
     'The entity evaluates security events to identify anomalies',
     'implemented', 1, 'audit_logs', 'Review security event detection mechanisms', 'monthly',
     datetime('now'), datetime('now', '+1 month'), 'Security', datetime('now'), datetime('now'));

-- ============================================================================
-- CONTROL EVIDENCE
-- ============================================================================

CREATE TABLE IF NOT EXISTS control_evidence (
    id TEXT PRIMARY KEY NOT NULL,
    control_id TEXT NOT NULL,
    evidence_date TEXT NOT NULL,
    evidence_type TEXT NOT NULL,
    evidence_description TEXT,
    evidence_location TEXT,
    tested_by TEXT,
    test_result TEXT,
    findings TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (control_id) REFERENCES soc2_controls(control_id)
);

CREATE INDEX IF NOT EXISTS idx_evidence_control ON control_evidence(control_id, evidence_date DESC);

-- ============================================================================
-- SECURITY INCIDENTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS security_incidents (
    id TEXT PRIMARY KEY NOT NULL,
    incident_id TEXT UNIQUE NOT NULL,
    incident_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    status TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    detected_by TEXT,
    affected_systems TEXT,
    affected_users TEXT,
    data_breach INTEGER NOT NULL DEFAULT 0,
    personal_data_affected INTEGER NOT NULL DEFAULT 0,
    sensitive_data_affected INTEGER NOT NULL DEFAULT 0,
    estimated_affected_count INTEGER,
    description TEXT NOT NULL,
    initial_response TEXT,
    containment_actions TEXT,
    eradication_actions TEXT,
    recovery_actions TEXT,
    lessons_learned TEXT,
    reported_to_authorities INTEGER DEFAULT 0,
    reported_at TEXT,
    authority_reference TEXT,
    notified_users INTEGER DEFAULT 0,
    notification_sent_at TEXT,
    assigned_to TEXT,
    resolved_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_incident_type ON security_incidents(incident_type, detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_incident_severity ON security_incidents(severity, status);
CREATE INDEX IF NOT EXISTS idx_incident_breach ON security_incidents(data_breach, detected_at DESC);

-- ============================================================================
-- BREACH NOTIFICATIONS (GDPR Art. 33-34)
-- ============================================================================

CREATE TABLE IF NOT EXISTS breach_notifications (
    id TEXT PRIMARY KEY NOT NULL,
    incident_id TEXT NOT NULL,
    notification_type TEXT NOT NULL,
    recipient_type TEXT NOT NULL,
    recipient TEXT NOT NULL,
    sent_at TEXT NOT NULL,
    delivery_status TEXT NOT NULL,
    notification_content TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (incident_id) REFERENCES security_incidents(incident_id)
);

CREATE INDEX IF NOT EXISTS idx_breach_notif_incident ON breach_notifications(incident_id, sent_at DESC);

-- ============================================================================
-- CHANGE REQUESTS (SOC 2)
-- ============================================================================

CREATE TABLE IF NOT EXISTS change_requests (
    id TEXT PRIMARY KEY NOT NULL,
    change_id TEXT UNIQUE NOT NULL,
    title TEXT NOT NULL,
    description TEXT NOT NULL,
    change_type TEXT NOT NULL,
    risk_level TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    requested_at TEXT NOT NULL DEFAULT (datetime('now')),
    approved_by TEXT,
    approved_at TEXT,
    implemented_by TEXT,
    implemented_at TEXT,
    verified_by TEXT,
    verified_at TEXT,
    status TEXT NOT NULL,
    rollback_plan TEXT,
    affected_systems TEXT,
    scheduled_start TEXT,
    scheduled_end TEXT,
    actual_start TEXT,
    actual_end TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_change_status ON change_requests(status, requested_at DESC);
CREATE INDEX IF NOT EXISTS idx_change_type ON change_requests(change_type, risk_level);

-- ============================================================================
-- ENCRYPTION KEY METADATA
-- ============================================================================

CREATE TABLE IF NOT EXISTS encryption_keys (
    id TEXT PRIMARY KEY NOT NULL,
    key_id TEXT UNIQUE NOT NULL,
    key_type TEXT NOT NULL,
    algorithm TEXT NOT NULL,
    key_length INTEGER NOT NULL,
    purpose TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    rotated_at TEXT,
    expires_at TEXT,
    rotation_period_days INTEGER NOT NULL DEFAULT 90,
    metadata TEXT DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_encryption_key_status ON encryption_keys(status, expires_at);

-- ============================================================================
-- COMPLIANCE CHECKS
-- ============================================================================

CREATE TABLE IF NOT EXISTS compliance_checks (
    id TEXT PRIMARY KEY NOT NULL,
    check_id TEXT UNIQUE NOT NULL,
    check_name TEXT NOT NULL,
    check_type TEXT NOT NULL,
    standard TEXT NOT NULL,
    description TEXT NOT NULL,
    automated INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    severity TEXT NOT NULL,
    last_run_at TEXT,
    next_run_at TEXT,
    run_frequency TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_check_type ON compliance_checks(check_type, enabled);
CREATE INDEX IF NOT EXISTS idx_check_schedule ON compliance_checks(next_run_at);

-- ============================================================================
-- COMPLIANCE CHECK RESULTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS compliance_check_results (
    id TEXT PRIMARY KEY NOT NULL,
    check_id TEXT NOT NULL,
    run_at TEXT NOT NULL,
    passed INTEGER NOT NULL,
    findings TEXT,
    severity TEXT NOT NULL,
    remediation TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (check_id) REFERENCES compliance_checks(check_id)
);

CREATE INDEX IF NOT EXISTS idx_check_results ON compliance_check_results(check_id, run_at DESC);
CREATE INDEX IF NOT EXISTS idx_check_passed ON compliance_check_results(passed, severity);

-- ============================================================================
-- RETENTION POLICIES
-- ============================================================================

CREATE TABLE IF NOT EXISTS retention_policies (
    id TEXT PRIMARY KEY NOT NULL,
    policy_name TEXT NOT NULL,
    data_type TEXT NOT NULL,
    data_classification TEXT NOT NULL,
    retention_days INTEGER NOT NULL,
    legal_basis TEXT NOT NULL,
    auto_delete INTEGER NOT NULL DEFAULT 0,
    archive_after_days INTEGER,
    active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Insert default retention policies
INSERT OR IGNORE INTO retention_policies VALUES
    (lower(hex(randomblob(16))), 'HIPAA Usage Records', 'usage_records', 'phi', 2555,
     'HIPAA 45 CFR 164.530(j)', 0, 90, 1, datetime('now'), datetime('now')),

    (lower(hex(randomblob(16))), 'SOC 2 Audit Logs', 'audit_events', 'confidential', 2190,
     'SOC 2 Type II', 0, 90, 1, datetime('now'), datetime('now')),

    (lower(hex(randomblob(16))), 'Security Incidents', 'security_incidents', 'confidential', 3650,
     'ISO 27001 A.16.1.7', 0, 365, 1, datetime('now'), datetime('now'));

CREATE INDEX IF NOT EXISTS idx_retention_data_type ON retention_policies(data_type, active);

-- ============================================================================
-- ANONYMIZATION LOG
-- ============================================================================

CREATE TABLE IF NOT EXISTS anonymization_log (
    id TEXT PRIMARY KEY NOT NULL,
    table_name TEXT NOT NULL,
    record_id TEXT NOT NULL,
    anonymization_method TEXT NOT NULL,
    anonymized_at TEXT NOT NULL DEFAULT (datetime('now')),
    anonymized_by TEXT NOT NULL,
    reason TEXT NOT NULL,
    fields_anonymized TEXT NOT NULL,
    metadata TEXT DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_anon_table ON anonymization_log(table_name, anonymized_at DESC);
CREATE INDEX IF NOT EXISTS idx_anon_record ON anonymization_log(table_name, record_id);
