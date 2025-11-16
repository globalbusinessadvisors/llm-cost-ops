# Video 09: Security & Compliance

## Metadata

- **Duration**: 22-25 minutes
- **Level**: Advanced
- **Prerequisites**: Videos 01, 02, 08
- **Target Audience**: Security engineers, compliance officers, platform engineers
- **Video ID**: LLMCO-V09-SECURITY
- **Version**: 1.0.0

## Learning Objectives

- Implement authentication and authorization
- Configure data encryption (at rest and in transit)
- Set up audit logging and compliance reporting
- Implement PII detection and redaction
- Configure role-based access control (RBAC)
- Meet SOC 2, GDPR, and HIPAA requirements
- Implement security scanning and vulnerability management

## Scene Breakdown

### Scene 1: Security Overview
**Duration**: 0:00-2:00

**Narration**:
"Security and compliance are critical for LLM applications. Today we'll implement authentication, encryption, audit logging, PII protection, and compliance controls. Let's secure your LLM Cost Ops deployment for enterprise use."

**On-Screen Text**:
- "Security Topics:"
  - "Authentication & Authorization"
  - "Data Encryption"
  - "Audit Logging"
  - "PII Detection & Redaction"
  - "RBAC & Permissions"
  - "Compliance (SOC 2, GDPR, HIPAA)"

---

### Scene 2: Authentication & SSO
**Duration**: 2:00-6:00

**Code/Demo**:
```typescript
// Multiple authentication methods
import { CostTracker, AuthConfig } from 'llm-cost-ops';

// 1. API Key Authentication (simplest)
const tracker = new CostTracker({
  apiKey: process.env.LCOPS_API_KEY!
});

// 2. JWT Authentication
const jwtTracker = new CostTracker({
  auth: {
    type: 'jwt',
    token: await getJWTToken(),
    refreshToken: await getRefreshToken(),
    onTokenExpire: async () => {
      return await refreshJWTToken();
    }
  }
});

// 3. OAuth 2.0 / OIDC
const oauthTracker = new CostTracker({
  auth: {
    type: 'oauth',
    clientId: process.env.OAUTH_CLIENT_ID!,
    clientSecret: process.env.OAUTH_CLIENT_SECRET!,
    tokenEndpoint: 'https://auth.company.com/oauth/token',
    scopes: ['llm-cost-ops:read', 'llm-cost-ops:write']
  }
});
```

**SSO Configuration (SAML/OIDC):**
```yaml
# config/auth.yaml
authentication:
  providers:
    - type: saml
      name: Okta
      entityId: https://company.okta.com
      ssoUrl: https://company.okta.com/app/saml/sso
      certificate: /etc/certs/okta.pem
      attributeMapping:
        email: email
        name: displayName
        groups: groups

    - type: oidc
      name: Google Workspace
      issuer: https://accounts.google.com
      clientId: ${GOOGLE_CLIENT_ID}
      clientSecret: ${GOOGLE_CLIENT_SECRET}
      scopes: [openid, email, profile]
      hostedDomain: company.com  # Restrict to company domain

  session:
    duration: 8h
    renewalWindow: 1h
    secure: true
    sameSite: strict

  mfa:
    required: true
    methods: [totp, webauthn]
    gracePeriod: 7d  # For new users
```

**Highlight**: "Multiple auth methods • SSO integration • MFA support"

---

### Scene 3: Authorization & RBAC
**Duration**: 6:00-10:00

**Code/Demo**:
```yaml
# Role-Based Access Control
apiVersion: llm-cost-ops.dev/v1
kind: Role
metadata:
  name: finance-analyst
spec:
  permissions:
    - resource: costs
      actions: [read, export]
    - resource: budgets
      actions: [read, create, update]
    - resource: reports
      actions: [read, create]
    - resource: dashboards
      actions: [read]
  # Cannot modify projects or system settings

---
apiVersion: llm-cost-ops.dev/v1
kind: Role
metadata:
  name: developer
spec:
  permissions:
    - resource: costs
      actions: [read]
      filters:
        projects: [owned, team]  # Only see own projects
    - resource: api_keys
      actions: [create, read, rotate]
      filters:
        projects: [owned]

---
apiVersion: llm-cost-ops.dev/v1
kind: Role
metadata:
  name: admin
spec:
  permissions:
    - resource: "*"
      actions: ["*"]
  # Full access
```

