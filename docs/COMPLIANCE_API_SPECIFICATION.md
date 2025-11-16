## Compliance API Specification
# LLM Cost Ops Platform

**Version:** 1.0
**Last Updated:** 2025-11-16

---

## Overview

This document specifies the REST API endpoints for the compliance control system, covering GDPR, SOC 2, HIPAA, ISO 27001, and PCI DSS compliance features.

### Base URL
```
Production: https://api.llm-cost-ops.com/v1
Staging: https://staging-api.llm-cost-ops.com/v1
Development: http://localhost:8080/v1
```

### Authentication
All endpoints require authentication via:
- **Bearer Token**: JWT token in Authorization header
- **API Key**: X-API-Key header

### Rate Limiting
- Standard tier: 1000 requests/hour
- Enterprise tier: 10000 requests/hour
- Compliance exports: 100 requests/day

---

## 1. Data Subject Requests (GDPR)

### 1.1 Create Data Subject Request

Create a new data subject request for GDPR compliance.

**Endpoint:** `POST /compliance/dsr`

**Request Body:**
```json
{
  "request_type": "access|erasure|portability|rectification|restriction|objection",
  "subject_id": "user_12345",
  "subject_email": "user@example.com",
  "organization_id": "org_abc",
  "verification_method": "email|manual",
  "notes": "Optional notes about the request"
}
```

**Response:** `201 Created`
```json
{
  "request_id": "dsr_xyz789",
  "status": "pending_verification",
  "verification_token_sent": true,
  "estimated_completion": "2025-11-23T00:00:00Z",
  "created_at": "2025-11-16T10:30:00Z"
}
```

**Permissions Required:** `compliance:dsr:create`

### 1.2 Verify Data Subject Request

Verify a data subject request using the verification token.

**Endpoint:** `POST /compliance/dsr/{request_id}/verify`

**Request Body:**
```json
{
  "verification_token": "abc123xyz"
}
```

**Response:** `200 OK`
```json
{
  "request_id": "dsr_xyz789",
  "status": "verified",
  "processing_started": true,
  "verified_at": "2025-11-16T10:35:00Z"
}
```

### 1.3 Get DSR Status

Get the status of a data subject request.

**Endpoint:** `GET /compliance/dsr/{request_id}`

**Response:** `200 OK`
```json
{
  "request_id": "dsr_xyz789",
  "request_type": "access",
  "subject_id": "user_12345",
  "subject_email": "user@example.com",
  "organization_id": "org_abc",
  "status": "processing|completed|failed",
  "progress": {
    "current_step": "exporting_data",
    "total_steps": 5,
    "percent_complete": 60
  },
  "requested_at": "2025-11-16T10:30:00Z",
  "verified_at": "2025-11-16T10:35:00Z",
  "completed_at": null,
  "export_url": null,
  "expiry_at": null
}
```

**Permissions Required:** `compliance:dsr:read`

### 1.4 Download Data Export

Download the exported data for an access request.

**Endpoint:** `GET /compliance/dsr/{request_id}/export`

**Query Parameters:**
- `format`: `json|csv|xml` (default: json)

**Response:** `200 OK`
```
Content-Type: application/json
Content-Disposition: attachment; filename="data_export_user_12345.json"

{
  "export_metadata": {
    "request_id": "dsr_xyz789",
    "subject_id": "user_12345",
    "organization_id": "org_abc",
    "exported_at": "2025-11-16T12:00:00Z",
    "data_categories": ["usage", "costs", "audit_logs"],
    "total_records": 1523,
    "checksum": "sha256:abc123..."
  },
  "usage_records": [...],
  "cost_records": [...],
  "audit_logs": [...]
}
```

**Permissions Required:** `compliance:dsr:export`

### 1.5 List Data Subject Requests

List all data subject requests with filtering.

**Endpoint:** `GET /compliance/dsr`

