use serde::{Deserialize, Serialize};
use shared::{Id, Timestamp};

/// Product entity marker type for type-safe IDs
#[derive(Debug, Clone)]
pub struct Product;

/// Product ID type alias
pub type ProductId = Id<Product>;

/// Product status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductStatus {
    Active,
    Inactive,
    Discontinued,
}

impl Default for ProductStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Product category enum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductCategory {
    Electronics,
    Clothing,
    Home,
    Books,
    Sports,
    Other,
}

/// Main Product entity following DDD principles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductEntity {
    pub id: ProductId,
    pub name: String,
    pub description: Option<String>,
    pub price: ProductPrice,
    pub category: ProductCategory,
    pub status: ProductStatus,
    pub inventory: ProductInventory,
    pub metadata: ProductMetadata,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

/// Product price value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductPrice {
    pub amount: i64,  // Price in cents to avoid floating point issues
    pub currency: String,  // ISO 4217 currency code
}

impl ProductPrice {
    pub fn new(amount_dollars: f64, currency: impl Into<String>) -> Self {
        Self {
            amount: (amount_dollars * 100.0) as i64,
            currency: currency.into(),
        }
    }

    pub fn amount_as_float(&self) -> f64 {
        self.amount as f64 / 100.0
    }

    pub fn is_valid(&self) -> bool {
        self.amount >= 0 && !self.currency.is_empty()
    }
}

/// Product inventory value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductInventory {
    pub quantity: i32,
    pub reserved: i32,
    pub min_stock: i32,
    pub max_stock: Option<i32>,
}

impl ProductInventory {
    pub fn new(quantity: i32, min_stock: i32, max_stock: Option<i32>) -> Self {
        Self {
            quantity,
            reserved: 0,
            min_stock,
            max_stock,
        }
    }

    pub fn available(&self) -> i32 {
        self.quantity - self.reserved
    }

    pub fn is_low_stock(&self) -> bool {
        self.available() <= self.min_stock
    }

    pub fn can_reserve(&self, amount: i32) -> bool {
        amount > 0 && self.available() >= amount
    }

    pub fn reserve(&mut self, amount: i32) -> Result<(), String> {
        if !self.can_reserve(amount) {
            return Err(format!("Cannot reserve {} items. Available: {}", amount, self.available()));
        }
        self.reserved += amount;
        Ok(())
    }

    pub fn release(&mut self, amount: i32) -> Result<(), String> {
        if amount <= 0 || self.reserved < amount {
            return Err(format!("Cannot release {} items. Reserved: {}", amount, self.reserved));
        }
        self.reserved -= amount;
        Ok(())
    }
}

/// Product metadata for additional attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMetadata {
    pub sku: Option<String>,
    pub barcode: Option<String>,
    pub weight: Option<ProductWeight>,
    pub dimensions: Option<ProductDimensions>,
    pub tags: Vec<String>,
    pub attributes: std::collections::HashMap<String, String>,
}

impl Default for ProductMetadata {
    fn default() -> Self {
        Self {
            sku: None,
            barcode: None,
            weight: None,
            dimensions: None,
            tags: Vec::new(),
            attributes: std::collections::HashMap::new(),
        }
    }
}

/// Product weight value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductWeight {
    pub value: f64,
    pub unit: WeightUnit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeightUnit {
    Grams,
    Kilograms,
    Pounds,
    Ounces,
}

/// Product dimensions value object
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProductDimensions {
    pub length: f64,
    pub width: f64,
    pub height: f64,
    pub unit: DimensionUnit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DimensionUnit {
    Centimeters,
    Inches,
    Meters,
}

/// Product domain events for event sourcing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProductEvent {
    Created {
        product: ProductEntity,
        timestamp: Timestamp,
    },
    Updated {
        product_id: ProductId,
        changes: ProductChanges,
        timestamp: Timestamp,
    },
    StatusChanged {
        product_id: ProductId,
        old_status: ProductStatus,
        new_status: ProductStatus,
        timestamp: Timestamp,
    },
    InventoryAdjusted {
        product_id: ProductId,
        old_quantity: i32,
        new_quantity: i32,
        reason: String,
        timestamp: Timestamp,
    },
    PriceChanged {
        product_id: ProductId,
        old_price: ProductPrice,
        new_price: ProductPrice,
        timestamp: Timestamp,
    },
}

