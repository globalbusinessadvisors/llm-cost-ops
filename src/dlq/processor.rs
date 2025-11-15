// DLQ processor for handling and retrying failed items

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use super::{
    config::DlqConfig,
    retry::RetryPolicy,
    storage::DlqStore,
    types::{DlqItem, DlqItemStatus},
    DlqError, DlqResult,
};

/// Result of processing a DLQ item
#[derive(Debug)]
pub enum ProcessingResult {
    /// Item successfully processed
    Success,

    /// Item failed, should be retried
    Retry { error: String },

    /// Item failed permanently
    Failed { error: String },

    /// Item requires manual review
    NeedsReview { reason: String },
}

/// Trait for handling DLQ item processing
#[async_trait]
pub trait DlqItemHandler: Send + Sync {
    /// Process a DLQ item
    async fn process(&self, item: &DlqItem) -> ProcessingResult;

    /// Called after successful processing
    async fn on_success(&self, item: &DlqItem) {
        debug!(id = %item.id, "DLQ item successfully processed");
    }

    /// Called after permanent failure
    async fn on_failure(&self, item: &DlqItem) {
        warn!(id = %item.id, "DLQ item permanently failed");
    }
}

/// DLQ processor that handles retry logic and processing
pub struct DlqProcessor<S: DlqStore, H: DlqItemHandler> {
    store: Arc<S>,
    handler: Arc<H>,
    retry_policy: RetryPolicy,
    config: DlqConfig,
    semaphore: Arc<Semaphore>,
}

impl<S: DlqStore + 'static, H: DlqItemHandler + 'static> DlqProcessor<S, H> {
    /// Create a new DLQ processor
    pub fn new(store: Arc<S>, handler: Arc<H>, config: DlqConfig) -> Self {
        let retry_policy = RetryPolicy::exponential(
            config.max_retries,
            config.initial_retry_delay_secs,
            config.backoff_multiplier,
            config.max_retry_delay_secs,
        );

        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_processing));

        Self {
            store,
            handler,
            retry_policy,
            config,
            semaphore,
        }
    }

    /// Process items ready for retry
    pub async fn process_ready_items(&self) -> DlqResult<ProcessingStats> {
        if !self.config.enabled {
            return Ok(ProcessingStats::default());
        }

        let items = self
            .store
            .get_ready_for_retry(self.config.batch_size)
            .await?;

        if items.is_empty() {
            return Ok(ProcessingStats::default());
        }

        info!(count = items.len(), "Processing DLQ items ready for retry");

        let mut stats = ProcessingStats::default();
        let mut handles = vec![];

        for item in items {
            let permit = self.semaphore.clone().acquire_owned().await.map_err(|e| {
                DlqError::ProcessingError(format!("Failed to acquire semaphore: {}", e))
            })?;

            let processor = self.clone_arc();
            let handle = tokio::spawn(async move {
                let result = processor.process_item(item).await;
                drop(permit); // Release semaphore
                result
            });

            handles.push(handle);
        }

        // Wait for all processing to complete
        for handle in handles {
            match handle.await {
                Ok(Ok(item_stats)) => stats.merge(item_stats),
                Ok(Err(e)) => {
                    error!(error = %e, "Error processing DLQ item");
                    stats.errors += 1;
                }
                Err(e) => {
                    error!(error = %e, "Task panicked while processing DLQ item");
                    stats.errors += 1;
                }
            }
        }

        info!(
            processed = stats.processed,
            succeeded = stats.succeeded,
            failed = stats.failed,
            retried = stats.retried,
            "DLQ processing batch completed"
        );

        Ok(stats)
    }

    /// Process a single DLQ item
    async fn process_item(&self, mut item: DlqItem) -> DlqResult<ProcessingStats> {
        let start = Instant::now();
        let mut stats = ProcessingStats::default();
        stats.processed = 1;

        // Mark as retrying
        item.status = DlqItemStatus::Retrying;
        self.store.update(item.clone()).await?;

        debug!(
            id = %item.id,
            org_id = %item.organization_id,
            retry_count = item.retry_count,
            "Processing DLQ item"
        );

        // Process the item
        let result = self.handler.process(&item).await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

        match result {
            ProcessingResult::Success => {
                item.record_retry(true, None, duration_ms);
                self.store.update(item.clone()).await?;
                self.handler.on_success(&item).await;

                stats.succeeded = 1;
                info!(id = %item.id, "DLQ item successfully processed");
            }
            ProcessingResult::Retry { error } => {
                item.record_retry(false, Some(error.clone()), duration_ms);

                if self.retry_policy.should_retry(&item) {
                    let next_retry = self.retry_policy.next_retry_time(&item);
                    item.schedule_retry(next_retry);
                    stats.retried = 1;

                    debug!(
                        id = %item.id,
                        next_retry = %next_retry,
                        "DLQ item scheduled for retry"
                    );
                } else {
                    item.status = DlqItemStatus::Failed;
                    stats.failed = 1;
                    self.handler.on_failure(&item).await;

                    warn!(
                        id = %item.id,
                        error = %error,
                        "DLQ item permanently failed after max retries"
                    );
                }

                self.store.update(item).await?;
            }
            ProcessingResult::Failed { error } => {
                item.record_retry(false, Some(error.clone()), duration_ms);
                item.status = DlqItemStatus::Failed;
                self.store.update(item.clone()).await?;
                self.handler.on_failure(&item).await;

                stats.failed = 1;
                error!(id = %item.id, error = %error, "DLQ item permanently failed");
            }
            ProcessingResult::NeedsReview { reason } => {
                item.mark_for_review();
                item.error_details = Some(reason.clone());
                self.store.update(item.clone()).await?;

                stats.needs_review = 1;
                warn!(id = %item.id, reason = %reason, "DLQ item marked for review");
            }
        }

        Ok(stats)
    }

    /// Cleanup expired items
    pub async fn cleanup_expired_items(&self) -> DlqResult<usize> {
        if !self.config.enabled {
            return Ok(0);
        }

        let count = self.store.cleanup_expired().await?;

        if count > 0 {
            info!(count, "Cleaned up expired DLQ items");
        }

        Ok(count)
    }

    /// Get DLQ statistics
    pub async fn get_stats(&self) -> DlqResult<DlqStats> {
        let total = self.store.count().await?;
        let pending = self
            .store
            .count_by_status(DlqItemStatus::Pending)
            .await?;
        let retrying = self
            .store
            .count_by_status(DlqItemStatus::Retrying)
            .await?;
        let processed = self
            .store
            .count_by_status(DlqItemStatus::Processed)
            .await?;
        let failed = self
            .store
            .count_by_status(DlqItemStatus::Failed)
            .await?;
        let needs_review = self
            .store
            .count_by_status(DlqItemStatus::ReviewRequired)
            .await?;

        Ok(DlqStats {
            total,
            pending,
            retrying,
            processed,
            failed,
            needs_review,
        })
    }

    /// Helper to clone Arc references
    fn clone_arc(&self) -> Arc<Self> {
        Arc::new(Self {
            store: self.store.clone(),
            handler: self.handler.clone(),
            retry_policy: self.retry_policy.clone(),
            config: self.config.clone(),
            semaphore: self.semaphore.clone(),
        })
    }
}

