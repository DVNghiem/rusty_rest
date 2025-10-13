use redis::{aio::ConnectionManager, AsyncCommands, Client};
use shared::{AppResult, AppError, cache_key, retry_async, RetryPolicy};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{instrument, warn};

/// Redis cache client wrapper with enterprise patterns
#[derive(Clone)]
pub struct RedisCache {
    connection: ConnectionManager,
    default_ttl: Duration,
    retry_policy: RetryPolicy,
}

impl RedisCache {
    /// Create a new Redis cache client
    pub async fn new(redis_url: &str, default_ttl: Duration) -> AppResult<Self> {
        let client = Client::open(redis_url)
            .map_err(|e| AppError::Cache(format!("Failed to create Redis client: {}", e)))?;
        
        let connection = ConnectionManager::new(client)
            .await
            .map_err(|e| AppError::Cache(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            connection,
            default_ttl,
            retry_policy: RetryPolicy::default(),
        })
    }

    /// Set a value with default TTL
    #[instrument(skip(self, value))]
    pub async fn set<T>(&self, key: &str, value: &T) -> AppResult<()>
    where
        T: Serialize,
    {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Set a value with custom TTL
    #[instrument(skip(self, value))]
    pub async fn set_with_ttl<T>(&self, key: &str, value: &T, ttl: Duration) -> AppResult<()>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_string(value)
            .map_err(|e| AppError::Cache(format!("Failed to serialize value: {}", e)))?;

        retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.set_ex::<_, _, ()>(key, &serialized, ttl.as_secs())
                    .await
                    .map_err(|e| AppError::Cache(format!("Redis SET failed: {}", e)))
            },
            &self.retry_policy,
        )
        .await?;

        Ok(())
    }

    /// Get a value from cache
    #[instrument(skip(self))]
    pub async fn get<T>(&self, key: &str) -> AppResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let result: Option<String> = retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.get(key).await
            },
            &self.retry_policy,
        )
        .await
        .map_err(|e| AppError::Cache(format!("Redis GET failed: {}", e)))?;

        match result {
            Some(serialized) => {
                let value = serde_json::from_str(&serialized)
                    .map_err(|e| AppError::Cache(format!("Failed to deserialize value: {}", e)))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Delete a key from cache
    #[instrument(skip(self))]
    pub async fn delete(&self, key: &str) -> AppResult<bool> {
        let deleted_count: i32 = retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.del(key).await
            },
            &self.retry_policy,
        )
        .await
        .map_err(|e| AppError::Cache(format!("Redis DEL failed: {}", e)))?;

        Ok(deleted_count > 0)
    }

    /// Check if key exists
    #[instrument(skip(self))]
    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        let result: bool = retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.exists(key).await
            },
            &self.retry_policy,
        )
        .await
        .map_err(|e| AppError::Cache(format!("Redis EXISTS failed: {}", e)))?;

        Ok(result)
    }

    /// Set TTL for existing key
    #[instrument(skip(self))]
    pub async fn expire(&self, key: &str, ttl: Duration) -> AppResult<bool> {
        let result: bool = retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.expire(key, ttl.as_secs() as i64).await
            },
            &self.retry_policy,
        )
        .await
        .map_err(|e| AppError::Cache(format!("Redis EXPIRE failed: {}", e)))?;

        Ok(result)
    }

    /// Get multiple keys at once
    #[instrument(skip(self))]
    pub async fn mget<T>(&self, keys: &[String]) -> AppResult<Vec<Option<T>>>
    where
        T: for<'de> Deserialize<'de>,
    {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        let values: Vec<Option<String>> = retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.get(keys).await
            },
            &self.retry_policy,
        )
        .await
        .map_err(|e| AppError::Cache(format!("Redis MGET failed: {}", e)))?;

        let mut deserialized = Vec::new();
        for value_opt in values {
            match value_opt {
                Some(serialized) => {
                    let value = serde_json::from_str(&serialized)
                        .map_err(|e| AppError::Cache(format!("Failed to deserialize value: {}", e)))?;
                    deserialized.push(Some(value));
                }
                None => deserialized.push(None),
            }
        }
        Ok(deserialized)
    }

    /// Set multiple key-value pairs
    #[instrument(skip(self, pairs))]
    pub async fn mset<T>(&self, pairs: &[(String, T)]) -> AppResult<()>
    where
        T: Serialize,
    {
        if pairs.is_empty() {
            return Ok(());
        }

        let mut serialized_pairs = Vec::new();
        for (key, value) in pairs {
            let serialized = serde_json::to_string(value)
                .map_err(|e| AppError::Cache(format!("Failed to serialize value: {}", e)))?;
            serialized_pairs.push((key.clone(), serialized));
        }

        let _: () = retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.mset(&serialized_pairs).await
            },
            &self.retry_policy,
        )
        .await
        .map_err(|e| AppError::Cache(format!("Redis MSET failed: {}", e)))?;

        Ok(())
    }

    /// Delete multiple keys
    #[instrument(skip(self))]
    pub async fn delete_many(&self, keys: &[String]) -> AppResult<u64> {
        if keys.is_empty() {
            return Ok(0);
        }

        let deleted_count: i32 = retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.del(keys).await
            },
            &self.retry_policy,
        )
        .await
        .map_err(|e| AppError::Cache(format!("Redis DEL failed: {}", e)))?;

        Ok(deleted_count as u64)
    }

    /// Increment counter
    #[instrument(skip(self))]
    pub async fn incr(&self, key: &str, delta: i64) -> AppResult<i64> {
        let result: i64 = retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.incr(key, delta).await
            },
            &self.retry_policy,
        )
        .await
        .map_err(|e| AppError::Cache(format!("Redis INCR failed: {}", e)))?;

        Ok(result)
    }

    /// Health check
    #[instrument(skip(self))]
    pub async fn ping(&self) -> AppResult<()> {
        // Use a simple command to test connectivity instead of ping
        let _: Option<String> = retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.get("__ping_test__").await
            },
            &self.retry_policy,
        )
        .await
        .map_err(|e| AppError::Cache(format!("Redis connectivity test failed: {}", e)))?;

        Ok(())
    }

    /// Clear all keys matching pattern (use with caution)
    #[instrument(skip(self))]
    pub async fn clear_pattern(&self, pattern: &str) -> AppResult<u64> {
        warn!(pattern, "Clearing cache keys matching pattern");

        // Get all keys matching pattern
        let keys_vec: Vec<String> = retry_async(
            || async {
                let mut conn = self.connection.clone();
                conn.keys(pattern).await
            },
            &self.retry_policy,
        )
        .await
        .map_err(|e| AppError::Cache(format!("Redis KEYS failed: {}", e)))?;

        if !keys_vec.is_empty() {
            self.delete_many(&keys_vec).await
        } else {
            Ok(0)
        }
    }
}

