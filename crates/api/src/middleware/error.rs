use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use shared::{AppError, ErrorResponse, ErrorDetail, RequestContext};
use tracing::{error, warn};

/// Error handling middleware - converts AppError to proper HTTP responses
pub async fn error_handler(request: Request, next: Next) -> Response {
    let response = next.run(request).await;
    
    // If the response is already an error, convert it to our standard format
    if response.status().is_client_error() || response.status().is_server_error() {
        // For simplicity, we'll let Axum handle error conversion
        // In a production system, you might want more sophisticated error handling
        return response;
    }

    response
}

/// Convert AppError to HTTP response with proper error format
pub fn convert_app_error(error: AppError, request_context: Option<RequestContext>) -> Response {
    let status_code = error.status_code();
    
    // Log the error appropriately
    if error.should_log() {
        error!(
            error = %error,
            error_code = error.error_code(),
            status_code = %status_code,
            "Application error occurred"
        );
    } else {
        warn!(
            error = %error,
            error_code = error.error_code(),
            status_code = %status_code,
            "Client error occurred"
        );
    }

    let error_response = ErrorResponse {
        error: ErrorDetail {
            code: error.error_code().to_string(),
            message: error.to_string(),
            details: None,
        },
        request_id: request_context.map(|ctx| ctx.request_id),
    };

    (status_code, Json(error_response)).into_response()
}

/// Rate limiting error
pub fn rate_limit_error() -> Response {
    let error_response = ErrorResponse {
        error: ErrorDetail {
            code: "RATE_LIMIT_EXCEEDED".to_string(),
            message: "Rate limit exceeded. Please try again later.".to_string(),
            details: None,
        },
        request_id: None,
    };

    (StatusCode::TOO_MANY_REQUESTS, Json(error_response)).into_response()
}

/// Validation error helper
pub fn validation_error(message: &str, details: Option<serde_json::Value>) -> Response {
    let error_response = ErrorResponse {
        error: ErrorDetail {
            code: "VALIDATION_ERROR".to_string(),
            message: message.to_string(),
            details,
        },
        request_id: None,
    };

    (StatusCode::BAD_REQUEST, Json(error_response)).into_response()
}

/// Authentication error helper
pub fn auth_error(message: &str) -> Response {
    let error_response = ErrorResponse {
        error: ErrorDetail {
            code: "AUTHENTICATION_ERROR".to_string(),
            message: message.to_string(),
            details: None,
        },
        request_id: None,
    };

    (StatusCode::UNAUTHORIZED, Json(error_response)).into_response()
}

/// Authorization error helper
pub fn authorization_error(message: &str) -> Response {
    let error_response = ErrorResponse {
        error: ErrorDetail {
            code: "AUTHORIZATION_ERROR".to_string(),
            message: message.to_string(),
            details: None,
        },
        request_id: None,
    };

    (StatusCode::FORBIDDEN, Json(error_response)).into_response()
}

/// Not found error helper
pub fn not_found_error(resource: &str) -> Response {
    let error_response = ErrorResponse {
        error: ErrorDetail {
            code: "NOT_FOUND".to_string(),
            message: format!("{} not found", resource),
            details: None,
        },
        request_id: None,
    };

    (StatusCode::NOT_FOUND, Json(error_response)).into_response()
}

/// Internal server error helper
pub fn internal_error(message: &str) -> Response {
    let error_response = ErrorResponse {
        error: ErrorDetail {
            code: "INTERNAL_ERROR".to_string(),
            message: message.to_string(),
            details: None,
        },
        request_id: None,
    };

    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
}

/// Service unavailable error helper
pub fn service_unavailable_error(service: &str) -> Response {
    let error_response = ErrorResponse {
        error: ErrorDetail {
            code: "SERVICE_UNAVAILABLE".to_string(),
            message: format!("Service {} is currently unavailable", service),
            details: None,
        },
        request_id: None,
    };

    (StatusCode::SERVICE_UNAVAILABLE, Json(error_response)).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validation_error_response() {
        let response = validation_error(
            "Invalid input",
            Some(json!({"field": "name", "reason": "too_short"}))
        );
        
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn test_not_found_error_response() {
        let response = not_found_error("Product");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_auth_error_response() {
        let response = auth_error("Invalid token");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}