/// Processing statistics
#[derive(Debug, Default, Clone)]
pub struct ProcessingStats {
    pub processed: usize,
    pub succeeded: usize,
    pub failed: usize,
    pub retried: usize,
    pub needs_review: usize,
    pub errors: usize,
}

impl ProcessingStats {
    fn merge(&mut self, other: ProcessingStats) {
        self.processed += other.processed;
        self.succeeded += other.succeeded;
        self.failed += other.failed;
        self.retried += other.retried;
        self.needs_review += other.needs_review;
        self.errors += other.errors;
    }
}

/// DLQ statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DlqStats {
    pub total: usize,
    pub pending: usize,
    pub retrying: usize,
    pub processed: usize,
    pub failed: usize,
    pub needs_review: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dlq::{storage::InMemoryDlqStore, types::FailureReason};

    struct TestHandler {
        should_succeed: bool,
    }

    #[async_trait]
    impl DlqItemHandler for TestHandler {
        async fn process(&self, _item: &DlqItem) -> ProcessingResult {
            if self.should_succeed {
                ProcessingResult::Success
            } else {
                ProcessingResult::Retry {
                    error: "Test error".to_string(),
                }
            }
        }
    }

    #[tokio::test]
    async fn test_process_item_success() {
        let store = Arc::new(InMemoryDlqStore::new());
        let handler = Arc::new(TestHandler {
            should_succeed: true,
        });
        let config = DlqConfig::development();

        let processor = DlqProcessor::new(store.clone(), handler, config);

        let mut item = DlqItem::new(
            "org-123".to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::NetworkError,
            "Test error".to_string(),
            3,
        );
        item.schedule_retry(chrono::Utc::now());

        store.add(item.clone()).await.unwrap();

        let stats = processor.process_item(item).await.unwrap();

        assert_eq!(stats.processed, 1);
        assert_eq!(stats.succeeded, 1);
        assert_eq!(stats.failed, 0);
    }

    #[tokio::test]
    async fn test_process_item_retry() {
        let store = Arc::new(InMemoryDlqStore::new());
        let handler = Arc::new(TestHandler {
            should_succeed: false,
        });
        let config = DlqConfig::development();

        let processor = DlqProcessor::new(store.clone(), handler, config);

        let mut item = DlqItem::new(
            "org-123".to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::NetworkError,
            "Test error".to_string(),
            3,
        );
        item.schedule_retry(chrono::Utc::now());

        store.add(item.clone()).await.unwrap();

        let stats = processor.process_item(item).await.unwrap();

        assert_eq!(stats.processed, 1);
        assert_eq!(stats.retried, 1);
        assert_eq!(stats.succeeded, 0);
    }

    #[tokio::test]
    async fn test_get_stats() {
        let store = Arc::new(InMemoryDlqStore::new());
        let handler = Arc::new(TestHandler {
            should_succeed: true,
        });
        let config = DlqConfig::development();

        let processor = DlqProcessor::new(store.clone(), handler, config);

        // Add items with different statuses
        let mut item1 = DlqItem::new(
            "org-123".to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::NetworkError,
            "Test".to_string(),
            3,
        );
        item1.status = DlqItemStatus::Pending;

        let mut item2 = item1.clone();
        item2.id = uuid::Uuid::new_v4();
        item2.status = DlqItemStatus::Failed;

        let mut item3 = item1.clone();
        item3.id = uuid::Uuid::new_v4();
        item3.status = DlqItemStatus::Processed;

        store.add(item1).await.unwrap();
        store.add(item2).await.unwrap();
        store.add(item3).await.unwrap();

        let stats = processor.get_stats().await.unwrap();

        assert_eq!(stats.total, 3);
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.processed, 1);
    }
}
