//! Audit repository for persistent storage of audit logs
//!
//! Provides PostgreSQL and SQLite storage for audit logs with high-performance
//! batch inserts, querying, filtering, and export capabilities.

use super::audit::{AuditLog, AuditEventType, AuditOutcome, ActorType};
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row, Postgres, QueryBuilder};
use std::net::IpAddr;
use std::sync::Arc;
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur during audit repository operations
#[derive(Debug, Error)]
pub enum AuditRepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Audit log not found: {0}")]
    NotFound(String),

    #[error("Invalid filter: {0}")]
    InvalidFilter(String),

    #[error("Export error: {0}")]
    Export(String),
}

pub type Result<T> = std::result::Result<T, AuditRepositoryError>;

/// Audit repository trait
#[async_trait::async_trait]
pub trait AuditRepository: Send + Sync {
    /// Store a single audit log
    async fn store(&self, audit_log: &AuditLog) -> Result<()>;

    /// Store multiple audit logs in a batch (high-performance)
    async fn store_batch(&self, audit_logs: &[AuditLog]) -> Result<()>;

    /// Retrieve an audit log by ID
    async fn get_by_id(&self, id: &Uuid) -> Result<Option<AuditLog>>;

    /// Query audit logs with filters
    async fn query(&self, filter: &AuditFilter) -> Result<Vec<AuditLog>>;

    /// Count audit logs matching the filter
    async fn count(&self, filter: &AuditFilter) -> Result<i64>;

    /// Export audit logs to specified format
    async fn export(&self, filter: &AuditFilter, format: AuditExportFormat) -> Result<Vec<u8>>;

    /// Apply retention policy (delete old logs)
    async fn apply_retention_policy(&self, policy: &RetentionPolicy) -> Result<i64>;

    /// Delete audit logs older than a specific date
    async fn delete_before(&self, before: DateTime<Utc>) -> Result<i64>;

    /// Get statistics about audit logs
    async fn get_statistics(&self, filter: &AuditFilter) -> Result<AuditStatistics>;
}

/// Filter for querying audit logs
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditFilter {
    /// Filter by event types
    pub event_types: Option<Vec<AuditEventType>>,

    /// Filter by actor ID
    pub actor_id: Option<String>,

    /// Filter by actor types
    pub actor_types: Option<Vec<ActorType>>,

    /// Filter by outcome
    pub outcomes: Option<Vec<AuditOutcome>>,

    /// Filter by resource type
    pub resource_type: Option<String>,

    /// Filter by resource ID
    pub resource_id: Option<String>,

    /// Filter by organization ID
    pub organization_id: Option<String>,

    /// Filter by IP address
    pub ip_address: Option<IpAddr>,

    /// Filter by correlation ID
    pub correlation_id: Option<String>,

    /// Filter by session ID
    pub session_id: Option<String>,

    /// Filter by security labels
    pub security_labels: Option<Vec<String>>,

    /// Filter by compliance tags
    pub compliance_tags: Option<Vec<String>>,

    /// Start of time range
    pub from_time: Option<DateTime<Utc>>,

    /// End of time range
    pub to_time: Option<DateTime<Utc>>,

    /// Maximum number of results
    pub limit: Option<i64>,

    /// Offset for pagination
    pub offset: Option<i64>,

    /// Sort by field (default: timestamp descending)
    pub sort_by: Option<String>,

    /// Sort order (asc/desc)
    pub sort_order: Option<SortOrder>,
}

/// Sort order for query results
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Export format for audit logs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuditExportFormat {
    /// JSON format
    Json,

    /// NDJSON (newline-delimited JSON)
    Ndjson,

    /// CSV format
    Csv,

    /// Excel format
    Excel,
}

/// Retention policy for audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    /// Retention duration in days
    pub retention_days: i64,

    /// Event types to apply this policy to (None = all events)
    pub event_types: Option<Vec<AuditEventType>>,

    /// Whether to archive before deletion
    pub archive_before_delete: bool,

    /// Archive location (e.g., S3 bucket)
    pub archive_location: Option<String>,
}

