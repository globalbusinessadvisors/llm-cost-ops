// Pagination support for API responses

use serde::{Deserialize, Serialize};

/// Pagination parameters
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-indexed)
    #[serde(default = "default_page")]
    pub page: u32,

    /// Page size (number of items per page)
    #[serde(default = "default_page_size")]
    pub page_size: u32,

    /// Sort field
    pub sort_by: Option<String>,

    /// Sort order (asc/desc)
    #[serde(default)]
    pub sort_order: SortOrder,
}

fn default_page() -> u32 {
    1
}

fn default_page_size() -> u32 {
    20
}

/// Sort order
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

impl PaginationParams {
    /// Get limit for SQL queries
    pub fn limit(&self) -> u32 {
        self.page_size.min(100) // Max 100 items per page
    }

    /// Get offset for SQL queries
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.limit()
    }

    /// Validate pagination parameters
    pub fn validate(&self) -> Result<(), String> {
        if self.page == 0 {
            return Err("Page must be greater than 0".to_string());
        }

        if self.page_size == 0 {
            return Err("Page size must be greater than 0".to_string());
        }

        if self.page_size > 100 {
            return Err("Page size cannot exceed 100".to_string());
        }

        Ok(())
    }
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// The data items
    pub data: Vec<T>,

    /// Pagination metadata
    pub pagination: PaginationMetadata,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMetadata {
    /// Current page number (1-indexed)
    pub page: u32,

    /// Page size (items per page)
    pub page_size: u32,

    /// Total number of items
    pub total_items: u64,

    /// Total number of pages
    pub total_pages: u32,

    /// Whether there is a next page
    pub has_next: bool,

    /// Whether there is a previous page
    pub has_prev: bool,
}

impl PaginationMetadata {
    /// Create pagination metadata
    pub fn new(page: u32, page_size: u32, total_items: u64) -> Self {
        let total_pages = ((total_items as f64) / (page_size as f64)).ceil() as u32;
        let has_next = page < total_pages;
        let has_prev = page > 1;

        Self {
            page,
            page_size,
            total_items,
            total_pages,
            has_next,
            has_prev,
        }
    }
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response
    pub fn new(data: Vec<T>, params: &PaginationParams, total_items: u64) -> Self {
        Self {
            data,
            pagination: PaginationMetadata::new(params.page, params.page_size, total_items),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params_defaults() {
        let params = PaginationParams {
            page: default_page(),
            page_size: default_page_size(),
            sort_by: None,
            sort_order: SortOrder::default(),
        };

        assert_eq!(params.page, 1);
        assert_eq!(params.page_size, 20);
        assert_eq!(params.limit(), 20);
        assert_eq!(params.offset(), 0);
    }

    #[test]
    fn test_pagination_offset_calculation() {
        let params = PaginationParams {
            page: 3,
            page_size: 10,
            sort_by: None,
            sort_order: SortOrder::Asc,
        };

        assert_eq!(params.offset(), 20); // (3-1) * 10
        assert_eq!(params.limit(), 10);
    }

    #[test]
    fn test_pagination_max_page_size() {
        let params = PaginationParams {
            page: 1,
            page_size: 200, // Exceeds max
            sort_by: None,
            sort_order: SortOrder::Asc,
        };

        assert_eq!(params.limit(), 100); // Capped at 100
    }

    #[test]
    fn test_pagination_validation() {
        let valid_params = PaginationParams {
            page: 1,
            page_size: 20,
            sort_by: None,
            sort_order: SortOrder::Asc,
        };
        assert!(valid_params.validate().is_ok());

        let invalid_page = PaginationParams {
            page: 0,
            page_size: 20,
            sort_by: None,
            sort_order: SortOrder::Asc,
        };
        assert!(invalid_page.validate().is_err());

        let invalid_page_size = PaginationParams {
            page: 1,
            page_size: 0,
            sort_by: None,
            sort_order: SortOrder::Asc,
        };
        assert!(invalid_page_size.validate().is_err());
    }

    #[test]
    fn test_pagination_metadata() {
        let metadata = PaginationMetadata::new(2, 10, 45);

        assert_eq!(metadata.page, 2);
        assert_eq!(metadata.page_size, 10);
        assert_eq!(metadata.total_items, 45);
        assert_eq!(metadata.total_pages, 5); // ceil(45/10)
        assert!(metadata.has_next);
        assert!(metadata.has_prev);
    }

    #[test]
    fn test_paginated_response_creation() {
        let data = vec![1, 2, 3];
        let params = PaginationParams {
            page: 1,
            page_size: 10,
            sort_by: None,
            sort_order: SortOrder::Asc,
        };

        let response = PaginatedResponse::new(data.clone(), &params, 100);

        assert_eq!(response.data.len(), 3);
        assert_eq!(response.pagination.total_items, 100);
        assert_eq!(response.pagination.total_pages, 10);
        assert!(response.pagination.has_next);
        assert!(!response.pagination.has_prev);
    }
}
