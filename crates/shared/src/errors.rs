use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

/// Application-wide error types following enterprise error handling patterns
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Cache error: {0}")]
    Cache(String),
    
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {resource}")]
    NotFound { resource: String },
    
    #[error("Unauthorized: {message}")]
    Unauthorized { message: String },
    
    #[error("Forbidden: {message}")]
    Forbidden { message: String },
    
    #[error("Conflict: {message}")]
    Conflict { message: String },
    
    #[error("Rate limit exceeded")]
    RateLimit,
    
    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },
    
    #[error("Internal server error: {0}")]
    Internal(#[from] anyhow::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Config error: {0}")]
    Config(String),
}

impl AppError {
    /// Get the HTTP status code for this error
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Cache(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::HttpClient(_) => StatusCode::BAD_GATEWAY,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound { .. } => StatusCode::NOT_FOUND,
            AppError::Unauthorized { .. } => StatusCode::UNAUTHORIZED,
            AppError::Forbidden { .. } => StatusCode::FORBIDDEN,
            AppError::Conflict { .. } => StatusCode::CONFLICT,
            AppError::RateLimit => StatusCode::TOO_MANY_REQUESTS,
            AppError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Config(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get error code for logging and monitoring
    pub fn error_code(&self) -> &'static str {
        match self {
            AppError::Database(_) => "DATABASE_ERROR",
            AppError::Cache(_) => "CACHE_ERROR",
            AppError::HttpClient(_) => "HTTP_CLIENT_ERROR",
            AppError::Validation(_) => "VALIDATION_ERROR",
            AppError::NotFound { .. } => "NOT_FOUND",
            AppError::Unauthorized { .. } => "UNAUTHORIZED",
            AppError::Forbidden { .. } => "FORBIDDEN",
            AppError::Conflict { .. } => "CONFLICT",
            AppError::RateLimit => "RATE_LIMIT",
            AppError::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            AppError::Internal(_) => "INTERNAL_ERROR",
            AppError::Io(_) => "IO_ERROR",
            AppError::Config(_) => "CONFIG_ERROR",
        }
    }

    /// Check if error should be logged (avoiding spam for client errors)
    pub fn should_log(&self) -> bool {
        matches!(
            self,
            AppError::Database(_)
                | AppError::Cache(_)
                | AppError::HttpClient(_)
                | AppError::ServiceUnavailable { .. }
                | AppError::Internal(_)
        )
    }
}

/// Standard error response format
#[derive(serde::Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
    pub request_id: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status_code = self.status_code();
        
        if self.should_log() {
            tracing::error!(
                error = %self,
                error_code = self.error_code(),
                status_code = %status_code,
                "Application error occurred"
            );
        } else {
            tracing::warn!(
                error = %self,
                error_code = self.error_code(),
                status_code = %status_code,
                "Client error occurred"
            );
        }

        let error_response = ErrorResponse {
            error: ErrorDetail {
                code: self.error_code().to_string(),
                message: self.to_string(),
                details: None,
            },
            request_id: None, // TODO: Extract from request context
        };

        let body = Json(json!(error_response));
        (status_code, body).into_response()
    }
}

/// Result type alias for convenience
pub type AppResult<T> = Result<T, AppError>;

/// Helper macros for common error patterns
#[macro_export]
macro_rules! not_found {
    ($resource:expr) => {
        AppError::NotFound {
            resource: $resource.to_string(),
        }
    };
}

#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        $crate::AppError::Validation($msg.to_string())
    };
}

#[macro_export]
macro_rules! cache_error {
    ($msg:expr) => {
        $crate::AppError::Cache($msg.to_string())
    };
}

// Additional From implementations for specific crates should be implemented in those crates
impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::Internal(anyhow::Error::from(error))
    }
}