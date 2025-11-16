# Compliance Architecture Design
## LLM Cost Ops Platform

**Version:** 1.0
**Last Updated:** 2025-11-16
**Status:** Design Specification

---

## Executive Summary

This document outlines the comprehensive compliance control system for the LLM Cost Ops platform, addressing GDPR, SOC 2, HIPAA, ISO 27001, and PCI DSS requirements. The architecture ensures data privacy, security, auditability, and regulatory compliance for enterprise customers in regulated industries.

### Key Compliance Objectives

- **Data Privacy (GDPR)**: Complete control over personal data with rights to access, rectify, delete, and export
- **Security Controls (SOC 2)**: Trust Service Criteria across security, availability, processing integrity, confidentiality, and privacy
- **Healthcare Protection (HIPAA)**: PHI safeguards, access controls, and audit trails
- **Information Security (ISO 27001)**: Systematic security management and risk controls
- **Payment Security (PCI DSS)**: Payment data protection (if applicable)

---

## 1. Compliance Landscape Analysis

### 1.1 GDPR Requirements

**Applicable Data:**
- User information (names, emails, user IDs)
- Organization data
- Usage metadata (IP addresses, user agents)
- Audit logs containing personal identifiers

**Key Requirements:**
- Data subject rights (access, rectification, erasure, portability)
- Consent management for data processing
- Data minimization and purpose limitation
- Privacy by design and default
- Breach notification (72 hours)
- Data Protection Impact Assessments (DPIA)
- International data transfers (adequacy decisions)

**Impact on System:**
- Implement data subject request (DSR) workflows
- Maintain consent records
- Enable data export in machine-readable format
- Support right to erasure with cascading deletes
- Log all personal data access
- Implement data retention policies

### 1.2 SOC 2 Requirements

**Trust Service Criteria:**

**Security (Common Criteria):**
- Access controls (authentication, authorization)
- Logical and physical access controls
- System operations (monitoring, incident response)
- Change management
- Risk mitigation

**Availability:**
- System availability monitoring
- Incident management
- Backup and recovery
- Capacity planning

**Processing Integrity:**
- Data quality and accuracy
- Processing completeness and validity
- Error detection and correction

**Confidentiality:**
- Data classification
- Encryption (in transit and at rest)
- Secure disposal

**Privacy:**
- Notice and consent
- Data subject rights
- Data quality and retention

**Impact on System:**
- Comprehensive audit logging
- Access control enforcement
- Encryption everywhere
- Monitoring and alerting
- Incident response procedures
- Regular security assessments

### 1.3 HIPAA Requirements

**Protected Health Information (PHI):**
- Any health-related data if customers are healthcare providers
- Requires strict access controls and encryption

**Key Requirements:**
- Administrative safeguards (policies, training, access management)
- Physical safeguards (facility access, workstation security)
- Technical safeguards (access control, audit controls, encryption)
- Business Associate Agreements (BAAs)

**Impact on System:**
- Role-based access control (RBAC)
- Encryption at rest and in transit
- Comprehensive audit trails (who accessed what, when)
- Automatic logoff
- Emergency access procedures
- Data backup and disaster recovery

### 1.4 ISO 27001 Requirements

**Information Security Management System (ISMS):**
- Risk assessment and treatment
- Security policies and procedures
- Asset management
- Access control
- Cryptography
- Operations security
- Communications security
- System acquisition, development, and maintenance
- Supplier relationships
- Incident management
- Business continuity
- Compliance monitoring

**Impact on System:**
- Policy management framework
- Risk register and assessment tools
- Security controls implementation
- Regular audits and reviews

### 1.5 PCI DSS Requirements (If Applicable)

**Applicable if:**
- Processing, storing, or transmitting payment card data

**Key Requirements:**
- Network security (firewalls, encryption)
- Cardholder data protection
- Vulnerability management
- Access control measures
- Network monitoring and testing
- Information security policy

**Impact on System:**
- Tokenization of payment data
- Secure storage (never store CVV)
- Network segmentation
- Regular security testing

---

## 2. Compliance Architecture Overview

### 2.1 Architecture Principles

1. **Privacy by Design**: Compliance built into every component
2. **Defense in Depth**: Multiple layers of security controls
3. **Least Privilege**: Minimum necessary access
4. **Auditability**: Complete activity tracking
5. **Transparency**: Clear data processing practices
6. **Accountability**: Defined responsibilities

