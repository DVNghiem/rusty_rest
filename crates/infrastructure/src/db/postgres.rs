use async_trait::async_trait;
use domain::{
    ProductEntity, ProductId, ProductRepository, ProductRepositoryExt, 
    ProductStatus, ProductCategory, ProductFilters, ProductSort, ProductStatistics, 
    InventorySummary, CategoryInventory
};
use shared::{AppResult, AppError, PaginationParams, PaginatedResponse, Timestamp};
use sqlx::{PgPool, Row};
use std::collections::HashMap;
use uuid::Uuid;

/// PostgreSQL implementation of ProductRepository
#[derive(Clone)]
pub struct PostgresProductRepository {
    pool: PgPool,
}

impl PostgresProductRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Initialize database tables
    pub async fn migrate(&self) -> AppResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS products (
                id UUID PRIMARY KEY,
                name VARCHAR NOT NULL,
                description TEXT,
                price_amount BIGINT NOT NULL,
                price_currency VARCHAR(3) NOT NULL,
                category VARCHAR NOT NULL,
                status VARCHAR NOT NULL,
                quantity INTEGER NOT NULL DEFAULT 0,
                reserved INTEGER NOT NULL DEFAULT 0,
                min_stock INTEGER NOT NULL DEFAULT 0,
                max_stock INTEGER,
                sku VARCHAR,
                barcode VARCHAR,
                weight_value DOUBLE PRECISION,
                weight_unit VARCHAR,
                length DOUBLE PRECISION,
                width DOUBLE PRECISION,
                height DOUBLE PRECISION,
                dimension_unit VARCHAR,
                tags TEXT[],
                attributes JSONB DEFAULT '{}',
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );

            CREATE INDEX IF NOT EXISTS idx_products_status ON products(status);
            CREATE INDEX IF NOT EXISTS idx_products_category ON products(category);
            CREATE INDEX IF NOT EXISTS idx_products_name ON products USING gin(to_tsvector('english', name));
            CREATE INDEX IF NOT EXISTS idx_products_created_at ON products(created_at);
            CREATE INDEX IF NOT EXISTS idx_products_price ON products(price_amount);
            CREATE INDEX IF NOT EXISTS idx_products_quantity ON products(quantity);
            CREATE INDEX IF NOT EXISTS idx_products_tags ON products USING gin(tags);
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl ProductRepository for PostgresProductRepository {
    async fn find_by_id(&self, id: &ProductId) -> AppResult<Option<ProductEntity>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, price_amount, price_currency, category, status,
                   quantity, reserved, min_stock, max_stock, sku, barcode,
                   weight_value, weight_unit, length, width, height, dimension_unit,
                   tags, attributes, created_at, updated_at
            FROM products WHERE id = $1
            "#,
        )
        .bind(id.value)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(Some(row_to_product(row)?)),
            None => Ok(None),
        }
    }

    async fn find_by_ids(&self, ids: &[ProductId]) -> AppResult<Vec<ProductEntity>> {
        let uuids: Vec<Uuid> = ids.iter().map(|id| id.value).collect();
        
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, price_amount, price_currency, category, status,
                   quantity, reserved, min_stock, max_stock, sku, barcode,
                   weight_value, weight_unit, length, width, height, dimension_unit,
                   tags, attributes, created_at, updated_at
            FROM products WHERE id = ANY($1)
            "#,
        )
        .bind(&uuids)
        .fetch_all(&self.pool)
        .await?;

        let mut products = Vec::new();
        for row in rows {
            products.push(row_to_product(row)?);
        }

        Ok(products)
    }

    async fn find_all(&self, pagination: PaginationParams) -> AppResult<PaginatedResponse<ProductEntity>> {
        let total = self.count_all().await?;
        
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, price_amount, price_currency, category, status,
                   quantity, reserved, min_stock, max_stock, sku, barcode,
                   weight_value, weight_unit, length, width, height, dimension_unit,
                   tags, attributes, created_at, updated_at
            FROM products 
            ORDER BY created_at DESC 
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(pagination.limit as i64)
        .bind(pagination.offset() as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut products = Vec::new();
        for row in rows {
            products.push(row_to_product(row)?);
        }

        Ok(PaginatedResponse::new(products, pagination, total))
    }

    async fn find_by_status(
        &self,
        status: ProductStatus,
        pagination: PaginationParams,
    ) -> AppResult<PaginatedResponse<ProductEntity>> {
        let total = self.count_by_status(status.clone()).await?;
        let status_str = status_to_string(&status);
        
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, price_amount, price_currency, category, status,
                   quantity, reserved, min_stock, max_stock, sku, barcode,
                   weight_value, weight_unit, length, width, height, dimension_unit,
                   tags, attributes, created_at, updated_at
            FROM products 
            WHERE status = $1
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(status_str)
        .bind(pagination.limit as i64)
        .bind(pagination.offset() as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut products = Vec::new();
        for row in rows {
            products.push(row_to_product(row)?);
        }

        Ok(PaginatedResponse::new(products, pagination, total))
    }

    async fn find_by_category(
        &self,
        category: ProductCategory,
        pagination: PaginationParams,
    ) -> AppResult<PaginatedResponse<ProductEntity>> {
        let total = self.count_by_category(category.clone()).await?;
        let category_str = category_to_string(&category);
        
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, price_amount, price_currency, category, status,
                   quantity, reserved, min_stock, max_stock, sku, barcode,
                   weight_value, weight_unit, length, width, height, dimension_unit,
                   tags, attributes, created_at, updated_at
            FROM products 
            WHERE category = $1
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(category_str)
        .bind(pagination.limit as i64)
        .bind(pagination.offset() as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut products = Vec::new();
        for row in rows {
            products.push(row_to_product(row)?);
        }

        Ok(PaginatedResponse::new(products, pagination, total))
    }

    async fn search_by_name(
        &self,
        query: &str,
        pagination: PaginationParams,
    ) -> AppResult<PaginatedResponse<ProductEntity>> {
        let search_query = format!("%{}%", query);
        
        let total_row = sqlx::query("SELECT COUNT(*) as count FROM products WHERE name ILIKE $1")
            .bind(&search_query)
            .fetch_one(&self.pool)
            .await?;
        let total: i64 = total_row.get("count");
        
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, price_amount, price_currency, category, status,
                   quantity, reserved, min_stock, max_stock, sku, barcode,
                   weight_value, weight_unit, length, width, height, dimension_unit,
                   tags, attributes, created_at, updated_at
            FROM products 
            WHERE name ILIKE $1
            ORDER BY created_at DESC 
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(&search_query)
        .bind(pagination.limit as i64)
        .bind(pagination.offset() as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut products = Vec::new();
        for row in rows {
            products.push(row_to_product(row)?);
        }

        Ok(PaginatedResponse::new(products, pagination, total as u64))
    }

    async fn find_low_stock(&self, pagination: PaginationParams) -> AppResult<PaginatedResponse<ProductEntity>> {
        let total_row = sqlx::query("SELECT COUNT(*) as count FROM products WHERE quantity - reserved <= min_stock")
            .fetch_one(&self.pool)
            .await?;
        let total: i64 = total_row.get("count");
        
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, price_amount, price_currency, category, status,
                   quantity, reserved, min_stock, max_stock, sku, barcode,
                   weight_value, weight_unit, length, width, height, dimension_unit,
                   tags, attributes, created_at, updated_at
            FROM products 
            WHERE quantity - reserved <= min_stock
            ORDER BY (quantity - reserved) ASC 
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(pagination.limit as i64)
        .bind(pagination.offset() as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut products = Vec::new();
        for row in rows {
            products.push(row_to_product(row)?);
        }

        Ok(PaginatedResponse::new(products, pagination, total as u64))
    }

    async fn save(&self, product: &ProductEntity) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO products (
                id, name, description, price_amount, price_currency, category, status,
                quantity, reserved, min_stock, max_stock, sku, barcode,
                weight_value, weight_unit, length, width, height, dimension_unit,
                tags, attributes, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23
            )
            "#,
        )
        .bind(product.id.value)
        .bind(&product.name)
        .bind(&product.description)
        .bind(product.price.amount)
        .bind(&product.price.currency)
        .bind(category_to_string(&product.category))
        .bind(status_to_string(&product.status))
        .bind(product.inventory.quantity)
        .bind(product.inventory.reserved)
        .bind(product.inventory.min_stock)
        .bind(product.inventory.max_stock)
        .bind(&product.metadata.sku)
        .bind(&product.metadata.barcode)
        .bind(product.metadata.weight.as_ref().map(|w| w.value))
        .bind(product.metadata.weight.as_ref().map(|w| weight_unit_to_string(&w.unit)))
        .bind(product.metadata.dimensions.as_ref().map(|d| d.length))
        .bind(product.metadata.dimensions.as_ref().map(|d| d.width))
        .bind(product.metadata.dimensions.as_ref().map(|d| d.height))
        .bind(product.metadata.dimensions.as_ref().map(|d| dimension_unit_to_string(&d.unit)))
        .bind(&product.metadata.tags)
        .bind(serde_json::to_value(&product.metadata.attributes)?)
        .bind(product.created_at.as_datetime())
        .bind(product.updated_at.as_datetime())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, product: &ProductEntity) -> AppResult<()> {
        let rows_affected = sqlx::query(
            r#"
            UPDATE products SET
                name = $2, description = $3, price_amount = $4, price_currency = $5,
                category = $6, status = $7, quantity = $8, reserved = $9,
                min_stock = $10, max_stock = $11, sku = $12, barcode = $13,
                weight_value = $14, weight_unit = $15, length = $16, width = $17,
                height = $18, dimension_unit = $19, tags = $20, attributes = $21,
                updated_at = $22
            WHERE id = $1
            "#,
        )
        .bind(product.id.value)
        .bind(&product.name)
        .bind(&product.description)
        .bind(product.price.amount)
        .bind(&product.price.currency)
        .bind(category_to_string(&product.category))
        .bind(status_to_string(&product.status))
        .bind(product.inventory.quantity)
        .bind(product.inventory.reserved)
        .bind(product.inventory.min_stock)
        .bind(product.inventory.max_stock)
        .bind(&product.metadata.sku)
        .bind(&product.metadata.barcode)
        .bind(product.metadata.weight.as_ref().map(|w| w.value))
        .bind(product.metadata.weight.as_ref().map(|w| weight_unit_to_string(&w.unit)))
        .bind(product.metadata.dimensions.as_ref().map(|d| d.length))
        .bind(product.metadata.dimensions.as_ref().map(|d| d.width))
        .bind(product.metadata.dimensions.as_ref().map(|d| d.height))
        .bind(product.metadata.dimensions.as_ref().map(|d| dimension_unit_to_string(&d.unit)))
        .bind(&product.metadata.tags)
        .bind(serde_json::to_value(&product.metadata.attributes)?)
        .bind(product.updated_at.as_datetime())
        .execute(&self.pool)
        .await?;

        if rows_affected.rows_affected() == 0 {
            return Err(shared::not_found!("Product"));
        }

        Ok(())
    }

    async fn delete(&self, id: &ProductId) -> AppResult<()> {
        let rows_affected = sqlx::query("DELETE FROM products WHERE id = $1")
            .bind(id.value)
            .execute(&self.pool)
            .await?;

        if rows_affected.rows_affected() == 0 {
            return Err(shared::not_found!("Product"));
        }

        Ok(())
    }

    async fn exists(&self, id: &ProductId) -> AppResult<bool> {
        let row = sqlx::query("SELECT EXISTS(SELECT 1 FROM products WHERE id = $1)")
            .bind(id.value)
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get(0))
    }

    async fn count_all(&self) -> AppResult<u64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM products")
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count as u64)
    }

    async fn count_by_status(&self, status: ProductStatus) -> AppResult<u64> {
        let status_str = status_to_string(&status);
        let row = sqlx::query("SELECT COUNT(*) as count FROM products WHERE status = $1")
            .bind(status_str)
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count as u64)
    }

    async fn count_by_category(&self, category: ProductCategory) -> AppResult<u64> {
        let category_str = category_to_string(&category);
        let row = sqlx::query("SELECT COUNT(*) as count FROM products WHERE category = $1")
            .bind(category_str)
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count as u64)
    }

    async fn save_batch(&self, products: &[ProductEntity]) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        
        for product in products {
            sqlx::query(
                r#"
                INSERT INTO products (
                    id, name, description, price_amount, price_currency, category, status,
                    quantity, reserved, min_stock, max_stock, sku, barcode,
                    weight_value, weight_unit, length, width, height, dimension_unit,
                    tags, attributes, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23
                )
                "#,
            )
            .bind(product.id.value)
            .bind(&product.name)
            .bind(&product.description)
            .bind(product.price.amount)
            .bind(&product.price.currency)
            .bind(category_to_string(&product.category))
            .bind(status_to_string(&product.status))
            .bind(product.inventory.quantity)
            .bind(product.inventory.reserved)
            .bind(product.inventory.min_stock)
            .bind(product.inventory.max_stock)
            .bind(&product.metadata.sku)
            .bind(&product.metadata.barcode)
            .bind(product.metadata.weight.as_ref().map(|w| w.value))
            .bind(product.metadata.weight.as_ref().map(|w| weight_unit_to_string(&w.unit)))
            .bind(product.metadata.dimensions.as_ref().map(|d| d.length))
            .bind(product.metadata.dimensions.as_ref().map(|d| d.width))
            .bind(product.metadata.dimensions.as_ref().map(|d| d.height))
            .bind(product.metadata.dimensions.as_ref().map(|d| dimension_unit_to_string(&d.unit)))
            .bind(&product.metadata.tags)
            .bind(serde_json::to_value(&product.metadata.attributes)?)
            .bind(product.created_at.as_datetime())
            .bind(product.updated_at.as_datetime())
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }

    async fn update_batch(&self, products: &[ProductEntity]) -> AppResult<()> {
        let mut tx = self.pool.begin().await?;
        
        for product in products {
            sqlx::query(
                r#"
                UPDATE products SET
                    name = $2, description = $3, price_amount = $4, price_currency = $5,
                    category = $6, status = $7, quantity = $8, reserved = $9,
                    min_stock = $10, max_stock = $11, sku = $12, barcode = $13,
                    weight_value = $14, weight_unit = $15, length = $16, width = $17,
                    height = $18, dimension_unit = $19, tags = $20, attributes = $21,
                    updated_at = $22
                WHERE id = $1
                "#,
            )
            .bind(product.id.value)
            .bind(&product.name)
            .bind(&product.description)
            .bind(product.price.amount)
            .bind(&product.price.currency)
            .bind(category_to_string(&product.category))
            .bind(status_to_string(&product.status))
            .bind(product.inventory.quantity)
            .bind(product.inventory.reserved)
            .bind(product.inventory.min_stock)
            .bind(product.inventory.max_stock)
            .bind(&product.metadata.sku)
            .bind(&product.metadata.barcode)
            .bind(product.metadata.weight.as_ref().map(|w| w.value))
            .bind(product.metadata.weight.as_ref().map(|w| weight_unit_to_string(&w.unit)))
            .bind(product.metadata.dimensions.as_ref().map(|d| d.length))
            .bind(product.metadata.dimensions.as_ref().map(|d| d.width))
            .bind(product.metadata.dimensions.as_ref().map(|d| d.height))
            .bind(product.metadata.dimensions.as_ref().map(|d| dimension_unit_to_string(&d.unit)))
            .bind(&product.metadata.tags)
            .bind(serde_json::to_value(&product.metadata.attributes)?)
            .bind(product.updated_at.as_datetime())
            .execute(&mut *tx)
            .await?;
        }
        
        tx.commit().await?;
        Ok(())
    }

    async fn delete_batch(&self, ids: &[ProductId]) -> AppResult<()> {
        let uuids: Vec<Uuid> = ids.iter().map(|id| id.value).collect();
        
        sqlx::query("DELETE FROM products WHERE id = ANY($1)")
            .bind(&uuids)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}

