//! Compliance task scheduler
//!
//! Schedules and executes automated compliance tasks.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::checks::ComplianceCheckEngine;
use super::reports::{ComplianceReport, ReportGenerator, ReportType};

/// Scheduler error types
#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("Task execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Invalid schedule: {0}")]
    InvalidSchedule(String),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Configuration error: {0}")]
    Configuration(String),
}

pub type SchedulerResult<T> = Result<T, SchedulerError>;

/// Task status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Task schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSchedule {
    pub cron_expression: String,
    pub timezone: String,
    pub enabled: bool,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
}

impl Default for TaskSchedule {
    fn default() -> Self {
        Self {
            cron_expression: "0 0 * * *".to_string(), // Daily at midnight
            timezone: "UTC".to_string(),
            enabled: true,
            max_retries: 3,
            retry_delay_seconds: 300,
        }
    }
}

/// Scheduled task definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub task_type: TaskType,
    pub schedule: TaskSchedule,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_execution: Option<DateTime<Utc>>,
    pub next_execution: Option<DateTime<Utc>>,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    ComplianceCheck,
    ReportGeneration,
    PolicyReview,
    DataRetentionCleanup,
    AuditLogArchive,
    GdprRequestProcessing,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: Uuid,
    pub execution_id: Uuid,
    pub status: TaskStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
    pub result_data: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub retry_count: u32,
}

/// Task execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecution {
    pub id: Uuid,
    pub task_id: Uuid,
    pub status: TaskStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<TaskResult>,
}

/// Task execution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistory {
    pub task_id: Uuid,
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub last_execution: Option<TaskExecution>,
    pub executions: Vec<TaskExecution>,
}

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub max_concurrent_tasks: usize,
    pub default_timeout_seconds: u64,
    pub retention_days: u32,
    pub enabled: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            default_timeout_seconds: 3600,
            retention_days: 90,
            enabled: true,
        }
    }
}

/// Compliance scheduler
pub struct ComplianceScheduler {
    _config: SchedulerConfig,
    tasks: Arc<RwLock<HashMap<Uuid, ScheduledTask>>>,
    executions: Arc<RwLock<HashMap<Uuid, Vec<TaskExecution>>>>,
    check_engine: Arc<ComplianceCheckEngine>,
    _report_generator: Arc<ReportGenerator>,
}

impl ComplianceScheduler {
    pub fn new(
        config: SchedulerConfig,
        check_engine: ComplianceCheckEngine,
        report_generator: ReportGenerator,
    ) -> Self {
        Self {
            _config: config,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(RwLock::new(HashMap::new())),
            check_engine: Arc::new(check_engine),
            _report_generator: Arc::new(report_generator),
        }
    }

    /// Schedule a new task
    pub async fn schedule_task(&self, task: ScheduledTask) -> SchedulerResult<Uuid> {
        let id = task.id;
        let mut tasks = self.tasks.write().await;
        tasks.insert(id, task);
        Ok(id)
    }

