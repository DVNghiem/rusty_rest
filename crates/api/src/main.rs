mod config;
mod routes;
mod middleware;

extern crate core as core_crate;

use axum::{
    extract::Extension,
    http::{header, Method},
    response::Json,
    routing::get,
    Router,
};
use config::AppConfig;
use core_crate::ProductService;
use infrastructure::{PostgresProductRepository, RedisCache};
use shared::{ApiResponse, AppResult, HealthResponse, HealthStatus, ServiceHealth, Timestamp};
use sqlx::PgPool;
use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    timeout::TimeoutLayer,
    compression::CompressionLayer,
};
use tracing::{info, instrument};



/// Application state shared across handlers
#[derive(Clone, Debug)]
pub struct AppState {
    pub product_service: Arc<ProductService>,
    pub config: Arc<AppConfig>,
    pub health_info: Arc<HealthInfo>,
}

#[derive(Clone, Debug)]
pub struct HealthInfo {
    pub started_at: Timestamp,
    pub version: String,
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logging
    shared::Logger::init();

    // Load configuration
    let config = Arc::new(AppConfig::load().map_err(|e| shared::AppError::Config(e.to_string()))?);
    info!("Configuration loaded successfully");

    // Initialize database connection
    let pool = PgPool::connect(&config.database.url).await?;
    info!("Connected to database");

    // Run database migrations
    let repository = PostgresProductRepository::new(pool.clone());
    repository.migrate().await?;
    info!("Database migrations completed");

    // Initialize Redis cache
    let cache = Arc::new(
        RedisCache::new(&config.redis.url, Duration::from_secs(config.redis.default_ttl_seconds))
            .await?,
    );
    info!("Connected to Redis");

    // Initialize services
    let product_service = Arc::new(ProductService::new(
        Arc::new(repository),
        cache.clone(),
        None, // External service client can be added later
        None, // Event publisher can be added later
    ));

    // Create application state
    let health_info = Arc::new(HealthInfo {
        started_at: Timestamp::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    });

    let app_state = AppState {
        product_service,
        config: config.clone(),
        health_info,
    };

    // Build application router
    let app = create_app(app_state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    info!(?addr, "Starting server");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Create the main application router with all middleware and routes
fn create_app(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", api_routes())
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .layer(
            ServiceBuilder::new()
                // Add tracing (simplified)
                .layer(TraceLayer::new_for_http())
                // Add timeout
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                // Add compression
                .layer(CompressionLayer::new())
                // Add CORS support
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH])
                        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE])
                        .expose_headers([header::CONTENT_LENGTH])
                        .max_age(Duration::from_secs(3600)),
                )
                // Add request context middleware
                .layer(axum::middleware::from_fn(middleware::request_context))
                // Add error handling middleware
                .layer(axum::middleware::from_fn(middleware::error_handler)),
        )
        .layer(Extension(state))
}

/// API routes under /api/v1
fn api_routes() -> Router {
    Router::new()
        .nest("/products", routes::product_routes())
        .route("/", get(api_info_handler))
}

/// Health check endpoint
#[instrument]
async fn health_handler(Extension(state): Extension<AppState>) -> Json<HealthResponse> {
    let mut services = HashMap::new();

    // Check product service health
    match state.product_service.health_check().await {
        Ok(health) => {
            services.insert("product_service".to_string(), health);
        }
        Err(_) => {
            services.insert(
                "product_service".to_string(),
                ServiceHealth {
                    status: HealthStatus::Unhealthy,
                    message: Some("Service unavailable".to_string()),
                    response_time_ms: None,
                },
            );
        }
    }

    // Determine overall status
    let overall_status = if services.values().all(|s| matches!(s.status, HealthStatus::Healthy)) {
        HealthStatus::Healthy
    } else if services.values().any(|s| matches!(s.status, HealthStatus::Healthy)) {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    };

    Json(HealthResponse {
        status: overall_status,
        services,
        timestamp: Timestamp::now(),
    })
}

/// Metrics endpoint (simplified - in production you'd use proper metrics)
#[instrument]
async fn metrics_handler(Extension(state): Extension<AppState>) -> Json<serde_json::Value> {
    let uptime = state.health_info.started_at.as_datetime().timestamp();
    let current_time = chrono::Utc::now().timestamp();
    let uptime_seconds = current_time - uptime;

    Json(serde_json::json!({
        "uptime_seconds": uptime_seconds,
        "version": state.health_info.version,
        "started_at": state.health_info.started_at,
        // Add more metrics as needed
    }))
}

/// API information endpoint
#[instrument]
async fn api_info_handler() -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse::new(serde_json::json!({
        "name": "Product API",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Enterprise Product Management API built with Axum",
        "endpoints": {
            "health": "/health",
            "metrics": "/metrics",
            "products": "/api/v1/products"
        }
    })))
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     
//     #[tokio::test]
//     async fn test_health_endpoint() {
//         // This would require setting up a test database and Redis
//         // For now, just test that the route is configured
//         // In a real application, you'd use test containers or mocks
//     }

//     #[tokio::test]
//     async fn test_api_info_endpoint() {
//         // Mock state for testing
//         let health_info = Arc::new(HealthInfo {
//             started_at: Timestamp::now(),
//             version: "test".to_string(),
//         });

//         // This test would be more comprehensive with proper mocking
//     }
// }