#[async_trait]
impl ProductRepositoryExt for PostgresProductRepository {
    async fn find_with_filters(
        &self,
        filters: ProductFilters,
        _sort: Option<ProductSort>,
        pagination: PaginationParams,
    ) -> AppResult<PaginatedResponse<ProductEntity>> {
        // This is a simplified implementation - in production you'd build dynamic SQL
        let mut where_conditions = Vec::new();
        let _params: Vec<Box<dyn std::any::Any + Send + Sync>> = Vec::new();
        let mut param_count = 0;

        if let Some(_status) = filters.status {
            param_count += 1;
            where_conditions.push(format!("status = ${}", param_count));
            // Note: In a real implementation, you'd properly handle parameters
        }

        let where_clause = if where_conditions.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", where_conditions.join(" AND "))
        };

        // Simplified query - in production, build this dynamically
        let count_query = format!("SELECT COUNT(*) as count FROM products {}", where_clause);
        let total_row = sqlx::query(&count_query).fetch_one(&self.pool).await?;
        let total: i64 = total_row.get("count");

        let query = format!(
            r#"
            SELECT id, name, description, price_amount, price_currency, category, status,
                   quantity, reserved, min_stock, max_stock, sku, barcode,
                   weight_value, weight_unit, length, width, height, dimension_unit,
                   tags, attributes, created_at, updated_at
            FROM products 
            {} 
            ORDER BY created_at DESC 
            LIMIT {} OFFSET {}
            "#,
            where_clause, pagination.limit, pagination.offset()
        );

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        let mut products = Vec::new();
        for row in rows {
            products.push(row_to_product(row)?);
        }