**Programmatic Permission Checks:**
```typescript
import { PermissionChecker } from 'llm-cost-ops/auth';

async function getCostReport(userId: string, projectId: string) {
  const checker = new PermissionChecker(userId);

  // Check permission
  if (!await checker.can('read', 'costs', { projectId })) {
    throw new UnauthorizedError('Insufficient permissions');
  }

  // Also check if user can export
  const canExport = await checker.can('export', 'costs', { projectId });

  const report = await tracker.getCosts({ projectId });

  return {
    data: report,
    permissions: {
      canExport,
      canModify: await checker.can('update', 'costs', { projectId })
    }
  };
}

// Attribute-Based Access Control (ABAC)
const abacPolicy = {
  effect: 'allow',
  actions: ['read'],
  resources: ['costs'],
  conditions: {
    // Only during business hours
    'time.hour': { $gte: 9, $lte: 17 },
    // Only from corporate network
    'request.ip': { $in: ['10.0.0.0/8'] },
    // Only for specific cost threshold
    'resource.cost': { $lte: 1000 }
  }
};
```

**Highlight**: "RBAC policies • Fine-grained permissions • ABAC support"

---

### Scene 4: Data Encryption
**Duration**: 10:00-13:00

**Code/Demo**:
```yaml
# Encryption configuration
encryption:
  # At-rest encryption
  atRest:
    enabled: true
    algorithm: AES-256-GCM
    keyManagement:
      type: kms  # AWS KMS, Google Cloud KMS, Azure Key Vault
      keyId: arn:aws:kms:us-east-1:123456789:key/abc-def
      rotationPeriod: 90d

  # In-transit encryption
  inTransit:
    tls:
      enabled: true
      minVersion: "1.3"
      cipherSuites:
        - TLS_AES_256_GCM_SHA384
        - TLS_AES_128_GCM_SHA256
      clientAuth: optional
      certificates:
        cert: /etc/certs/server.crt
        key: /etc/certs/server.key
        ca: /etc/certs/ca.crt

  # Field-level encryption for sensitive data
  fieldLevel:
    enabled: true
    fields:
      - prompt_content    # Encrypt actual LLM prompts
      - response_content  # Encrypt LLM responses
      - user_data        # Encrypt user information
    excludeFromLogs: true
```

**Application-level encryption:**
```typescript
import { FieldEncryption } from 'llm-cost-ops/security';

const encryption = new FieldEncryption({
  kmsKeyId: process.env.KMS_KEY_ID!
});

// Encrypt sensitive data before storage
const encryptedData = await tracker.track(
  llmResponse,
  {
    tags: { userId: '123' },
    encryption: {
      fields: ['prompt', 'response'],  // Encrypt these fields
      key: await encryption.getDataKey()
    }
  }
);

// Data is encrypted in database, decrypted only when needed
const decryptedData = await tracker.getCosts({
  projectId: 'abc',
  decrypt: true  // Requires decryption permission
});
```

**Highlight**: "End-to-end encryption • KMS integration • Field-level encryption"

---

### Scene 5: Audit Logging
**Duration**: 13:00-17:00

**Code/Demo**:
```typescript
// Comprehensive audit logging
import { AuditLogger } from 'llm-cost-ops/audit';

const auditLogger = new AuditLogger({
  destination: 's3://audit-logs/llm-cost-ops/',
  format: 'json',
  includeRequestBody: true,
  includeResponseBody: false,  // Security: don't log sensitive responses
  retention: '7y'  // 7 years for compliance
});

// Audit log entry structure
interface AuditLog {
  timestamp: string;
  eventType: string;
  userId: string;
  userEmail: string;
  ipAddress: string;
  userAgent: string;
  action: string;
  resource: string;
  resourceId: string;
  result: 'success' | 'failure';
  reason?: string;
  metadata: Record<string, any>;
  requestId: string;
}

// Example audit logs
{
  "timestamp": "2025-01-16T10:30:45Z",
  "eventType": "data_access",
  "userId": "user_123",
  "userEmail": "analyst@company.com",
  "action": "read",
  "resource": "costs",
  "resourceId": "project_abc",
  "result": "success",
  "metadata": {
    "filters": { "dateRange": "2025-01" },
    "recordsAccessed": 15420
  }
}

{
  "timestamp": "2025-01-16T10:35:12Z",
  "eventType": "data_export",
  "userId": "user_123",
  "action": "export",
  "resource": "costs",
  "format": "csv",
  "result": "success",
  "metadata": {
    "rowCount": 15420,
    "fileSize": "2.4MB",
    "destination": "local_download"
  }
}

{
  "timestamp": "2025-01-16T10:40:33Z",
  "eventType": "permission_denied",
  "userId": "user_456",
  "action": "delete",
  "resource": "project",
  "resourceId": "project_xyz",
  "result": "failure",
  "reason": "insufficient_permissions"
}
```

