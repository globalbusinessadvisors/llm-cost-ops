// DLQ storage trait and implementations

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::types::{DlqItem, DlqItemStatus};
use super::{DlqError, DlqResult};

/// Trait for DLQ storage backends
#[async_trait]
pub trait DlqStore: Send + Sync {
    /// Add a new item to the DLQ
    async fn add(&self, item: DlqItem) -> DlqResult<()>;

    /// Get an item by ID
    async fn get(&self, id: Uuid) -> DlqResult<Option<DlqItem>>;

    /// Update an existing item
    async fn update(&self, item: DlqItem) -> DlqResult<()>;

    /// Delete an item
    async fn delete(&self, id: Uuid) -> DlqResult<()>;

    /// Get items ready for retry
    async fn get_ready_for_retry(&self, limit: usize) -> DlqResult<Vec<DlqItem>>;

    /// Get items by status
    async fn get_by_status(&self, status: DlqItemStatus, limit: usize) -> DlqResult<Vec<DlqItem>>;

    /// Get items by organization
    async fn get_by_organization(&self, org_id: &str, limit: usize) -> DlqResult<Vec<DlqItem>>;

    /// Get total count of items
    async fn count(&self) -> DlqResult<usize>;

    /// Get count by status
    async fn count_by_status(&self, status: DlqItemStatus) -> DlqResult<usize>;

    /// Get expired items
    async fn get_expired(&self, limit: usize) -> DlqResult<Vec<DlqItem>>;

    /// Cleanup expired items
    async fn cleanup_expired(&self) -> DlqResult<usize>;

    /// Get items for review
    async fn get_for_review(&self, limit: usize) -> DlqResult<Vec<DlqItem>>;

    /// Search items by criteria
    async fn search(
        &self,
        organization_id: Option<String>,
        status: Option<DlqItemStatus>,
        item_type: Option<String>,
        limit: usize,
        offset: usize,
    ) -> DlqResult<Vec<DlqItem>>;
}

/// In-memory DLQ store (for development/testing)
#[derive(Debug, Clone)]
pub struct InMemoryDlqStore {
    items: Arc<RwLock<HashMap<Uuid, DlqItem>>>,
}