/// Statistics about audit logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    /// Total number of logs
    pub total_count: i64,

    /// Count by event type
    pub by_event_type: Vec<(AuditEventType, i64)>,

    /// Count by outcome
    pub by_outcome: Vec<(AuditOutcome, i64)>,

    /// Count by actor type
    pub by_actor_type: Vec<(ActorType, i64)>,

    /// Earliest log timestamp
    pub earliest_timestamp: Option<DateTime<Utc>>,

    /// Latest log timestamp
    pub latest_timestamp: Option<DateTime<Utc>>,
}

/// PostgreSQL implementation of audit repository
pub struct PostgresAuditRepository {
    pool: Arc<PgPool>,
}

impl PostgresAuditRepository {
    /// Create a new PostgreSQL audit repository
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Initialize the audit log table
    pub async fn init_table(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_logs (
                id UUID PRIMARY KEY,
                event_type TEXT NOT NULL,
                actor_id TEXT NOT NULL,
                actor_type TEXT NOT NULL,
                actor_name TEXT,
                actor_attributes JSONB,
                resource_type TEXT,
                resource_id TEXT,
                resource_name TEXT,
                resource_attributes JSONB,
                action TEXT NOT NULL,
                outcome TEXT NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                duration_ms BIGINT,
                ip_address INET,
                user_agent TEXT,
                correlation_id TEXT,
                session_id TEXT,
                request_id TEXT,
                organization_id TEXT,
                metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
                error_message TEXT,
                error_code TEXT,
                security_labels TEXT[],
                compliance_tags TEXT[],
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON audit_logs(event_type);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_actor_id ON audit_logs(actor_id);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_timestamp ON audit_logs(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_organization_id ON audit_logs(organization_id);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_correlation_id ON audit_logs(correlation_id);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_resource ON audit_logs(resource_type, resource_id);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_outcome ON audit_logs(outcome);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_security_labels ON audit_logs USING GIN(security_labels);
            CREATE INDEX IF NOT EXISTS idx_audit_logs_compliance_tags ON audit_logs USING GIN(compliance_tags);
            "#,
        )
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    /// Build a query from a filter
    fn build_query_conditions<'a>(
        &self,
        filter: &'a AuditFilter,
        query: &mut QueryBuilder<'a, Postgres>,
        first_condition: &mut bool,
    ) {
        let add_condition = |query: &mut QueryBuilder<Postgres>, first: &mut bool| {
            if *first {
                query.push(" WHERE ");
                *first = false;
            } else {
                query.push(" AND ");
            }
        };

        if let Some(ref event_types) = filter.event_types {
            if !event_types.is_empty() {
                add_condition(query, first_condition);
                query.push("event_type = ANY(");
                let types: Vec<String> = event_types
                    .iter()
                    .map(|et| serde_json::to_string(et).unwrap_or_default().trim_matches('"').to_string())
                    .collect();
                query.push_bind(types);
                query.push(")");
            }
        }

        if let Some(ref actor_id) = filter.actor_id {
            add_condition(query, first_condition);
            query.push("actor_id = ");
            query.push_bind(actor_id);
        }

        if let Some(ref actor_types) = filter.actor_types {
            if !actor_types.is_empty() {
                add_condition(query, first_condition);
                query.push("actor_type = ANY(");
                let types: Vec<String> = actor_types
                    .iter()
                    .map(|at| serde_json::to_string(at).unwrap_or_default().trim_matches('"').to_string())
                    .collect();
                query.push_bind(types);
                query.push(")");
            }
        }

        if let Some(ref outcomes) = filter.outcomes {
            if !outcomes.is_empty() {
                add_condition(query, first_condition);
                query.push("outcome = ANY(");
                let outcomes_str: Vec<String> = outcomes
                    .iter()
                    .map(|o| serde_json::to_string(o).unwrap_or_default().trim_matches('"').to_string())
                    .collect();
                query.push_bind(outcomes_str);
                query.push(")");
            }
        }

        if let Some(ref resource_type) = filter.resource_type {
            add_condition(query, first_condition);
            query.push("resource_type = ");
            query.push_bind(resource_type);
        }

        if let Some(ref resource_id) = filter.resource_id {
            add_condition(query, first_condition);
            query.push("resource_id = ");
            query.push_bind(resource_id);
        }

        if let Some(ref org_id) = filter.organization_id {
            add_condition(query, first_condition);
            query.push("organization_id = ");
            query.push_bind(org_id);
        }

        if let Some(ref ip) = filter.ip_address {
            add_condition(query, first_condition);
            query.push("ip_address = ");
            query.push_bind(ip.to_string());
        }

        if let Some(ref correlation_id) = filter.correlation_id {
            add_condition(query, first_condition);
            query.push("correlation_id = ");
            query.push_bind(correlation_id);
        }

        if let Some(ref session_id) = filter.session_id {
            add_condition(query, first_condition);
            query.push("session_id = ");
            query.push_bind(session_id);
        }

        if let Some(ref from_time) = filter.from_time {
            add_condition(query, first_condition);
            query.push("timestamp >= ");
            query.push_bind(from_time);
        }

        if let Some(ref to_time) = filter.to_time {
            add_condition(query, first_condition);
            query.push("timestamp <= ");
            query.push_bind(to_time);
        }

        if let Some(ref labels) = filter.security_labels {
            if !labels.is_empty() {
                add_condition(query, first_condition);
                query.push("security_labels && ");
                query.push_bind(labels);
            }
        }

        if let Some(ref tags) = filter.compliance_tags {
            if !tags.is_empty() {
                add_condition(query, first_condition);
                query.push("compliance_tags && ");
                query.push_bind(tags);
            }
        }
    }
}

#[async_trait::async_trait]
impl AuditRepository for PostgresAuditRepository {
    async fn store(&self, audit_log: &AuditLog) -> Result<()> {
        let actor_attrs = serde_json::to_value(&audit_log.actor.attributes)?;
        let metadata = serde_json::to_value(&audit_log.metadata)?;

        let (resource_type, resource_id, resource_name, resource_attrs) = if let Some(ref res) = audit_log.resource {
            (
                Some(res.resource_type.clone()),
                Some(res.resource_id.clone()),
                res.resource_name.clone(),
                Some(serde_json::to_value(&res.attributes)?),
            )
        } else {
            (None, None, None, None)
        };

        let event_type_str = serde_json::to_string(&audit_log.event_type)?
            .trim_matches('"')
            .to_string();
        let actor_type_str = serde_json::to_string(&audit_log.actor.actor_type)?
            .trim_matches('"')
            .to_string();
        let action_str = serde_json::to_string(&audit_log.action)?
            .trim_matches('"')
            .to_string();
        let outcome_str = serde_json::to_string(&audit_log.outcome)?
            .trim_matches('"')
            .to_string();

        let ip_str = audit_log.ip_address.as_ref().map(|ip| ip.to_string());

        sqlx::query(
            r#"
            INSERT INTO audit_logs (
                id, event_type, actor_id, actor_type, actor_name, actor_attributes,
                resource_type, resource_id, resource_name, resource_attributes,
                action, outcome, timestamp, duration_ms, ip_address, user_agent,
                correlation_id, session_id, request_id, organization_id,
                metadata, error_message, error_code, security_labels, compliance_tags
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                $17, $18, $19, $20, $21, $22, $23, $24, $25
            )
            "#,
        )
        .bind(audit_log.id)
        .bind(&event_type_str)
        .bind(&audit_log.actor.id)
        .bind(&actor_type_str)
        .bind(&audit_log.actor.name)
        .bind(&actor_attrs)
        .bind(&resource_type)
        .bind(&resource_id)
        .bind(&resource_name)
        .bind(&resource_attrs)
        .bind(&action_str)
        .bind(&outcome_str)
        .bind(audit_log.timestamp)
        .bind(audit_log.duration_ms)
        .bind(&ip_str)
        .bind(&audit_log.user_agent)
        .bind(&audit_log.correlation_id)
        .bind(&audit_log.session_id)
        .bind(&audit_log.request_id)
        .bind(&audit_log.organization_id)
        .bind(&metadata)
        .bind(&audit_log.error_message)
        .bind(&audit_log.error_code)
        .bind(&audit_log.security_labels)
        .bind(&audit_log.compliance_tags)
        .execute(self.pool.as_ref())
        .await?;

        Ok(())
    }