**Query Parameters:**
- `organization_id`: Filter by organization
- `status`: Filter by status
- `request_type`: Filter by type
- `from_date`: Start date (ISO 8601)
- `to_date`: End date (ISO 8601)
- `page`: Page number (default: 1)
- `per_page`: Items per page (default: 20, max: 100)

**Response:** `200 OK`
```json
{
  "data": [
    {
      "request_id": "dsr_xyz789",
      "request_type": "access",
      "subject_id": "user_12345",
      "status": "completed",
      "requested_at": "2025-11-16T10:30:00Z",
      "completed_at": "2025-11-16T12:00:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total_items": 45,
    "total_pages": 3
  }
}
```

**Permissions Required:** `compliance:dsr:list`

---

## 2. Consent Management

### 2.1 Record Consent

Record user consent for data processing.

**Endpoint:** `POST /compliance/consent`

**Request Body:**
```json
{
  "user_id": "user_12345",
  "organization_id": "org_abc",
  "consent_type": "data_processing|marketing|analytics|third_party_sharing",
  "purpose": "Detailed description of processing purpose",
  "granted": true,
  "consent_text": "I agree to the terms...",
  "consent_version": "v2.1",
  "ip_address": "192.168.1.1",
  "user_agent": "Mozilla/5.0..."
}
```

**Response:** `201 Created`
```json
{
  "consent_id": "consent_abc123",
  "user_id": "user_12345",
  "organization_id": "org_abc",
  "consent_type": "data_processing",
  "granted": true,
  "granted_at": "2025-11-16T10:30:00Z",
  "expires_at": "2026-11-16T10:30:00Z"
}
```

**Permissions Required:** `compliance:consent:create`

### 2.2 Revoke Consent

Revoke previously granted consent.

**Endpoint:** `DELETE /compliance/consent/{consent_id}`

**Request Body:**
```json
{
  "reason": "User requested consent withdrawal"
}
```

**Response:** `200 OK`
```json
{
  "consent_id": "consent_abc123",
  "status": "revoked",
  "revoked_at": "2025-11-16T10:35:00Z",
  "data_processing_stopped": true
}
```

**Permissions Required:** `compliance:consent:revoke`

### 2.3 Get User Consents

Get all consents for a user.

**Endpoint:** `GET /compliance/consent`

**Query Parameters:**
- `user_id`: User ID (required)
- `organization_id`: Organization ID (required)
- `consent_type`: Filter by type
- `status`: `active|revoked|expired`

**Response:** `200 OK`
```json
{
  "data": [
    {
      "consent_id": "consent_abc123",
      "consent_type": "data_processing",
      "purpose": "Cost tracking and billing",
      "granted": true,
      "granted_at": "2025-11-16T10:30:00Z",
      "expires_at": "2026-11-16T10:30:00Z",
      "consent_version": "v2.1"
    }
  ]
}
```

**Permissions Required:** `compliance:consent:read`

---

## 3. Compliance Reports

### 3.1 Generate Compliance Report

Generate a compliance report for a specific standard.

**Endpoint:** `POST /compliance/reports`

**Request Body:**
```json
{
  "report_type": "soc2_audit_trail|gdpr_dsr|hipaa_access_log|encryption_status|retention_compliance",
  "standard": "SOC2|GDPR|HIPAA|ISO27001",
  "start_date": "2025-01-01T00:00:00Z",
  "end_date": "2025-11-16T23:59:59Z",
  "organization_id": "org_abc",
  "filters": {
    "event_types": ["access_granted", "access_denied"],
    "severity": "critical|high|medium|low"
  },
  "format": "pdf|csv|json",
  "delivery": {
    "method": "download|email|webhook",
    "email": "compliance@example.com"
  }
}
```

**Response:** `202 Accepted`
```json
{
  "report_id": "report_xyz789",
  "status": "generating",
  "estimated_completion": "2025-11-16T10:35:00Z",
  "created_at": "2025-11-16T10:30:00Z"
}
```