### 2.2 System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Compliance Layer                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────┐ │
│  │  Audit Log    │  │  Data Privacy │  │  Policy Engine  │ │
│  │  System       │  │  Controls     │  │                 │ │
│  └───────────────┘  └───────────────┘  └─────────────────┘ │
│                                                               │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────┐ │
│  │  Retention    │  │  Encryption   │  │  Monitoring &   │ │
│  │  Manager      │  │  Service      │  │  Alerting       │ │
│  └───────────────┘  └───────────────┘  └─────────────────┘ │
│                                                               │
│  ┌───────────────┐  ┌───────────────┐  ┌─────────────────┐ │
│  │  Consent      │  │  Incident     │  │  Compliance     │ │
│  │  Manager      │  │  Response     │  │  Reporting      │ │
│  └───────────────┘  └───────────────┘  └─────────────────┘ │
│                                                               │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  API Layer      │  │  Storage Layer  │  │  Auth/RBAC      │
│  (Existing)     │  │  (Existing)     │  │  (Existing)     │
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

---

## 3. Audit Logging System

### 3.1 Enhanced Audit Requirements

**Extension of Existing System** (`src/auth/audit.rs`)

**Additional Event Types:**
```rust
pub enum AuditEventType {
    // Existing events...

    // GDPR events
    DataSubjectRequest,
    ConsentGranted,
    ConsentRevoked,
    DataExported,
    DataErased,

    // Compliance events
    PolicyViolation,
    ComplianceCheckFailed,
    RetentionPolicyApplied,
    DataBreachDetected,

    // Encryption events
    EncryptionKeyRotated,
    EncryptionFailed,
    DecryptionAttempt,

    // Backup and recovery
    BackupCreated,
    BackupRestored,
    DisasterRecoveryInitiated,
}
```

### 3.2 Audit Log Storage

**Requirements:**
- **Immutability**: Logs cannot be modified or deleted (except per retention policy)
- **Integrity**: Cryptographic hash chain to detect tampering
- **Long-term Storage**: 7 years for HIPAA, variable for others
- **Performance**: High-volume write capability
- **Searchability**: Efficient querying for compliance reports

**Storage Strategy:**
```
Primary Storage: PostgreSQL (structured queries)
├── Hot storage: Last 90 days (fast access)
├── Warm storage: 91 days - 1 year (moderate access)
└── Cold storage: > 1 year (archival)

Secondary Storage: Append-only object storage (S3/compatible)
└── Immutable audit trail with versioning enabled

Backup: Encrypted off-site backup
└── Daily incremental, weekly full
```

### 3.3 Audit Log Schema

```sql
-- Enhanced audit_events table
CREATE TABLE audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_id VARCHAR(255) UNIQUE NOT NULL,
    event_type VARCHAR(100) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),

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

    -- Integrity
    previous_event_hash VARCHAR(64),
    event_hash VARCHAR(64) NOT NULL,

    -- Partitioning
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
) PARTITION BY RANGE (created_at);

-- Create partitions (monthly)
CREATE TABLE audit_events_2025_01 PARTITION OF audit_events
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

-- Indexes
CREATE INDEX idx_audit_timestamp ON audit_events (timestamp DESC);
CREATE INDEX idx_audit_user ON audit_events (user_id, timestamp DESC);
CREATE INDEX idx_audit_org ON audit_events (organization_id, timestamp DESC);
CREATE INDEX idx_audit_event_type ON audit_events (event_type, timestamp DESC);
CREATE INDEX idx_audit_resource ON audit_events (resource_type, resource_id);
CREATE INDEX idx_audit_compliance ON audit_events (compliance_category);
CREATE INDEX idx_audit_metadata ON audit_events USING GIN (metadata);

-- Hash chain verification function
CREATE OR REPLACE FUNCTION verify_audit_chain(
    start_time TIMESTAMPTZ,
    end_time TIMESTAMPTZ
) RETURNS TABLE (
    valid BOOLEAN,
    total_events BIGINT,
    invalid_events BIGINT
) AS $$
-- Implementation to verify hash chain integrity
$$ LANGUAGE plpgsql;
```

### 3.4 Audit Log Retention

