// Scheduled reporting system

use async_trait::async_trait;
use chrono::{DateTime, Duration, Utc};
use chrono_tz::Tz;
use cron::Schedule;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

use super::config::{ScheduledReportConfig, ReportFiltersConfig};
use super::delivery::{DeliveryCoordinator, DeliveryRequest, DeliveryResponse};
use super::reports::{ReportGenerator, ReportRequest, ReportFilters};
use super::{ExportError, ExportResult};

/// Scheduled report execution result
#[derive(Debug, Clone)]
pub struct ScheduledExecutionResult {
    pub schedule_id: String,
    pub execution_id: String,
    pub executed_at: DateTime<Utc>,
    pub success: bool,
    pub deliveries: Vec<DeliveryResponse>,
    pub error: Option<String>,
}

/// Scheduled report status
#[derive(Debug, Clone)]
pub struct ScheduledReportStatus {
    pub schedule_id: String,
    pub enabled: bool,
    pub next_run: Option<DateTime<Utc>>,
    pub last_run: Option<DateTime<Utc>>,
    pub last_result: Option<ScheduledExecutionResult>,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
}

/// Report scheduler trait
#[async_trait]
pub trait ReportScheduler: Send + Sync {
    async fn add_schedule(&self, config: ScheduledReportConfig) -> ExportResult<()>;
    async fn remove_schedule(&self, schedule_id: &str) -> ExportResult<()>;
    async fn enable_schedule(&self, schedule_id: &str) -> ExportResult<()>;
    async fn disable_schedule(&self, schedule_id: &str) -> ExportResult<()>;
    async fn get_status(&self, schedule_id: &str) -> ExportResult<ScheduledReportStatus>;
    async fn list_schedules(&self) -> ExportResult<Vec<ScheduledReportStatus>>;
    async fn execute_now(&self, schedule_id: &str) -> ExportResult<ScheduledExecutionResult>;
}

/// Cron-based report scheduler implementation
pub struct CronScheduler {
    schedules: Arc<RwLock<HashMap<String, ScheduleEntry>>>,
    generator: Arc<ReportGenerator>,
    coordinator: Arc<DeliveryCoordinator>,
    shutdown_tx: Option<tokio::sync::broadcast::Sender<()>>,
    tasks: Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
}

struct ScheduleEntry {
    config: ScheduledReportConfig,
    schedule: Schedule,
    timezone: Tz,
    status: ScheduledReportStatus,
}