**Permissions Required:** `compliance:reports:create`

### 3.2 Get Report Status

Get the generation status of a report.

**Endpoint:** `GET /compliance/reports/{report_id}`

**Response:** `200 OK`
```json
{
  "report_id": "report_xyz789",
  "report_type": "soc2_audit_trail",
  "standard": "SOC2",
  "status": "completed|generating|failed",
  "progress": 100,
  "generated_at": "2025-11-16T10:33:00Z",
  "download_url": "https://...",
  "expires_at": "2025-11-23T10:33:00Z",
  "summary": {
    "total_events": 15234,
    "critical_events": 5,
    "violations": 2,
    "date_range": {
      "start": "2025-01-01T00:00:00Z",
      "end": "2025-11-16T23:59:59Z"
    }
  }
}
```

**Permissions Required:** `compliance:reports:read`

### 3.3 Download Report

Download a generated compliance report.

**Endpoint:** `GET /compliance/reports/{report_id}/download`

**Response:** `200 OK`
```
Content-Type: application/pdf
Content-Disposition: attachment; filename="soc2_audit_trail_2025-11-16.pdf"

[Binary PDF content]
```

**Permissions Required:** `compliance:reports:download`

### 3.4 List Reports

List all generated reports.

**Endpoint:** `GET /compliance/reports`

**Query Parameters:**
- `report_type`: Filter by type
- `standard`: Filter by standard
- `status`: Filter by status
- `from_date`: Start date
- `to_date`: End date
- `page`: Page number
- `per_page`: Items per page

**Response:** `200 OK`
```json
{
  "data": [
    {
      "report_id": "report_xyz789",
      "report_type": "soc2_audit_trail",
      "standard": "SOC2",
      "status": "completed",
      "generated_at": "2025-11-16T10:33:00Z",
      "download_url": "https://..."
    }
  ],
  "pagination": {...}
}
```

**Permissions Required:** `compliance:reports:list`

---

## 4. Policy Management

### 4.1 Create Policy

Create a new compliance policy.

**Endpoint:** `POST /compliance/policies`

**Request Body:**
```json
{
  "policy_id": "GDPR-RET-001",
  "policy_name": "GDPR Data Retention Policy",
  "policy_type": "retention|access|encryption|audit",
  "standard": "GDPR",
  "description": "Retention policy for personal data under GDPR",
  "effective_date": "2025-01-01",
  "review_date": "2026-01-01",
  "owner": "compliance_team",
  "rules": [
    {
      "rule_name": "Usage Record Retention",
      "rule_type": {
        "type": "retention_rule",
        "data_type": "usage_records",
        "classification": "pii",
        "period": {
          "type": "years",
          "years": 7
        },
        "auto_delete": false
      },
      "priority": 1,
      "enabled": true
    }
  ]
}
```

**Response:** `201 Created`
```json
{
  "policy_id": "GDPR-RET-001",
  "status": "draft",
  "created_at": "2025-11-16T10:30:00Z",
  "version": "1.0"
}
```

**Permissions Required:** `compliance:policies:create`

### 4.2 Activate Policy

Activate a policy to start enforcement.

**Endpoint:** `POST /compliance/policies/{policy_id}/activate`

**Response:** `200 OK`
```json
{
  "policy_id": "GDPR-RET-001",
  "status": "active",
  "activated_at": "2025-11-16T10:35:00Z",
  "enforcement_started": true
}
```

**Permissions Required:** `compliance:policies:manage`

### 4.3 Get Policy

Get details of a specific policy.

**Endpoint:** `GET /compliance/policies/{policy_id}`

**Response:** `200 OK`
```json
{
  "policy_id": "GDPR-RET-001",
  "policy_name": "GDPR Data Retention Policy",
  "policy_type": "retention",
  "standard": "GDPR",
  "status": "active",
  "description": "...",
  "effective_date": "2025-01-01",
  "review_date": "2026-01-01",
  "owner": "compliance_team",
  "version": "1.0",
  "rules": [...],
  "created_at": "2025-11-16T10:30:00Z",
  "updated_at": "2025-11-16T10:35:00Z"
}
```