```rust
pub struct RetentionPolicy {
    pub event_type: AuditEventType,
    pub retention_days: u32,
    pub archive_after_days: Option<u32>,
}

// Default retention policies
impl RetentionPolicy {
    pub fn default_policies() -> Vec<RetentionPolicy> {
        vec![
            // HIPAA: 6 years minimum
            RetentionPolicy {
                event_type: AuditEventType::ResourceRead,
                retention_days: 2555, // 7 years
                archive_after_days: Some(90),
            },
            // Security events: longer retention
            RetentionPolicy {
                event_type: AuditEventType::SecurityIncident,
                retention_days: 3650, // 10 years
                archive_after_days: Some(365),
            },
            // Standard events: SOC 2 requirement
            RetentionPolicy {
                event_type: AuditEventType::ResourceCreate,
                retention_days: 2190, // 6 years
                archive_after_days: Some(90),
            },
        ]
    }
}
```

---

## 4. Data Privacy Controls (GDPR)

### 4.1 Data Subject Rights

**Rights to Implement:**
1. Right of Access (Art. 15)
2. Right to Rectification (Art. 16)
3. Right to Erasure / "Right to be Forgotten" (Art. 17)
4. Right to Data Portability (Art. 20)
5. Right to Restrict Processing (Art. 18)
6. Right to Object (Art. 21)

### 4.2 Data Subject Request (DSR) Workflow

```rust
pub enum DataSubjectRequestType {
    Access,           // Export all data
    Rectification,    // Update incorrect data
    Erasure,          // Delete all data
    Portability,      // Export in machine-readable format
    Restriction,      // Limit processing
    Objection,        // Object to processing
}

pub struct DataSubjectRequest {
    pub id: String,
    pub request_type: DataSubjectRequestType,
    pub subject_id: String,
    pub subject_email: String,
    pub organization_id: String,
    pub requested_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: DsrStatus,
    pub verification_token: Option<String>,
    pub data_export_url: Option<String>,
}

pub enum DsrStatus {
    Pending,
    VerificationSent,
    Verified,
    Processing,
    Completed,
    Failed,
    Rejected,
}
```

### 4.3 Data Inventory and Classification

```sql
-- Data catalog for GDPR compliance
CREATE TABLE data_catalog (
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

-- Example entries
INSERT INTO data_catalog VALUES
    (gen_random_uuid(), 'usage_records', 'user_id', 'identifier', TRUE, FALSE,
     'contract', 'Cost tracking and billing', 2555, TRUE, NOW(), NOW()),
    (gen_random_uuid(), 'audit_events', 'ip_address', 'network_identifier', TRUE, FALSE,
     'legitimate_interest', 'Security and audit logging', 2555, TRUE, NOW(), NOW());

-- Data processing activities register
CREATE TABLE processing_activities (
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
```

### 4.4 Consent Management

```sql
-- Consent records
CREATE TABLE consent_records (
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
    consent_text TEXT,
    consent_version VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT consent_active CHECK (
        (granted = TRUE AND granted_at IS NOT NULL) OR
        (granted = FALSE AND revoked_at IS NOT NULL)
    )
);

CREATE INDEX idx_consent_user ON consent_records (user_id, organization_id);
CREATE INDEX idx_consent_type ON consent_records (consent_type, granted);
```

### 4.5 Data Erasure Implementation

```rust
pub struct DataErasureService {
    db_pool: Pool<Postgres>,
    audit_logger: Arc<AuditLogger>,
}

impl DataErasureService {
    pub async fn erase_user_data(
        &self,
        user_id: &str,
        organization_id: &str,
        request_id: &str,
    ) -> Result<ErasureReport, ComplianceError> {
        let mut report = ErasureReport::new(user_id, organization_id);

        // Erase in dependency order
        // 1. Cost records (depends on usage)
        let cost_deleted = self.erase_cost_records(user_id, organization_id).await?;
        report.add_table("cost_records", cost_deleted);

        // 2. Usage records
        let usage_deleted = self.erase_usage_records(user_id, organization_id).await?;
        report.add_table("usage_records", usage_deleted);

        // 3. Audit logs (anonymize, not delete - required for compliance)
        let audit_anonymized = self.anonymize_audit_logs(user_id).await?;
        report.add_table("audit_events (anonymized)", audit_anonymized);

        // 4. Consent records
        let consent_deleted = self.erase_consent_records(user_id, organization_id).await?;
        report.add_table("consent_records", consent_deleted);

        // 5. User profile data
        let profile_deleted = self.erase_user_profile(user_id, organization_id).await?;
        report.add_table("user_profiles", profile_deleted);

        // Log the erasure
        self.audit_logger.log(
            AuditEvent::new(AuditEventType::DataErased, "User data erased per GDPR request")
                .with_user(user_id.to_string(), None)
                .with_organization(organization_id.to_string())
                .add_metadata("request_id".to_string(), json!(request_id))
                .add_metadata("records_deleted".to_string(), json!(report.total_records()))
        ).await?;

        Ok(report)
    }
}
```

