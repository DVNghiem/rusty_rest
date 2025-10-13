use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::Json,
    routing::{delete, get, patch, post, put},
    Router,
};
use core::ProductService;
use domain::{ProductEntity, ProductId, ProductStatus, ProductCategory, ProductFilters, ProductSort, ProductChanges};
use serde::{Deserialize, Serialize};
use shared::{ApiResponse, AppResult, PaginationParams, PaginatedResponse, Timestamp};
use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::AppState;

/// Product routes
pub fn product_routes() -> Router {
    Router::new()
        .route("/", get(list_products).post(create_product))
        .route("/:id", get(get_product).put(update_product).delete(delete_product))
        .route("/:id/status", patch(change_product_status))
        .route("/:id/reserve", post(reserve_inventory))
        .route("/:id/release", post(release_inventory))
        .route("/search", get(search_products))
        .route("/statistics", get(get_statistics))
        .route("/inventory-summary", get(get_inventory_summary))
        .route("/batch", post(create_products_batch))
}

// Request/Response DTOs

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub price: ProductPriceDto,
    pub category: ProductCategory,
    pub inventory: ProductInventoryDto,
    pub metadata: Option<ProductMetadataDto>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub price: Option<ProductPriceDto>,
    pub category: Option<ProductCategory>,
    pub inventory: Option<ProductInventoryDto>,
    pub metadata: Option<ProductMetadataDto>,
}

#[derive(Debug, Deserialize)]
pub struct ChangeStatusRequest {
    pub status: ProductStatus,
}

#[derive(Debug, Deserialize)]
pub struct InventoryActionRequest {
    pub quantity: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductPriceDto {
    pub amount_dollars: f64,
    pub currency: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductInventoryDto {
    pub quantity: i32,
    pub min_stock: i32,
    pub max_stock: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductMetadataDto {
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub weight: Option<ProductWeightDto>,
    pub dimensions: Option<ProductDimensionsDto>,
    pub tags: Vec<String>,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductWeightDto {
    pub value: f64,
    pub unit: domain::WeightUnit,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductDimensionsDto {
    pub length: f64,
    pub width: f64,
    pub height: f64,
    pub unit: domain::DimensionUnit,
}

#[derive(Debug, Deserialize)]
pub struct ProductQueryParams {
    #[serde(flatten)]
    pub pagination: PaginationParams,
    pub status: Option<ProductStatus>,
    pub category: Option<ProductCategory>,
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,
    pub in_stock_only: Option<bool>,
    pub low_stock_only: Option<bool>,
    pub name_contains: Option<String>,
    pub tags: Option<String>, // Comma-separated tags
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub id: ProductId,
    pub name: String,
    pub description: Option<String>,
    pub price: ProductPriceDto,
    pub category: ProductCategory,
    pub status: ProductStatus,
    pub inventory: ProductInventoryDto,
    pub metadata: ProductMetadataDto,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

// Route Handlers

/// Create a new product
#[instrument(skip(state))]
async fn create_product(
    Extension(state): Extension<AppState>,
    Json(request): Json<CreateProductRequest>,
) -> Result<Json<ApiResponse<ProductResponse>>, StatusCode> {
    let product = ProductEntity::new(
        request.name,
        request.description,
        request.price.into(),
        request.category,
        request.inventory.into(),
        request.metadata.map(Into::into),
    );

    let created_product = state.product_service
        .create_product(product)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::new(created_product.into())))
}

/// Get product by ID
#[instrument(skip(state))]
async fn get_product(
    Extension(state): Extension<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<ProductResponse>>, StatusCode> {
    let product_id = ProductId::from_uuid(id);
    
    let product = state.product_service
        .get_product(&product_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ApiResponse::new(product.into())))
}

/// Update product
#[instrument(skip(state))]
async fn update_product(
    Extension(state): Extension<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateProductRequest>,
) -> Result<Json<ApiResponse<ProductResponse>>, StatusCode> {
    let product_id = ProductId::from_uuid(id);
    
    let changes = ProductChanges {
        name: request.name,
        description: request.description,
        price: request.price.map(Into::into),
        category: request.category,
        inventory: request.inventory.map(Into::into),
        metadata: request.metadata.map(Into::into),
    };

    let updated_product = state.product_service
        .update_product(&product_id, changes)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::new(updated_product.into())))
}

/// Delete product
#[instrument(skip(state))]
async fn delete_product(
    Extension(state): Extension<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let product_id = ProductId::from_uuid(id);
    
    state.product_service
        .delete_product(&product_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::NO_CONTENT)
}

/// List products with filtering and pagination
#[instrument(skip(state))]
async fn list_products(
    Extension(state): Extension<AppState>,
    Query(params): Query<ProductQueryParams>,
) -> Result<Json<PaginatedResponse<ProductResponse>>, StatusCode> {
    let filters = ProductFilters {
        status: params.status,
        category: params.category,
        min_price: params.min_price.map(|p| (p * 100.0) as i64), // Convert to cents
        max_price: params.max_price.map(|p| (p * 100.0) as i64),
        in_stock_only: params.in_stock_only,
        low_stock_only: params.low_stock_only,
        name_contains: params.name_contains,
        tags: params.tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
        created_after: None,
        created_before: None,
    };

    let products = state.product_service
        .list_products(filters, None, params.pagination)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Convert to response DTOs
    let response_products = PaginatedResponse {
        data: products.data.into_iter().map(Into::into).collect(),
        meta: products.meta,
    };

    Ok(Json(response_products))
}

/// Change product status
#[instrument(skip(state))]
async fn change_product_status(
    Extension(state): Extension<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ChangeStatusRequest>,
) -> Result<Json<ApiResponse<ProductResponse>>, StatusCode> {
    let product_id = ProductId::from_uuid(id);
    
    let updated_product = state.product_service
        .change_product_status(&product_id, request.status)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::new(updated_product.into())))
}

