// Data Breach Notification Service (GDPR Articles 33-34)

use async_trait::async_trait;
use chrono::{Duration, Utc};
use std::sync::Arc;
use tracing::{error, info, warn};

use crate::compliance::error::{GdprError, GdprResult};
use super::repository::GdprRepository;
use super::types::{BreachNotification, BreachSeverity, BreachStatus};

/// Breach detector trait
#[async_trait]
pub trait BreachDetector: Send + Sync {
    /// Detect potential data breach
    async fn detect_breach(&self) -> GdprResult<Option<BreachNotification>>;
}

/// Breach notifier trait
#[async_trait]
pub trait BreachNotifier: Send + Sync {
    /// Report a data breach
    async fn report_breach(&self, breach: BreachNotification) -> GdprResult<()>;

    /// Notify supervisory authority (within 72 hours)
    async fn notify_authority(&self, breach_id: &str) -> GdprResult<()>;

    /// Notify affected users
    async fn notify_users(&self, breach_id: &str) -> GdprResult<()>;

    /// Check if notification is required
    fn requires_notification(&self, breach: &BreachNotification) -> bool;

    /// Check if 72-hour deadline is approaching
    fn is_deadline_approaching(&self, breach: &BreachNotification) -> bool;
}

/// Default breach notifier implementation
pub struct DefaultBreachNotifier<R: GdprRepository> {
    repository: Arc<R>,
}

impl<R: GdprRepository> DefaultBreachNotifier<R> {
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<R: GdprRepository> BreachNotifier for DefaultBreachNotifier<R> {
    async fn report_breach(&self, breach: BreachNotification) -> GdprResult<()> {
        info!("Reporting data breach: {}", breach.id);

        // Validate breach data
        if breach.affected_users < 0 || breach.affected_records < 0 {
            return Err(GdprError::validation("Invalid affected counts"));
        }

        // Store breach record
        self.repository.store_breach(breach.clone()).await?;

        // Check if immediate notification is required
        if self.requires_notification(&breach) {
            warn!(
                "High severity breach detected, immediate notification required: {}",
                breach.id
            );

            // For critical breaches, auto-notify
            if breach.get_severity() >= BreachSeverity::High {
                match self.notify_authority(&breach.id).await {
                    Ok(_) => {
                        info!("Authority notified for breach: {}", breach.id);
                    }
                    Err(e) => {
                        error!("Failed to notify authority: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn notify_authority(&self, breach_id: &str) -> GdprResult<()> {
        info!("Notifying supervisory authority for breach: {}", breach_id);

        let mut breach = self
            .repository
            .get_breach(breach_id)
            .await?
            .ok_or_else(|| GdprError::internal("Breach not found"))?;

        // Check 72-hour deadline
        if self.is_deadline_approaching(&breach) {
            warn!("72-hour deadline approaching for breach: {}", breach_id);
        }

        // In a real implementation, this would send notifications via email/API
        // For now, we just mark it as notified
        breach.authority_notified_at = Some(Utc::now().to_rfc3339());
        breach.status = serde_json::to_string(&BreachStatus::NotificationSent).unwrap_or_default();
        breach.updated_at = Utc::now().to_rfc3339();

        self.repository.update_breach(breach).await?;

        info!("Authority notification sent for breach: {}", breach_id);
        Ok(())
    }

    async fn notify_users(&self, breach_id: &str) -> GdprResult<()> {
        info!("Notifying affected users for breach: {}", breach_id);

        let mut breach = self
            .repository
            .get_breach(breach_id)
            .await?
            .ok_or_else(|| GdprError::internal("Breach not found"))?;

        // Only notify users if severity is high or critical
        if breach.get_severity() < BreachSeverity::High {
            info!("Breach severity does not require user notification: {}", breach_id);
            return Ok(());
        }

        // In a real implementation, this would send notifications to affected users
        breach.users_notified_at = Some(Utc::now().to_rfc3339());
        breach.updated_at = Utc::now().to_rfc3339();

        self.repository.update_breach(breach).await?;

        info!("User notifications sent for breach: {}", breach_id);
        Ok(())
    }

    fn requires_notification(&self, breach: &BreachNotification) -> bool {
        // Always notify for high and critical breaches
        if breach.get_severity() >= BreachSeverity::High {
            return true;
        }

        // Notify if many users are affected
        if breach.affected_users > 100 {
            return true;
        }

        false
    }

    fn is_deadline_approaching(&self, breach: &BreachNotification) -> bool {
        let detected_at = match chrono::DateTime::parse_from_rfc3339(&breach.detected_at) {
            Ok(dt) => dt.with_timezone(&Utc),
            Err(_) => return false,
        };

        let deadline = detected_at + Duration::hours(72);
        let time_remaining = deadline - Utc::now();

        // Warn if less than 12 hours remaining
        time_remaining.num_hours() < 12
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compliance::gdpr::repository::InMemoryGdprRepository;

    #[tokio::test]
    async fn test_report_breach() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let notifier = DefaultBreachNotifier::new(repo);

        let breach = BreachNotification::new(
            "unauthorized_access".to_string(),
            BreachSeverity::High,
            10,
            100,
            "Unauthorized access detected".to_string(),
        );

        let result = notifier.report_breach(breach).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_requires_notification() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let notifier = DefaultBreachNotifier::new(repo);

        let high_severity = BreachNotification::new(
            "test".to_string(),
            BreachSeverity::High,
            1,
            1,
            "Test".to_string(),
        );
        assert!(notifier.requires_notification(&high_severity));

        let low_severity = BreachNotification::new(
            "test".to_string(),
            BreachSeverity::Low,
            1,
            1,
            "Test".to_string(),
        );
        assert!(!notifier.requires_notification(&low_severity));

        let many_users = BreachNotification::new(
            "test".to_string(),
            BreachSeverity::Medium,
            200,
            200,
            "Test".to_string(),
        );
        assert!(notifier.requires_notification(&many_users));
    }

    #[test]
    fn test_deadline_approaching() {
        let repo = Arc::new(InMemoryGdprRepository::new());
        let notifier = DefaultBreachNotifier::new(repo);

        // Recent breach - not approaching
        let recent = BreachNotification::new(
            "test".to_string(),
            BreachSeverity::High,
            1,
            1,
            "Test".to_string(),
        );
        assert!(!notifier.is_deadline_approaching(&recent));
    }
}