---

## 5. Security Controls (SOC 2)

### 5.1 Access Control Matrix

```sql
-- SOC 2 control mapping
CREATE TABLE soc2_controls (
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

-- Example controls
INSERT INTO soc2_controls VALUES
    (gen_random_uuid(), 'CC6.1', 'Logical Access Controls', 'Security',
     'The entity implements logical access security software, infrastructure, and architectures',
     'implemented', TRUE, 'audit_logs', 'Review RBAC enforcement in audit logs', 'quarterly',
     NOW(), NOW() + INTERVAL '3 months', 'Engineering', NOW(), NOW()),

    (gen_random_uuid(), 'CC6.6', 'Encryption at Rest', 'Security',
     'The entity encrypts sensitive data at rest',
     'implemented', TRUE, 'configuration_review', 'Verify database encryption settings', 'quarterly',
     NOW(), NOW() + INTERVAL '3 months', 'Infrastructure', NOW(), NOW()),

    (gen_random_uuid(), 'CC7.2', 'System Monitoring', 'Security',
     'The entity monitors system components and operations',
     'implemented', TRUE, 'metrics_dashboard', 'Review monitoring coverage', 'monthly',
     NOW(), NOW() + INTERVAL '1 month', 'SRE', NOW(), NOW());

-- Control evidence tracking
CREATE TABLE control_evidence (
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
```

### 5.2 Monitoring and Alerting

```rust
pub struct ComplianceMonitor {
    metrics_registry: Arc<MetricsRegistry>,
    alert_manager: Arc<AlertManager>,
}

impl ComplianceMonitor {
    pub fn register_metrics(&self) {
        // SOC 2 metrics
        self.metrics_registry.register_counter(
            "compliance_access_denied_total",
            "Total number of access denied events"
        );

        self.metrics_registry.register_counter(
            "compliance_failed_login_attempts_total",
            "Total number of failed login attempts"
        );

        self.metrics_registry.register_gauge(
            "compliance_encryption_coverage_percent",
            "Percentage of sensitive data encrypted"
        );

        self.metrics_registry.register_histogram(
            "compliance_audit_log_latency_seconds",
            "Audit log write latency"
        );
    }

    pub async fn check_compliance_violations(&self) -> Result<Vec<ComplianceViolation>> {
        let mut violations = Vec::new();

        // Check for excessive failed logins (potential brute force)
        let failed_logins = self.check_failed_logins_threshold().await?;
        if let Some(violation) = failed_logins {
            violations.push(violation);
            self.alert_manager.send_critical_alert(
                "Excessive failed login attempts detected",
                &violation
            ).await?;
        }

        // Check for unauthorized access attempts
        let unauthorized_access = self.check_unauthorized_access_pattern().await?;
        violations.extend(unauthorized_access);

        // Check for encryption coverage
        let encryption_gaps = self.check_encryption_coverage().await?;
        violations.extend(encryption_gaps);

        Ok(violations)
    }
}
```

### 5.3 Change Management

```sql
-- Change tracking for SOC 2 compliance
CREATE TABLE change_requests (
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
```

---

## 6. Encryption and Data Protection

### 6.1 Encryption Strategy

**Data at Rest:**
- Database: PostgreSQL Transparent Data Encryption (TDE) or disk-level encryption
- Object Storage: Server-side encryption with KMS
- Backups: Encrypted before transfer to backup storage

**Data in Transit:**
- TLS 1.3 for all external communications
- mTLS for service-to-service communication
- Certificate rotation every 90 days

