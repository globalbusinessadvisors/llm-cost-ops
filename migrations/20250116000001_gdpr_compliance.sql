-- GDPR Compliance Schema
-- Adds tables for consent management, data breach tracking, and processing restrictions

-- Consent records table
CREATE TABLE IF NOT EXISTS consent_records (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    purpose TEXT NOT NULL,
    status TEXT NOT NULL,
    given_at TEXT NOT NULL,
    withdrawn_at TEXT,
    expires_at TEXT,
    ip_address TEXT,
    user_agent TEXT,
    consent_text TEXT NOT NULL,
    version TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for consent_records
CREATE INDEX IF NOT EXISTS idx_consent_user ON consent_records(user_id);
CREATE INDEX IF NOT EXISTS idx_consent_org ON consent_records(organization_id);
CREATE INDEX IF NOT EXISTS idx_consent_status ON consent_records(status);
CREATE UNIQUE INDEX IF NOT EXISTS idx_consent_user_purpose ON consent_records(user_id, purpose);

-- Processing restrictions table (GDPR Article 18)
CREATE TABLE IF NOT EXISTS processing_restrictions (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    restricted_at TEXT NOT NULL,
    lifted_at TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for processing_restrictions
CREATE INDEX IF NOT EXISTS idx_restriction_user ON processing_restrictions(user_id);
CREATE INDEX IF NOT EXISTS idx_restriction_active ON processing_restrictions(is_active);

-- Data breach notifications table (GDPR Articles 33-34)
CREATE TABLE IF NOT EXISTS breach_notifications (
    id TEXT PRIMARY KEY NOT NULL,
    breach_type TEXT NOT NULL,
    severity TEXT NOT NULL,
    status TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    contained_at TEXT,
    resolved_at TEXT,
    affected_users INTEGER NOT NULL,
    affected_records INTEGER NOT NULL,
    description TEXT NOT NULL,
    impact_assessment TEXT NOT NULL DEFAULT '',
    mitigation_measures TEXT NOT NULL DEFAULT '',
    notification_sent_at TEXT,
    authority_notified_at TEXT,
    users_notified_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for breach_notifications
CREATE INDEX IF NOT EXISTS idx_breach_status ON breach_notifications(status);
CREATE INDEX IF NOT EXISTS idx_breach_severity ON breach_notifications(severity);
CREATE INDEX IF NOT EXISTS idx_breach_detected ON breach_notifications(detected_at DESC);

-- Data export requests table (GDPR Article 15)
CREATE TABLE IF NOT EXISTS data_export_requests (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    format TEXT NOT NULL,
    categories TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    requested_at TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    completed_at TEXT,
    download_url TEXT,
    expires_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for data_export_requests
CREATE INDEX IF NOT EXISTS idx_export_user ON data_export_requests(user_id);
CREATE INDEX IF NOT EXISTS idx_export_status ON data_export_requests(status);
CREATE INDEX IF NOT EXISTS idx_export_requested ON data_export_requests(requested_at DESC);

-- Data deletion requests table (GDPR Article 17)
CREATE TABLE IF NOT EXISTS data_deletion_requests (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    categories TEXT NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    requested_at TEXT NOT NULL,
    requested_by TEXT NOT NULL,
    completed_at TEXT,
    retain_audit_log INTEGER NOT NULL DEFAULT 1,
    deleted_counts TEXT NOT NULL DEFAULT '{}',
    retention_exceptions TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for data_deletion_requests
CREATE INDEX IF NOT EXISTS idx_deletion_user ON data_deletion_requests(user_id);
CREATE INDEX IF NOT EXISTS idx_deletion_status ON data_deletion_requests(status);
CREATE INDEX IF NOT EXISTS idx_deletion_requested ON data_deletion_requests(requested_at DESC);

-- Anonymization log table
CREATE TABLE IF NOT EXISTS anonymization_log (
    id TEXT PRIMARY KEY NOT NULL,
    record_type TEXT NOT NULL,
    record_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    method TEXT NOT NULL,
    reason TEXT NOT NULL,
    anonymized_at TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for anonymization_log
CREATE INDEX IF NOT EXISTS idx_anon_user ON anonymization_log(user_id);
CREATE INDEX IF NOT EXISTS idx_anon_record ON anonymization_log(record_type, record_id);
CREATE INDEX IF NOT EXISTS idx_anon_date ON anonymization_log(anonymized_at DESC);

-- Legal holds table
CREATE TABLE IF NOT EXISTS legal_holds (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    organization_id TEXT NOT NULL,
    case_reference TEXT NOT NULL,
    reason TEXT NOT NULL,
    applied_at TEXT NOT NULL,
    lifted_at TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    applied_by TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes for legal_holds
CREATE INDEX IF NOT EXISTS idx_hold_user ON legal_holds(user_id);
CREATE INDEX IF NOT EXISTS idx_hold_active ON legal_holds(is_active);
CREATE INDEX IF NOT EXISTS idx_hold_org ON legal_holds(organization_id);

-- Add GDPR-related fields to existing tables

-- Add anonymization marker to usage_records
ALTER TABLE usage_records ADD COLUMN is_anonymized INTEGER DEFAULT 0;
ALTER TABLE usage_records ADD COLUMN anonymized_at TEXT;

-- Add anonymization marker to cost_records
ALTER TABLE cost_records ADD COLUMN is_anonymized INTEGER DEFAULT 0;
ALTER TABLE cost_records ADD COLUMN anonymized_at TEXT;

-- Trigger to update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_consent_timestamp
AFTER UPDATE ON consent_records
FOR EACH ROW
BEGIN
    UPDATE consent_records SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_restriction_timestamp
AFTER UPDATE ON processing_restrictions
FOR EACH ROW
BEGIN
    UPDATE processing_restrictions SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_breach_timestamp
AFTER UPDATE ON breach_notifications
FOR EACH ROW
BEGIN
    UPDATE breach_notifications SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_export_timestamp
AFTER UPDATE ON data_export_requests
FOR EACH ROW
BEGIN
    UPDATE data_export_requests SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_deletion_timestamp
AFTER UPDATE ON data_deletion_requests
FOR EACH ROW
BEGIN
    UPDATE data_deletion_requests SET updated_at = datetime('now') WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_hold_timestamp
AFTER UPDATE ON legal_holds
FOR EACH ROW
BEGIN
    UPDATE legal_holds SET updated_at = datetime('now') WHERE id = NEW.id;
END;
