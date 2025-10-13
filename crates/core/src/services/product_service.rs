use async_trait::async_trait;
use domain::{
    ProductEntity, ProductId, ProductRepositoryExt, ProductStatus,
    ProductCategory, ProductFilters, ProductSort, ProductStatistics, InventorySummary,
    ProductChanges, ProductEvent,
};
use infrastructure::{RedisCache, ExternalServiceClient};
use shared::{AppResult, AppError, PaginationParams, PaginatedResponse, Timestamp};
use std::sync::Arc;
use tracing::{info, warn, instrument};

/// Product service implementation with caching and validation
pub struct ProductService {
    repository: Arc<dyn ProductRepositoryExt>,
    cache: Arc<RedisCache>,
    external_client: Option<Arc<ExternalServiceClient>>,
    event_publisher: Option<Arc<dyn EventPublisher>>,
}

impl std::fmt::Debug for ProductService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProductService")
            .field("repository", &"<repository>")
            .field("cache", &"<cache>")
            .field("external_client", &self.external_client.is_some())
            .field("event_publisher", &self.event_publisher.is_some())
            .finish()
    }
}

impl ProductService {
    pub fn new(
        repository: Arc<dyn ProductRepositoryExt>,
        cache: Arc<RedisCache>,
        external_client: Option<Arc<ExternalServiceClient>>,
        event_publisher: Option<Arc<dyn EventPublisher>>,
    ) -> Self {
        Self {
            repository,
            cache,
            external_client,
            event_publisher,
        }
    }

    /// Create a new product with validation and caching
    #[instrument(skip(self, product))]
    pub async fn create_product(&self, product: ProductEntity) -> AppResult<ProductEntity> {
        // Validate product before creation
        product.is_valid().map_err(|errors| {
            AppError::Validation(format!("Product validation failed: {:?}", errors))
        })?;

        // Check if product with same SKU exists (if SKU is provided)
        if let Some(sku) = &product.metadata.sku {
            if self.product_exists_by_sku(sku).await? {
                return Err(shared::validation_error!(&format!("Product with SKU {} already exists", sku)));
            }
        }

        // Save to database
        self.repository.save(&product).await?;

        // Cache the product
        if let Err(e) = self.cache.cache_product(&product).await {
            warn!(error = %e, product_id = %product.id, "Failed to cache product");
        }

        // Publish creation event
        if let Some(publisher) = &self.event_publisher {
            let event = ProductEvent::Created {
                product: product.clone(),
                timestamp: Timestamp::now(),
            };
            if let Err(e) = publisher.publish(event).await {
                warn!(error = %e, "Failed to publish product creation event");
            }
        }

        // Invalidate related caches
        self.invalidate_list_caches().await;

        info!(product_id = %product.id, "Product created successfully");
        Ok(product)
    }

    /// Get product by ID with cache-aside pattern
    #[instrument(skip(self))]
    pub async fn get_product(&self, id: &ProductId) -> AppResult<Option<ProductEntity>> {
        // Try cache first
        match self.cache.get_product(id).await {
            Ok(Some(product)) => {
                info!(product_id = %id, "Product found in cache");
                return Ok(Some(product));
            }
            Ok(None) => {}
            Err(e) => {
                warn!(error = %e, product_id = %id, "Cache lookup failed");
            }
        }

        // Fallback to database
        let product = self.repository.find_by_id(id).await?;

        // Cache the result if found
        if let Some(ref product) = product {
            if let Err(e) = self.cache.cache_product(product).await {
                warn!(error = %e, product_id = %id, "Failed to cache product");
            }
        }

        Ok(product)
    }

    /// Update product with optimistic locking and event publishing
    #[instrument(skip(self, changes))]
    pub async fn update_product(
        &self,
        id: &ProductId,
        changes: ProductChanges,
    ) -> AppResult<ProductEntity> {
        // Get current product
        let mut product = self.get_product(id).await?
            .ok_or_else(|| shared::not_found!("Product"))?;

        let old_product = product.clone();

        // Apply changes
        product.update(changes.clone());

        // Validate updated product
        product.is_valid().map_err(|errors| {
            AppError::Validation(format!("Product validation failed: {:?}", errors))
        })?;

        // Save to database
        self.repository.update(&product).await?;

        // Update cache
        if let Err(e) = self.cache.cache_product(&product).await {
            warn!(error = %e, product_id = %id, "Failed to update cached product");
        }

        // Publish update event
        if let Some(publisher) = &self.event_publisher {
            let event = ProductEvent::Updated {
                product_id: id.clone(),
                changes,
                timestamp: Timestamp::now(),
            };
            if let Err(e) = publisher.publish(event).await {
                warn!(error = %e, "Failed to publish product update event");
            }
        }

        // Check for significant changes that require cache invalidation
        if old_product.status != product.status 
            || old_product.category != product.category 
            || old_product.price.amount != product.price.amount {
            self.invalidate_list_caches().await;
        }

        info!(product_id = %id, "Product updated successfully");
        Ok(product)
    }