**Key Management:**
- External KMS (AWS KMS, Azure Key Vault, HashiCorp Vault)
- Key rotation every 90 days
- Separate keys per environment
- Key usage auditing

### 6.2 Field-Level Encryption

```rust
pub struct EncryptionService {
    kms_client: Arc<dyn KeyManagementService>,
    cache: Arc<RwLock<HashMap<String, CachedKey>>>,
}

#[derive(Clone)]
pub struct CachedKey {
    key_id: String,
    data_key: Vec<u8>,
    expires_at: DateTime<Utc>,
}

impl EncryptionService {
    pub async fn encrypt_field(
        &self,
        plaintext: &[u8],
        context: EncryptionContext,
    ) -> Result<EncryptedField, EncryptionError> {
        // Get or create data encryption key
        let data_key = self.get_data_key(&context).await?;

        // Encrypt with AES-256-GCM
        let nonce = self.generate_nonce();
        let ciphertext = aes_gcm_encrypt(plaintext, &data_key.data_key, &nonce)?;

        Ok(EncryptedField {
            key_id: data_key.key_id,
            algorithm: "AES-256-GCM".to_string(),
            ciphertext,
            nonce,
            context,
        })
    }

    pub async fn decrypt_field(
        &self,
        encrypted: &EncryptedField,
    ) -> Result<Vec<u8>, EncryptionError> {
        // Get data key from cache or KMS
        let data_key = self.get_data_key_for_decryption(&encrypted.key_id).await?;

        // Decrypt
        let plaintext = aes_gcm_decrypt(
            &encrypted.ciphertext,
            &data_key.data_key,
            &encrypted.nonce
        )?;

        // Audit decryption
        self.audit_decryption(&encrypted.key_id, &encrypted.context).await?;

        Ok(plaintext)
    }
}
```

### 6.3 Sensitive Data Detection

```rust
pub struct SensitiveDataScanner;

impl SensitiveDataScanner {
    pub fn scan_for_pii(text: &str) -> Vec<PiiDetection> {
        let mut detections = Vec::new();

        // Email addresses
        if let Some(emails) = self.detect_emails(text) {
            detections.extend(emails);
        }

        // Phone numbers
        if let Some(phones) = self.detect_phone_numbers(text) {
            detections.extend(phones);
        }

        // Social security numbers
        if let Some(ssns) = self.detect_ssn(text) {
            detections.extend(ssns);
        }

        // IP addresses
        if let Some(ips) = self.detect_ip_addresses(text) {
            detections.extend(ips);
        }

        detections
    }
}

pub enum PiiType {
    Email,
    PhoneNumber,
    SSN,
    IpAddress,
    CreditCard,
    ApiKey,
}

pub struct PiiDetection {
    pub pii_type: PiiType,
    pub value: String,
    pub position: (usize, usize),
    pub confidence: f32,
}
```

---

## 7. Compliance Reporting System

### 7.1 Report Types

```rust
pub enum ComplianceReportType {
    // SOC 2 reports
    Soc2ControlEvidence,
    Soc2AuditTrail,
    Soc2SystemDescription,

    // GDPR reports
    GdprDataProcessingActivities,
    GdprDataSubjectRequests,
    GdprBreachNotifications,
    GdprConsentRecords,

    // HIPAA reports
    HipaaAccessLog,
    HipaaSecurityIncidents,
    HipaaBreachReport,

    // General compliance
    UserAccessReport,
    EncryptionCoverageReport,
    RetentionPolicyComplianceReport,
    PolicyViolationsReport,
}

pub struct ComplianceReportGenerator {
    db_pool: Pool<Postgres>,
    audit_store: Arc<dyn AuditStore>,
}

impl ComplianceReportGenerator {
    pub async fn generate_report(
        &self,
        report_type: ComplianceReportType,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<ComplianceReport, ComplianceError> {
        match report_type {
            ComplianceReportType::Soc2AuditTrail => {
                self.generate_soc2_audit_trail(start_date, end_date).await
            }
            ComplianceReportType::GdprDataSubjectRequests => {
                self.generate_gdpr_dsr_report(start_date, end_date).await
            }
            ComplianceReportType::HipaaAccessLog => {
                self.generate_hipaa_access_log(start_date, end_date).await
            }
            // ... other report types
        }
    }

    async fn generate_soc2_audit_trail(
        &self,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> Result<ComplianceReport, ComplianceError> {
        // Query audit events relevant to SOC 2
        let query = AuditQuery::new()
            .with_time_range(start_date, end_date);

        let events = self.audit_store.query(query).await?;

        // Analyze for SOC 2 criteria
        let mut report = ComplianceReport::new(
            ComplianceReportType::Soc2AuditTrail,
            start_date,
            end_date,
        );

        report.add_section("Access Control Events", events.iter()
            .filter(|e| matches!(e.event_type,
                AuditEventType::AccessGranted |
                AuditEventType::AccessDenied))
            .collect());

        report.add_section("Security Events", events.iter()
            .filter(|e| e.severity == AuditSeverity::Critical)
            .collect());

        Ok(report)
    }
}
```

