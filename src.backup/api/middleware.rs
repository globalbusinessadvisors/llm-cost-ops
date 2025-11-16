// API middleware

use axum::{
    extract::Request,
    http::header,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

/// Request ID middleware
pub async fn request_id_middleware(mut request: Request, next: Next) -> Response {
    let request_id = Uuid::new_v4().to_string();
    request.headers_mut().insert(
        header::HeaderName::from_static("x-request-id"),
        header::HeaderValue::from_str(&request_id).unwrap(),
    );

    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::HeaderName::from_static("x-request-id"),
        header::HeaderValue::from_str(&request_id).unwrap(),
    );

    response
}