    /// Delete product with cascade operations
    #[instrument(skip(self))]
    pub async fn delete_product(&self, id: &ProductId) -> AppResult<()> {
        // Check if product exists
        let product = self.get_product(id).await?
            .ok_or_else(|| shared::not_found!("Product"))?;

        // Check business rules (e.g., can't delete product with reserved inventory)
        if product.inventory.reserved > 0 {
            return Err(shared::validation_error!(
                "Cannot delete product with reserved inventory"
            ));
        }

        // Delete from database
        self.repository.delete(id).await?;

        // Remove from cache
        if let Err(e) = self.cache.invalidate_product(id).await {
            warn!(error = %e, product_id = %id, "Failed to invalidate cached product");
        }

        // Invalidate related caches
        self.invalidate_list_caches().await;

        info!(product_id = %id, "Product deleted successfully");
        Ok(())
    }

    /// List products with advanced filtering and caching
    #[instrument(skip(self))]
    pub async fn list_products(
        &self,
        filters: ProductFilters,
        sort: Option<ProductSort>,
        pagination: PaginationParams,
    ) -> AppResult<PaginatedResponse<ProductEntity>> {
        // For simple queries, try cache first
        let cache_key = self.build_list_cache_key(&filters, &pagination);
        let is_cacheable = filters.is_cacheable();
        
        if is_cacheable {
            match self.cache.get::<PaginatedResponse<ProductEntity>>(&cache_key).await {
                Ok(Some(cached)) => {
                    info!(cache_key, "Products list found in cache");
                    return Ok(cached);
                }
                Ok(None) => {}
                Err(e) => {
                    warn!(error = %e, cache_key, "Failed to get cached product list");
                }
            }
        }

        // Query database
        let products = self.repository.find_with_filters(filters, sort, pagination).await?;

        // Cache the result if it's cacheable
        if is_cacheable {
            if let Err(e) = self.cache.set_with_ttl(
                &cache_key,
                &products,
                std::time::Duration::from_secs(300), // 5 minutes
            ).await {
                warn!(error = %e, cache_key, "Failed to cache product list");
            }
        }

        Ok(products)
    }

    /// Change product status with business rule validation
    #[instrument(skip(self))]
    pub async fn change_product_status(
        &self,
        id: &ProductId,
        new_status: ProductStatus,
    ) -> AppResult<ProductEntity> {
        let mut product = self.get_product(id).await?
            .ok_or_else(|| shared::not_found!("Product"))?;

        let old_status = product.status.clone();

        // Business rules for status changes
        match (old_status.clone(), new_status.clone()) {
            (ProductStatus::Discontinued, ProductStatus::Active) => {
                return Err(shared::validation_error!(
                    "Cannot activate discontinued product"
                ));
            }
            (ProductStatus::Active, ProductStatus::Discontinued) => {
                // Check if product has reserved inventory
                if product.inventory.reserved > 0 {
                    return Err(shared::validation_error!(
                        "Cannot discontinue product with reserved inventory"
                    ));
                }
            }
            _ => {} // Other transitions are allowed
        }

        product.change_status(new_status);
        self.repository.update(&product).await?;

        // Update cache
        if let Err(e) = self.cache.cache_product(&product).await {
            warn!(error = %e, product_id = %id, "Failed to update cached product");
        }

        // Publish status change event
        if let Some(publisher) = &self.event_publisher {
            let event = ProductEvent::StatusChanged {
                product_id: id.clone(),
                old_status,
                new_status,
                timestamp: Timestamp::now(),
            };
            if let Err(e) = publisher.publish(event).await {
                warn!(error = %e, "Failed to publish status change event");
            }
        }

        // Invalidate status-based caches
        self.invalidate_list_caches().await;

        info!(product_id = %id, ?new_status, "Product status changed");
        Ok(product)
    }