### 7.2 Automated Compliance Checks

```rust
pub struct ComplianceChecker {
    db_pool: Pool<Postgres>,
    policy_engine: Arc<PolicyEngine>,
}

pub struct ComplianceCheckResult {
    pub check_id: String,
    pub check_name: String,
    pub standard: String,
    pub passed: bool,
    pub findings: Vec<String>,
    pub severity: ComplianceSeverity,
    pub checked_at: DateTime<Utc>,
}

impl ComplianceChecker {
    pub async fn run_all_checks(&self) -> Result<Vec<ComplianceCheckResult>> {
        let mut results = Vec::new();

        // GDPR checks
        results.push(self.check_consent_validity().await?);
        results.push(self.check_data_retention_compliance().await?);
        results.push(self.check_dsr_response_time().await?);

        // SOC 2 checks
        results.push(self.check_access_control_enforcement().await?);
        results.push(self.check_encryption_coverage().await?);
        results.push(self.check_audit_log_integrity().await?);
        results.push(self.check_password_policy_compliance().await?);

        // HIPAA checks
        results.push(self.check_phi_access_controls().await?);
        results.push(self.check_automatic_logoff().await?);
        results.push(self.check_emergency_access_procedures().await?);

        Ok(results)
    }

    async fn check_consent_validity(&self) -> Result<ComplianceCheckResult> {
        // Check for expired consents
        let expired = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM consent_records
            WHERE granted = TRUE
              AND expires_at < NOW()
              AND revoked_at IS NULL
            "#
        )
        .fetch_one(&self.db_pool)
        .await?;

        let passed = expired.count == Some(0);

        Ok(ComplianceCheckResult {
            check_id: "GDPR-001".to_string(),
            check_name: "Consent Validity Check".to_string(),
            standard: "GDPR".to_string(),
            passed,
            findings: if !passed {
                vec![format!("Found {} expired consent records", expired.count.unwrap_or(0))]
            } else {
                vec![]
            },
            severity: if !passed {
                ComplianceSeverity::High
            } else {
                ComplianceSeverity::Info
            },
            checked_at: Utc::now(),
        })
    }
}
```

---

## 8. Policy Management Framework

### 8.1 Policy Engine

```sql
-- Compliance policies
CREATE TABLE compliance_policies (
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

-- Policy rules (executable)
CREATE TABLE policy_rules (
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

-- Policy violations
CREATE TABLE policy_violations (
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
```

### 8.2 Policy Examples

```rust
pub struct PolicyEngine {
    policies: Arc<RwLock<HashMap<String, Policy>>>,
    db_pool: Pool<Postgres>,
}

pub struct Policy {
    pub id: String,
    pub name: String,
    pub rules: Vec<PolicyRule>,
    pub enabled: bool,
}

pub struct PolicyRule {
    pub id: String,
    pub name: String,
    pub condition: PolicyCondition,
    pub action: PolicyAction,
    pub severity: ComplianceSeverity,
}

pub enum PolicyCondition {
    // Data retention
    DataOlderThan { days: u32, data_type: String },

    // Access control
    AccessWithoutMfa { resource_type: Resource },
    UnauthorizedAccess { user_role: RoleType, resource: Resource },

    // Encryption
    UnencryptedSensitiveData { data_category: String },

    // Consent
    ProcessingWithoutConsent { purpose: String },
    ExpiredConsent { user_id: String },

    // Custom condition (CEL or similar)
    Custom { expression: String },
}

pub enum PolicyAction {
    Block,
    Warn,
    LogOnly,
    RequireApproval { approvers: Vec<String> },
    Encrypt { algorithm: String },
    Delete,
    Anonymize,
}

impl PolicyEngine {
    pub async fn evaluate(
        &self,
        context: PolicyContext,
    ) -> Result<PolicyDecision, PolicyError> {
        let policies = self.policies.read().await;

        for policy in policies.values() {
            if !policy.enabled {
                continue;
            }

            for rule in &policy.rules {
                if self.evaluate_condition(&rule.condition, &context).await? {
                    return Ok(PolicyDecision {
                        allowed: matches!(rule.action, PolicyAction::Warn | PolicyAction::LogOnly),
                        action: rule.action.clone(),
                        reason: format!("Policy '{}' rule '{}' triggered", policy.name, rule.name),
                        policy_id: policy.id.clone(),
                        rule_id: rule.id.clone(),
                    });
                }
            }
        }

        Ok(PolicyDecision::allow())
    }
}
```

