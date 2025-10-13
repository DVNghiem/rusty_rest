pub mod error;

pub use error::*;

use axum::{
    extract::Request,
    http::HeaderMap,
    middleware::Next,
    response::Response,
};
use shared::{RequestContext, Timestamp};
use std::time::Instant;
use tracing::{info, warn, Span};
use uuid::Uuid;

/// Request context middleware - adds request ID and timing
pub async fn request_context(mut request: Request, next: Next) -> Response {
    let start_time = Instant::now();
    let request_id = Uuid::new_v4().to_string();
    
    // Add request ID to headers for downstream services
    request.headers_mut().insert(
        "x-request-id",
        request_id.parse().unwrap(),
    );

    // Create request context
    let context = RequestContext {
        request_id: request_id.clone(),
        user_id: extract_user_id(request.headers()),
        trace_id: extract_trace_id(request.headers()),
        timestamp: Timestamp::now(),
    };

    // Add context to request extensions
    request.extensions_mut().insert(context);

    // Add request ID to tracing span
    Span::current().record("request_id", &request_id);

    let response = next.run(request).await;

    let duration = start_time.elapsed();
    info!(
        request_id = %request_id,
        duration_ms = %duration.as_millis(),
        status = %response.status(),
        "Request completed"
    );

    response
}

/// Extract user ID from authorization header (simplified)
fn extract_user_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|auth| {
            if auth.starts_with("Bearer ") {
                // In a real implementation, you'd decode the JWT token
                Some("user_from_token".to_string())
            } else {
                None
            }
        })
}

/// Extract trace ID from headers (for distributed tracing)
fn extract_trace_id(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-trace-id")
        .and_then(|header| header.to_str().ok())
        .map(|s| s.to_string())
}

/// Tracing callbacks for HTTP requests
pub fn on_request(request: &Request, _span: &Span) {
    info!(
        method = %request.method(),
        uri = %request.uri(),
        version = ?request.version(),
        "Processing request"
    );
}

pub fn on_response(response: &Response, latency: std::time::Duration, _span: &Span) {
    let status = response.status();
    if status.is_success() {
        info!(
            status = %status,
            latency = ?latency,
            "Request succeeded"
        );
    } else if status.is_client_error() {
        warn!(
            status = %status,
            latency = ?latency,
            "Client error"
        );
    } else {
        warn!(
            status = %status,
            latency = ?latency,
            "Request failed"
        );
    }
}

pub fn on_failure(
    error: tower_http::classify::ServerErrorsFailureClass,
    latency: std::time::Duration,
    _span: &Span,
) {
    warn!(
        error = ?error,
        latency = ?latency,
        "Request failed with error"
    );
}