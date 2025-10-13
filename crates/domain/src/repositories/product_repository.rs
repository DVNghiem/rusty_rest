use async_trait::async_trait;
use shared::{AppResult, PaginationParams, PaginatedResponse};
use crate::entities::{ProductEntity, ProductId, ProductStatus, ProductCategory};

/// Repository trait for Product operations following Repository pattern
#[async_trait]
pub trait ProductRepository: Send + Sync {
    /// Find product by ID
    async fn find_by_id(&self, id: &ProductId) -> AppResult<Option<ProductEntity>>;
    
    /// Find products by multiple IDs
    async fn find_by_ids(&self, ids: &[ProductId]) -> AppResult<Vec<ProductEntity>>;
    
    /// Find all products with pagination
    async fn find_all(&self, pagination: PaginationParams) -> AppResult<PaginatedResponse<ProductEntity>>;
    
    /// Find products by status
    async fn find_by_status(
        &self, 
        status: ProductStatus, 
        pagination: PaginationParams
    ) -> AppResult<PaginatedResponse<ProductEntity>>;
    
    /// Find products by category
    async fn find_by_category(
        &self, 
        category: ProductCategory, 
        pagination: PaginationParams
    ) -> AppResult<PaginatedResponse<ProductEntity>>;
    
    /// Search products by name (partial match)
    async fn search_by_name(
        &self, 
        query: &str, 
        pagination: PaginationParams
    ) -> AppResult<PaginatedResponse<ProductEntity>>;
    
    /// Find products with low stock
    async fn find_low_stock(&self, pagination: PaginationParams) -> AppResult<PaginatedResponse<ProductEntity>>;
    
    /// Save a new product
    async fn save(&self, product: &ProductEntity) -> AppResult<()>;
    
    /// Update an existing product
    async fn update(&self, product: &ProductEntity) -> AppResult<()>;
    
    /// Delete a product by ID
    async fn delete(&self, id: &ProductId) -> AppResult<()>;
    
    /// Check if product exists by ID
    async fn exists(&self, id: &ProductId) -> AppResult<bool>;
    
    /// Count total products
    async fn count_all(&self) -> AppResult<u64>;
    
    /// Count products by status
    async fn count_by_status(&self, status: ProductStatus) -> AppResult<u64>;
    
    /// Count products by category
    async fn count_by_category(&self, category: ProductCategory) -> AppResult<u64>;
    
    /// Batch operations for better performance
    async fn save_batch(&self, products: &[ProductEntity]) -> AppResult<()>;
    async fn update_batch(&self, products: &[ProductEntity]) -> AppResult<()>;
    async fn delete_batch(&self, ids: &[ProductId]) -> AppResult<()>;
}

/// Advanced query filters for complex product searches
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProductFilters {
    pub status: Option<ProductStatus>,
    pub category: Option<ProductCategory>,
    pub min_price: Option<i64>,
    pub max_price: Option<i64>,
    pub in_stock_only: Option<bool>,
    pub low_stock_only: Option<bool>,
    pub name_contains: Option<String>,
    pub tags: Option<Vec<String>>,
    pub created_after: Option<shared::Timestamp>,
    pub created_before: Option<shared::Timestamp>,
}

/// Sorting options for product queries
#[derive(Debug, Clone)]
pub enum ProductSortBy {
    Name,
    Price,
    CreatedAt,
    UpdatedAt,
    Quantity,
    Status,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone)]
pub struct ProductSort {
    pub field: ProductSortBy,
    pub order: SortOrder,
}

impl Default for ProductSort {
    fn default() -> Self {
        Self {
            field: ProductSortBy::CreatedAt,
            order: SortOrder::Desc,
        }
    }
}

/// Extended repository trait for advanced queries
#[async_trait]
pub trait ProductRepositoryExt: ProductRepository {
    /// Advanced search with filters and sorting
    async fn find_with_filters(
        &self,
        filters: ProductFilters,
        sort: Option<ProductSort>,
        pagination: PaginationParams,
    ) -> AppResult<PaginatedResponse<ProductEntity>>;
    
    /// Get product statistics
    async fn get_statistics(&self) -> AppResult<ProductStatistics>;
    
    /// Get inventory summary
    async fn get_inventory_summary(&self) -> AppResult<InventorySummary>;
}

/// Product statistics for dashboard/reporting
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProductStatistics {
    pub total_products: u64,
    pub active_products: u64,
    pub inactive_products: u64,
    pub discontinued_products: u64,
    pub low_stock_products: u64,
    pub out_of_stock_products: u64,
    pub total_inventory_value: i64,
    pub average_price: f64,
}

/// Inventory summary for monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InventorySummary {
    pub total_items: i64,
    pub total_reserved: i64,
    pub total_available: i64,
    pub products_needing_restock: u64,
    pub categories_breakdown: std::collections::HashMap<ProductCategory, CategoryInventory>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CategoryInventory {
    pub product_count: u64,
    pub total_quantity: i64,
    pub total_value: i64,
}

impl ProductFilters {
    /// Determine if this filter combination is cacheable
    pub fn is_cacheable(&self) -> bool {
        // Simple filters are cacheable, complex ones are not
        self.name_contains.is_none() && self.tags.as_ref().map_or(true, |tags| tags.is_empty())
    }
    
    /// Calculate cache key for this filter combination
    pub fn cache_key(&self) -> String {
        // Create a deterministic key based on the filter values
        let mut key_parts = Vec::new();
        
        if let Some(status) = &self.status {
            key_parts.push(format!("status:{:?}", status));
        }
        if let Some(category) = &self.category {
            key_parts.push(format!("category:{:?}", category));
        }
        if let Some(min_price) = self.min_price {
            key_parts.push(format!("min_price:{}", min_price));
        }
        if let Some(max_price) = self.max_price {
            key_parts.push(format!("max_price:{}", max_price));
        }
        if let Some(in_stock) = self.in_stock_only {
            key_parts.push(format!("in_stock:{}", in_stock));
        }
        if let Some(low_stock) = self.low_stock_only {
            key_parts.push(format!("low_stock:{}", low_stock));
        }
        if let Some(after) = &self.created_after {
            key_parts.push(format!("after:{}", after.0.timestamp()));
        }
        if let Some(before) = &self.created_before {
            key_parts.push(format!("before:{}", before.0.timestamp()));
        }
        
        format!("product_filters:{}", key_parts.join("|"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_product_filters_default() {
        let filters = ProductFilters::default();
        assert!(filters.status.is_none());
        assert!(filters.category.is_none());
        assert!(filters.min_price.is_none());
    }
    
    #[test]
    fn test_product_sort_default() {
        let sort = ProductSort::default();
        assert!(matches!(sort.field, ProductSortBy::CreatedAt));
        assert!(matches!(sort.order, SortOrder::Desc));
    }
}