impl CronScheduler {
    pub fn new(
        generator: Arc<ReportGenerator>,
        coordinator: Arc<DeliveryCoordinator>,
    ) -> Self {
        Self {
            schedules: Arc::new(RwLock::new(HashMap::new())),
            generator,
            coordinator,
            shutdown_tx: None,
            tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&mut self) -> ExportResult<()> {
        let (shutdown_tx, _) = tokio::sync::broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        // Start scheduler loop
        let schedules = Arc::clone(&self.schedules);
        let generator = Arc::clone(&self.generator);
        let coordinator = Arc::clone(&self.coordinator);
        let tasks = Arc::clone(&self.tasks);
        let mut shutdown_rx = shutdown_tx.subscribe();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        Self::check_and_execute_schedules(
                            &schedules,
                            &generator,
                            &coordinator,
                            &tasks,
                        ).await;
                    }
                    _ = shutdown_rx.recv() => {
                        info!("Scheduler shutting down");
                        break;
                    }
                }
            }
        });

        info!("Report scheduler started");
        Ok(())
    }

    pub async fn shutdown(&self) -> ExportResult<()> {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(());
        }

        // Wait for all tasks to complete
        let tasks = self.tasks.read().await;
        for (id, handle) in tasks.iter() {
            if !handle.is_finished() {
                info!("Waiting for schedule {} to complete", id);
            }
        }

        info!("Scheduler shutdown complete");
        Ok(())
    }

    async fn check_and_execute_schedules(
        schedules: &Arc<RwLock<HashMap<String, ScheduleEntry>>>,
        generator: &Arc<ReportGenerator>,
        coordinator: &Arc<DeliveryCoordinator>,
        tasks: &Arc<RwLock<HashMap<String, JoinHandle<()>>>>,
    ) {
        let now = Utc::now();
        let schedules_read = schedules.read().await;

        for (id, entry) in schedules_read.iter() {
            if !entry.config.enabled {
                continue;
            }

            // Calculate next execution time in the configured timezone
            let now_tz = now.with_timezone(&entry.timezone);
            if let Some(next_run) = entry.status.next_run {
                if now >= next_run {
                    // Time to execute
                    let schedule_id = id.clone();
                    let schedule_id_for_task = schedule_id.clone();
                    let config = entry.config.clone();
                    let gen = Arc::clone(generator);
                    let coord = Arc::clone(coordinator);
                    let scheds = Arc::clone(schedules);

                    // Spawn execution task
                    let task = tokio::spawn(async move {
                        let result = Self::execute_schedule(
                            &schedule_id_for_task,
                            &config,
                            &gen,
                            &coord,
                        ).await;

                        // Update status
                        let mut schedules_write = scheds.write().await;
                        if let Some(entry) = schedules_write.get_mut(&schedule_id_for_task) {
                            entry.status.last_run = Some(Utc::now());
                            entry.status.last_result = Some(result.clone());
                            entry.status.total_executions += 1;

                            if result.success {
                                entry.status.successful_executions += 1;
                            } else {
                                entry.status.failed_executions += 1;
                            }

                            // Calculate next run time
                            if let Some(next) = entry.schedule.upcoming(entry.timezone).next() {
                                entry.status.next_run = Some(next.with_timezone(&Utc));
                            }
                        }
                    });

                    // Store task handle
                    tasks.write().await.insert(schedule_id.clone(), task);
                }
            }
        }
    }

    async fn execute_schedule(
        schedule_id: &str,
        config: &ScheduledReportConfig,
        generator: &Arc<ReportGenerator>,
        coordinator: &Arc<DeliveryCoordinator>,
    ) -> ScheduledExecutionResult {
        let execution_id = uuid::Uuid::new_v4().to_string();
        info!(
            "Executing scheduled report: {} (execution: {})",
            schedule_id, execution_id
        );

        let executed_at = Utc::now();

        // Create report request
        let end_date = Utc::now();
        let start_date = match config.report_type {
            super::reports::ReportType::Cost
            | super::reports::ReportType::Usage => end_date - Duration::days(1),
            super::reports::ReportType::Forecast => end_date,
            super::reports::ReportType::Audit => end_date - Duration::days(7),
            super::reports::ReportType::Budget => end_date,
            super::reports::ReportType::Summary => end_date - Duration::days(30),
        };

        let request = ReportRequest {
            report_type: config.report_type,
            start_date,
            end_date,
            organization_id: config.filters.organization_id.clone(),
            filters: Self::convert_filters(&config.filters),
        };

        // Generate report
        match generator.generate(request).await {
            Ok(report) => {
                // Deliver to all targets
                match coordinator
                    .deliver_to_targets(report, config.format, config.delivery.clone())
                    .await
                {
                    Ok(deliveries) => {
                        info!(
                            "Successfully executed scheduled report {} with {} deliveries",
                            schedule_id,
                            deliveries.len()
                        );
                        ScheduledExecutionResult {
                            schedule_id: schedule_id.to_string(),
                            execution_id,
                            executed_at,
                            success: true,
                            deliveries,
                            error: None,
                        }
                    }
                    Err(e) => {
                        error!(
                            "Failed to deliver scheduled report {}: {}",
                            schedule_id, e
                        );
                        ScheduledExecutionResult {
                            schedule_id: schedule_id.to_string(),
                            execution_id,
                            executed_at,
                            success: false,
                            deliveries: Vec::new(),
                            error: Some(format!("Delivery failed: {}", e)),
                        }
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to generate scheduled report {}: {}",
                    schedule_id, e
                );
                ScheduledExecutionResult {
                    schedule_id: schedule_id.to_string(),
                    execution_id,
                    executed_at,
                    success: false,
                    deliveries: Vec::new(),
                    error: Some(format!("Generation failed: {}", e)),
                }
            }
        }
    }

    fn convert_filters(config_filters: &ReportFiltersConfig) -> ReportFilters {
        ReportFilters {
            provider: config_filters.provider.clone(),
            model: config_filters.model.clone(),
            user_id: config_filters.user_id.clone(),
            resource_type: config_filters.resource_type.clone(),
        }
    }

    fn parse_schedule(schedule_str: &str) -> ExportResult<Schedule> {
        Schedule::from_str(schedule_str).map_err(|e| {
            ExportError::GenerationError(format!("Invalid cron schedule: {}", e))
        })
    }

    fn parse_timezone(tz_str: &str) -> ExportResult<Tz> {
        tz_str.parse::<Tz>().map_err(|e| {
            ExportError::GenerationError(format!("Invalid timezone: {}", e))
        })
    }
}

