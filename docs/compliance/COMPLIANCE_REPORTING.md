# Compliance Reporting Guide

## Overview

This guide covers the compliance reporting capabilities of the LLM Cost Ops platform, including available report types, scheduling, export formats, and dashboard usage. Compliance reports provide evidence of adherence to regulatory requirements and organizational policies.

## Table of Contents

1. [Available Reports](#available-reports)
2. [Scheduling Reports](#scheduling-reports)
3. [Export Formats](#export-formats)
4. [Dashboard Usage](#dashboard-usage)
5. [Custom Reports](#custom-reports)
6. [Integration](#integration)
7. [Best Practices](#best-practices)

## Available Reports

### 1. Audit Trail Report

Comprehensive log of all system activities and user actions.

**Purpose**: Demonstrate accountability and traceability for compliance audits.

**Includes**:
- Authentication events (logins, logouts, failures)
- Authorization decisions (access granted/denied)
- Resource operations (create, read, update, delete)
- User and role management activities
- API key operations
- Data exports and imports
- System configuration changes
- Security incidents

**API Endpoint**:
```http
POST /api/v1/reports/audit-trail
Authorization: Bearer {token}
Content-Type: application/json

{
  "start_date": "2024-10-01T00:00:00Z",
  "end_date": "2024-10-31T23:59:59Z",
  "event_types": ["auth_login", "auth_failed", "access_denied"],
  "user_id": "user_12345",
  "severity": "warning",
  "format": "excel"
}
```

**Code Example**:
```rust
use llm_cost_ops::export::reports::{ReportGenerator, ReportRequest, ReportType};
use chrono::{Utc, Duration};

let generator = ReportGenerator::new();

let request = ReportRequest {
    report_type: ReportType::Audit,
    start_date: Utc::now() - Duration::days(30),
    end_date: Utc::now(),
    organization_id: Some("org_12345".to_string()),
    filters: ReportFilters {
        event_types: Some(vec![
            "auth_login",
            "auth_failed",
            "access_denied",
        ]),
        severity: Some("warning"),
        ..Default::default()
    },
};

let report = generator.generate(request).await?;

// Export to Excel
let excel_data = report.export(ExportFormat::Excel)?;
std::fs::write("audit_trail.xlsx", excel_data)?;
```

### 2. Access Control Report

Summary of user access rights, permissions, and access reviews.

**Purpose**: Demonstrate proper access control implementation (SOC 2 CC6).

**Includes**:
- User list with roles and permissions
- Recent access provisioning events
- Access deprovisioning events
- Failed access attempts
- Privileged user activities
- Access review status
- Separation of duties validation

**Report Structure**:
```
# Access Control Report

## Summary
- Total Users: 250
- Active Users: 245
- Inactive Users: 5
- Admin Users: 12
- Service Accounts: 8

## Access Reviews
- Last Review Date: 2024-10-15
- Next Review Due: 2025-01-15
- Users Requiring Review: 3

## Recent Changes
- Users Added: 5
- Users Removed: 2
- Role Changes: 8
- Permission Updates: 12

## Failed Access Attempts
- Total Failures: 45
- Unique Users: 12
- Critical Failures: 3

## Privileged Access
- Admin Logins: 67
- System Config Changes: 12
- Permission Grants: 8
```

**Code Example**:
```rust
let request = ReportRequest {
    report_type: ReportType::AccessControl,
    start_date: Utc::now() - Duration::days(30),
    end_date: Utc::now(),
    organization_id: Some("org_12345".to_string()),
    filters: ReportFilters::default(),
};

let report = generator.generate(request).await?;
```

### 3. Data Protection Report

Overview of data protection measures and compliance.

**Purpose**: Demonstrate GDPR and data protection compliance.

**Includes**:
- Encryption status (at rest and in transit)
- Data classification summary
- Data retention compliance
- Personal data processing records
- Data subject requests (access, deletion, portability)
- Data breach incidents
- Backup and recovery status

**Report Metrics**:
```rust
#[derive(Debug, Serialize)]
pub struct DataProtectionReport {
    pub encryption_coverage: EncryptionCoverage,
    pub retention_compliance: RetentionCompliance,
    pub data_subject_requests: DataSubjectRequests,
    pub data_breaches: Vec<DataBreachIncident>,
    pub backup_status: BackupStatus,
}

#[derive(Debug, Serialize)]
pub struct EncryptionCoverage {
    pub total_data_stores: u32,
    pub encrypted_at_rest: u32,
    pub encrypted_in_transit: u32,
    pub encryption_percentage: f64,
}

#[derive(Debug, Serialize)]
pub struct DataSubjectRequests {
    pub access_requests: u32,
    pub deletion_requests: u32,
    pub portability_requests: u32,
    pub average_response_time_days: f64,
    pub compliance_rate: f64,  // % completed within SLA
}
```

### 4. Security Compliance Report

Summary of security controls and compliance status.

**Purpose**: Demonstrate SOC 2 security compliance.

**Includes**:
- Security control effectiveness
- Vulnerability management
- Incident response metrics
- Security training completion
- MFA adoption rate
- Password policy compliance
- Security patch status
- Penetration test results

**Code Example**:
```rust
let request = ReportRequest {
    report_type: ReportType::SecurityCompliance,
    start_date: Utc::now() - Duration::days(90),
    end_date: Utc::now(),
    organization_id: Some("org_12345".to_string()),
    filters: ReportFilters::default(),
};

let report = generator.generate(request).await?;

println!("Security Compliance Score: {}/100", report.summary.compliance_score);
```

### 5. Policy Compliance Report

Status of policy compliance across the organization.

**Purpose**: Track policy violations and remediation.

**Includes**:
- Active policies
- Policy compliance rate
- Policy violations (by severity)
- Top violated policies
- Remediation status
- Trend analysis
- Policy effectiveness metrics

**Report Format**:
```json
{
  "report_id": "rpt_policy_123",
  "generated_at": "2024-11-16T10:00:00Z",
  "period": {
    "start": "2024-10-01T00:00:00Z",
    "end": "2024-10-31T23:59:59Z"
  },
  "summary": {
    "total_policies": 45,
    "active_policies": 42,
    "compliance_score": 87.5,
    "total_violations": 125,
    "critical_violations": 5,
    "warning_violations": 45,
    "info_violations": 75
  },
  "top_violations": [
    {
      "policy_name": "Password Complexity",
      "violation_count": 23,
      "remediated": 18,
      "open": 5
    }
  ],
  "trends": {
    "compliance_trend": "improving",
    "violation_trend": "decreasing"
  }
}
```

### 6. User Activity Report

Detailed user activity and behavior analysis.

**Purpose**: Monitor user behavior and detect anomalies.

**Includes**:
- User login patterns
- API usage by user
- Resource access patterns
- Data export activities
- Failed authentication attempts
- Suspicious activities
- User session statistics

### 7. Cost and Usage Report

Financial and usage metrics for compliance and budgeting.

**Purpose**: Budget compliance and cost transparency.

**Includes**:
- Total costs by organization
- Cost by provider and model
- Usage statistics (tokens, requests)
- Budget vs. actual spending
- Cost trends
- Anomaly detection
- Forecast vs. actual comparison

**Code Example**:
```rust
let request = ReportRequest {
    report_type: ReportType::Cost,
    start_date: Utc::now() - Duration::days(30),
    end_date: Utc::now(),
    organization_id: Some("org_12345".to_string()),
    filters: ReportFilters {
        provider: Some("openai".to_string()),
        ..Default::default()
    },
};

let report = generator.generate(request).await?;

// Export to CSV for analysis
let csv_data = report.export(ExportFormat::Csv)?;
```

### 8. Executive Summary Report

High-level overview for executive stakeholders.

**Purpose**: Executive visibility into compliance posture.

**Includes**:
- Overall compliance score
- Key metrics dashboard
- Critical findings
- Trend indicators
- Risk summary
- Recommendations
- Action items

**Report Template**:
```markdown
# Executive Compliance Summary
**Period**: October 2024

## Overall Compliance Score: 92/100

### Key Metrics
- Active Policies: 42
- Policy Compliance: 95%
- Security Score: 94/100
- Data Protection Score: 91/100

### Highlights
✅ No critical security incidents
✅ All data subject requests completed within SLA
✅ 100% MFA adoption for admin users
✅ Zero unpatched critical vulnerabilities

### Areas for Improvement
⚠️ 5 pending access reviews (due this month)
⚠️ Password policy violations trending up
⚠️ 3 policies in monitor mode need enforcement

### Action Items
1. Complete pending access reviews by month-end
2. Increase password awareness training
3. Enable enforcement for monitor-mode policies

### Trends
📈 Compliance score up 3% from last month
📉 Policy violations down 12% from last month
```

## Scheduling Reports

### Report Scheduler

```rust
use llm_cost_ops::export::scheduler::{ReportScheduler, ScheduledReportConfig};
use llm_cost_ops::export::delivery::{DeliveryMethod, EmailDelivery};

let scheduler = ReportScheduler::new(db_pool);

// Schedule daily audit report
let audit_schedule = ScheduledReportConfig {
    id: "schedule_daily_audit".to_string(),
    name: "Daily Audit Report".to_string(),
    report_type: ReportType::Audit,
    schedule: "0 9 * * *".to_string(),  // Every day at 9 AM
    filters: ReportFilters::default(),
    export_format: ExportFormat::Excel,
    delivery: DeliveryMethod::Email(EmailDelivery {
        recipients: vec![
            "security@example.com".to_string(),
            "compliance@example.com".to_string(),
        ],
        subject: "Daily Audit Report - {date}".to_string(),
        template: "audit_report_email".to_string(),
        attach_report: true,
    }),
    enabled: true,
    timezone: "America/New_York".to_string(),
};

scheduler.create_schedule(audit_schedule).await?;
```

### Schedule Configuration

```toml
# config/report_schedules.toml

[[schedules]]
name = "Weekly Security Report"
report_type = "security_compliance"
cron = "0 9 * * 1"  # Every Monday at 9 AM
format = "pdf"
delivery = "email"
recipients = ["security@example.com"]
enabled = true

[[schedules]]
name = "Monthly Executive Summary"
report_type = "executive_summary"
cron = "0 9 1 * *"  # First day of month at 9 AM
format = "pdf"
delivery = "email"
recipients = ["executives@example.com"]
enabled = true

[[schedules]]
name = "Quarterly Compliance Report"
report_type = "policy_compliance"
cron = "0 9 1 */3 *"  # First day of quarter at 9 AM
format = "excel"
delivery = "email"
recipients = ["compliance@example.com", "auditors@example.com"]
enabled = true
```

### Cron Schedule Examples

```
# Every hour
"0 * * * *"

# Every 6 hours
"0 */6 * * *"

# Every day at 2 AM
"0 2 * * *"

# Every Monday at 9 AM
"0 9 * * 1"

# First day of month at midnight
"0 0 1 * *"

# Every quarter (Jan 1, Apr 1, Jul 1, Oct 1)
"0 0 1 1,4,7,10 *"

# Last day of month at 11:59 PM
"59 23 L * *"

# Weekdays at 8 AM
"0 8 * * 1-5"
```

### On-Demand Reports

```rust
// Generate report immediately
let request = ReportRequest {
    report_type: ReportType::Audit,
    start_date: Utc::now() - Duration::days(7),
    end_date: Utc::now(),
    organization_id: Some("org_12345".to_string()),
    filters: ReportFilters::default(),
};

let report = generator.generate(request).await?;

// Deliver via email
let delivery = EmailDelivery {
    recipients: vec!["manager@example.com".to_string()],
    subject: "On-Demand Audit Report".to_string(),
    template: "audit_report_email".to_string(),
    attach_report: true,
};

delivery_coordinator.deliver_email(&report, &delivery).await?;
```

## Export Formats

### 1. PDF Format

Professional report format suitable for executives and auditors.

**Features**:
- Executive summary page
- Table of contents
- Charts and graphs
- Formatted tables
- Page numbering
- Header/footer with metadata

**Code Example**:
```rust
let exporter = PdfExporter::new();

let pdf_options = PdfExportOptions {
    include_toc: true,
    include_charts: true,
    page_size: PageSize::Letter,
    orientation: Orientation::Portrait,
    font_family: "Arial".to_string(),
};

let pdf_data = exporter.export(&report, pdf_options)?;
std::fs::write("compliance_report.pdf", pdf_data)?;
```

### 2. Excel Format

Detailed data format for analysis and manipulation.

**Features**:
- Multiple worksheets
- Formatted data tables
- Pivot tables
- Charts
- Cell formulas
- Conditional formatting

**Worksheets**:
1. Summary
2. Detailed Data
3. Charts
4. Pivot Tables
5. Raw Data

**Code Example**:
```rust
let exporter = ExcelExporter::new();

let excel_options = ExcelExportOptions {
    include_charts: true,
    include_pivot_tables: true,
    freeze_header_row: true,
    auto_filter: true,
};

let excel_data = exporter.export(&report, excel_options)?;
std::fs::write("compliance_data.xlsx", excel_data)?;
```

### 3. CSV Format

Simple format for data import and analysis.

**Features**:
- UTF-8 encoding
- Comma-separated values
- Header row
- Quoted fields

**Code Example**:
```rust
let exporter = CsvExporter::new();
let csv_data = exporter.export(&report)?;
std::fs::write("compliance_data.csv", csv_data)?;
```

### 4. JSON Format

Structured data format for programmatic access.

**Features**:
- Hierarchical structure
- Machine-readable
- API-friendly
- Complete data preservation

**Code Example**:
```rust
let json_data = serde_json::to_string_pretty(&report)?;
std::fs::write("compliance_data.json", json_data)?;
```

### 5. HTML Format

Web-friendly format for viewing in browsers.

**Features**:
- Interactive tables
- Embedded charts
- Responsive design
- Printable

**Code Example**:
```rust
let exporter = HtmlExporter::new();
let html_data = exporter.export(&report)?;
std::fs::write("compliance_report.html", html_data)?;
```

## Dashboard Usage

### Compliance Dashboard

Access the compliance dashboard at: `https://your-instance.com/compliance/dashboard`

**Features**:
- Real-time compliance score
- Policy compliance status
- Recent violations
- Trend charts
- Quick access to reports

### Dashboard Components

#### 1. Compliance Score Widget
```
┌─────────────────────────────┐
│   Compliance Score          │
│                             │
│         92/100              │
│                             │
│   ████████████████░░        │
│                             │
│   ▲ +3% from last month    │
└─────────────────────────────┘
```

#### 2. Policy Status Widget
```
┌─────────────────────────────┐
│   Policy Compliance         │
│                             │
│   Active: 42                │
│   Compliant: 39 (93%)       │
│   Violations: 125           │
│                             │
│   Critical:  █ 5            │
│   Warning:   ███ 45         │
│   Info:      ███████ 75     │
└─────────────────────────────┘
```

#### 3. Recent Activity Widget
```
┌─────────────────────────────┐
│   Recent Activity           │
│                             │
│   🔴 Critical violation     │
│       Password policy       │
│       2 minutes ago         │
│                             │
│   🟡 Warning violation      │
│       Budget exceeded       │
│       15 minutes ago        │
│                             │
│   ✅ Policy updated         │
│       MFA requirement       │
│       1 hour ago            │
└─────────────────────────────┘
```

### Dashboard API

```http
# Get dashboard data
GET /api/v1/compliance/dashboard
Authorization: Bearer {token}

# Response
{
  "compliance_score": 92,
  "active_policies": 42,
  "violations_24h": 15,
  "critical_violations": 2,
  "trends": {
    "compliance": "improving",
    "violations": "decreasing"
  },
  "recent_activity": [...]
}
```

### Custom Dashboards

```rust
// Create custom dashboard
let dashboard = DashboardBuilder::new()
    .add_widget(Widget::ComplianceScore {
        period: Period::Last30Days,
    })
    .add_widget(Widget::PolicyStatus {
        policy_types: vec![PolicyType::Security, PolicyType::DataProtection],
    })
    .add_widget(Widget::ViolationTrend {
        period: Period::Last90Days,
        group_by: GroupBy::Week,
    })
    .add_widget(Widget::AuditActivity {
        event_types: vec!["security_incident", "access_denied"],
        limit: 10,
    })
    .build()?;

dashboard.save("security_dashboard").await?;
```

## Custom Reports

### Report Builder

```rust
use llm_cost_ops::export::builder::{ReportBuilder, Section, Chart};

let report = ReportBuilder::new()
    .title("Custom Compliance Report")
    .description("Tailored compliance metrics for Q4 2024")
    .add_section(Section {
        title: "Executive Summary".to_string(),
        content: SectionContent::Summary {
            metrics: vec![
                ("Compliance Score", "92/100"),
                ("Active Policies", "42"),
                ("Violations", "125"),
            ],
        },
    })
    .add_section(Section {
        title: "Security Metrics".to_string(),
        content: SectionContent::Table {
            headers: vec!["Metric", "Value", "Target", "Status"],
            rows: vec![
                vec!["MFA Adoption", "98%", "95%", "✅"],
                vec!["Password Compliance", "87%", "90%", "⚠️"],
                vec!["Patch Coverage", "100%", "100%", "✅"],
            ],
        },
    })
    .add_section(Section {
        title: "Violation Trends".to_string(),
        content: SectionContent::Chart {
            chart_type: ChartType::Line,
            data: violation_trend_data,
        },
    })
    .build()?;

// Export to PDF
let pdf_data = report.export(ExportFormat::Pdf)?;
```

### Custom Queries

```rust
// Build custom report from audit data
let custom_report = CustomReportBuilder::new()
    .name("Failed Authentication Analysis")
    .query(AuditQuery::new()
        .with_event_type(AuditEventType::AuthFailed)
        .with_time_range(start_date, end_date)
    )
    .aggregate(vec![
        Aggregation::CountBy("user_email"),
        Aggregation::CountBy("ip_address"),
        Aggregation::GroupBy("hour_of_day"),
    ])
    .visualize(vec![
        Chart::bar("Failed Logins by User"),
        Chart::line("Failed Logins Over Time"),
        Chart::heatmap("Failed Logins by Hour"),
    ])
    .build()?;

let report = custom_report.generate().await?;
```

## Integration

### SIEM Integration

```rust
// Export to SIEM (Splunk, ELK, etc.)
let siem_exporter = SiemExporter::new(SiemType::Splunk {
    url: "https://splunk.example.com:8088".to_string(),
    token: "your-hec-token".to_string(),
});

let report = generator.generate(request).await?;
siem_exporter.export(&report).await?;
```

### GRC Tool Integration

```rust
// Export to GRC platform
let grc_exporter = GrcExporter::new(GrcPlatform::ServiceNow {
    instance: "your-instance.service-now.com".to_string(),
    credentials: credentials,
});

grc_exporter.export_compliance_report(&report).await?;
```

### BI Tool Integration

```rust
// Export to BI platform (Tableau, Power BI)
let bi_exporter = BiExporter::new(BiPlatform::Tableau);

// Export as Tableau data source
bi_exporter.export_data_source(&report, "compliance_metrics.tds")?;
```

## Best Practices

### 1. Regular Reporting Schedule

```toml
# Recommended schedule
[reports]
daily = ["audit_trail", "security_incidents"]
weekly = ["policy_compliance", "access_control"]
monthly = ["data_protection", "executive_summary"]
quarterly = ["comprehensive_compliance", "risk_assessment"]
annual = ["soc2_evidence", "gdpr_compliance"]
```

### 2. Report Retention

```rust
// Retain reports for compliance
let retention_policy = ReportRetentionPolicy {
    audit_reports: Duration::days(2555),  // 7 years
    compliance_reports: Duration::days(2555),  // 7 years
    operational_reports: Duration::days(365),  // 1 year
};

scheduler.apply_retention_policy(retention_policy).await?;
```

### 3. Access Control

```rust
// Restrict report access
let report_acl = ReportAccessControl {
    audit_reports: vec![Role::Auditor, Role::Admin],
    compliance_reports: vec![Role::ComplianceOfficer, Role::Admin],
    security_reports: vec![Role::SecurityTeam, Role::Admin],
    executive_reports: vec![Role::Executive, Role::Admin],
};
```

### 4. Report Validation

```rust
// Validate report data
let validator = ReportValidator::new();

let validation = validator.validate(&report)?;

if !validation.is_valid {
    for error in validation.errors {
        eprintln!("Validation Error: {}", error);
    }
    return Err(ReportError::ValidationFailed);
}
```

### 5. Audit Trail for Reports

```rust
// Log report generation
let audit_event = AuditEvent::new(
    AuditEventType::ReportGenerated,
    format!("Generated {} report", report.report_type)
)
.with_user(user_id, Some(user_email))
.with_metadata("report_id", json!(report.id))
.with_metadata("report_type", json!(report.report_type))
.with_metadata("export_format", json!(export_format));

audit_logger.log(audit_event).await?;
```

## Resources

### Documentation
- Report API Reference: `/docs/api/reports-api.md`
- Dashboard Guide: `/docs/compliance/dashboard-guide.md`
- Report Templates: `/docs/compliance/report-templates/`

### Tools
- Report Builder: Web UI at `/reports/builder`
- Schedule Manager: Web UI at `/reports/schedules`
- Report Viewer: Web UI at `/reports/viewer`

### Support
- **Email**: reports@llm-cost-ops.io
- **Documentation**: https://docs.llm-cost-ops.io/reporting
- **Training**: Monthly reporting webinars

---

**Last Updated**: November 2024
**Version**: 1.0.0
**Reviewed By**: Compliance Team
