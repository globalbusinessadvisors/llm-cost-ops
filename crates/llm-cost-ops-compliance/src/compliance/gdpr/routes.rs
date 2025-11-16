// GDPR API Routes

use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::handlers;
use super::repository::GdprRepository;
use super::service::GdprService;

/// Create GDPR router
pub fn create_gdpr_router<R: GdprRepository + 'static>(
    service: Arc<GdprService<R>>,
) -> Router {
    Router::new()
        // Data Export (Article 15)
        .route("/export", post(handlers::export_user_data::<R>))
        // Data Deletion (Article 17)
        .route("/delete", post(handlers::delete_user_data::<R>))
        // Consent Management
        .route("/consent", post(handlers::record_consent::<R>))
        .route("/consent/withdraw", post(handlers::withdraw_consent::<R>))
        .route("/consent/:user_id", get(handlers::get_user_consents::<R>))
        // Breach Notification (Articles 33-34)
        .route("/breach", post(handlers::report_breach::<R>))
        .route("/breach/:breach_id/notify-authority", post(handlers::notify_authority::<R>))
        .route("/breach/:breach_id/notify-users", post(handlers::notify_users::<R>))
        .with_state(service)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::gdpr::repository::InMemoryGdprRepository;

    #[test]
    fn test_create_router() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let service = Arc::new(GdprService::new(repo));
        let _router = create_gdpr_router(service);
        // Router created successfully
    }
}