/// Changes made to a product for tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductChanges {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub price: Option<ProductPrice>,
    pub category: Option<ProductCategory>,
    pub inventory: Option<ProductInventory>,
    pub metadata: Option<ProductMetadata>,
}

impl ProductEntity {
    /// Create a new product
    pub fn new(
        name: String,
        description: Option<String>,
        price: ProductPrice,
        category: ProductCategory,
        inventory: ProductInventory,
        metadata: Option<ProductMetadata>,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: ProductId::new(),
            name,
            description,
            price,
            category,
            status: ProductStatus::default(),
            inventory,
            metadata: metadata.unwrap_or_default(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// Update product fields
    pub fn update(&mut self, changes: ProductChanges) {
        if let Some(name) = changes.name {
            self.name = name;
        }
        if let Some(description) = changes.description {
            self.description = description;
        }
        if let Some(price) = changes.price {
            self.price = price;
        }
        if let Some(category) = changes.category {
            self.category = category;
        }
        if let Some(inventory) = changes.inventory {
            self.inventory = inventory;
        }
        if let Some(metadata) = changes.metadata {
            self.metadata = metadata;
        }
        self.updated_at = Timestamp::now();
    }

    /// Change product status
    pub fn change_status(&mut self, new_status: ProductStatus) {
        if self.status != new_status {
            self.status = new_status;
            self.updated_at = Timestamp::now();
        }
    }

    /// Validate product data
    pub fn is_valid(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.name.trim().is_empty() {
            errors.push("Product name cannot be empty".to_string());
        }

        if !self.price.is_valid() {
            errors.push("Product price is invalid".to_string());
        }

        if self.inventory.quantity < 0 {
            errors.push("Product quantity cannot be negative".to_string());
        }

        if self.inventory.reserved < 0 {
            errors.push("Reserved quantity cannot be negative".to_string());
        }

        if self.inventory.min_stock < 0 {
            errors.push("Minimum stock cannot be negative".to_string());
        }

        if let Some(max_stock) = self.inventory.max_stock {
            if max_stock < self.inventory.min_stock {
                errors.push("Maximum stock cannot be less than minimum stock".to_string());
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check if product is available for purchase
    pub fn is_available(&self) -> bool {
        self.status == ProductStatus::Active && self.inventory.available() > 0
    }

    /// Reserve inventory for an order
    pub fn reserve_inventory(&mut self, amount: i32) -> Result<(), String> {
        self.inventory.reserve(amount)?;
        self.updated_at = Timestamp::now();
        Ok(())
    }

    /// Release reserved inventory
    pub fn release_inventory(&mut self, amount: i32) -> Result<(), String> {
        self.inventory.release(amount)?;
        self.updated_at = Timestamp::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_creation() {
        let price = ProductPrice::new(29.99, "USD");
        let inventory = ProductInventory::new(100, 10, Some(500));
        
        let product = ProductEntity::new(
            "Test Product".to_string(),
            Some("A test product".to_string()),
            price,
            ProductCategory::Electronics,
            inventory,
            None,
        );

        assert!(!product.id.value.to_string().is_empty());
        assert_eq!(product.name, "Test Product");
        assert_eq!(product.category, ProductCategory::Electronics);
        assert!(product.is_valid().is_ok());
    }

    #[test]
    fn test_price_calculations() {
        let price = ProductPrice::new(29.99, "USD");
        assert_eq!(price.amount, 2999);
        assert_eq!(price.amount_as_float(), 29.99);
        assert!(price.is_valid());
    }

    #[test]
    fn test_inventory_operations() {
        let mut inventory = ProductInventory::new(100, 10, Some(500));
        
        assert_eq!(inventory.available(), 100);
        assert!(!inventory.is_low_stock());
        
        assert!(inventory.reserve(20).is_ok());
        assert_eq!(inventory.available(), 80);
        assert_eq!(inventory.reserved, 20);
        
        assert!(inventory.release(10).is_ok());
        assert_eq!(inventory.available(), 90);
        assert_eq!(inventory.reserved, 10);
    }

    #[test]
    fn test_product_validation() {
        let price = ProductPrice::new(-10.0, "USD");
        let inventory = ProductInventory::new(-5, 10, Some(500));
        
        let product = ProductEntity::new(
            "".to_string(),
            None,
            price,
            ProductCategory::Electronics,
            inventory,
            None,
        );

        let validation_result = product.is_valid();
        assert!(validation_result.is_err());
        
        let errors = validation_result.unwrap_err();
        assert!(errors.len() > 0);
    }
}