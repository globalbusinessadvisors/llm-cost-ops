# GDPR Compliance Guide

## Overview

This guide details how LLM Cost Ops complies with the General Data Protection Regulation (GDPR) and provides instructions for organizations to maintain GDPR compliance when using the platform.

## Table of Contents

1. [GDPR Requirements Met](#gdpr-requirements-met)
2. [Data Subject Rights](#data-subject-rights)
3. [API Endpoints](#api-endpoints)
4. [Usage Examples](#usage-examples)
5. [Privacy Policy Template](#privacy-policy-template)
6. [Data Processing](#data-processing)
7. [Security Measures](#security-measures)
8. [Compliance Checklist](#compliance-checklist)

## GDPR Requirements Met

### Article 5: Principles of Processing

#### Lawfulness, Fairness, and Transparency
- ✅ Clear privacy policy and terms of service
- ✅ Transparent data collection and processing
- ✅ Lawful basis for processing (contract, legitimate interest, consent)

#### Purpose Limitation
- ✅ Data collected only for specified purposes
- ✅ No processing for incompatible purposes
- ✅ Clear documentation of processing purposes

#### Data Minimization
- ✅ Only necessary data collected
- ✅ No excessive data collection
- ✅ Regular data minimization reviews

#### Accuracy
- ✅ Mechanisms to update and correct data
- ✅ Regular data quality checks
- ✅ User self-service data correction

#### Storage Limitation
- ✅ Configurable retention periods
- ✅ Automated data deletion
- ✅ Retention policy enforcement

#### Integrity and Confidentiality
- ✅ Encryption at rest (AES-256)
- ✅ Encryption in transit (TLS 1.3)
- ✅ Access controls and authentication
- ✅ Security monitoring and audit logging

#### Accountability
- ✅ Comprehensive audit logs
- ✅ Data protection impact assessments (DPIA)
- ✅ Documentation of processing activities

### Article 12-22: Data Subject Rights

| Right | Status | Implementation |
|-------|--------|----------------|
| Right to Access (Art. 15) | ✅ Implemented | API endpoint + self-service portal |
| Right to Rectification (Art. 16) | ✅ Implemented | Update API + user portal |
| Right to Erasure (Art. 17) | ✅ Implemented | Deletion API + automated workflow |
| Right to Data Portability (Art. 20) | ✅ Implemented | Export API (JSON, CSV, Excel) |
| Right to Object (Art. 21) | ✅ Implemented | Opt-out mechanisms |
| Right to Restriction (Art. 18) | ✅ Implemented | Account suspension |
| Rights Related to Automated Decision-Making (Art. 22) | ✅ Implemented | Manual review option |

### Article 25: Data Protection by Design and Default

- ✅ Privacy built into architecture
- ✅ Secure defaults (opt-in, not opt-out)
- ✅ Minimal data access by default
- ✅ Pseudonymization where applicable
- ✅ Regular privacy reviews

### Article 30: Records of Processing Activities

- ✅ Comprehensive ROPA (Record of Processing Activities)
- ✅ Documented data flows
- ✅ Clear controller/processor roles
- ✅ International transfer documentation

### Article 32: Security of Processing

- ✅ State-of-the-art encryption
- ✅ Regular security assessments
- ✅ Incident response procedures
- ✅ Business continuity planning
- ✅ Regular security training

### Article 33-34: Breach Notification

- ✅ 72-hour breach notification capability
- ✅ Automated breach detection
- ✅ Incident response workflow
- ✅ Data subject notification procedures

### Article 35: Data Protection Impact Assessment (DPIA)

- ✅ DPIA template and process
- ✅ High-risk processing identification
- ✅ Mitigation measures
- ✅ Regular DPIA reviews

## Data Subject Rights

### 1. Right to Access (Article 15)

Data subjects can request access to their personal data.

#### API Endpoint
```http
GET /api/v1/gdpr/access/:user_id
Authorization: Bearer {token}
```

#### Response Format
```json
{
  "user_id": "user_12345",
  "request_date": "2024-11-16T10:00:00Z",
  "data_categories": {
    "personal_information": {
      "user_id": "user_12345",
      "email": "user@example.com",
      "created_at": "2024-01-01T00:00:00Z",
      "last_login": "2024-11-16T09:00:00Z"
    },
    "usage_data": {
      "total_api_calls": 15420,
      "total_tokens": 2450000,
      "providers_used": ["openai", "anthropic"],
      "models_used": ["gpt-4", "claude-3-opus"]
    },
    "cost_data": {
      "total_spend": 1234.56,
      "currency": "USD",
      "period": "2024-01-01 to 2024-11-16"
    },
    "audit_logs": [
      {
        "event_type": "auth_login",
        "timestamp": "2024-11-16T09:00:00Z",
        "ip_address": "192.168.1.1"
      }
    ]
  },
  "processing_purposes": [
    "Service delivery",
    "Cost tracking and billing",
    "Analytics and optimization"
  ],
  "legal_basis": "Contract (Terms of Service)",
  "retention_period": "3 years from last activity",
  "recipients": ["Internal systems only"],
  "export_available": true
}
```

#### Code Example
```rust
use llm_cost_ops::auth::gdpr::GdprService;

let gdpr_service = GdprService::new(db_pool);

// Process access request
let access_request = gdpr_service
    .process_access_request("user_12345")
    .await?;

println!("User data: {:?}", access_request.data);
```

### 2. Right to Rectification (Article 16)

Data subjects can request correction of inaccurate data.

#### API Endpoint
```http
PATCH /api/v1/gdpr/rectify/:user_id
Authorization: Bearer {token}
Content-Type: application/json

{
  "field": "email",
  "old_value": "old@example.com",
  "new_value": "new@example.com",
  "reason": "Email address changed"
}
```

#### Response
```json
{
  "request_id": "rect_67890",
  "status": "completed",
  "updated_at": "2024-11-16T10:05:00Z",
  "changes": {
    "field": "email",
    "old_value": "old@example.com",
    "new_value": "new@example.com"
  }
}
```

### 3. Right to Erasure (Article 17)

Data subjects can request deletion of their personal data ("right to be forgotten").

#### API Endpoint
```http
DELETE /api/v1/gdpr/erase/:user_id
Authorization: Bearer {token}
Content-Type: application/json

{
  "confirmation": "DELETE_ALL_DATA",
  "reason": "user_request",
  "notify_email": "user@example.com"
}
```

#### Deletion Process
```
1. Request Validation
   ↓
2. Create Deletion Job
   ↓
3. Mark Data for Deletion
   ↓
4. Audit Log Entry (immutable)
   ↓
5. Delete Personal Data
   ↓
6. Anonymize Usage Records
   ↓
7. Remove API Keys
   ↓
8. Revoke Sessions
   ↓
9. Send Confirmation Email
   ↓
10. Generate Deletion Certificate
```

#### Response
```json
{
  "request_id": "del_11111",
  "status": "processing",
  "estimated_completion": "2024-11-16T12:00:00Z",
  "deletion_scope": [
    "user_profile",
    "api_keys",
    "sessions",
    "preferences"
  ],
  "retained_data": [
    "anonymized_usage_statistics",
    "audit_logs (legal requirement)"
  ],
  "certificate_available_at": "/api/v1/gdpr/deletion-certificate/del_11111"
}
```

#### Code Example
```rust
use llm_cost_ops::auth::gdpr::{GdprService, ErasureRequest};

let gdpr_service = GdprService::new(db_pool);

let erasure_request = ErasureRequest {
    user_id: "user_12345".to_string(),
    reason: "user_request".to_string(),
    confirmation: "DELETE_ALL_DATA".to_string(),
};

let result = gdpr_service
    .process_erasure_request(erasure_request)
    .await?;

println!("Deletion request ID: {}", result.request_id);
```

### 4. Right to Data Portability (Article 20)

Data subjects can receive their data in a structured, machine-readable format.

#### API Endpoint
```http
POST /api/v1/gdpr/export/:user_id
Authorization: Bearer {token}
Content-Type: application/json

{
  "format": "json",  // Options: json, csv, excel
  "include_categories": [
    "personal_information",
    "usage_data",
    "cost_data",
    "audit_logs"
  ],
  "delivery_method": "download"  // Options: download, email
}
```

#### Export Formats

**JSON Format**
```json
{
  "export_metadata": {
    "export_id": "exp_22222",
    "generated_at": "2024-11-16T10:10:00Z",
    "user_id": "user_12345",
    "format": "json",
    "version": "1.0"
  },
  "personal_information": { ... },
  "usage_data": [ ... ],
  "cost_data": [ ... ],
  "audit_logs": [ ... ]
}
```

**CSV Format**
Multiple CSV files in a ZIP archive:
- `personal_information.csv`
- `usage_data.csv`
- `cost_data.csv`
- `audit_logs.csv`
- `README.txt` (explains the data)

**Excel Format**
Single Excel workbook with multiple sheets:
- Sheet 1: Personal Information
- Sheet 2: Usage Data
- Sheet 3: Cost Data
- Sheet 4: Audit Logs
- Sheet 5: Data Dictionary

#### Code Example
```rust
use llm_cost_ops::export::{ExportFormat, ReportGenerator};

let generator = ReportGenerator::new();

let export_data = generator
    .export_user_data("user_12345", ExportFormat::Json)
    .await?;

// Save to file
std::fs::write("user_data.json", export_data.content)?;
```

### 5. Right to Object (Article 21)

Data subjects can object to processing of their data.

#### API Endpoint
```http
POST /api/v1/gdpr/object/:user_id
Authorization: Bearer {token}
Content-Type: application/json

{
  "processing_type": "marketing",  // Options: marketing, profiling, analytics
  "reason": "No longer wish to receive marketing communications"
}
```

### 6. Right to Restriction (Article 18)

Data subjects can request restriction of processing.

#### API Endpoint
```http
POST /api/v1/gdpr/restrict/:user_id
Authorization: Bearer {token}
Content-Type: application/json

{
  "restriction_type": "temporary_suspension",
  "reason": "Disputing data accuracy",
  "duration_days": 30
}
```

## API Endpoints

### Complete GDPR API Reference

#### Access Request
```http
GET /api/v1/gdpr/access/:user_id
```

#### Rectification Request
```http
PATCH /api/v1/gdpr/rectify/:user_id
```

#### Erasure Request
```http
DELETE /api/v1/gdpr/erase/:user_id
```

#### Data Export
```http
POST /api/v1/gdpr/export/:user_id
```

#### Object to Processing
```http
POST /api/v1/gdpr/object/:user_id
```

#### Restrict Processing
```http
POST /api/v1/gdpr/restrict/:user_id
```

#### Consent Management
```http
GET /api/v1/gdpr/consent/:user_id
POST /api/v1/gdpr/consent/:user_id
DELETE /api/v1/gdpr/consent/:user_id
```

#### Deletion Certificate
```http
GET /api/v1/gdpr/deletion-certificate/:request_id
```

#### Processing Record
```http
GET /api/v1/gdpr/processing-record
```

## Usage Examples

### Example 1: Complete Data Access Request

```rust
use llm_cost_ops::auth::gdpr::GdprService;
use llm_cost_ops::export::ExportFormat;

async fn handle_access_request(user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let gdpr_service = GdprService::new(db_pool);

    // 1. Verify user identity (implement your authentication)
    verify_user_identity(user_id).await?;

    // 2. Process access request
    let access_data = gdpr_service
        .process_access_request(user_id)
        .await?;

    // 3. Generate export
    let export = gdpr_service
        .export_data(user_id, ExportFormat::Json)
        .await?;

    // 4. Send to user
    send_export_to_user(user_id, export).await?;

    // 5. Log the access request
    gdpr_service.log_access_request(user_id).await?;

    Ok(())
}
```

### Example 2: Automated Deletion Workflow

```rust
use llm_cost_ops::auth::gdpr::{GdprService, ErasureRequest};
use chrono::Utc;

async fn process_deletion_request(
    user_id: &str,
    reason: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let gdpr_service = GdprService::new(db_pool);

    // 1. Create erasure request
    let request = ErasureRequest {
        user_id: user_id.to_string(),
        reason,
        confirmation: "DELETE_ALL_DATA".to_string(),
        requested_at: Utc::now(),
    };

    // 2. Validate request
    gdpr_service.validate_erasure_request(&request).await?;

    // 3. Process deletion
    let result = gdpr_service.process_erasure_request(request).await?;

    // 4. Generate deletion certificate
    let certificate = gdpr_service
        .generate_deletion_certificate(&result.request_id)
        .await?;

    // 5. Send confirmation
    send_deletion_confirmation(user_id, &certificate).await?;

    Ok(result.request_id)
}
```

### Example 3: Data Export with Scheduling

```rust
use llm_cost_ops::export::{ReportScheduler, ScheduledReportConfig};
use llm_cost_ops::export::delivery::{DeliveryMethod, EmailDelivery};

async fn schedule_monthly_data_export(
    user_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let scheduler = ReportScheduler::new(db_pool);

    let config = ScheduledReportConfig {
        name: format!("Monthly Data Export - {}", user_id),
        report_type: ReportType::DataExport,
        schedule: "0 0 1 * *".to_string(), // First day of month
        user_id: user_id.to_string(),
        delivery: DeliveryMethod::Email(EmailDelivery {
            recipient: user_email.to_string(),
            subject: "Your Monthly Data Export".to_string(),
            template: "data_export".to_string(),
        }),
        enabled: true,
    };

    scheduler.schedule_report(config).await?;

    Ok(())
}
```

## Privacy Policy Template

### Data Processing Information

```markdown
## What Personal Data We Collect

### Account Information
- Email address
- User ID (generated)
- Organization name
- Account creation date

### Usage Information
- API calls made
- Models and providers used
- Token counts
- Cost calculations
- Timestamps

### Technical Information
- IP addresses
- User agents
- Authentication tokens
- Session data

### Audit Information
- Access logs
- Action logs
- Security events

## How We Use Your Data

### Primary Purposes
1. **Service Delivery**: Provide cost tracking and analytics
2. **Billing**: Calculate and track usage costs
3. **Security**: Protect accounts and prevent fraud
4. **Support**: Respond to inquiries and issues
5. **Improvement**: Enhance platform features

### Legal Basis
- **Contract**: Necessary for service delivery
- **Legitimate Interest**: Security, fraud prevention, improvement
- **Consent**: Marketing communications (opt-in)
- **Legal Obligation**: Compliance with applicable laws

## How Long We Keep Your Data

- **Account Data**: Duration of account + 90 days
- **Usage Data**: 3 years from last activity
- **Cost Records**: 7 years (tax requirements)
- **Audit Logs**: 7 years (compliance requirements)
- **Backups**: 90 days

## Your Rights

You have the right to:
- **Access** your personal data
- **Rectify** inaccurate data
- **Erase** your data ("right to be forgotten")
- **Export** your data in a portable format
- **Object** to certain types of processing
- **Restrict** processing in certain circumstances

To exercise these rights, contact privacy@llm-cost-ops.io

## Data Security

We protect your data using:
- AES-256 encryption at rest
- TLS 1.3 encryption in transit
- Multi-factor authentication
- Regular security audits
- Access controls and monitoring

## Data Sharing

We do not sell your personal data. We may share data with:
- **Service Providers**: Cloud infrastructure, email delivery
- **Legal Requirements**: When required by law
- **Business Transfers**: In case of merger or acquisition

## International Transfers

Data may be transferred to and processed in countries outside your jurisdiction. We use:
- Standard Contractual Clauses (SCCs)
- Adequacy decisions
- Appropriate safeguards

## Cookies and Tracking

We use cookies for:
- Authentication
- Session management
- Analytics (with consent)

You can control cookies through your browser settings.

## Children's Privacy

Our service is not intended for children under 16. We do not knowingly collect data from children.

## Changes to This Policy

We will notify you of material changes via email or platform notification.

## Contact Us

**Data Protection Officer**: dpo@llm-cost-ops.io
**Privacy Team**: privacy@llm-cost-ops.io
**Address**: [Your Address]
```

## Data Processing

### Record of Processing Activities (ROPA)

| Processing Activity | Purpose | Legal Basis | Data Categories | Recipients | Retention |
|---------------------|---------|-------------|-----------------|------------|-----------|
| User Authentication | Access control | Contract | Email, password hash | Internal systems | Account lifetime |
| Usage Tracking | Service delivery | Contract | API calls, tokens, models | Internal systems | 3 years |
| Cost Calculation | Billing | Contract | Usage data, pricing | Internal systems | 7 years |
| Audit Logging | Security, compliance | Legitimate interest | All user actions | Internal systems | 7 years |
| Analytics | Service improvement | Legitimate interest | Aggregated usage | Internal systems | 3 years |
| Email Communications | Support, notifications | Contract | Email address | Email service provider | Account lifetime |

### Data Flow Diagram

```
┌─────────────┐
│    User     │
└──────┬──────┘
       │
       ▼
┌─────────────────────────────────┐
│      API Gateway (TLS 1.3)      │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│    Authentication Layer         │
│    - JWT validation             │
│    - MFA verification           │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│    Processing Layer             │
│    - Audit logging              │
│    - Access control (RBAC)      │
└────────────┬────────────────────┘
             │
             ▼
┌─────────────────────────────────┐
│    Data Storage (Encrypted)     │
│    - AES-256 at rest            │
│    - Key management             │
└─────────────────────────────────┘
```

## Security Measures

### Technical Measures

1. **Encryption**
   - At rest: AES-256
   - In transit: TLS 1.3
   - Backups: Encrypted with separate keys

2. **Access Control**
   - Role-based access control (RBAC)
   - Multi-factor authentication
   - API key rotation
   - Session management

3. **Monitoring**
   - 24/7 security monitoring
   - Intrusion detection
   - Anomaly detection
   - Real-time alerts

4. **Audit Logging**
   - Comprehensive event logging
   - Immutable audit trail
   - Log integrity verification
   - Long-term retention

### Organizational Measures

1. **Staff Training**
   - Annual GDPR training
   - Security awareness
   - Incident response drills
   - Privacy by design principles

2. **Access Management**
   - Least privilege principle
   - Regular access reviews
   - Separation of duties
   - Contractor management

3. **Vendor Management**
   - Due diligence assessments
   - Data processing agreements
   - Regular audits
   - Sub-processor approval

4. **Incident Response**
   - 24/7 incident response team
   - Escalation procedures
   - Communication plans
   - Post-incident reviews

## Compliance Checklist

### Pre-Deployment

- [ ] Privacy policy published
- [ ] Data processing agreement template ready
- [ ] ROPA (Record of Processing Activities) documented
- [ ] DPIA (Data Protection Impact Assessment) completed
- [ ] Encryption enabled (at rest and in transit)
- [ ] GDPR API endpoints configured
- [ ] Audit logging enabled
- [ ] Data retention policies configured
- [ ] Breach notification procedures in place
- [ ] Staff training completed

### Operational

- [ ] Monthly access reviews
- [ ] Quarterly privacy audits
- [ ] Annual DPIA review
- [ ] Regular security assessments
- [ ] Data subject request handling
- [ ] Vendor compliance monitoring
- [ ] Incident response testing
- [ ] Policy updates as needed

### Documentation

- [ ] Privacy policy
- [ ] Cookie policy
- [ ] Data processing agreement
- [ ] ROPA maintained and updated
- [ ] DPIA documentation
- [ ] Incident response plan
- [ ] Training records
- [ ] Audit reports

## Resources

### Templates
- Privacy Policy Template (above)
- Data Processing Agreement: `/docs/legal/DPA_TEMPLATE.md`
- DPIA Template: `/docs/compliance/DPIA_TEMPLATE.md`
- Consent Form: `/docs/legal/CONSENT_FORM.md`

### Tools
- GDPR API Client: `/sdk/gdpr-client/`
- Compliance Dashboard: Web UI at `/compliance/gdpr`
- Audit Log Viewer: `/tools/audit-viewer/`

### Support
- **Email**: gdpr@llm-cost-ops.io
- **Documentation**: https://docs.llm-cost-ops.io/gdpr
- **Training**: Monthly GDPR webinars
- **DPO Contact**: dpo@llm-cost-ops.io

---

**Last Updated**: November 2024
**Version**: 1.0.0
**Reviewed By**: Data Protection Officer