    /// Reserve inventory for orders
    #[instrument(skip(self))]
    pub async fn reserve_inventory(
        &self,
        id: &ProductId,
        quantity: i32,
    ) -> AppResult<ProductEntity> {
        if quantity <= 0 {
            return Err(shared::validation_error!("Quantity must be positive"));
        }

        let mut product = self.get_product(id).await?
            .ok_or_else(|| shared::not_found!("Product"))?;

        // Check if product is available for reservation
        if product.status != ProductStatus::Active {
            return Err(shared::validation_error!("Product is not active"));
        }

        // Reserve inventory
        product.reserve_inventory(quantity)
            .map_err(|e| shared::validation_error!(&e))?;

        // Update in database
        self.repository.update(&product).await?;

        // Update cache
        if let Err(e) = self.cache.cache_product(&product).await {
            warn!(error = %e, product_id = %id, "Failed to update cached product");
        }

        // Publish inventory adjustment event
        if let Some(publisher) = &self.event_publisher {
            let event = ProductEvent::InventoryAdjusted {
                product_id: id.clone(),
                old_quantity: product.inventory.quantity + quantity,
                new_quantity: product.inventory.quantity,
                reason: format!("Reserved {} items", quantity),
                timestamp: Timestamp::now(),
            };
            if let Err(e) = publisher.publish(event).await {
                warn!(error = %e, "Failed to publish inventory event");
            }
        }

        info!(product_id = %id, quantity, "Inventory reserved");
        Ok(product)
    }

    /// Release reserved inventory
    #[instrument(skip(self))]
    pub async fn release_inventory(
        &self,
        id: &ProductId,
        quantity: i32,
    ) -> AppResult<ProductEntity> {
        if quantity <= 0 {
            return Err(shared::validation_error!("Quantity must be positive"));
        }

        let mut product = self.get_product(id).await?
            .ok_or_else(|| shared::not_found!("Product"))?;

        // Release inventory
        product.release_inventory(quantity)
            .map_err(|e| shared::validation_error!(&e))?;

        // Update in database
        self.repository.update(&product).await?;

        // Update cache
        if let Err(e) = self.cache.cache_product(&product).await {
            warn!(error = %e, product_id = %id, "Failed to update cached product");
        }

        info!(product_id = %id, quantity, "Inventory released");
        Ok(product)
    }

    /// Get product statistics with caching
    #[instrument(skip(self))]
    pub async fn get_statistics(&self) -> AppResult<ProductStatistics> {
        let cache_key = RedisCache::product_statistics_key();

        // Try cache first
        match self.cache.get::<ProductStatistics>(&cache_key).await {
            Ok(Some(stats)) => {
                info!("Product statistics found in cache");
                return Ok(stats);
            }
            Ok(None) => {}
            Err(e) => {
                warn!(error = %e, "Failed to get cached statistics");
            }
        }

        // Calculate from database
        let stats = self.repository.get_statistics().await?;

        // Cache for 10 minutes
        if let Err(e) = self.cache.set_with_ttl(
            &cache_key,
            &stats,
            std::time::Duration::from_secs(600),
        ).await {
            warn!(error = %e, "Failed to cache statistics");
        }

        Ok(stats)
    }

    /// Get inventory summary with caching
    #[instrument(skip(self))]
    pub async fn get_inventory_summary(&self) -> AppResult<InventorySummary> {
        let cache_key = RedisCache::inventory_summary_key();

        // Try cache first
        match self.cache.get::<InventorySummary>(&cache_key).await {
            Ok(Some(summary)) => {
                info!("Inventory summary found in cache");
                return Ok(summary);
            }
            Ok(None) => {}
            Err(e) => {
                warn!(error = %e, "Failed to get cached inventory summary");
            }
        }

        // Calculate from database
        let summary = self.repository.get_inventory_summary().await?;

        // Cache for 5 minutes
        if let Err(e) = self.cache.set_with_ttl(
            &cache_key,
            &summary,
            std::time::Duration::from_secs(300),
        ).await {
            warn!(error = %e, "Failed to cache inventory summary");
        }

        Ok(summary)
    }