#[async_trait]
impl ReportScheduler for CronScheduler {
    async fn add_schedule(&self, config: ScheduledReportConfig) -> ExportResult<()> {
        let schedule = Self::parse_schedule(&config.schedule)?;
        let timezone = Self::parse_timezone(&config.timezone)?;

        // Calculate next run time
        let next_run = schedule
            .upcoming(timezone)
            .next()
            .map(|dt| dt.with_timezone(&Utc));

        let status = ScheduledReportStatus {
            schedule_id: config.id.clone(),
            enabled: config.enabled,
            next_run,
            last_run: None,
            last_result: None,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
        };

        let entry = ScheduleEntry {
            config: config.clone(),
            schedule,
            timezone,
            status,
        };

        let mut schedules = self.schedules.write().await;
        schedules.insert(config.id.clone(), entry);

        info!("Added scheduled report: {}", config.id);
        Ok(())
    }

    async fn remove_schedule(&self, schedule_id: &str) -> ExportResult<()> {
        let mut schedules = self.schedules.write().await;
        schedules.remove(schedule_id).ok_or_else(|| {
            ExportError::NotFound(format!("Schedule not found: {}", schedule_id))
        })?;

        // Cancel any running task
        let mut tasks = self.tasks.write().await;
        if let Some(handle) = tasks.remove(schedule_id) {
            handle.abort();
        }

        info!("Removed scheduled report: {}", schedule_id);
        Ok(())
    }

    async fn enable_schedule(&self, schedule_id: &str) -> ExportResult<()> {
        let mut schedules = self.schedules.write().await;
        let entry = schedules.get_mut(schedule_id).ok_or_else(|| {
            ExportError::NotFound(format!("Schedule not found: {}", schedule_id))
        })?;

        entry.config.enabled = true;
        entry.status.enabled = true;

        // Recalculate next run
        entry.status.next_run = entry
            .schedule
            .upcoming(entry.timezone)
            .next()
            .map(|dt| dt.with_timezone(&Utc));

        info!("Enabled scheduled report: {}", schedule_id);
        Ok(())
    }

    async fn disable_schedule(&self, schedule_id: &str) -> ExportResult<()> {
        let mut schedules = self.schedules.write().await;
        let entry = schedules.get_mut(schedule_id).ok_or_else(|| {
            ExportError::NotFound(format!("Schedule not found: {}", schedule_id))
        })?;

        entry.config.enabled = false;
        entry.status.enabled = false;
        entry.status.next_run = None;

        info!("Disabled scheduled report: {}", schedule_id);
        Ok(())
    }