impl InMemoryDlqStore {
    /// Create a new in-memory DLQ store
    pub fn new() -> Self {
        Self {
            items: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a store with pre-populated items
    pub fn with_items(items: Vec<DlqItem>) -> Self {
        let store = Self::new();
        let mut map = HashMap::new();
        for item in items {
            map.insert(item.id, item);
        }
        *store.items.blocking_write() = map;
        store
    }

    /// Get all items (for testing)
    pub async fn get_all(&self) -> Vec<DlqItem> {
        self.items.read().await.values().cloned().collect()
    }

    /// Clear all items (for testing)
    pub async fn clear(&self) {
        self.items.write().await.clear();
    }
}

impl Default for InMemoryDlqStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DlqStore for InMemoryDlqStore {
    async fn add(&self, item: DlqItem) -> DlqResult<()> {
        let mut items = self.items.write().await;
        items.insert(item.id, item);
        Ok(())
    }

    async fn get(&self, id: Uuid) -> DlqResult<Option<DlqItem>> {
        let items = self.items.read().await;
        Ok(items.get(&id).cloned())
    }

    async fn update(&self, item: DlqItem) -> DlqResult<()> {
        let mut items = self.items.write().await;

        if !items.contains_key(&item.id) {
            return Err(DlqError::ItemNotFound(item.id.to_string()));
        }

        items.insert(item.id, item);
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> DlqResult<()> {
        let mut items = self.items.write().await;
        items.remove(&id);
        Ok(())
    }

    async fn get_ready_for_retry(&self, limit: usize) -> DlqResult<Vec<DlqItem>> {
        let items = self.items.read().await;
        let mut ready: Vec<DlqItem> = items
            .values()
            .filter(|item| item.is_ready_for_retry())
            .cloned()
            .collect();

        // Sort by created_at (oldest first)
        ready.sort_by_key(|item| item.created_at);
        ready.truncate(limit);

        Ok(ready)
    }

    async fn get_by_status(&self, status: DlqItemStatus, limit: usize) -> DlqResult<Vec<DlqItem>> {
        let items = self.items.read().await;
        let mut result: Vec<DlqItem> = items
            .values()
            .filter(|item| item.status == status)
            .cloned()
            .collect();

        result.sort_by_key(|item| item.created_at);
        result.truncate(limit);

        Ok(result)
    }

    async fn get_by_organization(&self, org_id: &str, limit: usize) -> DlqResult<Vec<DlqItem>> {
        let items = self.items.read().await;
        let mut result: Vec<DlqItem> = items
            .values()
            .filter(|item| item.organization_id == org_id)
            .cloned()
            .collect();

        result.sort_by_key(|item| item.created_at);
        result.truncate(limit);

        Ok(result)
    }

    async fn count(&self) -> DlqResult<usize> {
        let items = self.items.read().await;
        Ok(items.len())
    }

    async fn count_by_status(&self, status: DlqItemStatus) -> DlqResult<usize> {
        let items = self.items.read().await;
        Ok(items.values().filter(|item| item.status == status).count())
    }

    async fn get_expired(&self, limit: usize) -> DlqResult<Vec<DlqItem>> {
        let items = self.items.read().await;
        let mut expired: Vec<DlqItem> = items
            .values()
            .filter(|item| item.is_expired())
            .cloned()
            .collect();

        expired.sort_by_key(|item| item.created_at);
        expired.truncate(limit);

        Ok(expired)
    }

    async fn cleanup_expired(&self) -> DlqResult<usize> {
        let mut items = self.items.write().await;

        let expired_ids: Vec<Uuid> = items
            .values()
            .filter(|item| item.is_expired())
            .map(|item| item.id)
            .collect();

        let count = expired_ids.len();

        for id in expired_ids {
            items.remove(&id);
        }

        Ok(count)
    }

    async fn get_for_review(&self, limit: usize) -> DlqResult<Vec<DlqItem>> {
        self.get_by_status(DlqItemStatus::ReviewRequired, limit)
            .await
    }

    async fn search(
        &self,
        organization_id: Option<String>,
        status: Option<DlqItemStatus>,
        item_type: Option<String>,
        limit: usize,
        offset: usize,
    ) -> DlqResult<Vec<DlqItem>> {
        let items = self.items.read().await;

        let mut result: Vec<DlqItem> = items
            .values()
            .filter(|item| {
                if let Some(ref org_id) = organization_id {
                    if &item.organization_id != org_id {
                        return false;
                    }
                }

                if let Some(ref status_filter) = status {
                    if &item.status != status_filter {
                        return false;
                    }
                }

                if let Some(ref type_filter) = item_type {
                    if &item.item_type != type_filter {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect();

        result.sort_by_key(|item| item.created_at);

        // Apply offset and limit
        if offset < result.len() {
            result = result.into_iter().skip(offset).collect();
        } else {
            result.clear();
        }

        result.truncate(limit);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dlq::types::FailureReason;
    use chrono::Utc;

    async fn create_test_item(org_id: &str, status: DlqItemStatus) -> DlqItem {
        let mut item = DlqItem::new(
            org_id.to_string(),
            "{}".to_string(),
            "test".to_string(),
            FailureReason::NetworkError,
            "Test error".to_string(),
            3,
        );
        item.status = status;
        item
    }

    #[tokio::test]
    async fn test_add_and_get() {
        let store = InMemoryDlqStore::new();
        let item = create_test_item("org-123", DlqItemStatus::Pending).await;
        let id = item.id;

        store.add(item).await.unwrap();

        let retrieved = store.get(id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().organization_id, "org-123");
    }

    #[tokio::test]
    async fn test_update() {
        let store = InMemoryDlqStore::new();
        let mut item = create_test_item("org-123", DlqItemStatus::Pending).await;
        let id = item.id;

        store.add(item.clone()).await.unwrap();

        item.status = DlqItemStatus::Processed;
        store.update(item).await.unwrap();

        let retrieved = store.get(id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, DlqItemStatus::Processed);
    }

    #[tokio::test]
    async fn test_delete() {
        let store = InMemoryDlqStore::new();
        let item = create_test_item("org-123", DlqItemStatus::Pending).await;
        let id = item.id;

        store.add(item).await.unwrap();
        assert!(store.get(id).await.unwrap().is_some());

        store.delete(id).await.unwrap();
        assert!(store.get(id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_count() {
        let store = InMemoryDlqStore::new();

        store
            .add(create_test_item("org-123", DlqItemStatus::Pending).await)
            .await
            .unwrap();
        store
            .add(create_test_item("org-123", DlqItemStatus::Processed).await)
            .await
            .unwrap();
        store
            .add(create_test_item("org-456", DlqItemStatus::Failed).await)
            .await
            .unwrap();

        assert_eq!(store.count().await.unwrap(), 3);
        assert_eq!(
            store.count_by_status(DlqItemStatus::Pending).await.unwrap(),
            1
        );
        assert_eq!(
            store.count_by_status(DlqItemStatus::Processed).await.unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_get_by_organization() {
        let store = InMemoryDlqStore::new();

        store
            .add(create_test_item("org-123", DlqItemStatus::Pending).await)
            .await
            .unwrap();
        store
            .add(create_test_item("org-123", DlqItemStatus::Processed).await)
            .await
            .unwrap();
        store
            .add(create_test_item("org-456", DlqItemStatus::Failed).await)
            .await
            .unwrap();

        let org123_items = store.get_by_organization("org-123", 10).await.unwrap();
        assert_eq!(org123_items.len(), 2);

        let org456_items = store.get_by_organization("org-456", 10).await.unwrap();
        assert_eq!(org456_items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_ready_for_retry() {
        let store = InMemoryDlqStore::new();

        let mut item1 = create_test_item("org-123", DlqItemStatus::Pending).await;
        item1.next_retry_at = Some(Utc::now() - chrono::Duration::seconds(10)); // Ready now

        let mut item2 = create_test_item("org-123", DlqItemStatus::Pending).await;
        item2.next_retry_at = Some(Utc::now() + chrono::Duration::seconds(60)); // Not ready yet

        store.add(item1).await.unwrap();
        store.add(item2).await.unwrap();

        let ready = store.get_ready_for_retry(10).await.unwrap();
        assert_eq!(ready.len(), 1);
    }

    #[tokio::test]
    async fn test_search() {
        let store = InMemoryDlqStore::new();

        store
            .add(create_test_item("org-123", DlqItemStatus::Pending).await)
            .await
            .unwrap();
        store
            .add(create_test_item("org-123", DlqItemStatus::Failed).await)
            .await
            .unwrap();
        store
            .add(create_test_item("org-456", DlqItemStatus::Pending).await)
            .await
            .unwrap();

        // Search by organization
        let results = store
            .search(Some("org-123".to_string()), None, None, 10, 0)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        // Search by status
        let results = store
            .search(None, Some(DlqItemStatus::Pending), None, 10, 0)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);

        // Search by organization and status
        let results = store
            .search(
                Some("org-123".to_string()),
                Some(DlqItemStatus::Pending),
                None,
                10,
                0,
            )
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_cleanup_expired() {
        let store = InMemoryDlqStore::new();

        let mut expired_item = create_test_item("org-123", DlqItemStatus::Pending).await;
        expired_item.expires_at = Some(Utc::now() - chrono::Duration::hours(1));

        let active_item = create_test_item("org-123", DlqItemStatus::Pending).await;

        store.add(expired_item).await.unwrap();
        store.add(active_item).await.unwrap();

        assert_eq!(store.count().await.unwrap(), 2);

        let cleaned = store.cleanup_expired().await.unwrap();
        assert_eq!(cleaned, 1);
        assert_eq!(store.count().await.unwrap(), 1);
    }
}
