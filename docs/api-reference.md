# API Reference

Complete API documentation for the Rust Backend API.

## Base URL

```
http://localhost:3000
```

## Authentication

Currently, the API does not require authentication. This can be extended with JWT tokens, OAuth2, or other authentication mechanisms.

## Error Handling

All errors follow a consistent format:

```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {
      "field": "Additional error details"
    }
  },
  "meta": {
    "timestamp": "2024-01-01T00:00:00Z",
    "request_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

### Error Codes

| Code | Description | HTTP Status |
|------|-------------|-------------|
| `VALIDATION_ERROR` | Input validation failed | 400 |
| `NOT_FOUND` | Resource not found | 404 |
| `CONFLICT` | Resource already exists | 409 |
| `INTERNAL_ERROR` | Internal server error | 500 |
| `DATABASE_ERROR` | Database operation failed | 500 |
| `CACHE_ERROR` | Cache operation failed | 500 |
| `EXTERNAL_SERVICE_ERROR` | External service call failed | 502 |

## Health Endpoints

### Basic Health Check

**GET** `/health`

Returns basic health status.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "ok",
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

### Detailed Health Check

**GET** `/health/detailed`

Returns detailed health status of all components.

**Response:**
```json
{
  "success": true,
  "data": {
    "status": "ok",
    "components": {
      "database": {
        "status": "healthy",
        "connection_pool": {
          "active": 5,
          "idle": 15,
          "max": 20
        }
      },
      "cache": {
        "status": "healthy",
        "redis_info": {
          "connected_clients": 1,
          "used_memory": "1.2M"
        }
      },
      "external_services": {
        "status": "healthy",
        "services": {
          "example_api": "reachable"
        }
      }
    },
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

## Products API

### Create Product

**POST** `/api/v1/products`

Creates a new product.

**Request Body:**
```json
{
  "name": "Product Name",
  "description": "Product description",
  "price": 29.99,
  "stock_quantity": 100,
  "category": "electronics",
  "sku": "PROD-001"
}
```

**Validation Rules:**
- `name`: Required, 1-255 characters
- `description`: Optional, max 1000 characters
- `price`: Required, >= 0, max 2 decimal places
- `stock_quantity`: Required, >= 0
- `category`: Optional, max 100 characters
- `sku`: Optional, max 100 characters, must be unique

**Response (201 Created):**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Product Name",
    "description": "Product description",
    "price": 29.99,
    "stock_quantity": 100,
    "category": "electronics",
    "sku": "PROD-001",
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  },
  "meta": {
    "timestamp": "2024-01-01T00:00:00Z",
    "request_id": "550e8400-e29b-41d4-a716-446655440001"
  }
}
```

### Get Product

**GET** `/api/v1/products/{id}`

Retrieves a single product by ID.

**Parameters:**
- `id` (path): Product UUID

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Product Name",
    "description": "Product description",
    "price": 29.99,
    "stock_quantity": 100,
    "category": "electronics",
    "sku": "PROD-001",
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

### List Products

**GET** `/api/v1/products`

Retrieves a paginated list of products.

**Query Parameters:**
- `page` (optional): Page number, default 1
- `limit` (optional): Items per page, default 20, max 100
- `category` (optional): Filter by category
- `is_active` (optional): Filter by active status (true/false)
- `min_price` (optional): Filter by minimum price
- `max_price` (optional): Filter by maximum price
- `sort_by` (optional): Sort field (name, price, created_at), default created_at
- `sort_order` (optional): Sort order (asc, desc), default desc

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "products": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Product Name",
        "description": "Product description",
        "price": 29.99,
        "stock_quantity": 100,
        "category": "electronics",
        "sku": "PROD-001",
        "is_active": true,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
      }
    ],
    "pagination": {
      "current_page": 1,
      "total_pages": 5,
      "total_count": 100,
      "page_size": 20,
      "has_next": true,
      "has_previous": false
    }
  }
}
```

### Update Product

**PUT** `/api/v1/products/{id}`

Updates an existing product.

**Parameters:**
- `id` (path): Product UUID