**Audit log querying:**
```typescript
// Query audit logs for compliance reports
const auditReport = await auditLogger.query({
  eventTypes: ['data_access', 'data_export', 'data_modification'],
  dateRange: {
    start: '2025-01-01',
    end: '2025-01-31'
  },
  users: ['analyst@company.com'],
  resources: ['costs', 'budgets']
});

// Generate compliance report
const complianceReport = await auditLogger.generateReport({
  template: 'SOC2',
  period: 'Q1-2025',
  includeFailedAttempts: true,
  groupBy: ['user', 'resource', 'action']
});
```

**Highlight**: "Comprehensive audit trail • Compliance reporting • 7-year retention"

---

### Scene 6: PII Detection & Redaction
**Duration**: 17:00-20:00

**Code/Demo**:
```typescript
// Automatic PII detection and redaction
import { PIIDetector, PIIRedactor } from 'llm-cost-ops/privacy';

const piiDetector = new PIIDetector({
  patterns: {
    email: true,
    phone: true,
    ssn: true,
    creditCard: true,
    customPatterns: [
      {
        name: 'employeeId',
        pattern: /EMP-\d{6}/g,
        category: 'identifier'
      }
    ]
  },
  confidence: 0.85  // 85% confidence threshold
});

const piiRedactor = new PIIRedactor({
  strategy: 'mask',  // 'mask', 'hash', 'remove', 'tokenize'
  preserveFormat: true
});

// Scan and redact before storing
tracker.addPreprocessor(async (request) => {
  // Detect PII in prompt
  const piiFound = await piiDetector.scan(request.prompt);

  if (piiFound.length > 0) {
    console.warn(`PII detected: ${piiFound.map(p => p.type).join(', ')}`);

    // Redact PII
    request.prompt = await piiRedactor.redact(request.prompt, piiFound);

    // Log PII detection
    await auditLogger.log({
      eventType: 'pii_detected',
      piiTypes: piiFound.map(p => p.type),
      redacted: true
    });
  }

  return request;
});

// Example redaction
const originalPrompt = "Send invoice to john.doe@company.com, phone: 555-123-4567";
const redacted = await piiRedactor.redact(originalPrompt);
// Result: "Send invoice to [EMAIL_REDACTED], phone: [PHONE_REDACTED]"

// Tokenization (reversible for authorized users)
const tokenized = await piiRedactor.tokenize(originalPrompt);
// Result: "Send invoice to [TOKEN:abc123], phone: [TOKEN:def456]"

// Detokenize (requires permission)
const original = await piiRedactor.detokenize(tokenized, {
  userId: 'admin@company.com',
  reason: 'compliance_audit'
});
```

**GDPR Right to be Forgotten:**
```typescript
// Implement GDPR data deletion
async function deleteUserData(userId: string, reason: string) {
  // Audit the deletion request
  await auditLogger.log({
    eventType: 'gdpr_deletion',
    userId,
    reason
  });

  // Delete or anonymize all user data
  await tracker.anonymizeData({
    userId,
    method: 'irreversible',
    scope: ['prompts', 'responses', 'metadata']
  });

  // Verify deletion
  const remainingData = await tracker.searchUserData(userId);
  if (remainingData.length > 0) {
    throw new Error('Data deletion incomplete');
  }

  // Log completion
  await auditLogger.log({
    eventType: 'gdpr_deletion_complete',
    userId
  });
}
```

**Highlight**: "Automatic PII detection • Redaction strategies • GDPR compliance"

---

### Scene 7: Compliance Frameworks
**Duration**: 20:00-23:00