    /// Bulk operations for better performance
    #[instrument(skip(self, products))]
    pub async fn create_products_batch(&self, products: Vec<ProductEntity>) -> AppResult<Vec<ProductEntity>> {
        // Validate all products
        for product in &products {
            product.is_valid().map_err(|errors| {
                AppError::Validation(format!("Product validation failed: {:?}", errors))
            })?;
        }

        // Save in batch
        self.repository.save_batch(&products).await?;

        // Cache products individually (fire and forget)
        for product in &products {
            if let Err(e) = self.cache.cache_product(product).await {
                warn!(error = %e, product_id = %product.id, "Failed to cache product in batch");
            }
        }

        // Invalidate list caches
        self.invalidate_list_caches().await;

        info!(count = products.len(), "Products created in batch");
        Ok(products)
    }

    // Private helper methods

    async fn product_exists_by_sku(&self, sku: &str) -> AppResult<bool> {
        // This is a simplified check - in reality you'd have a proper query
        let filters = ProductFilters {
            name_contains: Some(sku.to_string()), // Simplified - would need proper SKU search
            ..Default::default()
        };
        
        let result = self.repository.find_with_filters(
            filters,
            None,
            PaginationParams::new(1, 1),
        ).await?;

        Ok(!result.data.is_empty())
    }

    fn build_list_cache_key(&self, filters: &ProductFilters, pagination: &PaginationParams) -> String {
        // Simplified cache key generation - in production, you'd use a more sophisticated approach
        format!("products:list:{}:{}", 
                serde_json::to_string(filters).unwrap_or_default(), 
                format!("{}:{}", pagination.page, pagination.limit))
    }

    async fn invalidate_list_caches(&self) {
        if let Err(e) = self.cache.invalidate_pattern("products:list:*").await {
            warn!(error = %e, "Failed to invalidate product list caches");
        }
        
        // Also invalidate statistics and summary caches
        let _ = self.cache.delete(&RedisCache::product_statistics_key()).await;
        let _ = self.cache.delete(&RedisCache::inventory_summary_key()).await;
    }
}

/// Event publishing trait for domain events
#[async_trait]
pub trait EventPublisher: Send + Sync {
    async fn publish(&self, event: ProductEvent) -> AppResult<()>;
}



/// Health check service for product-related components
impl ProductService {
    pub async fn health_check(&self) -> AppResult<shared::ServiceHealth> {
        let mut checks = Vec::new();

        // Check cache connectivity
        let cache_start = std::time::Instant::now();
        let cache_healthy = self.cache.ping().await.is_ok();
        let cache_duration = cache_start.elapsed();
        
        checks.push(("cache", cache_healthy, Some(cache_duration.as_millis() as u64)));

        // Check database connectivity (simplified)
        let db_start = std::time::Instant::now();
        let db_healthy = self.repository.count_all().await.is_ok();
        let db_duration = db_start.elapsed();
        
        checks.push(("database", db_healthy, Some(db_duration.as_millis() as u64)));

        // Check external service if configured
        if let Some(client) = &self.external_client {
            let ext_start = std::time::Instant::now();
            let ext_healthy = client.health_check().await.unwrap_or(false);
            let ext_duration = ext_start.elapsed();
            
            checks.push(("external_service", ext_healthy, Some(ext_duration.as_millis() as u64)));
        }

        // Determine overall health
        let all_healthy = checks.iter().all(|(_, healthy, _)| *healthy);
        let any_healthy = checks.iter().any(|(_, healthy, _)| *healthy);

        let status = if all_healthy {
            shared::HealthStatus::Healthy
        } else if any_healthy {
            shared::HealthStatus::Degraded
        } else {
            shared::HealthStatus::Unhealthy
        };

        let response_time = checks.iter()
            .filter_map(|(_, _, duration)| *duration)
            .max()
            .unwrap_or(0);

        Ok(shared::ServiceHealth {
            status,
            message: if all_healthy {
                None
            } else {
                Some("Some components are unhealthy".to_string())
            },
            response_time_ms: Some(response_time),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_product_filters_cacheable() {
        let cacheable = ProductFilters {
            status: Some(ProductStatus::Active),
            category: Some(ProductCategory::Electronics),
            ..Default::default()
        };
        assert!(cacheable.is_cacheable());

        let not_cacheable = ProductFilters {
            name_contains: Some("test".to_string()),
            ..Default::default()
        };
        assert!(!not_cacheable.is_cacheable());
    }
}