---

## 9. Incident Response and Breach Notification

### 9.1 Incident Management

```sql
-- Security incidents
CREATE TABLE security_incidents (
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

-- Breach notifications (GDPR 72-hour requirement)
CREATE TABLE breach_notifications (
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
```

### 9.2 Breach Detection

```rust
pub struct BreachDetector {
    audit_store: Arc<dyn AuditStore>,
    alert_manager: Arc<AlertManager>,
    incident_manager: Arc<IncidentManager>,
}

impl BreachDetector {
    pub async fn detect_potential_breaches(&self) -> Result<Vec<PotentialBreach>> {
        let mut breaches = Vec::new();

        // Detect unusual data access patterns
        breaches.extend(self.detect_unusual_access().await?);

        // Detect data exfiltration
        breaches.extend(self.detect_data_exfiltration().await?);

        // Detect unauthorized access
        breaches.extend(self.detect_unauthorized_access().await?);

        // Trigger incident response if breaches detected
        for breach in &breaches {
            if breach.severity == ComplianceSeverity::Critical {
                self.incident_manager.create_incident(
                    IncidentType::DataBreach,
                    breach.description.clone(),
                    breach.evidence.clone(),
                ).await?;

                // GDPR requires notification within 72 hours
                self.alert_manager.send_critical_alert(
                    "Potential data breach detected - GDPR notification required",
                    breach,
                ).await?;
            }
        }

        Ok(breaches)
    }

    async fn detect_unusual_access(&self) -> Result<Vec<PotentialBreach>> {
        // Query for unusual access patterns
        let query = AuditQuery::new()
            .with_event_type(AuditEventType::ResourceRead)
            .with_time_range(
                Utc::now() - chrono::Duration::hours(24),
                Utc::now(),
            );

        let events = self.audit_store.query(query).await?;

        // Analyze patterns
        let mut user_access: HashMap<String, Vec<AuditEvent>> = HashMap::new();
        for event in events {
            if let Some(user_id) = &event.user_id {
                user_access.entry(user_id.clone())
                    .or_default()
                    .push(event);
            }
        }

        let mut breaches = Vec::new();

        for (user_id, accesses) in user_access {
            // Check for mass data access
            if accesses.len() > 1000 {
                breaches.push(PotentialBreach {
                    breach_type: BreachType::MassDataAccess,
                    severity: ComplianceSeverity::High,
                    user_id: Some(user_id.clone()),
                    description: format!(
                        "User {} accessed {} records in 24 hours",
                        user_id,
                        accesses.len()
                    ),
                    evidence: json!({ "access_count": accesses.len() }),
                    detected_at: Utc::now(),
                });
            }
        }

        Ok(breaches)
    }
}
```

---

## 10. Implementation Plan

### Phase 1: Foundation (Weeks 1-2)
- [ ] Enhance database schema with compliance tables
- [ ] Extend audit logging system
- [ ] Implement encryption service
- [ ] Set up data catalog

### Phase 2: Privacy Controls (Weeks 3-4)
- [ ] Implement Data Subject Request workflow
- [ ] Build consent management system
- [ ] Create data erasure service
- [ ] Implement data export functionality

### Phase 3: Security Controls (Weeks 5-6)
- [ ] Implement SOC 2 control framework
- [ ] Set up compliance monitoring
- [ ] Build policy engine
- [ ] Create automated compliance checks