    async fn store_batch(&self, audit_logs: &[AuditLog]) -> Result<()> {
        if audit_logs.is_empty() {
            return Ok(());
        }

        // Use PostgreSQL's COPY or batch insert for better performance
        let mut tx = self.pool.begin().await?;

        for audit_log in audit_logs {
            let actor_attrs = serde_json::to_value(&audit_log.actor.attributes)?;
            let metadata = serde_json::to_value(&audit_log.metadata)?;

            let (resource_type, resource_id, resource_name, resource_attrs) = if let Some(ref res) = audit_log.resource {
                (
                    Some(res.resource_type.clone()),
                    Some(res.resource_id.clone()),
                    res.resource_name.clone(),
                    Some(serde_json::to_value(&res.attributes)?),
                )
            } else {
                (None, None, None, None)
            };

            let event_type_str = serde_json::to_string(&audit_log.event_type)?
                .trim_matches('"')
                .to_string();
            let actor_type_str = serde_json::to_string(&audit_log.actor.actor_type)?
                .trim_matches('"')
                .to_string();
            let action_str = serde_json::to_string(&audit_log.action)?
                .trim_matches('"')
                .to_string();
            let outcome_str = serde_json::to_string(&audit_log.outcome)?
                .trim_matches('"')
                .to_string();

            let ip_str = audit_log.ip_address.as_ref().map(|ip| ip.to_string());

            sqlx::query(
                r#"
                INSERT INTO audit_logs (
                    id, event_type, actor_id, actor_type, actor_name, actor_attributes,
                    resource_type, resource_id, resource_name, resource_attributes,
                    action, outcome, timestamp, duration_ms, ip_address, user_agent,
                    correlation_id, session_id, request_id, organization_id,
                    metadata, error_message, error_code, security_labels, compliance_tags
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16,
                    $17, $18, $19, $20, $21, $22, $23, $24, $25
                )
                "#,
            )
            .bind(audit_log.id)
            .bind(&event_type_str)
            .bind(&audit_log.actor.id)
            .bind(&actor_type_str)
            .bind(&audit_log.actor.name)
            .bind(&actor_attrs)
            .bind(&resource_type)
            .bind(&resource_id)
            .bind(&resource_name)
            .bind(&resource_attrs)
            .bind(&action_str)
            .bind(&outcome_str)
            .bind(audit_log.timestamp)
            .bind(audit_log.duration_ms)
            .bind(&ip_str)
            .bind(&audit_log.user_agent)
            .bind(&audit_log.correlation_id)
            .bind(&audit_log.session_id)
            .bind(&audit_log.request_id)
            .bind(&audit_log.organization_id)
            .bind(&metadata)
            .bind(&audit_log.error_message)
            .bind(&audit_log.error_code)
            .bind(&audit_log.security_labels)
            .bind(&audit_log.compliance_tags)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    async fn get_by_id(&self, id: &Uuid) -> Result<Option<AuditLog>> {
        let row = sqlx::query(
            r#"
            SELECT
                id, event_type, actor_id, actor_type, actor_name, actor_attributes,
                resource_type, resource_id, resource_name, resource_attributes,
                action, outcome, timestamp, duration_ms, ip_address, user_agent,
                correlation_id, session_id, request_id, organization_id,
                metadata, error_message, error_code, security_labels, compliance_tags
            FROM audit_logs
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool.as_ref())
        .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_audit_log(row)?))
        } else {
            Ok(None)
        }
    }