**Code/Demo**:

**SOC 2 Compliance:**
```yaml
# SOC 2 controls mapping
soc2:
  controls:
    CC6.1:  # Logical Access Controls
      - authentication_required: true
      - mfa_enforced: true
      - session_timeout: 8h
      - password_policy: strong

    CC6.6:  # Encryption
      - data_at_rest_encrypted: true
      - data_in_transit_encrypted: true
      - key_rotation: 90d

    CC7.2:  # System Monitoring
      - audit_logging_enabled: true
      - log_retention: 7y
      - intrusion_detection: enabled
      - vulnerability_scanning: weekly

  reports:
    - type: access_report
      frequency: monthly
      recipients: [security@company.com]
    - type: change_log
      frequency: daily
      recipients: [audit@company.com]
```

**HIPAA Compliance:**
```yaml
hipaa:
  # PHI protection
  phi_detection: enabled
  phi_encryption: required
  phi_access_logging: comprehensive

  # Minimum necessary rule
  access_controls:
    role_based: true
    need_to_know: true
    audit_all_access: true

  # Business Associate Agreement (BAA)
  baa:
    required: true
    attestation: annual

  # Breach notification
  breach_detection:
    enabled: true
    notification_window: 60d
    affected_threshold: 500
```

**GDPR Compliance:**
```yaml
gdpr:
  # Data subject rights
  rights:
    - type: access
      response_time: 30d
      automated: true

    - type: rectification
      response_time: 30d
      automated: true

    - type: erasure
      response_time: 30d
      verification: required

    - type: portability
      format: [json, csv]
      automated: true

  # Data processing
  lawful_basis:
    - legitimate_interest
    - consent

  data_retention:
    default: 2y
    with_consent: unlimited
    deletion_policy: automatic

  # Privacy by design
  privacy_controls:
    data_minimization: true
    pseudonymization: true
    encryption: required
```

**Compliance Dashboard:**
```typescript
// Generate compliance status
const complianceStatus = await tracker.getComplianceStatus({
  frameworks: ['SOC2', 'GDPR', 'HIPAA']
});

console.log('Compliance Status:');
console.log('- SOC 2:', complianceStatus.soc2.status);  // 'compliant'
console.log('- GDPR:', complianceStatus.gdpr.status);   // 'compliant'
console.log('- HIPAA:', complianceStatus.hipaa.status); // 'compliant'

// Issues requiring attention
complianceStatus.issues.forEach(issue => {
  console.log(`- ${issue.framework}: ${issue.description}`);
});
```

**Highlight**: "SOC 2 • HIPAA • GDPR • Automated compliance reporting"

---

### Scene 8: Security Scanning & Best Practices
**Duration**: 23:00-24:00

**Code/Demo**:
```yaml
# Automated security scanning
security:
  vulnerability_scanning:
    enabled: true
    frequency: daily
    scanners: [trivy, snyk]
    severity_threshold: high

  dependency_scanning:
    enabled: true
    auto_update: patch_only

  secrets_scanning:
    enabled: true
    block_commits: true

  penetration_testing:
    frequency: quarterly
    external_auditor: required
```

**Highlight**: "Continuous scanning • Vulnerability management • Regular audits"

---

### Scene 9: Recap
**Duration**: 24:00-25:00

**Narration**:
"You now have enterprise-grade security! We covered authentication, encryption, audit logging, PII protection, RBAC, and compliance frameworks. Your LLM Cost Ops deployment is secure and compliant. Final video: troubleshooting and support!"

**On-Screen Text**:
- "Security Checklist:"
  - "✅ SSO/MFA enabled"
  - "✅ End-to-end encryption"
  - "✅ Comprehensive audit logs"
  - "✅ PII detection active"
  - "✅ RBAC configured"
  - "✅ Compliance controls"
- "Next: Video 10 - Troubleshooting"

---

## Post-Production Notes

### Chapter Markers
- 0:00 - Security Overview
- 2:00 - Authentication & SSO
- 6:00 - Authorization & RBAC
- 10:00 - Data Encryption
- 13:00 - Audit Logging
- 17:00 - PII Detection
- 20:00 - Compliance
- 23:00 - Security Scanning
- 24:00 - Recap

**Script Version**: 1.0.0
**Last Updated**: 2025-11-16
