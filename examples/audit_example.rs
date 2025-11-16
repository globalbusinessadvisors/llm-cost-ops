//! Example demonstrating the audit logging system
//!
//! This example shows how to:
//! 1. Set up the audit repository
//! 2. Add audit middleware to Axum
//! 3. Manually log critical operations
//! 4. Query audit logs
//! 5. Export audit reports

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use chrono::{Duration, Utc};
use llm_cost_ops::compliance::{
    create_audit_layer, Actor, ActionType, AuditChanges, AuditEventType as ComplianceAuditEventType,
    AuditExportFormat, AuditFilter, AuditLog, AuditOutcome, AuditRepository,
    AuditRetentionPolicy, PostgresAuditRepository, ResourceInfo,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Application state
#[derive(Clone)]
struct AppState {
    audit_repo: Arc<PostgresAuditRepository>,
}

/// User model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
    role: String,
}

/// Project model
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Project {
    id: String,
    name: String,
    budget: f64,
    status: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/llm_cost_ops".to_string());

    let pool = PgPool::connect(&database_url).await?;
    let pool = Arc::new(pool);

    // Create audit repository
    let audit_repo = Arc::new(PostgresAuditRepository::new(pool.clone()));

    // Initialize audit table
    audit_repo.init_table().await?;

    // Create application state
    let state = AppState {
        audit_repo: audit_repo.clone(),
    };

    // Create audit middleware layer
    let audit_layer = create_audit_layer(audit_repo.clone());

    // Build application router
    let app = Router::new()
        // User routes
        .route("/api/users", get(list_users).post(create_user))
        .route("/api/users/:id", get(get_user).delete(delete_user))
        // Project routes
        .route("/api/projects", get(list_projects).post(create_project))
        .route(
            "/api/projects/:id",
            get(get_project).delete(delete_project),
        )
        .route("/api/projects/:id/budget", post(update_project_budget))
        // Audit routes
        .route("/api/audit/logs", get(query_audit_logs))
        .route("/api/audit/export", get(export_audit_logs))
        .route("/api/audit/stats", get(get_audit_stats))
        // Add audit middleware
        .layer(audit_layer)
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Server listening on {}", addr);

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// User Handlers
// ============================================================================

async fn list_users() -> Json<Vec<User>> {
    // Mock implementation
    Json(vec![])
}

async fn get_user(Path(user_id): Path<String>) -> Result<Json<User>, StatusCode> {
    // Mock implementation
    Ok(Json(User {
        id: user_id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        role: "admin".to_string(),
    }))
}

async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<User>,
) -> Result<Json<User>, StatusCode> {
    // Create the user (mock)
    let created_user = user.clone();

    // Manual audit logging for critical operation
    let actor = Actor::system("user-service".to_string());
    let resource = ResourceInfo::new("user".to_string(), user.id.clone())
        .with_name(user.name.clone());

    let audit_log = AuditLog::new(
        ComplianceAuditEventType::UserCreated,
        actor,
        ActionType::Create,
        AuditOutcome::Success,
    )
    .with_resource(resource)
    .add_metadata("email".to_string(), serde_json::json!(user.email))
    .add_metadata("role".to_string(), serde_json::json!(user.role))
    .add_security_label("pii".to_string())
    .add_compliance_tag("SOC2".to_string());

    // Store audit log
    if let Err(e) = state.audit_repo.store(&audit_log).await {
        tracing::error!("Failed to store audit log: {}", e);
    }

    Ok(Json(created_user))
}

