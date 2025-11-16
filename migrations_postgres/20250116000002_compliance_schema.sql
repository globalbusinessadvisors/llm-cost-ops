-- Compliance Schema Migration (PostgreSQL)
-- Supports GDPR, SOC 2, HIPAA, ISO 27001, PCI DSS compliance

-- ============================================================================
-- AUDIT EVENTS (Enhanced)
-- ============================================================================

CREATE TABLE IF NOT EXISTS audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id VARCHAR(255) UNIQUE NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,

    -- User context
    user_id VARCHAR(255),
    user_email VARCHAR(255),
    organization_id VARCHAR(255),

    -- Resource context
    resource_type VARCHAR(100),
    resource_id VARCHAR(255),
    action VARCHAR(100),

    -- Request context
    ip_address INET,
    user_agent TEXT,
    request_id VARCHAR(255),
    session_id VARCHAR(255),

    -- Event details
    description TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    error TEXT,

    -- Change tracking
    changes_before JSONB,
    changes_after JSONB,

    -- Compliance fields
    compliance_category VARCHAR(50),
    retention_period_days INTEGER,
    is_personal_data BOOLEAN DEFAULT FALSE,

    -- Integrity (hash chain)
    previous_event_hash VARCHAR(64),
    event_hash VARCHAR(64) NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create partitions for audit events (by month)
CREATE TABLE IF NOT EXISTS audit_events_default PARTITION OF audit_events DEFAULT;

CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_events (timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_events (user_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_org ON audit_events (organization_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_event_type ON audit_events (event_type, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_resource ON audit_events (resource_type, resource_id);
CREATE INDEX IF NOT EXISTS idx_audit_compliance ON audit_events (compliance_category);
CREATE INDEX IF NOT EXISTS idx_audit_metadata ON audit_events USING GIN (metadata);
CREATE INDEX IF NOT EXISTS idx_audit_event_hash ON audit_events (event_hash);

-- Hash chain verification function
CREATE OR REPLACE FUNCTION verify_audit_chain(
    start_time TIMESTAMPTZ,
    end_time TIMESTAMPTZ
) RETURNS TABLE (
    valid BOOLEAN,
    total_events BIGINT,
    invalid_events BIGINT
) AS $$
DECLARE
    prev_hash VARCHAR(64);
    curr_event RECORD;
    invalid_count BIGINT := 0;
    total_count BIGINT := 0;
BEGIN
    FOR curr_event IN
        SELECT * FROM audit_events
        WHERE timestamp BETWEEN start_time AND end_time
        ORDER BY timestamp ASC
    LOOP
        total_count := total_count + 1;

        IF prev_hash IS NOT NULL AND curr_event.previous_event_hash != prev_hash THEN
            invalid_count := invalid_count + 1;
        END IF;

        prev_hash := curr_event.event_hash;
    END LOOP;

    RETURN QUERY SELECT (invalid_count = 0), total_count, invalid_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- DATA SUBJECT REQUESTS (GDPR)
-- ============================================================================

CREATE TABLE IF NOT EXISTS data_subject_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_type VARCHAR(50) NOT NULL,
    subject_id VARCHAR(255) NOT NULL,
    subject_email VARCHAR(255) NOT NULL,
    organization_id VARCHAR(255) NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL,
    verification_token VARCHAR(255),
    data_export_url TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_dsr_subject ON data_subject_requests(subject_id, organization_id);
CREATE INDEX IF NOT EXISTS idx_dsr_status ON data_subject_requests(status, requested_at DESC);
CREATE INDEX IF NOT EXISTS idx_dsr_org ON data_subject_requests(organization_id, requested_at DESC);

-- ============================================================================
-- CONSENT RECORDS (GDPR Art. 7)
-- ============================================================================

CREATE TABLE IF NOT EXISTS consent_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id VARCHAR(255) NOT NULL,
    organization_id VARCHAR(255) NOT NULL,
    consent_type VARCHAR(100) NOT NULL,
    purpose TEXT NOT NULL,
    granted BOOLEAN NOT NULL,
    granted_at TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    consent_method VARCHAR(50),
    ip_address INET,
    user_agent TEXT,
    consent_text TEXT NOT NULL,
    consent_version VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT consent_active CHECK (
        (granted = TRUE AND granted_at IS NOT NULL) OR
        (granted = FALSE AND revoked_at IS NOT NULL)
    )
);

CREATE INDEX IF NOT EXISTS idx_consent_user ON consent_records (user_id, organization_id);
CREATE INDEX IF NOT EXISTS idx_consent_type ON consent_records (consent_type, granted);
CREATE INDEX IF NOT EXISTS idx_consent_expires ON consent_records (expires_at) WHERE granted = TRUE;

-- ============================================================================
-- DATA CATALOG (GDPR Art. 30)
-- ============================================================================

CREATE TABLE IF NOT EXISTS data_catalog (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR(255) NOT NULL,
    column_name VARCHAR(255) NOT NULL,
    data_category VARCHAR(100) NOT NULL,
    contains_pii BOOLEAN NOT NULL DEFAULT FALSE,
    contains_sensitive BOOLEAN NOT NULL DEFAULT FALSE,
    legal_basis VARCHAR(100),
    processing_purpose TEXT,
    retention_period_days INTEGER,
    encryption_required BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE (table_name, column_name)
);

-- Populate data catalog
INSERT INTO data_catalog (table_name, column_name, data_category, contains_pii, legal_basis,
                          processing_purpose, retention_period_days, encryption_required)
VALUES
    ('usage_records', 'user_id', 'identifier', TRUE, 'contract',
     'Cost tracking and billing', 2555, TRUE),
    ('usage_records', 'organization_id', 'identifier', TRUE, 'contract',
     'Cost tracking and billing', 2555, TRUE),
    ('audit_events', 'ip_address', 'network_identifier', TRUE, 'legitimate_interest',
     'Security and audit logging', 2555, TRUE),
    ('audit_events', 'user_email', 'contact', TRUE, 'legitimate_interest',
     'Security and audit logging', 2555, TRUE)
ON CONFLICT (table_name, column_name) DO NOTHING;

-- ============================================================================
-- PROCESSING ACTIVITIES (GDPR Art. 30)
-- ============================================================================

CREATE TABLE IF NOT EXISTS processing_activities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    activity_name VARCHAR(255) NOT NULL,
    purpose TEXT NOT NULL,
    legal_basis VARCHAR(100) NOT NULL,
    data_categories TEXT[] NOT NULL,
    data_subjects TEXT[] NOT NULL,
    recipients TEXT[],
    international_transfers BOOLEAN DEFAULT FALSE,
    safeguards TEXT,
    retention_period TEXT NOT NULL,
    technical_measures TEXT,
    organizational_measures TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert sample processing activities
INSERT INTO processing_activities (activity_name, purpose, legal_basis, data_categories,
                                   data_subjects, recipients, international_transfers,
                                   retention_period, technical_measures, organizational_measures)
VALUES
    ('Usage Tracking', 'Track LLM usage for billing', 'contract',
     ARRAY['usage_data', 'cost_data'], ARRAY['customers', 'end_users'],
     ARRAY['billing_system'], FALSE, '7 years',
     'Encryption at rest and in transit, access controls, audit logging',
     'RBAC, employee training, incident response plan')
ON CONFLICT DO NOTHING;

-- ============================================================================
-- COMPLIANCE POLICIES
-- ============================================================================

CREATE TABLE IF NOT EXISTS compliance_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id VARCHAR(50) UNIQUE NOT NULL,
    policy_name VARCHAR(255) NOT NULL,
    policy_type VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    standard VARCHAR(50) NOT NULL,
    policy_document_url TEXT,
    effective_date DATE NOT NULL,
    review_date DATE NOT NULL,
    owner VARCHAR(255) NOT NULL,
    status VARCHAR(50) NOT NULL,
    version VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policy_type ON compliance_policies(policy_type, status);
CREATE INDEX IF NOT EXISTS idx_policy_standard ON compliance_policies(standard);
CREATE INDEX IF NOT EXISTS idx_policy_effective ON compliance_policies(effective_date);

-- ============================================================================
-- POLICY RULES
-- ============================================================================

CREATE TABLE IF NOT EXISTS policy_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_id VARCHAR(50) NOT NULL REFERENCES compliance_policies(policy_id),
    rule_name VARCHAR(255) NOT NULL,
    rule_type VARCHAR(50) NOT NULL,
    condition JSONB NOT NULL,
    action JSONB NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    severity VARCHAR(20) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_policy_rules_policy ON policy_rules(policy_id, enabled);
CREATE INDEX IF NOT EXISTS idx_policy_rules_condition ON policy_rules USING GIN (condition);

-- ============================================================================
-- POLICY VIOLATIONS
-- ============================================================================

CREATE TABLE IF NOT EXISTS policy_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    violation_id VARCHAR(50) UNIQUE NOT NULL,
    policy_id VARCHAR(50) NOT NULL REFERENCES compliance_policies(policy_id),
    rule_id UUID REFERENCES policy_rules(id),
    resource_type VARCHAR(100),
    resource_id VARCHAR(255),
    user_id VARCHAR(255),
    organization_id VARCHAR(255),
    violation_time TIMESTAMPTZ NOT NULL,
    description TEXT NOT NULL,
    severity VARCHAR(20) NOT NULL,
    status VARCHAR(50) NOT NULL,
    resolved_at TIMESTAMPTZ,
    resolved_by VARCHAR(255),
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_violation_policy ON policy_violations(policy_id, violation_time DESC);
CREATE INDEX IF NOT EXISTS idx_violation_status ON policy_violations(status, severity);
CREATE INDEX IF NOT EXISTS idx_violation_org ON policy_violations(organization_id, violation_time DESC);

-- ============================================================================
-- SOC 2 CONTROLS
-- ============================================================================

CREATE TABLE IF NOT EXISTS soc2_controls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    control_id VARCHAR(50) UNIQUE NOT NULL,
    control_name VARCHAR(255) NOT NULL,
    trust_service_category VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    implementation_status VARCHAR(50) NOT NULL,
    automated BOOLEAN NOT NULL DEFAULT FALSE,
    evidence_type VARCHAR(100),
    test_procedure TEXT,
    test_frequency VARCHAR(50),
    last_tested_at TIMESTAMPTZ,
    next_test_at TIMESTAMPTZ,
    responsible_team VARCHAR(100),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert key SOC 2 controls
INSERT INTO soc2_controls (control_id, control_name, trust_service_category, description,
                           implementation_status, automated, evidence_type, test_procedure,
                           test_frequency, last_tested_at, next_test_at, responsible_team)
VALUES
    ('CC6.1', 'Logical Access Controls', 'Security',
     'The entity implements logical access security software, infrastructure, and architectures',
     'implemented', TRUE, 'audit_logs', 'Review RBAC enforcement in audit logs', 'quarterly',
     NOW(), NOW() + INTERVAL '3 months', 'Engineering'),

    ('CC6.6', 'Encryption at Rest', 'Security',
     'The entity encrypts sensitive data at rest',
     'implemented', TRUE, 'configuration_review', 'Verify database encryption settings', 'quarterly',
     NOW(), NOW() + INTERVAL '3 months', 'Infrastructure'),

    ('CC7.2', 'System Monitoring', 'Security',
     'The entity monitors system components and operations',
     'implemented', TRUE, 'metrics_dashboard', 'Review monitoring coverage', 'monthly',
     NOW(), NOW() + INTERVAL '1 month', 'SRE'),

    ('CC7.3', 'Security Event Detection', 'Security',
     'The entity evaluates security events to identify anomalies',
     'implemented', TRUE, 'audit_logs', 'Review security event detection mechanisms', 'monthly',
     NOW(), NOW() + INTERVAL '1 month', 'Security')
ON CONFLICT (control_id) DO NOTHING;

-- ============================================================================
-- CONTROL EVIDENCE
-- ============================================================================

CREATE TABLE IF NOT EXISTS control_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    control_id VARCHAR(50) NOT NULL REFERENCES soc2_controls(control_id),
    evidence_date TIMESTAMPTZ NOT NULL,
    evidence_type VARCHAR(100) NOT NULL,
    evidence_description TEXT,
    evidence_location TEXT,
    tested_by VARCHAR(255),
    test_result VARCHAR(50),
    findings TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_evidence_control ON control_evidence(control_id, evidence_date DESC);

-- ============================================================================
-- SECURITY INCIDENTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS security_incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id VARCHAR(50) UNIQUE NOT NULL,
    incident_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    status VARCHAR(50) NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL,
    detected_by VARCHAR(255),
    affected_systems TEXT[],
    affected_users TEXT[],
    data_breach BOOLEAN NOT NULL DEFAULT FALSE,
    personal_data_affected BOOLEAN NOT NULL DEFAULT FALSE,
    sensitive_data_affected BOOLEAN NOT NULL DEFAULT FALSE,
    estimated_affected_count INTEGER,
    description TEXT NOT NULL,
    initial_response TEXT,
    containment_actions TEXT,
    eradication_actions TEXT,
    recovery_actions TEXT,
    lessons_learned TEXT,
    reported_to_authorities BOOLEAN DEFAULT FALSE,
    reported_at TIMESTAMPTZ,
    authority_reference VARCHAR(255),
    notified_users BOOLEAN DEFAULT FALSE,
    notification_sent_at TIMESTAMPTZ,
    assigned_to VARCHAR(255),
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_incident_type ON security_incidents(incident_type, detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_incident_severity ON security_incidents(severity, status);
CREATE INDEX IF NOT EXISTS idx_incident_breach ON security_incidents(data_breach, detected_at DESC);

-- ============================================================================
-- BREACH NOTIFICATIONS (GDPR Art. 33-34)
-- ============================================================================

CREATE TABLE IF NOT EXISTS breach_notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_id VARCHAR(50) NOT NULL REFERENCES security_incidents(incident_id),
    notification_type VARCHAR(50) NOT NULL,
    recipient_type VARCHAR(50) NOT NULL,
    recipient VARCHAR(255) NOT NULL,
    sent_at TIMESTAMPTZ NOT NULL,
    delivery_status VARCHAR(50) NOT NULL,
    notification_content TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_breach_notif_incident ON breach_notifications(incident_id, sent_at DESC);

-- ============================================================================
-- CHANGE REQUESTS (SOC 2)
-- ============================================================================

CREATE TABLE IF NOT EXISTS change_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    change_id VARCHAR(50) UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    change_type VARCHAR(50) NOT NULL,
    risk_level VARCHAR(20) NOT NULL,
    requested_by VARCHAR(255) NOT NULL,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_by VARCHAR(255),
    approved_at TIMESTAMPTZ,
    implemented_by VARCHAR(255),
    implemented_at TIMESTAMPTZ,
    verified_by VARCHAR(255),
    verified_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL,
    rollback_plan TEXT,
    affected_systems TEXT[],
    scheduled_start TIMESTAMPTZ,
    scheduled_end TIMESTAMPTZ,
    actual_start TIMESTAMPTZ,
    actual_end TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_change_status ON change_requests(status, requested_at DESC);
CREATE INDEX IF NOT EXISTS idx_change_type ON change_requests(change_type, risk_level);

-- ============================================================================
-- ENCRYPTION KEY METADATA
-- ============================================================================

CREATE TABLE IF NOT EXISTS encryption_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key_id VARCHAR(255) UNIQUE NOT NULL,
    key_type VARCHAR(50) NOT NULL,
    algorithm VARCHAR(50) NOT NULL,
    key_length INTEGER NOT NULL,
    purpose TEXT NOT NULL,
    status VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rotated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    rotation_period_days INTEGER NOT NULL DEFAULT 90,
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_encryption_key_status ON encryption_keys(status, expires_at);

-- ============================================================================
-- COMPLIANCE CHECKS
-- ============================================================================

CREATE TABLE IF NOT EXISTS compliance_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    check_id VARCHAR(50) UNIQUE NOT NULL,
    check_name VARCHAR(255) NOT NULL,
    check_type VARCHAR(50) NOT NULL,
    standard VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    automated BOOLEAN NOT NULL DEFAULT FALSE,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    severity VARCHAR(20) NOT NULL,
    last_run_at TIMESTAMPTZ,
    next_run_at TIMESTAMPTZ,
    run_frequency VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_check_type ON compliance_checks(check_type, enabled);
CREATE INDEX IF NOT EXISTS idx_check_schedule ON compliance_checks(next_run_at) WHERE enabled = TRUE;

-- ============================================================================
-- COMPLIANCE CHECK RESULTS
-- ============================================================================

CREATE TABLE IF NOT EXISTS compliance_check_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    check_id VARCHAR(50) NOT NULL REFERENCES compliance_checks(check_id),
    run_at TIMESTAMPTZ NOT NULL,
    passed BOOLEAN NOT NULL,
    findings TEXT,
    severity VARCHAR(20) NOT NULL,
    remediation TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_check_results ON compliance_check_results(check_id, run_at DESC);
CREATE INDEX IF NOT EXISTS idx_check_passed ON compliance_check_results(passed, severity);

-- ============================================================================
-- RETENTION POLICIES
-- ============================================================================

CREATE TABLE IF NOT EXISTS retention_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_name VARCHAR(255) NOT NULL,
    data_type VARCHAR(100) NOT NULL,
    data_classification VARCHAR(50) NOT NULL,
    retention_days INTEGER NOT NULL,
    legal_basis TEXT NOT NULL,
    auto_delete BOOLEAN NOT NULL DEFAULT FALSE,
    archive_after_days INTEGER,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default retention policies
INSERT INTO retention_policies (policy_name, data_type, data_classification, retention_days,
                                legal_basis, auto_delete, archive_after_days)
VALUES
    ('HIPAA Usage Records', 'usage_records', 'phi', 2555,
     'HIPAA 45 CFR 164.530(j)', FALSE, 90),

    ('SOC 2 Audit Logs', 'audit_events', 'confidential', 2190,
     'SOC 2 Type II', FALSE, 90),

    ('Security Incidents', 'security_incidents', 'confidential', 3650,
     'ISO 27001 A.16.1.7', FALSE, 365)
ON CONFLICT DO NOTHING;

CREATE INDEX IF NOT EXISTS idx_retention_data_type ON retention_policies(data_type, active);

-- ============================================================================
-- ANONYMIZATION LOG
-- ============================================================================

CREATE TABLE IF NOT EXISTS anonymization_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    table_name VARCHAR(255) NOT NULL,
    record_id VARCHAR(255) NOT NULL,
    anonymization_method VARCHAR(50) NOT NULL,
    anonymized_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    anonymized_by VARCHAR(255) NOT NULL,
    reason TEXT NOT NULL,
    fields_anonymized TEXT[] NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX IF NOT EXISTS idx_anon_table ON anonymization_log(table_name, anonymized_at DESC);
CREATE INDEX IF NOT EXISTS idx_anon_record ON anonymization_log(table_name, record_id);
