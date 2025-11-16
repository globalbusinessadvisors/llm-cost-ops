// REST API module for LLM Cost Ops platform

pub mod types;
pub mod error;
pub mod routes;
pub mod handlers;
pub mod middleware;
pub mod server;
pub mod pagination;
pub mod validation;

pub use types::{ApiVersion, ApiResponse, ApiError as ApiErrorResponse};
pub use error::{ApiError, ApiResult};
pub use server::{ApiServer, ApiServerConfig, create_api_router};
pub use pagination::{PaginationParams, PaginatedResponse};

/// API version constant
pub const API_VERSION: &str = "v1";
pub const API_PREFIX: &str = "/api/v1";
