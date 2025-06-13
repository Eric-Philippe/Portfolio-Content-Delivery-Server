use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use tracing::warn;

/// Middleware to check for API key in requests
/// 
/// Expects the API key to be provided in the `X-API-Key` header
pub async fn api_key_auth(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get the expected API key from environment
    let expected_api_key = std::env::var("API_KEY")
        .map_err(|_| {
            warn!("API_KEY environment variable not set");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get the API key from the request headers
    let api_key = headers
        .get("X-API-Key")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            warn!("Missing X-API-Key header");
            StatusCode::UNAUTHORIZED
        })?;

    // Check if the API key matches
    if api_key != expected_api_key {
        warn!("Invalid API key provided");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // If API key is valid, continue with the request
    Ok(next.run(request).await)
}