**Request Body (partial update supported):**
```json
{
  "name": "Updated Product Name",
  "price": 39.99,
  "stock_quantity": 150
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "name": "Updated Product Name",
    "description": "Product description",
    "price": 39.99,
    "stock_quantity": 150,
    "category": "electronics",
    "sku": "PROD-001",
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:05:00Z"
  }
}
```

### Delete Product

**DELETE** `/api/v1/products/{id}`

Soft deletes a product (sets is_active to false).

**Parameters:**
- `id` (path): Product UUID

**Response (204 No Content):**
```json
{
  "success": true,
  "data": null
}
```

### Search Products

**GET** `/api/v1/products/search`

Performs full-text search on products.

**Query Parameters:**
- `q` (required): Search query
- `page` (optional): Page number, default 1
- `limit` (optional): Items per page, default 20, max 100

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "products": [
      {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "Product Name",
        "description": "Product description",
        "price": 29.99,
        "stock_quantity": 100,
        "category": "electronics",
        "sku": "PROD-001",
        "is_active": true,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z",
        "relevance_score": 0.95
      }
    ],
    "pagination": {
      "current_page": 1,
      "total_pages": 2,
      "total_count": 35,
      "page_size": 20,
      "has_next": true,
      "has_previous": false
    },
    "search_metadata": {
      "query": "electronics smartphone",
      "execution_time_ms": 15
    }
  }
}
```

### Product Statistics

**GET** `/api/v1/products/stats`

Returns product statistics.

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "total_products": 1250,
    "active_products": 1180,
    "total_value": 125000.50,
    "categories": {
      "electronics": 450,
      "clothing": 320,
      "books": 280,
      "home": 130,
      "sports": 70
    },
    "low_stock_count": 25,
    "out_of_stock_count": 8,
    "average_price": 42.35
  }
}
```

### Update Product Stock

**PATCH** `/api/v1/products/{id}/stock`

Updates product stock quantity with inventory tracking.

**Parameters:**
- `id` (path): Product UUID

**Request Body:**
```json
{
  "quantity": 50,
  "operation": "add", // "add", "subtract", or "set"
  "reason": "Restock from supplier"
}
```

**Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "previous_stock": 100,
    "new_stock": 150,
    "operation": "add",
    "quantity_changed": 50,
    "timestamp": "2024-01-01T00:00:00Z"
  }
}
```

## Rate Limiting

API endpoints are rate limited to prevent abuse:

- **Products endpoints**: 100 requests per minute per IP
- **Search endpoints**: 50 requests per minute per IP
- **Health endpoints**: 1000 requests per minute per IP

Rate limit headers are included in responses:

```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640995200
```

## Caching

Responses are cached using Redis with the following TTL:

- **Product details**: 5 minutes
- **Product lists**: 2 minutes
- **Search results**: 1 minute
- **Statistics**: 10 minutes

Cache headers indicate cache status:

```
X-Cache-Status: HIT
X-Cache-TTL: 180
```

## Webhooks (Future Enhancement)

The API is designed to support webhooks for real-time notifications:

**Supported Events:**
- `product.created`
- `product.updated`
- `product.deleted`
- `product.stock.low`
- `product.stock.out`

## SDK Examples

### cURL Examples

**Create Product:**
```bash
curl -X POST http://localhost:3000/api/v1/products \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Smartphone",
    "description": "Latest model smartphone",
    "price": 699.99,
    "stock_quantity": 50,
    "category": "electronics",
    "sku": "PHONE-001"
  }'
```

**Search Products:**
```bash
curl -X GET "http://localhost:3000/api/v1/products/search?q=smartphone&limit=5"
```

### JavaScript/Fetch Example

```javascript
// Create product
const response = await fetch('http://localhost:3000/api/v1/products', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json',
  },
  body: JSON.stringify({
    name: 'Laptop',
    description: 'High-performance laptop',
    price: 1299.99,
    stock_quantity: 25,
    category: 'electronics',
    sku: 'LAPTOP-001'
  })
});

const result = await response.json();
console.log(result);
```

### Python/Requests Example

```python
import requests

# List products with filtering
response = requests.get(
    'http://localhost:3000/api/v1/products',
    params={
        'category': 'electronics',
        'min_price': 100,
        'max_price': 1000,
        'page': 1,
        'limit': 10
    }
)

products = response.json()
print(products)
```