    async fn get_status(&self, schedule_id: &str) -> ExportResult<ScheduledReportStatus> {
        let schedules = self.schedules.read().await;
        let entry = schedules.get(schedule_id).ok_or_else(|| {
            ExportError::NotFound(format!("Schedule not found: {}", schedule_id))
        })?;

        Ok(entry.status.clone())
    }

    async fn list_schedules(&self) -> ExportResult<Vec<ScheduledReportStatus>> {
        let schedules = self.schedules.read().await;
        Ok(schedules.values().map(|e| e.status.clone()).collect())
    }

    async fn execute_now(&self, schedule_id: &str) -> ExportResult<ScheduledExecutionResult> {
        let schedules = self.schedules.read().await;
        let entry = schedules.get(schedule_id).ok_or_else(|| {
            ExportError::NotFound(format!("Schedule not found: {}", schedule_id))
        })?;

        let result = Self::execute_schedule(
            schedule_id,
            &entry.config,
            &self.generator,
            &self.coordinator,
        )
        .await;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::config::DeliveryTarget;

    #[test]
    fn test_parse_schedule() {
        let schedule = CronScheduler::parse_schedule("0 0 * * *");
        assert!(schedule.is_ok());

        let invalid = CronScheduler::parse_schedule("invalid");
        assert!(invalid.is_err());
    }

    #[test]
    fn test_parse_timezone() {
        let tz = CronScheduler::parse_timezone("UTC");
        assert!(tz.is_ok());

        let tz = CronScheduler::parse_timezone("America/New_York");
        assert!(tz.is_ok());

        let invalid = CronScheduler::parse_timezone("Invalid/Timezone");
        assert!(invalid.is_err());
    }

    #[tokio::test]
    async fn test_add_remove_schedule() {
        let generator = Arc::new(ReportGenerator::new());
        let coordinator = Arc::new(DeliveryCoordinator::new());
        let scheduler = CronScheduler::new(generator, coordinator);

        let config = ScheduledReportConfig {
            id: "test-schedule".to_string(),
            report_type: super::super::reports::ReportType::Cost,
            schedule: "0 0 * * *".to_string(),
            format: super::super::formats::ExportFormat::Csv,
            delivery: vec![DeliveryTarget::Storage { path: None }],
            filters: ReportFiltersConfig::default(),
            enabled: true,
            timezone: "UTC".to_string(),
        };

        let result = scheduler.add_schedule(config).await;
        assert!(result.is_ok());

        let status = scheduler.get_status("test-schedule").await;
        assert!(status.is_ok());

        let result = scheduler.remove_schedule("test-schedule").await;
        assert!(result.is_ok());

        let status = scheduler.get_status("test-schedule").await;
        assert!(status.is_err());
    }

    #[tokio::test]
    async fn test_enable_disable_schedule() {
        let generator = Arc::new(ReportGenerator::new());
        let coordinator = Arc::new(DeliveryCoordinator::new());
        let scheduler = CronScheduler::new(generator, coordinator);

        let config = ScheduledReportConfig {
            id: "test-schedule-2".to_string(),
            report_type: super::super::reports::ReportType::Usage,
            schedule: "0 0 * * *".to_string(),
            format: super::super::formats::ExportFormat::Json,
            delivery: vec![],
            filters: ReportFiltersConfig::default(),
            enabled: true,
            timezone: "UTC".to_string(),
        };

        scheduler.add_schedule(config).await.unwrap();

        let result = scheduler.disable_schedule("test-schedule-2").await;
        assert!(result.is_ok());

        let status = scheduler.get_status("test-schedule-2").await.unwrap();
        assert!(!status.enabled);
        assert!(status.next_run.is_none());

        let result = scheduler.enable_schedule("test-schedule-2").await;
        assert!(result.is_ok());

        let status = scheduler.get_status("test-schedule-2").await.unwrap();
        assert!(status.enabled);
        assert!(status.next_run.is_some());
    }
}