        Ok(PaginatedResponse::new(products, pagination, total as u64))
    }

    async fn get_statistics(&self) -> AppResult<ProductStatistics> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(*) as total_products,
                COUNT(CASE WHEN status = 'active' THEN 1 END) as active_products,
                COUNT(CASE WHEN status = 'inactive' THEN 1 END) as inactive_products,
                COUNT(CASE WHEN status = 'discontinued' THEN 1 END) as discontinued_products,
                COUNT(CASE WHEN quantity - reserved <= min_stock THEN 1 END) as low_stock_products,
                COUNT(CASE WHEN quantity - reserved <= 0 THEN 1 END) as out_of_stock_products,
                COALESCE(SUM(price_amount * quantity), 0) as total_inventory_value,
                COALESCE(AVG(price_amount), 0) as average_price
            FROM products
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(ProductStatistics {
            total_products: row.get::<i64, _>("total_products") as u64,
            active_products: row.get::<i64, _>("active_products") as u64,
            inactive_products: row.get::<i64, _>("inactive_products") as u64,
            discontinued_products: row.get::<i64, _>("discontinued_products") as u64,
            low_stock_products: row.get::<i64, _>("low_stock_products") as u64,
            out_of_stock_products: row.get::<i64, _>("out_of_stock_products") as u64,
            total_inventory_value: row.get::<i64, _>("total_inventory_value"),
            average_price: row.get::<f64, _>("average_price") / 100.0, // Convert from cents
        })
    }

    async fn get_inventory_summary(&self) -> AppResult<InventorySummary> {
        let summary_row = sqlx::query(
            r#"
            SELECT 
                COALESCE(SUM(quantity), 0) as total_items,
                COALESCE(SUM(reserved), 0) as total_reserved,
                COALESCE(SUM(quantity - reserved), 0) as total_available,
                COUNT(CASE WHEN quantity - reserved <= min_stock THEN 1 END) as products_needing_restock
            FROM products
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let category_rows = sqlx::query(
            r#"
            SELECT 
                category,
                COUNT(*) as product_count,
                COALESCE(SUM(quantity), 0) as total_quantity,
                COALESCE(SUM(price_amount * quantity), 0) as total_value
            FROM products
            GROUP BY category
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut categories_breakdown = HashMap::new();
        for row in category_rows {
            let category_str: String = row.get("category");
            let category = string_to_category(&category_str)?;
            let inventory = CategoryInventory {
                product_count: row.get::<i64, _>("product_count") as u64,
                total_quantity: row.get::<i64, _>("total_quantity"),
                total_value: row.get::<i64, _>("total_value"),
            };
            categories_breakdown.insert(category, inventory);
        }

        Ok(InventorySummary {
            total_items: summary_row.get::<i64, _>("total_items"),
            total_reserved: summary_row.get::<i64, _>("total_reserved"),
            total_available: summary_row.get::<i64, _>("total_available"),
            products_needing_restock: summary_row.get::<i64, _>("products_needing_restock") as u64,
            categories_breakdown,
        })
    }
}

/// Helper functions for converting between domain types and database representations

fn row_to_product(row: sqlx::postgres::PgRow) -> AppResult<ProductEntity> {
    use domain::*;

    let price = ProductPrice {
        amount: row.get("price_amount"),
        currency: row.get("price_currency"),
    };

    let inventory = ProductInventory {
        quantity: row.get("quantity"),
        reserved: row.get("reserved"),
        min_stock: row.get("min_stock"),
        max_stock: row.get("max_stock"),
    };

    let weight = if let (Some(value), Some(unit_str)) = (
        row.get::<Option<f64>, _>("weight_value"),
        row.get::<Option<String>, _>("weight_unit"),
    ) {
        Some(ProductWeight {
            value,
            unit: string_to_weight_unit(&unit_str)?,
        })
    } else {
        None
    };

    let dimensions = if let (Some(length), Some(width), Some(height), Some(unit_str)) = (
        row.get::<Option<f64>, _>("length"),
        row.get::<Option<f64>, _>("width"),
        row.get::<Option<f64>, _>("height"),
        row.get::<Option<String>, _>("dimension_unit"),
    ) {
        Some(ProductDimensions {
            length,
            width,
            height,
            unit: string_to_dimension_unit(&unit_str)?,
        })
    } else {
        None
    };

    let attributes: serde_json::Value = row.get("attributes");
    let attributes_map: HashMap<String, String> = serde_json::from_value(attributes)?;

    let metadata = ProductMetadata {
        sku: row.get("sku"),
        barcode: row.get("barcode"),
        weight,
        dimensions,
        tags: row.get::<Vec<String>, _>("tags"),
        attributes: attributes_map,
    };

    Ok(ProductEntity {
        id: ProductId::from_uuid(row.get("id")),
        name: row.get("name"),
        description: row.get("description"),
        price,
        category: string_to_category(&row.get::<String, _>("category"))?,
        status: string_to_status(&row.get::<String, _>("status"))?,
        inventory,
        metadata,
        created_at: Timestamp::from(row.get::<chrono::DateTime<chrono::Utc>, _>("created_at")),
        updated_at: Timestamp::from(row.get::<chrono::DateTime<chrono::Utc>, _>("updated_at")),
    })
}

fn status_to_string(status: &ProductStatus) -> String {
    match status {
        ProductStatus::Active => "active".to_string(),
        ProductStatus::Inactive => "inactive".to_string(),
        ProductStatus::Discontinued => "discontinued".to_string(),
    }
}

fn string_to_status(s: &str) -> AppResult<ProductStatus> {
    match s {
        "active" => Ok(ProductStatus::Active),
        "inactive" => Ok(ProductStatus::Inactive),
        "discontinued" => Ok(ProductStatus::Discontinued),
        _ => Err(shared::validation_error!(&format!("Invalid status: {}", s))),
    }
}

fn category_to_string(category: &ProductCategory) -> String {
    match category {
        ProductCategory::Electronics => "electronics".to_string(),
        ProductCategory::Clothing => "clothing".to_string(),
        ProductCategory::Home => "home".to_string(),
        ProductCategory::Books => "books".to_string(),
        ProductCategory::Sports => "sports".to_string(),
        ProductCategory::Other => "other".to_string(),
    }
}

fn string_to_category(s: &str) -> AppResult<ProductCategory> {
    match s {
        "electronics" => Ok(ProductCategory::Electronics),
        "clothing" => Ok(ProductCategory::Clothing),
        "home" => Ok(ProductCategory::Home),
        "books" => Ok(ProductCategory::Books),
        "sports" => Ok(ProductCategory::Sports),
        "other" => Ok(ProductCategory::Other),
        _ => Err(shared::validation_error!(&format!("Invalid category: {}", s))),
    }
}

fn weight_unit_to_string(unit: &domain::WeightUnit) -> String {
    match unit {
        domain::WeightUnit::Grams => "grams".to_string(),
        domain::WeightUnit::Kilograms => "kilograms".to_string(),
        domain::WeightUnit::Pounds => "pounds".to_string(),
        domain::WeightUnit::Ounces => "ounces".to_string(),
    }
}

fn string_to_weight_unit(s: &str) -> AppResult<domain::WeightUnit> {
    match s {
        "grams" => Ok(domain::WeightUnit::Grams),
        "kilograms" => Ok(domain::WeightUnit::Kilograms),
        "pounds" => Ok(domain::WeightUnit::Pounds),
        "ounces" => Ok(domain::WeightUnit::Ounces),
        _ => Err(shared::validation_error!(&format!("Invalid weight unit: {}", s))),
    }
}

fn dimension_unit_to_string(unit: &domain::DimensionUnit) -> String {
    match unit {
        domain::DimensionUnit::Centimeters => "centimeters".to_string(),
        domain::DimensionUnit::Inches => "inches".to_string(),
        domain::DimensionUnit::Meters => "meters".to_string(),
    }
}

fn string_to_dimension_unit(s: &str) -> AppResult<domain::DimensionUnit> {
    match s {
        "centimeters" => Ok(domain::DimensionUnit::Centimeters),
        "inches" => Ok(domain::DimensionUnit::Inches),
        "meters" => Ok(domain::DimensionUnit::Meters),
        _ => Err(shared::validation_error!(&format!("Invalid dimension unit: {}", s))),
    }
}