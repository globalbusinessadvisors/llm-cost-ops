# Policy Management Guide

## Overview

This guide provides comprehensive information on creating, enforcing, monitoring, and managing compliance policies in the LLM Cost Ops platform. Policies define rules and requirements that govern system behavior, data access, and resource usage.

## Table of Contents

1. [Policy Types](#policy-types)
2. [Creating Policies](#creating-policies)
3. [Enforcement Mechanisms](#enforcement-mechanisms)
4. [Monitoring Compliance](#monitoring-compliance)
5. [Violation Handling](#violation-handling)
6. [Policy Templates](#policy-templates)
7. [Best Practices](#best-practices)
8. [Automation](#automation)

## Policy Types

### 1. Access Control Policies

Define who can access what resources and under what conditions.

**Examples**:
- Role-based access control (RBAC) policies
- Attribute-based access control (ABAC) policies
- Time-based access restrictions
- IP-based access control
- MFA requirements for sensitive operations

### 2. Data Protection Policies

Govern how data is protected, encrypted, and handled.

**Examples**:
- Encryption requirements (at rest and in transit)
- Data classification policies
- Data retention and deletion policies
- Backup and recovery policies
- Data masking policies

### 3. Security Policies

Define security requirements and controls.

**Examples**:
- Password complexity requirements
- Session timeout policies
- API key rotation policies
- Vulnerability management policies
- Incident response policies

### 4. Usage Policies

Control how resources are used.

**Examples**:
- Rate limiting policies
- Cost budget policies
- Resource quota policies
- API usage policies
- Fair use policies

### 5. Compliance Policies

Ensure compliance with regulations and standards.

**Examples**:
- GDPR compliance policies
- SOC 2 control policies
- HIPAA security policies
- Audit logging policies
- Data residency policies

## Creating Policies

### Policy Definition Structure

```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    /// Unique policy identifier
    pub id: String,

    /// Policy name
    pub name: String,

    /// Policy description
    pub description: String,

    /// Policy type
    pub policy_type: PolicyType,

    /// Policy rules
    pub rules: Vec<PolicyRule>,

    /// Enforcement mode (Enforce, Monitor, Disabled)
    pub enforcement_mode: EnforcementMode,

    /// Severity of violations (Info, Warning, Critical)
    pub severity: ViolationSeverity,

    /// Actions to take on violation
    pub violation_actions: Vec<ViolationAction>,

    /// Policy scope (Global, Organization, User)
    pub scope: PolicyScope,

    /// Optional scope identifier (org_id, user_id, etc.)
    pub scope_id: Option<String>,

    /// Policy enabled/disabled
    pub enabled: bool,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,

    /// Created by user
    pub created_by: String,

    /// Policy metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    /// Rule identifier
    pub id: String,

    /// Rule condition (expression to evaluate)
    pub condition: String,

    /// Rule description
    pub description: String,

    /// Rule enabled/disabled
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EnforcementMode {
    /// Actively enforce policy (block violations)
    Enforce,

    /// Monitor violations (alert only, don't block)
    Monitor,

    /// Policy disabled
    Disabled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationAction {
    /// Log the violation
    Log,

    /// Send alert notification
    Alert { recipients: Vec<String> },

    /// Block the action
    Block,

    /// Require approval
    RequireApproval { approvers: Vec<String> },

    /// Temporary suspension
    Suspend { duration_minutes: u64 },

    /// Trigger webhook
    Webhook { url: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyScope {
    /// Global policy (applies to all)
    Global,

    /// Organization-level policy
    Organization,

    /// User-level policy
    User,

    /// Resource-level policy
    Resource,
}
```

### Example Policies

#### Password Policy
```rust
let password_policy = Policy {
    id: "policy_password_001".to_string(),
    name: "Strong Password Policy".to_string(),
    description: "Enforce strong password requirements for all users".to_string(),
    policy_type: PolicyType::Security,
    rules: vec![
        PolicyRule {
            id: "rule_pwd_length".to_string(),
            condition: "password.length >= 12".to_string(),
            description: "Password must be at least 12 characters".to_string(),
            enabled: true,
        },
        PolicyRule {
            id: "rule_pwd_complexity".to_string(),
            condition: "password.has_uppercase && password.has_lowercase && password.has_number && password.has_special".to_string(),
            description: "Password must contain uppercase, lowercase, number, and special character".to_string(),
            enabled: true,
        },
        PolicyRule {
            id: "rule_pwd_history".to_string(),
            condition: "!password.in_history(5)".to_string(),
            description: "Password must not be one of the last 5 passwords".to_string(),
            enabled: true,
        },
    ],
    enforcement_mode: EnforcementMode::Enforce,
    severity: ViolationSeverity::Critical,
    violation_actions: vec![
        ViolationAction::Log,
        ViolationAction::Block,
    ],
    scope: PolicyScope::Global,
    scope_id: None,
    enabled: true,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    created_by: "admin".to_string(),
    metadata: HashMap::new(),
};
```

#### Cost Budget Policy
```rust
let budget_policy = Policy {
    id: "policy_budget_001".to_string(),
    name: "Monthly Cost Budget".to_string(),
    description: "Alert when monthly costs exceed 80% of budget".to_string(),
    policy_type: PolicyType::Usage,
    rules: vec![
        PolicyRule {
            id: "rule_budget_warning".to_string(),
            condition: "organization.monthly_cost >= organization.budget * 0.8".to_string(),
            description: "Alert at 80% budget utilization".to_string(),
            enabled: true,
        },
        PolicyRule {
            id: "rule_budget_critical".to_string(),
            condition: "organization.monthly_cost >= organization.budget * 0.95".to_string(),
            description: "Critical alert at 95% budget utilization".to_string(),
            enabled: true,
        },
        PolicyRule {
            id: "rule_budget_exceeded".to_string(),
            condition: "organization.monthly_cost >= organization.budget".to_string(),
            description: "Budget exceeded - require approval for new requests".to_string(),
            enabled: true,
        },
    ],
    enforcement_mode: EnforcementMode::Enforce,
    severity: ViolationSeverity::Warning,
    violation_actions: vec![
        ViolationAction::Log,
        ViolationAction::Alert {
            recipients: vec![
                "finance@example.com".to_string(),
                "admin@example.com".to_string(),
            ],
        },
        ViolationAction::RequireApproval {
            approvers: vec!["finance_manager".to_string()],
        },
    ],
    scope: PolicyScope::Organization,
    scope_id: Some("org_12345".to_string()),
    enabled: true,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    created_by: "finance_admin".to_string(),
    metadata: {
        let mut m = HashMap::new();
        m.insert("budget_currency".to_string(), json!("USD"));
        m.insert("budget_period".to_string(), json!("monthly"));
        m
    },
};
```

#### Data Retention Policy
```rust
let retention_policy = Policy {
    id: "policy_retention_001".to_string(),
    name: "Data Retention Policy".to_string(),
    description: "Automatically delete old data according to retention schedules".to_string(),
    policy_type: PolicyType::DataProtection,
    rules: vec![
        PolicyRule {
            id: "rule_retention_usage".to_string(),
            condition: "data.type == 'usage' && data.age_days > 1095".to_string(),
            description: "Delete usage data older than 3 years".to_string(),
            enabled: true,
        },
        PolicyRule {
            id: "rule_retention_audit".to_string(),
            condition: "data.type == 'audit_log' && data.age_days > 2555".to_string(),
            description: "Archive audit logs older than 7 years".to_string(),
            enabled: true,
        },
        PolicyRule {
            id: "rule_retention_personal".to_string(),
            condition: "data.type == 'personal' && account.deleted && data.age_days > 90".to_string(),
            description: "Delete personal data 90 days after account deletion".to_string(),
            enabled: true,
        },
    ],
    enforcement_mode: EnforcementMode::Enforce,
    severity: ViolationSeverity::Info,
    violation_actions: vec![
        ViolationAction::Log,
    ],
    scope: PolicyScope::Global,
    scope_id: None,
    enabled: true,
    created_at: Utc::now(),
    updated_at: Utc::now(),
    created_by: "compliance_admin".to_string(),
    metadata: HashMap::new(),
};
```

### Creating Policies via API

#### REST API
```http
POST /api/v1/policies
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "MFA Requirement for Admins",
  "description": "Require multi-factor authentication for all admin users",
  "policy_type": "security",
  "rules": [
    {
      "condition": "user.role == 'admin' && !auth.mfa_verified",
      "description": "Admin users must use MFA"
    }
  ],
  "enforcement_mode": "enforce",
  "severity": "critical",
  "violation_actions": [
    {"type": "log"},
    {"type": "block"}
  ],
  "scope": "global",
  "enabled": true
}
```

#### Rust SDK
```rust
use llm_cost_ops::compliance::policy::{PolicyManager, Policy, PolicyRule};

let policy_manager = PolicyManager::new(db_pool);

let policy = Policy {
    name: "MFA Requirement for Admins".to_string(),
    description: "Require multi-factor authentication for all admin users".to_string(),
    policy_type: PolicyType::Security,
    rules: vec![
        PolicyRule {
            condition: "user.role == 'admin' && !auth.mfa_verified".to_string(),
            description: "Admin users must use MFA".to_string(),
            enabled: true,
        }
    ],
    enforcement_mode: EnforcementMode::Enforce,
    severity: ViolationSeverity::Critical,
    violation_actions: vec![
        ViolationAction::Log,
        ViolationAction::Block,
    ],
    scope: PolicyScope::Global,
    enabled: true,
    ..Default::default()
};

let created_policy = policy_manager.create_policy(policy).await?;
```

## Enforcement Mechanisms

### Policy Evaluation Engine

```rust
use llm_cost_ops::compliance::policy::{PolicyEngine, EvaluationContext};

let policy_engine = PolicyEngine::new(policy_manager);

// Evaluate policies
let context = EvaluationContext {
    user_id: Some("user_12345".to_string()),
    organization_id: Some("org_98765".to_string()),
    action: Action::Create,
    resource: Resource::ApiKey,
    metadata: {
        let mut m = HashMap::new();
        m.insert("ip_address".to_string(), json!("203.0.113.42"));
        m.insert("user_agent".to_string(), json!("Mozilla/5.0..."));
        m
    },
};

let evaluation = policy_engine.evaluate(&context).await?;

match evaluation.result {
    EvaluationResult::Allow => {
        // Proceed with action
    },
    EvaluationResult::Deny { reason } => {
        // Block action
        return Err(PolicyError::Denied(reason));
    },
    EvaluationResult::RequireApproval { approvers } => {
        // Request approval
        approval_system.request_approval(approvers).await?;
    },
}

// Log violations
for violation in evaluation.violations {
    audit_logger.log_policy_violation(violation).await?;
}
```

### Real-Time Enforcement

```rust
// Middleware for policy enforcement
pub async fn policy_enforcement_middleware(
    policy_engine: Arc<PolicyEngine>,
    request: Request,
    next: Next,
) -> Result<Response, PolicyError> {
    // Extract context from request
    let context = extract_context(&request)?;

    // Evaluate applicable policies
    let evaluation = policy_engine.evaluate(&context).await?;

    match evaluation.result {
        EvaluationResult::Allow => {
            // Continue processing
            Ok(next.run(request).await)
        },
        EvaluationResult::Deny { reason } => {
            // Return 403 Forbidden
            Err(PolicyError::Denied(reason))
        },
        EvaluationResult::RequireApproval { approvers } => {
            // Return 202 Accepted with approval request
            Ok(Response::builder()
                .status(StatusCode::ACCEPTED)
                .body(json!({
                    "status": "pending_approval",
                    "approvers": approvers
                }))
                .unwrap())
        },
    }
}
```

### Batch Enforcement

```rust
// Evaluate multiple policies
let policies = policy_manager.get_policies_for_scope(
    PolicyScope::Organization,
    "org_12345"
).await?;

let mut violations = Vec::new();

for policy in policies {
    if !policy.enabled {
        continue;
    }

    let evaluation = policy_engine.evaluate_policy(&policy, &context).await?;

    if !evaluation.compliant {
        violations.push(PolicyViolation {
            policy_id: policy.id,
            policy_name: policy.name,
            violated_rules: evaluation.violated_rules,
            severity: policy.severity,
        });
    }
}

// Handle violations
if !violations.is_empty() {
    handle_violations(violations).await?;
}
```

## Monitoring Compliance

### Compliance Dashboard

```rust
use llm_cost_ops::compliance::monitor::{ComplianceMonitor, ComplianceMetrics};

let monitor = ComplianceMonitor::new(policy_manager, audit_logger);

// Get compliance metrics
let metrics = monitor.get_metrics().await?;

println!("Compliance Score: {}/100", metrics.compliance_score);
println!("Active Policies: {}", metrics.active_policies);
println!("Violations (24h): {}", metrics.violations_24h);
println!("Critical Violations: {}", metrics.critical_violations);

// Compliance by policy type
for (policy_type, score) in metrics.compliance_by_type {
    println!("{:?}: {}%", policy_type, score);
}
```

### Compliance Metrics

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceMetrics {
    /// Overall compliance score (0-100)
    pub compliance_score: f64,

    /// Number of active policies
    pub active_policies: u64,

    /// Policy compliance by type
    pub compliance_by_type: HashMap<PolicyType, f64>,

    /// Violations in last 24 hours
    pub violations_24h: u64,

    /// Critical violations
    pub critical_violations: u64,

    /// Warning violations
    pub warning_violations: u64,

    /// Info violations
    pub info_violations: u64,

    /// Violations by policy
    pub violations_by_policy: HashMap<String, u64>,

    /// Trend (improving, declining, stable)
    pub trend: ComplianceTrend,

    /// Last updated
    pub updated_at: DateTime<Utc>,
}
```

### Real-Time Monitoring

```rust
// Set up real-time compliance monitoring
let monitor = ComplianceMonitor::new(policy_manager, audit_logger);

// Monitor for violations
monitor.on_violation(|violation| async move {
    println!("Policy Violation: {}", violation.policy_name);

    match violation.severity {
        ViolationSeverity::Critical => {
            // Immediate alert
            alert_system.send_critical_alert(violation).await?;

            // Create incident
            incident_system.create_incident(violation).await?;
        },
        ViolationSeverity::Warning => {
            // Send notification
            notification_system.notify(violation).await?;
        },
        ViolationSeverity::Info => {
            // Log only
            audit_logger.log_violation(violation).await?;
        },
    }

    Ok(())
});

// Start monitoring
monitor.start().await?;
```

### Scheduled Compliance Checks

```rust
use llm_cost_ops::compliance::scheduler::ComplianceScheduler;

let scheduler = ComplianceScheduler::new(monitor);

// Daily compliance check
scheduler.schedule(
    "daily_compliance_check",
    "0 9 * * *",  // Every day at 9 AM
    || async {
        let metrics = monitor.get_metrics().await?;

        // Generate report
        let report = generate_compliance_report(metrics)?;

        // Send to compliance team
        email_system.send(
            vec!["compliance@example.com".to_string()],
            "Daily Compliance Report",
            report
        ).await?;

        Ok(())
    }
).await?;

// Weekly detailed analysis
scheduler.schedule(
    "weekly_compliance_analysis",
    "0 9 * * 1",  // Every Monday at 9 AM
    || async {
        let analysis = monitor.analyze_trends().await?;

        // Send detailed report
        send_detailed_report(analysis).await?;

        Ok(())
    }
).await?;
```

## Violation Handling

### Violation Workflow

```
Detection → Classification → Action → Notification → Remediation → Review
```

### Violation Processing

```rust
use llm_cost_ops::compliance::violations::{ViolationHandler, PolicyViolation};

let handler = ViolationHandler::new(policy_manager, audit_logger);

// Process violation
async fn process_violation(
    handler: &ViolationHandler,
    violation: PolicyViolation,
) -> Result<(), ViolationError> {
    // 1. Log the violation
    handler.log_violation(&violation).await?;

    // 2. Execute violation actions
    for action in &violation.actions {
        match action {
            ViolationAction::Log => {
                // Already logged above
            },
            ViolationAction::Alert { recipients } => {
                handler.send_alert(&violation, recipients).await?;
            },
            ViolationAction::Block => {
                // Block is handled by policy engine
            },
            ViolationAction::RequireApproval { approvers } => {
                handler.request_approval(&violation, approvers).await?;
            },
            ViolationAction::Suspend { duration_minutes } => {
                handler.suspend_user(&violation, *duration_minutes).await?;
            },
            ViolationAction::Webhook { url } => {
                handler.trigger_webhook(&violation, url).await?;
            },
        }
    }

    // 3. Update metrics
    handler.update_metrics(&violation).await?;

    // 4. Check for remediation
    handler.check_remediation(&violation).await?;

    Ok(())
}
```

### Violation Remediation

```rust
// Automatic remediation
let remediator = ViolationRemediator::new();

remediator.register_handler(
    PolicyType::Security,
    |violation| async move {
        match violation.violated_rule.as_str() {
            "password_expired" => {
                // Force password reset
                force_password_reset(&violation.user_id).await?;
            },
            "mfa_not_enabled" => {
                // Require MFA enrollment
                require_mfa_enrollment(&violation.user_id).await?;
            },
            "suspicious_login" => {
                // Lock account and notify user
                lock_account(&violation.user_id).await?;
                notify_user_suspicious_activity(&violation.user_id).await?;
            },
            _ => {},
        }
        Ok(())
    }
);

// Manual remediation workflow
let workflow = RemediationWorkflow::new();

workflow.create_ticket(violation, |ticket| async move {
    ticket
        .assign_to("security_team")
        .set_priority(Priority::High)
        .set_due_date(Utc::now() + Duration::days(1))
        .add_comment("Investigate and remediate security policy violation")
        .save()
        .await?;
    Ok(())
});
```

### Violation Reporting

```rust
// Generate violation report
let report = ViolationReporter::new(audit_logger);

let summary = report.generate_summary(
    Utc::now() - Duration::days(7),
    Utc::now()
).await?;

println!("Violation Summary (Last 7 Days):");
println!("Total Violations: {}", summary.total_violations);
println!("By Severity:");
println!("  Critical: {}", summary.critical_count);
println!("  Warning: {}", summary.warning_count);
println!("  Info: {}", summary.info_count);
println!("\nTop Violated Policies:");
for (policy_name, count) in summary.top_violations.iter().take(10) {
    println!("  {}: {} violations", policy_name, count);
}
println!("\nRemediation Status:");
println!("  Resolved: {}", summary.resolved_count);
println!("  In Progress: {}", summary.in_progress_count);
println!("  Open: {}", summary.open_count);
```

## Policy Templates

### Built-in Templates

#### 1. GDPR Compliance Template
```toml
# config/policy_templates/gdpr_compliance.toml

[[policies]]
name = "GDPR Data Access Request"
description = "Respond to data access requests within 30 days"
policy_type = "compliance"
enforcement_mode = "monitor"
severity = "warning"

[[policies.rules]]
condition = "request.type == 'data_access' && request.age_days > 30"
description = "Data access request must be fulfilled within 30 days"

[[policies]]
name = "GDPR Data Retention"
description = "Delete personal data according to retention schedule"
policy_type = "data_protection"
enforcement_mode = "enforce"
severity = "info"

[[policies.rules]]
condition = "data.type == 'personal' && data.age_days > retention_days"
description = "Delete personal data after retention period"
```

#### 2. SOC 2 Security Template
```toml
# config/policy_templates/soc2_security.toml

[[policies]]
name = "SOC 2 - Strong Authentication"
description = "Require strong authentication for all users"
policy_type = "security"
enforcement_mode = "enforce"
severity = "critical"

[[policies.rules]]
condition = "password.length < 12"
description = "Password must be at least 12 characters"

[[policies.rules]]
condition = "user.role == 'admin' && !auth.mfa_enabled"
description = "Admin users must have MFA enabled"

[[policies]]
name = "SOC 2 - Access Review"
description = "Review user access quarterly"
policy_type = "compliance"
enforcement_mode = "monitor"
severity = "warning"

[[policies.rules]]
condition = "user.last_access_review_days > 90"
description = "User access must be reviewed quarterly"
```

#### 3. Rate Limiting Template
```toml
# config/policy_templates/rate_limiting.toml

[[policies]]
name = "API Rate Limiting"
description = "Limit API requests per user"
policy_type = "usage"
enforcement_mode = "enforce"
severity = "warning"

[[policies.rules]]
condition = "api.requests_per_minute > 100"
description = "Maximum 100 requests per minute per user"

[[policies.rules]]
condition = "api.requests_per_day > 10000"
description = "Maximum 10,000 requests per day per user"
```

### Custom Templates

```rust
// Create custom policy template
let template = PolicyTemplate {
    name: "Custom Security Policy".to_string(),
    description: "Organization-specific security requirements".to_string(),
    category: "security".to_string(),
    variables: vec![
        TemplateVariable {
            name: "max_failed_attempts".to_string(),
            description: "Maximum failed login attempts".to_string(),
            default_value: json!(5),
            var_type: VariableType::Integer,
        },
        TemplateVariable {
            name: "lockout_duration_minutes".to_string(),
            description: "Account lockout duration".to_string(),
            default_value: json!(30),
            var_type: VariableType::Integer,
        },
    ],
    policies: vec![
        PolicyDefinition {
            name: "Account Lockout".to_string(),
            rules: vec![
                "failed_logins >= {{max_failed_attempts}}".to_string(),
            ],
            actions: vec![
                json!({"type": "block"}),
                json!({"type": "suspend", "duration_minutes": "{{lockout_duration_minutes}}"}),
            ],
        },
    ],
};

// Instantiate template with custom values
let policy = template.instantiate(hashmap! {
    "max_failed_attempts" => json!(3),
    "lockout_duration_minutes" => json!(60),
})?;

policy_manager.create_policy(policy).await?;
```

## Best Practices

### 1. Start with Monitor Mode

```rust
// Initially deploy policies in monitor mode
let policy = Policy {
    enforcement_mode: EnforcementMode::Monitor,  // Start monitoring
    ..Default::default()
};

// After validation, switch to enforce mode
policy_manager.update_enforcement_mode(
    policy_id,
    EnforcementMode::Enforce
).await?;
```

### 2. Use Appropriate Severity Levels

```rust
// Critical: Security risks, compliance violations
ViolationSeverity::Critical

// Warning: Policy violations that should be addressed
ViolationSeverity::Warning

// Info: Informational violations
ViolationSeverity::Info
```

### 3. Test Policies Before Deployment

```rust
// Test policy evaluation
let test_context = EvaluationContext {
    user_id: Some("test_user".to_string()),
    // ... test data
};

let evaluation = policy_engine.evaluate_policy(&policy, &test_context).await?;

assert_eq!(evaluation.result, EvaluationResult::Allow);
```

### 4. Version Control Policies

```rust
// Track policy changes
let policy_version = PolicyVersion {
    policy_id: policy.id.clone(),
    version: 2,
    changes: "Updated password length requirement from 8 to 12 characters".to_string(),
    changed_by: "security_admin".to_string(),
    changed_at: Utc::now(),
    previous_policy: serde_json::to_value(&old_policy)?,
    current_policy: serde_json::to_value(&policy)?,
};

policy_manager.create_version(policy_version).await?;
```

### 5. Regular Policy Reviews

```bash
# Schedule quarterly policy review
cargo run --bin policy-review -- \
    --review-type quarterly \
    --output policy_review_2024_q4.pdf \
    --notify compliance@example.com
```

## Automation

### Automated Policy Creation

```rust
// Auto-generate policies from compliance framework
let generator = PolicyGenerator::new();

let gdpr_policies = generator.generate_from_framework(
    ComplianceFramework::GDPR,
    PolicyGenerationOptions {
        enforcement_mode: EnforcementMode::Monitor,
        scope: PolicyScope::Global,
    }
).await?;

for policy in gdpr_policies {
    policy_manager.create_policy(policy).await?;
}
```

### Automated Compliance Testing

```rust
// Run compliance tests
let tester = ComplianceTester::new(policy_manager, policy_engine);

let test_results = tester.run_tests().await?;

println!("Compliance Test Results:");
println!("Total Tests: {}", test_results.total_tests);
println!("Passed: {}", test_results.passed);
println!("Failed: {}", test_results.failed);

if test_results.failed > 0 {
    println!("\nFailed Tests:");
    for failure in test_results.failures {
        println!("  - {}: {}", failure.policy_name, failure.reason);
    }
}
```

### CI/CD Integration

```yaml
# .github/workflows/compliance.yml
name: Compliance Checks

on:
  pull_request:
  push:
    branches: [main]

jobs:
  compliance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Run Policy Tests
        run: cargo test --test policy_tests

      - name: Validate Policies
        run: cargo run --bin validate-policies

      - name: Check Compliance
        run: cargo run --bin compliance-check

      - name: Generate Report
        run: cargo run --bin compliance-report --output compliance_report.pdf

      - name: Upload Report
        uses: actions/upload-artifact@v2
        with:
          name: compliance-report
          path: compliance_report.pdf
```

## Resources

### Documentation
- Policy API Reference: `/docs/api/policy-api.md`
- Policy Language Specification: `/docs/compliance/policy-language.md`
- Compliance Frameworks: `/docs/compliance/frameworks/`

### Tools
- Policy Editor: Web UI at `/policies/editor`
- Policy Validator: `cargo run --bin validate-policy`
- Policy Simulator: Web UI at `/policies/simulator`

### Support
- **Email**: policy@llm-cost-ops.io
- **Documentation**: https://docs.llm-cost-ops.io/policy-management
- **Training**: Monthly policy management workshops

---

**Last Updated**: November 2024
**Version**: 1.0.0
**Reviewed By**: Compliance Team