**Permissions Required:** `compliance:policies:read`

### 4.4 List Policies

List all compliance policies.

**Endpoint:** `GET /compliance/policies`

**Query Parameters:**
- `policy_type`: Filter by type
- `standard`: Filter by standard
- `status`: Filter by status
- `page`: Page number
- `per_page`: Items per page

**Response:** `200 OK`
```json
{
  "data": [
    {
      "policy_id": "GDPR-RET-001",
      "policy_name": "GDPR Data Retention Policy",
      "policy_type": "retention",
      "standard": "GDPR",
      "status": "active",
      "effective_date": "2025-01-01"
    }
  ],
  "pagination": {...}
}
```

**Permissions Required:** `compliance:policies:list`

---

## 5. Policy Violations

### 5.1 List Violations

List policy violations.

**Endpoint:** `GET /compliance/violations`

**Query Parameters:**
- `policy_id`: Filter by policy
- `severity`: Filter by severity
- `status`: Filter by status (open|resolved)
- `from_date`: Start date
- `to_date`: End date
- `page`: Page number
- `per_page`: Items per page

**Response:** `200 OK`
```json
{
  "data": [
    {
      "violation_id": "viol_abc123",
      "policy_id": "GDPR-RET-001",
      "policy_name": "GDPR Data Retention Policy",
      "resource_type": "usage_records",
      "resource_id": "usage_xyz",
      "user_id": "user_12345",
      "organization_id": "org_abc",
      "violation_time": "2025-11-16T10:00:00Z",
      "description": "Data retention period exceeded",
      "severity": "high",
      "status": "open",
      "created_at": "2025-11-16T10:01:00Z"
    }
  ],
  "pagination": {...},
  "summary": {
    "total_violations": 15,
    "by_severity": {
      "critical": 2,
      "high": 5,
      "medium": 6,
      "low": 2
    },
    "by_status": {
      "open": 10,
      "resolved": 5
    }
  }
}
```

**Permissions Required:** `compliance:violations:list`

### 5.2 Get Violation Details

Get detailed information about a violation.

**Endpoint:** `GET /compliance/violations/{violation_id}`

**Response:** `200 OK`
```json
{
  "violation_id": "viol_abc123",
  "policy_id": "GDPR-RET-001",
  "rule_id": "rule_xyz",
  "resource_type": "usage_records",
  "resource_id": "usage_xyz",
  "user_id": "user_12345",
  "organization_id": "org_abc",
  "violation_time": "2025-11-16T10:00:00Z",
  "description": "Data retention period exceeded by 30 days",
  "severity": "high",
  "status": "open",
  "remediation_steps": [
    "Review data retention requirements",
    "Delete or anonymize expired data",
    "Update retention policy if needed"
  ],
  "created_at": "2025-11-16T10:01:00Z"
}
```

**Permissions Required:** `compliance:violations:read`

### 5.3 Resolve Violation

Mark a violation as resolved.

**Endpoint:** `PUT /compliance/violations/{violation_id}/resolve`

**Request Body:**
```json
{
  "resolution_notes": "Data was anonymized and retention policy updated",
  "resolved_by": "admin_user"
}
```

**Response:** `200 OK`
```json
{
  "violation_id": "viol_abc123",
  "status": "resolved",
  "resolved_at": "2025-11-16T10:30:00Z",
  "resolved_by": "admin_user"
}
```

**Permissions Required:** `compliance:violations:resolve`

---

## 6. Compliance Checks

### 6.1 Run Compliance Check

Manually trigger a compliance check.

**Endpoint:** `POST /compliance/checks/run`