async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    // Delete the user (mock)

    // Manual audit logging for sensitive operation
    let actor = Actor::system("user-service".to_string());
    let resource = ResourceInfo::new("user".to_string(), user_id);

    let audit_log = AuditLog::new(
        ComplianceAuditEventType::UserDeleted,
        actor,
        ActionType::Delete,
        AuditOutcome::Success,
    )
    .with_resource(resource)
    .add_security_label("pii".to_string())
    .add_compliance_tag("GDPR".to_string());

    if let Err(e) = state.audit_repo.store(&audit_log).await {
        tracing::error!("Failed to store audit log: {}", e);
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Project Handlers
// ============================================================================

async fn list_projects() -> Json<Vec<Project>> {
    Json(vec![])
}

async fn get_project(Path(project_id): Path<String>) -> Result<Json<Project>, StatusCode> {
    Ok(Json(Project {
        id: project_id,
        name: "AI Research".to_string(),
        budget: 10000.0,
        status: "active".to_string(),
    }))
}

async fn create_project(
    State(state): State<AppState>,
    Json(project): Json<Project>,
) -> Result<Json<Project>, StatusCode> {
    let created_project = project.clone();

    let actor = Actor::system("project-service".to_string());
    let resource = ResourceInfo::new("project".to_string(), project.id.clone())
        .with_name(project.name.clone());

    let audit_log = AuditLog::new(
        ComplianceAuditEventType::DataCreate,
        actor,
        ActionType::Create,
        AuditOutcome::Success,
    )
    .with_resource(resource)
    .add_metadata("budget".to_string(), serde_json::json!(project.budget))
    .add_metadata("status".to_string(), serde_json::json!(project.status));

    if let Err(e) = state.audit_repo.store(&audit_log).await {
        tracing::error!("Failed to store audit log: {}", e);
    }

    Ok(Json(created_project))
}

async fn delete_project(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let actor = Actor::system("project-service".to_string());
    let resource = ResourceInfo::new("project".to_string(), project_id);

    let audit_log = AuditLog::new(
        ComplianceAuditEventType::DataDelete,
        actor,
        ActionType::Delete,
        AuditOutcome::Success,
    )
    .with_resource(resource)
    .add_security_label("confidential".to_string());

    if let Err(e) = state.audit_repo.store(&audit_log).await {
        tracing::error!("Failed to store audit log: {}", e);
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
struct BudgetUpdate {
    new_budget: f64,
}

async fn update_project_budget(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(update): Json<BudgetUpdate>,
) -> Result<Json<Project>, StatusCode> {
    // Mock: get old budget
    let old_budget = 10000.0;
    let new_budget = update.new_budget;

    // Update the project (mock)
    let updated_project = Project {
        id: project_id.clone(),
        name: "AI Research".to_string(),
        budget: new_budget,
        status: "active".to_string(),
    };

    // Track the change with AuditChanges
    let mut before = HashMap::new();
    before.insert("budget".to_string(), serde_json::json!(old_budget));

    let mut after = HashMap::new();
    after.insert("budget".to_string(), serde_json::json!(new_budget));

    let changes = AuditChanges::new(before, after);

    let actor = Actor::system("project-service".to_string());
    let resource = ResourceInfo::new("project".to_string(), project_id);

    let audit_log = AuditLog::new(
        ComplianceAuditEventType::DataUpdate,
        actor,
        ActionType::Update,
        AuditOutcome::Success,
    )
    .with_resource(resource)
    .with_changes(changes)
    .add_compliance_tag("SOC2".to_string());

    if let Err(e) = state.audit_repo.store(&audit_log).await {
        tracing::error!("Failed to store audit log: {}", e);
    }

    Ok(Json(updated_project))
}

// ============================================================================
// Audit Query Handlers
// ============================================================================

#[derive(Debug, Deserialize)]
struct QueryParams {
    actor_id: Option<String>,
    event_type: Option<String>,
    from_days: Option<i64>,
    limit: Option<i64>,
}

async fn query_audit_logs(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<QueryParams>,
) -> Result<Json<Vec<AuditLog>>, StatusCode> {
    let from_time = params
        .from_days
        .map(|days| Utc::now() - Duration::days(days));

    let filter = AuditFilter {
        actor_id: params.actor_id,
        from_time,
        to_time: Some(Utc::now()),
        limit: params.limit,
        ..Default::default()
    };

    match state.audit_repo.query(&filter).await {
        Ok(logs) => Ok(Json(logs)),
        Err(e) => {
            tracing::error!("Failed to query audit logs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn export_audit_logs(
    State(state): State<AppState>,
) -> Result<Vec<u8>, StatusCode> {
    let filter = AuditFilter {
        from_time: Some(Utc::now() - Duration::days(30)),
        to_time: Some(Utc::now()),
        ..Default::default()
    };

    match state
        .audit_repo
        .export(&filter, AuditExportFormat::Csv)
        .await
    {
        Ok(csv_data) => Ok(csv_data),
        Err(e) => {
            tracing::error!("Failed to export audit logs: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_audit_stats(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let filter = AuditFilter {
        from_time: Some(Utc::now() - Duration::days(7)),
        to_time: Some(Utc::now()),
        ..Default::default()
    };

    match state.audit_repo.get_statistics(&filter).await {
        Ok(stats) => Ok(Json(serde_json::json!({
            "total_count": stats.total_count,
            "earliest_timestamp": stats.earliest_timestamp,
            "latest_timestamp": stats.latest_timestamp,
        }))),
        Err(e) => {
            tracing::error!("Failed to get audit statistics: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// ============================================================================
// Background Jobs
// ============================================================================

/// Run retention policy cleanup (would be scheduled via cron)
async fn run_retention_cleanup(audit_repo: Arc<PostgresAuditRepository>) {
    // Standard retention: 90 days
    let policy = AuditRetentionPolicy {
        retention_days: 90,
        event_types: None,
        archive_before_delete: true,
        archive_location: Some("s3://audit-archive/".to_string()),
    };

    match audit_repo.apply_retention_policy(&policy).await {
        Ok(deleted) => {
            tracing::info!("Retention policy deleted {} audit logs", deleted);
        }
        Err(e) => {
            tracing::error!("Failed to apply retention policy: {}", e);
        }
    }
}