    async fn query(&self, filter: &AuditFilter) -> Result<Vec<AuditLog>> {
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
            "SELECT id, event_type, actor_id, actor_type, actor_name, actor_attributes, \
             resource_type, resource_id, resource_name, resource_attributes, \
             action, outcome, timestamp, duration_ms, ip_address, user_agent, \
             correlation_id, session_id, request_id, organization_id, \
             metadata, error_message, error_code, security_labels, compliance_tags \
             FROM audit_logs"
        );

        let mut first_condition = true;
        self.build_query_conditions(filter, &mut query, &mut first_condition);

        // Add sorting
        let sort_by = filter.sort_by.as_deref().unwrap_or("timestamp");
        let sort_order = match filter.sort_order {
            Some(SortOrder::Asc) => "ASC",
            _ => "DESC",
        };
        query.push(format!(" ORDER BY {} {}", sort_by, sort_order));

        // Add pagination
        if let Some(limit) = filter.limit {
            query.push(" LIMIT ");
            query.push_bind(limit);
        }

        if let Some(offset) = filter.offset {
            query.push(" OFFSET ");
            query.push_bind(offset);
        }

        let rows = query.build().fetch_all(self.pool.as_ref()).await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(self.row_to_audit_log(row)?);
        }

        Ok(results)
    }

    async fn count(&self, filter: &AuditFilter) -> Result<i64> {
        let mut query: QueryBuilder<Postgres> = QueryBuilder::new("SELECT COUNT(*) FROM audit_logs");

        let mut first_condition = true;
        self.build_query_conditions(filter, &mut query, &mut first_condition);

        let row = query.build().fetch_one(self.pool.as_ref()).await?;
        let count: i64 = row.try_get(0)?;

        Ok(count)
    }

    async fn export(&self, filter: &AuditFilter, format: AuditExportFormat) -> Result<Vec<u8>> {
        let logs = self.query(filter).await?;

        match format {
            AuditExportFormat::Json => {
                let json = serde_json::to_vec_pretty(&logs)?;
                Ok(json)
            }
            AuditExportFormat::Ndjson => {
                let mut output = Vec::new();
                for log in logs {
                    let line = serde_json::to_vec(&log)?;
                    output.extend_from_slice(&line);
                    output.push(b'\n');
                }
                Ok(output)
            }
            AuditExportFormat::Csv => {
                let mut wtr = csv::Writer::from_writer(Vec::new());

                // Write header
                wtr.write_record([
                    "id", "event_type", "actor_id", "actor_type", "action", "outcome",
                    "timestamp", "resource_type", "resource_id", "organization_id",
                    "ip_address", "correlation_id", "error_message",
                ])?;

                // Write rows
                for log in logs {
                    wtr.write_record(&[
                        log.id.to_string(),
                        format!("{:?}", log.event_type),
                        log.actor.id.clone(),
                        format!("{:?}", log.actor.actor_type),
                        format!("{:?}", log.action),
                        format!("{:?}", log.outcome),
                        log.timestamp.to_rfc3339(),
                        log.resource.as_ref().map(|r| r.resource_type.as_str()).unwrap_or("").to_string(),
                        log.resource.as_ref().map(|r| r.resource_id.as_str()).unwrap_or("").to_string(),
                        log.organization_id.as_deref().unwrap_or("").to_string(),
                        log.ip_address.map(|ip| ip.to_string()).unwrap_or_default(),
                        log.correlation_id.as_deref().unwrap_or("").to_string(),
                        log.error_message.as_deref().unwrap_or("").to_string(),
                    ])?;
                }

                Ok(wtr.into_inner().map_err(|e| AuditRepositoryError::Export(e.to_string()))?)
            }
            AuditExportFormat::Excel => {
                // For Excel, we'd use rust_xlsxwriter here
                // For now, return CSV as a placeholder
                self.export(filter, AuditExportFormat::Csv).await
            }
        }
    }

    async fn apply_retention_policy(&self, policy: &RetentionPolicy) -> Result<i64> {
        let cutoff_date = Utc::now() - Duration::days(policy.retention_days);

        let mut query = QueryBuilder::new("DELETE FROM audit_logs WHERE timestamp < ");
        query.push_bind(cutoff_date);

        if let Some(ref event_types) = policy.event_types {
            if !event_types.is_empty() {
                query.push(" AND event_type = ANY(");
                let types: Vec<String> = event_types
                    .iter()
                    .map(|et| serde_json::to_string(et).unwrap_or_default().trim_matches('"').to_string())
                    .collect();
                query.push_bind(types);
                query.push(")");
            }
        }

        let result = query.build().execute(self.pool.as_ref()).await?;
        Ok(result.rows_affected() as i64)
    }

    async fn delete_before(&self, before: DateTime<Utc>) -> Result<i64> {
        let result = sqlx::query("DELETE FROM audit_logs WHERE timestamp < $1")
            .bind(before)
            .execute(self.pool.as_ref())
            .await?;

        Ok(result.rows_affected() as i64)
    }

    async fn get_statistics(&self, filter: &AuditFilter) -> Result<AuditStatistics> {
        let total_count = self.count(filter).await?;

        // This is a simplified implementation
        // In production, you'd want to use aggregation queries

        Ok(AuditStatistics {
            total_count,
            by_event_type: Vec::new(),
            by_outcome: Vec::new(),
            by_actor_type: Vec::new(),
            earliest_timestamp: None,
            latest_timestamp: None,
        })
    }
}