**Request Body:**
```json
{
  "check_ids": ["GDPR-001", "SOC2-002"],
  "organization_id": "org_abc"
}
```

**Response:** `202 Accepted`
```json
{
  "job_id": "job_xyz789",
  "checks_scheduled": 2,
  "estimated_completion": "2025-11-16T10:35:00Z"
}
```

**Permissions Required:** `compliance:checks:run`

### 6.2 Get Check Results

Get the results of compliance checks.

**Endpoint:** `GET /compliance/checks/results`

**Query Parameters:**
- `check_id`: Filter by check
- `standard`: Filter by standard
- `passed`: Filter by pass/fail
- `from_date`: Start date
- `to_date`: End date
- `page`: Page number
- `per_page`: Items per page

**Response:** `200 OK`
```json
{
  "data": [
    {
      "check_id": "GDPR-001",
      "check_name": "Consent Validity Check",
      "standard": "GDPR",
      "run_at": "2025-11-16T10:00:00Z",
      "passed": false,
      "severity": "high",
      "findings": [
        "Found 5 expired consent records"
      ],
      "remediation": "Update consent records or remove data"
    }
  ],
  "pagination": {...},
  "summary": {
    "total_checks": 10,
    "passed": 8,
    "failed": 2,
    "by_severity": {
      "critical": 0,
      "high": 2,
      "medium": 0,
      "low": 0
    }
  }
}
```

**Permissions Required:** `compliance:checks:read`

### 6.3 Get Compliance Status

Get overall compliance status dashboard.

**Endpoint:** `GET /compliance/status`

**Query Parameters:**
- `organization_id`: Organization ID
- `standard`: Filter by standard

**Response:** `200 OK`
```json
{
  "organization_id": "org_abc",
  "overall_score": 92,
  "last_updated": "2025-11-16T10:00:00Z",
  "standards": {
    "GDPR": {
      "score": 95,
      "status": "compliant",
      "open_violations": 1,
      "last_audit": "2025-11-01T00:00:00Z"
    },
    "SOC2": {
      "score": 90,
      "status": "compliant",
      "open_violations": 3,
      "last_audit": "2025-10-15T00:00:00Z"
    },
    "HIPAA": {
      "score": 91,
      "status": "compliant",
      "open_violations": 2,
      "last_audit": "2025-11-05T00:00:00Z"
    }
  },
  "recent_checks": [...],
  "recent_violations": [...],
  "upcoming_reviews": [...]
}
```

**Permissions Required:** `compliance:status:read`

---

## 7. Security Incidents

### 7.1 Report Incident

Report a security incident.

**Endpoint:** `POST /compliance/incidents`

**Request Body:**
```json
{
  "incident_type": "data_breach|unauthorized_access|system_compromise|phishing",
  "severity": "critical|high|medium|low",
  "description": "Detailed description of the incident",
  "detected_at": "2025-11-16T09:00:00Z",
  "detected_by": "security_team",
  "affected_systems": ["api_server", "database"],
  "affected_users": ["user_123", "user_456"],
  "data_breach": true,
  "personal_data_affected": true,
  "sensitive_data_affected": false,
  "estimated_affected_count": 100
}
```

**Response:** `201 Created`
```json
{
  "incident_id": "inc_xyz789",
  "status": "detected",
  "created_at": "2025-11-16T10:30:00Z",
  "requires_notification": true,
  "notification_deadline": "2025-11-19T10:30:00Z"
}
```

**Permissions Required:** `compliance:incidents:create`

### 7.2 Update Incident

Update incident status and details.

**Endpoint:** `PUT /compliance/incidents/{incident_id}`

**Request Body:**
```json
{
  "status": "investigating|contained|resolved",
  "containment_actions": "Isolated affected systems, reset credentials",
  "eradication_actions": "Patched vulnerability, removed malware",
  "recovery_actions": "Restored systems from backup",
  "lessons_learned": "Need better monitoring on API endpoints"
}
```

