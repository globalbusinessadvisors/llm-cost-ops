// API request validation utilities

use axum::{
    async_trait,
    extract::{FromRequest, Request},
    Json,
};
use serde::de::DeserializeOwned;
use validator::Validate;

use super::error::{ApiError, ApiResult};

/// Validated JSON extractor
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
{
    type Rejection = ApiError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|_| ApiError::BadRequest("Invalid JSON".to_string()))?;

        value.validate()?;

        Ok(ValidatedJson(value))
    }
}

/// Validate query parameters
pub fn validate_query_params<T: Validate>(params: &T) -> ApiResult<()> {
    params.validate().map_err(|e| e.into())
}