/// Reserve inventory
#[instrument(skip(state))]
async fn reserve_inventory(
    Extension(state): Extension<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<InventoryActionRequest>,
) -> Result<Json<ApiResponse<ProductResponse>>, StatusCode> {
    let product_id = ProductId::from_uuid(id);
    
    let updated_product = state.product_service
        .reserve_inventory(&product_id, request.quantity)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::new(updated_product.into())))
}

/// Release inventory
#[instrument(skip(state))]
async fn release_inventory(
    Extension(state): Extension<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<InventoryActionRequest>,
) -> Result<Json<ApiResponse<ProductResponse>>, StatusCode> {
    let product_id = ProductId::from_uuid(id);
    
    let updated_product = state.product_service
        .release_inventory(&product_id, request.quantity)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::new(updated_product.into())))
}

/// Search products (alias for list with search parameters)
#[instrument(skip(state))]
async fn search_products(
    Extension(state): Extension<AppState>,
    Query(params): Query<ProductQueryParams>,
) -> Result<Json<PaginatedResponse<ProductResponse>>, StatusCode> {
    list_products(Extension(state), Query(params)).await
}

/// Get product statistics
#[instrument(skip(state))]
async fn get_statistics(
    Extension(state): Extension<AppState>,
) -> Result<Json<ApiResponse<domain::ProductStatistics>>, StatusCode> {
    let statistics = state.product_service
        .get_statistics()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::new(statistics)))
}

/// Get inventory summary
#[instrument(skip(state))]
async fn get_inventory_summary(
    Extension(state): Extension<AppState>,
) -> Result<Json<ApiResponse<domain::InventorySummary>>, StatusCode> {
    let summary = state.product_service
        .get_inventory_summary()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(ApiResponse::new(summary)))
}