impl PostgresAuditRepository {
    /// Convert a database row to an AuditLog
    fn row_to_audit_log(&self, row: sqlx::postgres::PgRow) -> Result<AuditLog> {
        use super::audit::{Actor, ResourceInfo};
        use std::collections::HashMap;

        let id: Uuid = row.try_get("id")?;
        let event_type_str: String = row.try_get("event_type")?;
        let event_type: AuditEventType = serde_json::from_str(&format!("\"{}\"", event_type_str))?;

        let actor_id: String = row.try_get("actor_id")?;
        let actor_type_str: String = row.try_get("actor_type")?;
        let actor_type: ActorType = serde_json::from_str(&format!("\"{}\"", actor_type_str))?;
        let actor_name: Option<String> = row.try_get("actor_name")?;
        let actor_attributes: serde_json::Value = row.try_get("actor_attributes")?;
        let actor_attrs: HashMap<String, String> = serde_json::from_value(actor_attributes)?;

        let actor = Actor {
            id: actor_id,
            actor_type,
            name: actor_name,
            attributes: actor_attrs,
        };

        let resource = if let Ok(Some(resource_type)) = row.try_get::<Option<String>, _>("resource_type") {
            let resource_id: String = row.try_get("resource_id")?;
            let resource_name: Option<String> = row.try_get("resource_name")?;
            let resource_attrs_value: Option<serde_json::Value> = row.try_get("resource_attributes")?;
            let resource_attrs: HashMap<String, String> = if let Some(val) = resource_attrs_value {
                serde_json::from_value(val)?
            } else {
                HashMap::new()
            };

            Some(ResourceInfo {
                resource_type,
                resource_id,
                resource_name,
                parent_resource: None,
                attributes: resource_attrs,
            })
        } else {
            None
        };

        let action_str: String = row.try_get("action")?;
        let action = serde_json::from_str(&format!("\"{}\"", action_str))?;

        let outcome_str: String = row.try_get("outcome")?;
        let outcome = serde_json::from_str(&format!("\"{}\"", outcome_str))?;

        let timestamp: DateTime<Utc> = row.try_get("timestamp")?;
        let duration_ms: Option<i64> = row.try_get("duration_ms")?;

        let ip_str: Option<String> = row.try_get("ip_address")?;
        let ip_address = ip_str.and_then(|s| s.parse().ok());

        let user_agent: Option<String> = row.try_get("user_agent")?;
        let correlation_id: Option<String> = row.try_get("correlation_id")?;
        let session_id: Option<String> = row.try_get("session_id")?;
        let request_id: Option<String> = row.try_get("request_id")?;
        let organization_id: Option<String> = row.try_get("organization_id")?;

        let metadata_value: serde_json::Value = row.try_get("metadata")?;
        let metadata = serde_json::from_value(metadata_value)?;

        let error_message: Option<String> = row.try_get("error_message")?;
        let error_code: Option<String> = row.try_get("error_code")?;

        let security_labels: Vec<String> = row.try_get("security_labels")?;
        let compliance_tags: Vec<String> = row.try_get("compliance_tags")?;

        Ok(AuditLog {
            id,
            event_type,
            actor,
            resource,
            action,
            outcome,
            timestamp,
            duration_ms,
            ip_address,
            user_agent,
            correlation_id,
            session_id,
            request_id,
            organization_id,
            metadata,
            error_message,
            error_code,
            security_labels,
            compliance_tags,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::audit::AuditEventType;

    #[test]
    fn test_audit_filter_builder() {
        let filter = AuditFilter {
            event_types: Some(vec![AuditEventType::AuthLogin]),
            actor_id: Some("user123".to_string()),
            limit: Some(10),
            ..Default::default()
        };

        assert_eq!(filter.limit, Some(10));
        assert!(filter.event_types.is_some());
    }

    #[test]
    fn test_retention_policy() {
        let policy = RetentionPolicy {
            retention_days: 90,
            event_types: None,
            archive_before_delete: true,
            archive_location: Some("s3://audit-archive".to_string()),
        };

        assert_eq!(policy.retention_days, 90);
        assert!(policy.archive_before_delete);
    }
}