**Response:** `200 OK`
```json
{
  "incident_id": "inc_xyz789",
  "status": "resolved",
  "updated_at": "2025-11-16T15:00:00Z"
}
```

**Permissions Required:** `compliance:incidents:update`

### 7.3 Send Breach Notification

Send breach notification to authorities or users.

**Endpoint:** `POST /compliance/incidents/{incident_id}/notify`

**Request Body:**
```json
{
  "notification_type": "authority|user",
  "recipients": ["supervisory.authority@example.com"],
  "notification_content": "Detailed breach notification content"
}
```

**Response:** `200 OK`
```json
{
  "notification_id": "notif_abc123",
  "sent_at": "2025-11-16T10:30:00Z",
  "recipients": 1,
  "delivery_status": "sent"
}
```

**Permissions Required:** `compliance:incidents:notify`

---

## 8. Audit Logs

### 8.1 Query Audit Logs

Query audit logs with advanced filtering.

**Endpoint:** `GET /compliance/audit`

**Query Parameters:**
- `event_types`: Comma-separated event types
- `user_id`: Filter by user
- `organization_id`: Filter by organization
- `resource_type`: Filter by resource
- `severity`: Filter by severity
- `status`: Filter by status
- `from_date`: Start date
- `to_date`: End date
- `ip_address`: Filter by IP
- `page`: Page number
- `per_page`: Items per page (max: 1000)

**Response:** `200 OK`
```json
{
  "data": [
    {
      "event_id": "evt_abc123",
      "event_type": "access_granted",
      "severity": "info",
      "status": "success",
      "timestamp": "2025-11-16T10:30:00Z",
      "user_id": "user_12345",
      "user_email": "user@example.com",
      "organization_id": "org_abc",
      "resource_type": "usage_records",
      "resource_id": "usage_xyz",
      "action": "read",
      "ip_address": "192.168.1.1",
      "description": "User accessed usage record",
      "compliance_category": "SOC2"
    }
  ],
  "pagination": {...},
  "summary": {
    "total_events": 15234,
    "by_severity": {...},
    "by_status": {...}
  }
}
```

**Permissions Required:** `compliance:audit:read`

### 8.2 Export Audit Logs

Export audit logs for compliance purposes.

**Endpoint:** `POST /compliance/audit/export`

**Request Body:**
```json
{
  "filters": {
    "from_date": "2025-01-01T00:00:00Z",
    "to_date": "2025-11-16T23:59:59Z",
    "organization_id": "org_abc"
  },
  "format": "json|csv|xml",
  "include_hash_chain": true,
  "delivery": {
    "method": "download|email",
    "email": "auditor@example.com"
  }
}
```

**Response:** `202 Accepted`
```json
{
  "export_id": "exp_xyz789",
  "status": "generating",
  "estimated_completion": "2025-11-16T10:35:00Z"
}
```

**Permissions Required:** `compliance:audit:export`

### 8.3 Verify Audit Integrity

Verify the integrity of audit logs using hash chain.

**Endpoint:** `POST /compliance/audit/verify`

**Request Body:**
```json
{
  "start_date": "2025-11-01T00:00:00Z",
  "end_date": "2025-11-16T23:59:59Z"
}
```

**Response:** `200 OK`
```json
{
  "valid": true,
  "total_events": 15234,
  "invalid_events": 0,
  "verified_at": "2025-11-16T10:30:00Z",
  "hash_chain_intact": true
}
```

**Permissions Required:** `compliance:audit:verify`

---

## Error Responses

All endpoints follow standard error response format:

```json
{
  "error": {
    "code": "COMPLIANCE_ERROR_CODE",
    "message": "Human-readable error message",
    "details": {
      "field": "Additional context"
    },
    "request_id": "req_xyz789"
  }
}
```

### Common Error Codes