### Phase 4: Reporting and Documentation (Weeks 7-8)
- [ ] Build compliance report generator
- [ ] Create incident management system
- [ ] Implement breach detection
- [ ] Generate compliance documentation

### Phase 5: Testing and Validation (Weeks 9-10)
- [ ] Compliance testing suite
- [ ] Security penetration testing
- [ ] Audit log integrity testing
- [ ] Policy enforcement testing

### Phase 6: Documentation and Training (Weeks 11-12)
- [ ] Compliance runbooks
- [ ] API documentation
- [ ] Training materials
- [ ] External audit preparation

---

## 11. Integration Points

### 11.1 Existing Systems

**Authentication/Authorization** (`src/auth/`)
- Extend with MFA support
- Add session management
- Implement automatic logoff
- Enhance API key security

**Audit Logging** (`src/auth/audit.rs`)
- Add compliance event types
- Implement hash chain integrity
- Add retention policies
- Enable long-term archival

**Storage** (`src/storage/`)
- Add field-level encryption
- Implement data classification
- Add anonymization support
- Enable secure deletion

**Export System** (`src/export/`)
- Add GDPR export format
- Implement secure delivery
- Add encryption for exports
- Support data portability

### 11.2 API Endpoints

```
POST   /api/v1/compliance/dsr                    - Create data subject request
GET    /api/v1/compliance/dsr/:id                - Get DSR status
POST   /api/v1/compliance/dsr/:id/verify         - Verify DSR
GET    /api/v1/compliance/dsr/:id/export         - Download data export

POST   /api/v1/compliance/consent                - Grant/update consent
GET    /api/v1/compliance/consent                - List consents
DELETE /api/v1/compliance/consent/:id            - Revoke consent

GET    /api/v1/compliance/reports                - List available reports
POST   /api/v1/compliance/reports                - Generate report
GET    /api/v1/compliance/reports/:id            - Download report

GET    /api/v1/compliance/policies               - List policies
POST   /api/v1/compliance/policies               - Create policy
GET    /api/v1/compliance/policies/:id           - Get policy
PUT    /api/v1/compliance/policies/:id           - Update policy

GET    /api/v1/compliance/violations             - List violations
GET    /api/v1/compliance/violations/:id         - Get violation
PUT    /api/v1/compliance/violations/:id/resolve - Resolve violation

GET    /api/v1/compliance/checks                 - Run compliance checks
GET    /api/v1/compliance/checks/status          - Get check status

POST   /api/v1/compliance/incidents              - Create incident
GET    /api/v1/compliance/incidents              - List incidents
GET    /api/v1/compliance/incidents/:id          - Get incident
PUT    /api/v1/compliance/incidents/:id          - Update incident
```

---

## 12. Metrics and KPIs

### Compliance Metrics

```
# GDPR Metrics
gdpr_dsr_requests_total{type="access|erasure|portability"}
gdpr_dsr_response_time_seconds
gdpr_dsr_completion_rate
gdpr_consent_grant_total
gdpr_consent_revoke_total

# SOC 2 Metrics
soc2_access_denied_total{resource="*"}
soc2_failed_auth_total{reason="*"}
soc2_encryption_coverage_percent
soc2_audit_log_writes_total
soc2_audit_log_write_latency_seconds
soc2_policy_violations_total{policy="*"}

# HIPAA Metrics
hipaa_phi_access_total{user="*"}
hipaa_unauthorized_access_attempts_total
hipaa_automatic_logoff_total
hipaa_emergency_access_total

# General Compliance
compliance_check_runs_total{check="*"}
compliance_check_failures_total{check="*"}
compliance_report_generation_total{type="*"}
compliance_incident_total{severity="*"}
```

---

## Conclusion

This compliance architecture provides comprehensive coverage for GDPR, SOC 2, HIPAA, ISO 27001, and PCI DSS requirements. The design emphasizes:

1. **Privacy by Design**: Compliance built into every component
2. **Auditability**: Complete tracking of all actions
3. **Security**: Defense in depth with encryption and access controls
4. **Automation**: Automated compliance checks and reporting
5. **Flexibility**: Policy-driven framework adaptable to changing regulations

The implementation plan spans 12 weeks and integrates seamlessly with existing systems while adding enterprise-grade compliance capabilities required for Fortune 500 customers in regulated industries.
