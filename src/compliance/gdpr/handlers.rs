// GDPR API Handlers

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Removed unused GDPR error imports
use super::repository::GdprRepository;
use super::service::GdprService;
use super::types::{
    BreachSeverity, ConsentPurpose, DataExportFormat, DataExportRequest, DeletionRequest,
    PersonalDataCategory,
};

/// API Response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(error: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(error),
        }
    }
}

/// Export data request payload
#[derive(Debug, Deserialize)]
pub struct ExportDataPayload {
    pub user_id: String,
    pub organization_id: String,
    pub format: DataExportFormat,
    pub categories: Option<Vec<PersonalDataCategory>>,
}

/// Delete data request payload
#[derive(Debug, Deserialize)]
pub struct DeleteDataPayload {
    pub user_id: String,
    pub organization_id: String,
    pub categories: Option<Vec<PersonalDataCategory>>,
    pub reason: String,
    pub retain_audit_log: Option<bool>,
}

/// Consent request payload
#[derive(Debug, Deserialize)]
pub struct ConsentPayload {
    pub user_id: String,
    pub organization_id: String,
    pub purpose: ConsentPurpose,
    pub consent_text: String,
    pub version: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

/// Withdraw consent payload
#[derive(Debug, Deserialize)]
pub struct WithdrawConsentPayload {
    pub user_id: String,
    pub purpose: ConsentPurpose,
}

/// Breach report payload
#[derive(Debug, Deserialize)]
pub struct BreachReportPayload {
    pub breach_type: String,
    pub severity: BreachSeverity,
    pub affected_users: i64,
    pub affected_records: i64,
    pub description: String,
}

/// Export user data (Article 15)
pub async fn export_user_data<R: GdprRepository + 'static>(
    State(service): State<Arc<GdprService<R>>>,
    Json(payload): Json<ExportDataPayload>,
) -> Result<Json<ApiResponse<super::types::DataExportResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let request = DataExportRequest {
        user_id: payload.user_id,
        organization_id: payload.organization_id,
        format: payload.format,
        categories: payload
            .categories
            .unwrap_or_else(|| vec![PersonalDataCategory::All]),
        requested_at: Utc::now(),
        requested_by: "api".to_string(),
    };

    match service.export_user_data(request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )),
    }
}

/// Delete user data (Article 17)
pub async fn delete_user_data<R: GdprRepository + 'static>(
    State(service): State<Arc<GdprService<R>>>,
    Json(payload): Json<DeleteDataPayload>,
) -> Result<Json<ApiResponse<super::types::DeletionResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let request = DeletionRequest {
        user_id: payload.user_id,
        organization_id: payload.organization_id,
        categories: payload
            .categories
            .unwrap_or_else(|| vec![PersonalDataCategory::All]),
        reason: payload.reason,
        requested_at: Utc::now(),
        requested_by: "api".to_string(),
        retain_audit_log: payload.retain_audit_log.unwrap_or(true),
    };

    match service.delete_user_data(request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )),
    }
}

/// Record user consent
pub async fn record_consent<R: GdprRepository + 'static>(
    State(service): State<Arc<GdprService<R>>>,
    Json(payload): Json<ConsentPayload>,
) -> Result<Json<ApiResponse<super::types::ConsentRecord>>, (StatusCode, Json<ApiResponse<()>>)> {
    match service
        .record_consent(
            payload.user_id,
            payload.organization_id,
            payload.purpose,
            payload.consent_text,
            payload.version,
            payload.ip_address,
            payload.user_agent,
        )
        .await
    {
        Ok(consent) => Ok(Json(ApiResponse::success(consent))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )),
    }
}

/// Withdraw user consent
pub async fn withdraw_consent<R: GdprRepository + 'static>(
    State(service): State<Arc<GdprService<R>>>,
    Json(payload): Json<WithdrawConsentPayload>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    match service.withdraw_consent(&payload.user_id, payload.purpose).await {
        Ok(_) => Ok(Json(ApiResponse::success("Consent withdrawn successfully".to_string()))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )),
    }
}

/// Get user consents
pub async fn get_user_consents<R: GdprRepository + 'static>(
    State(service): State<Arc<GdprService<R>>>,
    Path(user_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<super::types::ConsentRecord>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match service.get_user_consents(&user_id).await {
        Ok(consents) => Ok(Json(ApiResponse::success(consents))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )),
    }
}

/// Report data breach (Articles 33-34)
pub async fn report_breach<R: GdprRepository + 'static>(
    State(service): State<Arc<GdprService<R>>>,
    Json(payload): Json<BreachReportPayload>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    let breach = super::types::BreachNotification::new(
        payload.breach_type,
        payload.severity,
        payload.affected_users,
        payload.affected_records,
        payload.description,
    );

    let breach_id = breach.id.clone();

    match service.report_breach(breach).await {
        Ok(_) => Ok(Json(ApiResponse::success(format!(
            "Breach reported successfully. ID: {}",
            breach_id
        )))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )),
    }
}

/// Notify supervisory authority
pub async fn notify_authority<R: GdprRepository + 'static>(
    State(service): State<Arc<GdprService<R>>>,
    Path(breach_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    match service.notify_authority(&breach_id).await {
        Ok(_) => Ok(Json(ApiResponse::success("Authority notified successfully".to_string()))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )),
    }
}

/// Notify affected users
pub async fn notify_users<R: GdprRepository + 'static>(
    State(service): State<Arc<GdprService<R>>>,
    Path(breach_id): Path<String>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    match service.notify_users(&breach_id).await {
        Ok(_) => Ok(Json(ApiResponse::success("Users notified successfully".to_string()))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(e.to_string())),
        )),
    }
}