/// Create multiple products in batch
#[instrument(skip(state))]
async fn create_products_batch(
    Extension(state): Extension<AppState>,
    Json(requests): Json<Vec<CreateProductRequest>>,
) -> Result<Json<ApiResponse<Vec<ProductResponse>>>, StatusCode> {
    let products: Vec<ProductEntity> = requests
        .into_iter()
        .map(|req| {
            ProductEntity::new(
                req.name,
                req.description,
                req.price.into(),
                req.category,
                req.inventory.into(),
                req.metadata.map(Into::into),
            )
        })
        .collect();

    let created_products = state.product_service
        .create_products_batch(products)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response_products: Vec<ProductResponse> = created_products
        .into_iter()
        .map(Into::into)
        .collect();

    Ok(Json(ApiResponse::new(response_products)))
}

// DTO Conversions

impl From<ProductPriceDto> for domain::ProductPrice {
    fn from(dto: ProductPriceDto) -> Self {
        Self::new(dto.amount_dollars, dto.currency)
    }
}

impl From<domain::ProductPrice> for ProductPriceDto {
    fn from(price: domain::ProductPrice) -> Self {
        Self {
            amount_dollars: price.amount_as_float(),
            currency: price.currency,
        }
    }
}

impl From<ProductInventoryDto> for domain::ProductInventory {
    fn from(dto: ProductInventoryDto) -> Self {
        Self::new(dto.quantity, dto.min_stock, dto.max_stock)
    }
}

impl From<domain::ProductInventory> for ProductInventoryDto {
    fn from(inventory: domain::ProductInventory) -> Self {
        Self {
            quantity: inventory.quantity,
            min_stock: inventory.min_stock,
            max_stock: inventory.max_stock,
        }
    }
}

impl From<ProductMetadataDto> for domain::ProductMetadata {
    fn from(dto: ProductMetadataDto) -> Self {
        Self {
            sku: dto.sku,
            barcode: dto.barcode,
            weight: dto.weight.map(|w| domain::ProductWeight {
                value: w.value,
                unit: w.unit,
            }),
            dimensions: dto.dimensions.map(|d| domain::ProductDimensions {
                length: d.length,
                width: d.width,
                height: d.height,
                unit: d.unit,
            }),
            tags: dto.tags,
            attributes: dto.attributes,
        }
    }
}

impl From<domain::ProductMetadata> for ProductMetadataDto {
    fn from(metadata: domain::ProductMetadata) -> Self {
        Self {
            sku: metadata.sku,
            barcode: metadata.barcode,
            weight: metadata.weight.map(|w| ProductWeightDto {
                value: w.value,
                unit: w.unit,
            }),
            dimensions: metadata.dimensions.map(|d| ProductDimensionsDto {
                length: d.length,
                width: d.width,
                height: d.height,
                unit: d.unit,
            }),
            tags: metadata.tags,
            attributes: metadata.attributes,
        }
    }
}

impl From<ProductEntity> for ProductResponse {
    fn from(entity: ProductEntity) -> Self {
        Self {
            id: entity.id,
            name: entity.name,
            description: entity.description,
            price: entity.price.into(),
            category: entity.category,
            status: entity.status,
            inventory: entity.inventory.into(),
            metadata: entity.metadata.into(),
            created_at: entity.created_at,
            updated_at: entity.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_dto_conversion() {
        let dto = ProductPriceDto {
            amount_dollars: 29.99,
            currency: "USD".to_string(),
        };
        
        let price: domain::ProductPrice = dto.into();
        assert_eq!(price.amount, 2999); // Should be in cents
        assert_eq!(price.currency, "USD");
        
        let back_to_dto: ProductPriceDto = price.into();
        assert_eq!(back_to_dto.amount_dollars, 29.99);
    }

    #[test]
    fn test_inventory_dto_conversion() {
        let dto = ProductInventoryDto {
            quantity: 100,
            min_stock: 10,
            max_stock: Some(500),
        };
        
        let inventory: domain::ProductInventory = dto.into();
        assert_eq!(inventory.quantity, 100);
        assert_eq!(inventory.min_stock, 10);
        assert_eq!(inventory.max_stock, Some(500));
        
        let back_to_dto: ProductInventoryDto = inventory.into();
        assert_eq!(back_to_dto.quantity, 100);
    }
}