    /// Get a scheduled task
    pub async fn get_task(&self, task_id: Uuid) -> SchedulerResult<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks
            .get(&task_id)
            .cloned()
            .ok_or_else(|| SchedulerError::TaskNotFound(task_id.to_string()))
    }

    /// List all scheduled tasks
    pub async fn list_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// Execute a task immediately
    pub async fn execute_task(&self, task_id: Uuid) -> SchedulerResult<TaskResult> {
        let task = self.get_task(task_id).await?;

        if !task.schedule.enabled {
            return Err(SchedulerError::ExecutionFailed(
                "Task is disabled".to_string(),
            ));
        }

        let execution_id = Uuid::new_v4();
        let started_at = Utc::now();

        let execution = TaskExecution {
            id: execution_id,
            task_id,
            status: TaskStatus::Running,
            started_at,
            completed_at: None,
            result: None,
        };

        // Store execution record
        {
            let mut executions = self.executions.write().await;
            executions.entry(task_id).or_insert_with(Vec::new).push(execution.clone());
        }

        // Execute based on task type
        let result = match task.task_type {
            TaskType::ComplianceCheck => self.execute_compliance_check(task_id, execution_id).await,
            TaskType::ReportGeneration => self.execute_report_generation(task_id, execution_id).await,
            TaskType::PolicyReview => self.execute_policy_review(task_id, execution_id).await,
            TaskType::DataRetentionCleanup => self.execute_retention_cleanup(task_id, execution_id).await,
            TaskType::AuditLogArchive => self.execute_audit_archive(task_id, execution_id).await,
            TaskType::GdprRequestProcessing => self.execute_gdpr_processing(task_id, execution_id).await,
        };

        // Update execution record
        let completed_at = Utc::now();
        let duration_ms = completed_at
            .signed_duration_since(started_at)
            .num_milliseconds() as u64;

        let task_result = match result {
            Ok(data) => TaskResult {
                task_id,
                execution_id,
                status: TaskStatus::Completed,
                started_at,
                completed_at: Some(completed_at),
                duration_ms: Some(duration_ms),
                result_data: Some(data),
                error_message: None,
                retry_count: 0,
            },
            Err(e) => TaskResult {
                task_id,
                execution_id,
                status: TaskStatus::Failed,
                started_at,
                completed_at: Some(completed_at),
                duration_ms: Some(duration_ms),
                result_data: None,
                error_message: Some(e.to_string()),
                retry_count: 0,
            },
        };

        // Update execution with result
        {
            let mut executions = self.executions.write().await;
            if let Some(task_executions) = executions.get_mut(&task_id) {
                if let Some(exec) = task_executions.iter_mut().find(|e| e.id == execution_id) {
                    exec.status = task_result.status;
                    exec.completed_at = task_result.completed_at;
                    exec.result = Some(task_result.clone());
                }
            }
        }

        Ok(task_result)
    }

    async fn execute_compliance_check(
        &self,
        _task_id: Uuid,
        _execution_id: Uuid,
    ) -> SchedulerResult<serde_json::Value> {
        let results = self.check_engine.run_all_checks().await;
        serde_json::to_value(&results).map_err(|e| SchedulerError::ExecutionFailed(e.to_string()))
    }

    async fn execute_report_generation(
        &self,
        _task_id: Uuid,
        _execution_id: Uuid,
    ) -> SchedulerResult<serde_json::Value> {
        // Mock report generation
        let report = ComplianceReport {
            metadata: super::reports::ReportMetadata {
                id: Uuid::new_v4(),
                report_type: ReportType::AuditLogSummary,
                title: "Scheduled Compliance Report".to_string(),
                description: "Automated compliance report".to_string(),
                generated_at: Utc::now(),
                generated_by: "scheduler".to_string(),
                period_start: Utc::now() - chrono::Duration::days(7),
                period_end: Utc::now(),
                total_records: 0,
                format: super::reports::ReportFormat::Json,
                tags: vec![],
            },
            executive_summary: "Automated compliance check completed successfully.".to_string(),
            sections: vec![],
            findings: vec![],
            recommendations: vec![],
        };

        serde_json::to_value(&report).map_err(|e| SchedulerError::ExecutionFailed(e.to_string()))
    }

    async fn execute_policy_review(
        &self,
        _task_id: Uuid,
        _execution_id: Uuid,
    ) -> SchedulerResult<serde_json::Value> {
        // Mock policy review
        Ok(serde_json::json!({
            "policies_reviewed": 45,
            "policies_needing_update": 3,
            "timestamp": Utc::now()
        }))
    }

    async fn execute_retention_cleanup(
        &self,
        _task_id: Uuid,
        _execution_id: Uuid,
    ) -> SchedulerResult<serde_json::Value> {
        // Mock retention cleanup
        Ok(serde_json::json!({
            "data_sets_deleted": 12,
            "space_freed_mb": 4500,
            "timestamp": Utc::now()
        }))
    }

    async fn execute_audit_archive(
        &self,
        _task_id: Uuid,
        _execution_id: Uuid,
    ) -> SchedulerResult<serde_json::Value> {
        // Mock audit archival
        Ok(serde_json::json!({
            "events_archived": 50000,
            "archive_size_mb": 250,
            "timestamp": Utc::now()
        }))
    }

    async fn execute_gdpr_processing(
        &self,
        _task_id: Uuid,
        _execution_id: Uuid,
    ) -> SchedulerResult<serde_json::Value> {
        // Mock GDPR processing
        Ok(serde_json::json!({
            "requests_processed": 8,
            "requests_pending": 2,
            "timestamp": Utc::now()
        }))
    }

    /// Get task execution history
    pub async fn get_task_history(&self, task_id: Uuid) -> Option<TaskHistory> {
        let executions = self.executions.read().await;
        let task_executions = executions.get(&task_id)?;

        let total = task_executions.len();
        let successful = task_executions
            .iter()
            .filter(|e| e.status == TaskStatus::Completed)
            .count();
        let failed = task_executions
            .iter()
            .filter(|e| e.status == TaskStatus::Failed)
            .count();

        Some(TaskHistory {
            task_id,
            total_executions: total,
            successful_executions: successful,
            failed_executions: failed,
            last_execution: task_executions.last().cloned(),
            executions: task_executions.clone(),
        })
    }

    /// Cancel a running task
    pub async fn cancel_task(&self, task_id: Uuid) -> SchedulerResult<()> {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.schedule.enabled = false;
            Ok(())
        } else {
            Err(SchedulerError::TaskNotFound(task_id.to_string()))
        }
    }

    /// Delete a scheduled task
    pub async fn delete_task(&self, task_id: Uuid) -> SchedulerResult<()> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(&task_id).ok_or_else(|| SchedulerError::TaskNotFound(task_id.to_string()))?;

        let mut executions = self.executions.write().await;
        executions.remove(&task_id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schedule_task() {
        let scheduler = ComplianceScheduler::new(
            SchedulerConfig::default(),
            ComplianceCheckEngine::new(),
            ReportGenerator::new(),
        );

        let task = ScheduledTask {
            id: Uuid::new_v4(),
            name: "Daily Compliance Check".to_string(),
            description: "Run all compliance checks daily".to_string(),
            task_type: TaskType::ComplianceCheck,
            schedule: TaskSchedule::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_execution: None,
            next_execution: None,
            config: serde_json::json!({}),
        };

        let id = scheduler.schedule_task(task).await.unwrap();
        let retrieved = scheduler.get_task(id).await.unwrap();

        assert_eq!(retrieved.name, "Daily Compliance Check");
    }

    #[tokio::test]
    async fn test_execute_task() {
        let scheduler = ComplianceScheduler::new(
            SchedulerConfig::default(),
            ComplianceCheckEngine::new(),
            ReportGenerator::new(),
        );

        let task = ScheduledTask {
            id: Uuid::new_v4(),
            name: "Test Task".to_string(),
            description: "Test task".to_string(),
            task_type: TaskType::PolicyReview,
            schedule: TaskSchedule::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_execution: None,
            next_execution: None,
            config: serde_json::json!({}),
        };

        let id = scheduler.schedule_task(task).await.unwrap();
        let result = scheduler.execute_task(id).await.unwrap();

        assert_eq!(result.status, TaskStatus::Completed);
    }
}