- `UNAUTHORIZED`: Authentication required
- `FORBIDDEN`: Insufficient permissions
- `NOT_FOUND`: Resource not found
- `VALIDATION_ERROR`: Invalid request parameters
- `RATE_LIMIT_EXCEEDED`: Too many requests
- `DSR_VERIFICATION_FAILED`: Verification token invalid
- `POLICY_VIOLATION`: Action violates compliance policy
- `RETENTION_CONFLICT`: Cannot delete data due to retention policy
- `CONSENT_REQUIRED`: User consent required for operation
- `INCIDENT_NOTIFICATION_REQUIRED`: Breach notification deadline approaching

---

## Webhooks

The compliance system can send webhooks for important events:

### Webhook Events

1. `compliance.dsr.completed` - Data subject request completed
2. `compliance.dsr.failed` - Data subject request failed
3. `compliance.policy.violated` - Policy violation detected
4. `compliance.incident.detected` - Security incident detected
5. `compliance.check.failed` - Compliance check failed
6. `compliance.consent.revoked` - User consent revoked
7. `compliance.retention.due` - Data retention period ending soon
8. `compliance.notification.required` - Breach notification required

### Webhook Payload

```json
{
  "event": "compliance.dsr.completed",
  "timestamp": "2025-11-16T10:30:00Z",
  "data": {
    "request_id": "dsr_xyz789",
    "request_type": "access",
    "subject_id": "user_12345",
    "completed_at": "2025-11-16T10:30:00Z",
    "export_url": "https://..."
  },
  "webhook_id": "wh_abc123"
}
```

---

## Rate Limits

| Endpoint Category | Requests per Hour | Burst Limit |
|------------------|-------------------|-------------|
| DSR Operations | 100 | 10 |
| Consent Management | 1000 | 50 |
| Report Generation | 50 | 5 |
| Policy Management | 200 | 20 |
| Audit Log Queries | 500 | 50 |
| Compliance Checks | 100 | 10 |

---

## Compliance Standards Matrix

| Endpoint | GDPR | SOC 2 | HIPAA | ISO 27001 |
|----------|------|-------|-------|-----------|
| DSR Endpoints | ✓ | - | - | - |
| Consent Management | ✓ | - | - | - |
| Audit Logs | ✓ | ✓ | ✓ | ✓ |
| Security Incidents | ✓ | ✓ | ✓ | ✓ |
| Policy Management | ✓ | ✓ | ✓ | ✓ |
| Compliance Reports | ✓ | ✓ | ✓ | ✓ |
| Encryption Status | ✓ | ✓ | ✓ | ✓ |

---

## Appendix: Example Workflows

### GDPR Right to Erasure Workflow

1. User submits erasure request: `POST /compliance/dsr`
2. System sends verification email
3. User verifies: `POST /compliance/dsr/{id}/verify`
4. System processes erasure (30 days max)
5. User checks status: `GET /compliance/dsr/{id}`
6. System completes and sends confirmation

### SOC 2 Audit Preparation

1. Generate audit trail: `POST /compliance/reports` (type: soc2_audit_trail)
2. Run compliance checks: `POST /compliance/checks/run`
3. Review violations: `GET /compliance/violations`
4. Resolve violations: `PUT /compliance/violations/{id}/resolve`
5. Export audit logs: `POST /compliance/audit/export`
6. Verify integrity: `POST /compliance/audit/verify`
7. Download report: `GET /compliance/reports/{id}/download`

### Data Breach Response (GDPR Art. 33)

1. Detect breach and report: `POST /compliance/incidents`
2. Update status: `PUT /compliance/incidents/{id}` (status: investigating)
3. Assess impact and severity
4. If high risk: `POST /compliance/incidents/{id}/notify` (authorities within 72h)
5. If high risk to individuals: `POST /compliance/incidents/{id}/notify` (users)
6. Document actions: `PUT /compliance/incidents/{id}`
7. Generate breach report: `POST /compliance/reports` (type: gdpr_breach)