/// High-level cache operations for common patterns
impl RedisCache {
    /// Cache-aside pattern: get from cache, fallback to function
    #[instrument(skip(self, fallback))]
    pub async fn get_or_set<T, F, Fut>(&self, key: &str, fallback: F) -> AppResult<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = AppResult<T>>,
    {
        // Try to get from cache first
        if let Some(cached_value) = self.get(key).await? {
            return Ok(cached_value);
        }

        // Cache miss - call fallback function
        let value = fallback().await?;
        
        // Store in cache for next time (fire and forget)
        if let Err(e) = self.set(key, &value).await {
            warn!(error = %e, key, "Failed to cache value");
        }

        Ok(value)
    }

    /// Cache-aside with custom TTL
    #[instrument(skip(self, fallback))]
    pub async fn get_or_set_with_ttl<T, F, Fut>(
        &self,
        key: &str,
        ttl: Duration,
        fallback: F,
    ) -> AppResult<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Clone,
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = AppResult<T>>,
    {
        if let Some(cached_value) = self.get(key).await? {
            return Ok(cached_value);
        }

        let value = fallback().await?;
        
        if let Err(e) = self.set_with_ttl(key, &value, ttl).await {
            warn!(error = %e, key, "Failed to cache value");
        }

        Ok(value)
    }

    /// Invalidate cache by pattern (for cache warming scenarios)
    #[instrument(skip(self))]
    pub async fn invalidate_pattern(&self, pattern: &str) -> AppResult<u64> {
        self.clear_pattern(pattern).await
    }
}

/// Product-specific cache operations
impl RedisCache {
    /// Cache key generators for products
    pub fn product_key(id: &domain::ProductId) -> String {
        cache_key("product", &id.to_string())
    }

    pub fn products_by_category_key(category: &domain::ProductCategory, page: u32) -> String {
        cache_key("products_category", &format!("{}:{}", category_to_string(category), page))
    }

    pub fn products_by_status_key(status: &domain::ProductStatus, page: u32) -> String {
        cache_key("products_status", &format!("{}:{}", status_to_string(status), page))
    }

    pub fn product_statistics_key() -> String {
        cache_key("product", "statistics")
    }

    pub fn inventory_summary_key() -> String {
        cache_key("inventory", "summary")
    }

    /// Cache a product
    pub async fn cache_product(&self, product: &domain::ProductEntity) -> AppResult<()> {
        let key = Self::product_key(&product.id);
        self.set_with_ttl(&key, product, Duration::from_secs(3600)).await // 1 hour TTL
    }

    /// Get cached product
    pub async fn get_product(&self, id: &domain::ProductId) -> AppResult<Option<domain::ProductEntity>> {
        let key = Self::product_key(id);
        self.get(&key).await
    }

    /// Invalidate product cache
    pub async fn invalidate_product(&self, id: &domain::ProductId) -> AppResult<bool> {
        let key = Self::product_key(id);
        self.delete(&key).await
    }

    /// Invalidate all product-related caches
    pub async fn invalidate_all_products(&self) -> AppResult<u64> {
        self.clear_pattern("product:*").await
    }
}

// Helper functions for consistency with database layer
fn category_to_string(category: &domain::ProductCategory) -> String {
    match category {
        domain::ProductCategory::Electronics => "electronics".to_string(),
        domain::ProductCategory::Clothing => "clothing".to_string(),
        domain::ProductCategory::Home => "home".to_string(),
        domain::ProductCategory::Books => "books".to_string(),
        domain::ProductCategory::Sports => "sports".to_string(),
        domain::ProductCategory::Other => "other".to_string(),
    }
}

fn status_to_string(status: &domain::ProductStatus) -> String {
    match status {
        domain::ProductStatus::Active => "active".to_string(),
        domain::ProductStatus::Inactive => "inactive".to_string(),
        domain::ProductStatus::Discontinued => "discontinued".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn test_cache_key_generation() {
        let id = domain::ProductId::new();
        let key = RedisCache::product_key(&id);
        assert!(key.starts_with("product:"));
        assert!(key.contains(&id.to_string()));
    }

    #[test]
    fn test_category_string_conversion() {
        assert_eq!(category_to_string(&domain::ProductCategory::Electronics), "electronics");
        assert_eq!(category_to_string(&domain::ProductCategory::Clothing), "clothing");
    }

    #[test]
    fn test_status_string_conversion() {
        assert_eq!(status_to_string(&domain::ProductStatus::Active), "active");
        assert_eq!(status_to_string(&domain::ProductStatus::Inactive), "inactive");
    }
}