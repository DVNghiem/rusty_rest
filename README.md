# Enterprise Rust Backend API

A production-ready, scalable backend API built with Axum, PostgreSQL, and Redis, following enterprise design patterns and clean architecture principles.

## 🏗️ Architecture Overview

This project implements a **Clean Architecture** (Hexagonal Architecture) pattern with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                        API Layer (Axum)                    │
│                    Routes | Middleware                     │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────────┐
│                     Core Layer                              │
│                  Business Logic                            │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────────┐
│                 Infrastructure Layer                        │
│           PostgreSQL | Redis | HTTP Client                 │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────────┐
│                    Domain Layer                             │
│                 Entities | Traits                          │
└─────────────────────┬───────────────────────────────────────┘
                      │
┌─────────────────────┴───────────────────────────────────────┐
│                    Shared Layer                             │
│            Common Types | Utilities | Errors               │
└─────────────────────────────────────────────────────────────┘
```

### Crate Structure

- **`shared`**: Common utilities, error handling, and types
- **`domain`**: Core business entities and repository traits
- **`infrastructure`**: Database, cache, and external service implementations
- **`core`**: Business logic and use cases
- **`api`**: HTTP layer with Axum server, routes, and middleware

## 🚀 Key Features

### Enterprise Patterns

- **Clean Architecture** with dependency inversion
- **Repository Pattern** for data access abstraction
- **Domain-Driven Design** with rich domain models
- **CQRS** ready structure for command/query separation
- **Event Sourcing** support with domain events
- **Cache-Aside Pattern** for performance optimization
- **Circuit Breaker** for resilience
- **Retry Policies** with exponential backoff

### Technical Features

- **Type-Safe IDs** to prevent entity confusion
- **Comprehensive Error Handling** with structured responses
- **Request Tracing** and observability
- **Health Checks** for monitoring
- **Graceful Shutdown** handling
- **Configuration Management** with environment support
- **Database Migrations** with SQLx
- **Batch Operations** for performance
- **Pagination** support
- **Input Validation** and sanitization

### Performance & Scalability

- **Connection Pooling** for database and Redis
- **Caching Strategy** with TTL management
- **Async/Await** throughout the stack
- **Efficient JSON Serialization** with Serde
- **Compression** and middleware optimization
- **Rate Limiting** ready infrastructure

## 📋 Prerequisites

- **Rust** 1.70+ (latest stable recommended)
- **PostgreSQL** 14+ 
- **Redis** 6+
- **Docker** (optional, for local development)

## 🛠️ Quick Start

### 1. Clone the Repository

```bash
git clone https://github.com/your-org/rust-backend-api.git
cd rust-backend-api
```

### 2. Set Up Environment

Copy the example environment file and configure it:

```bash
cp .env.example .env
```

Edit `.env` with your configuration:

```env
# Server Configuration
APP_SERVER__HOST=0.0.0.0
APP_SERVER__PORT=8080

# Database Configuration
APP_DATABASE__URL=postgresql://postgres:password@localhost:5432/products
APP_DATABASE__MAX_CONNECTIONS=10
APP_DATABASE__MIN_CONNECTIONS=1

# Redis Configuration
APP_REDIS__URL=redis://localhost:6379
APP_REDIS__POOL_SIZE=10
APP_REDIS__DEFAULT_TTL_SECONDS=3600

# Logging
APP_LOGGING__LEVEL=info
APP_LOGGING__FORMAT=json

# Environment
APP_ENV=development
```

### 3. Start Infrastructure (Docker Compose)

```bash
# Create docker-compose.yml for local development
docker-compose up -d postgres redis
```

Example `docker-compose.yml`:

```yaml
version: '3.8'
services:
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: products
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

### 4. Build and Run

```bash
# Build the project
cargo build --release

# Run the API server
cargo run --bin api

# Or run in development mode with auto-reload
cargo watch -x "run --bin api"
```

The API will be available at `http://localhost:8080`

## 📚 API Documentation

### Health Check

```bash
# Check API health
curl http://localhost:8080/health
```

Response:
```json
{
  "status": "healthy",
  "services": {
    "product_service": {
      "status": "healthy",
      "response_time_ms": 5
    }
  },
  "timestamp": "2024-01-15T10:30:00Z"
}
```

### Products API

#### Create Product

```bash
curl -X POST http://localhost:8080/api/v1/products \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Laptop",
    "description": "High-performance laptop",
    "price": {
      "amount_dollars": 999.99,
      "currency": "USD"
    },
    "category": "electronics",
    "inventory": {
      "quantity": 50,
      "min_stock": 5,
      "max_stock": 200
    },
    "metadata": {
      "sku": "LAP001",
      "tags": ["computer", "portable"]
    }
  }'
```

#### Get Product

```bash
curl http://localhost:8080/api/v1/products/{product_id}
```

#### List Products with Filters

```bash
curl "http://localhost:8080/api/v1/products?category=electronics&page=1&limit=10&in_stock_only=true"
```

#### Update Product

```bash
curl -X PUT http://localhost:8080/api/v1/products/{product_id} \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Updated Laptop Name",
    "price": {
      "amount_dollars": 1199.99,
      "currency": "USD"
    }
  }'
```

#### Reserve Inventory

```bash
curl -X POST http://localhost:8080/api/v1/products/{product_id}/reserve \
  -H "Content-Type: application/json" \
  -d '{"quantity": 5}'
```

#### Get Statistics

```bash
curl http://localhost:8080/api/v1/products/statistics
```

## 🏗️ Development

### Project Structure

```
my_project/
├── Cargo.toml                 # Workspace configuration
├── README.md                  # This file
├── .env.example              # Environment template
├── docker-compose.yml        # Local development infrastructure
└── crates/
    ├── api/                  # HTTP API layer
    │   ├── src/
    │   │   ├── main.rs       # Server entry point
    │   │   ├── config.rs     # Configuration management
    │   │   ├── routes/       # HTTP route handlers
    │   │   └── middleware/   # HTTP middleware
    │   └── Cargo.toml
    ├── core/                 # Business logic layer
    │   ├── src/
    │   │   └── services/     # Business services
    │   └── Cargo.toml
    ├── infrastructure/       # External dependencies
    │   ├── src/
    │   │   ├── db/          # Database implementations
    │   │   ├── cache/       # Caching implementations
    │   │   └── http/        # HTTP client
    │   └── Cargo.toml
    ├── domain/              # Core business logic
    │   ├── src/
    │   │   ├── entities/    # Domain entities
    │   │   └── repositories/ # Repository traits
    │   └── Cargo.toml
    └── shared/              # Common utilities
        ├── src/
        │   ├── errors.rs    # Error handling
        │   ├── types.rs     # Common types
        │   └── utils.rs     # Utilities
        └── Cargo.toml
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with coverage
cargo install cargo-tarpaulin
cargo tarpaulin --verbose --all-features --workspace --timeout 120

# Run specific crate tests
cargo test -p shared
cargo test -p domain
cargo test -p infrastructure
cargo test -p core
cargo test -p api
```

### Database Migrations

The application automatically runs migrations on startup. For manual migration management:

```bash
# Install sqlx CLI
cargo install sqlx-cli

# Create new migration
sqlx migrate add create_products_table

# Run migrations manually
sqlx migrate run --database-url $DATABASE_URL

# Revert last migration
sqlx migrate revert --database-url $DATABASE_URL
```

### Adding New Features

1. **Define Domain Entity** (if needed) in `domain/src/entities/`
2. **Create Repository Trait** in `domain/src/repositories/`
3. **Implement Repository** in `infrastructure/src/db/`
4. **Add Business Logic** in `core/src/services/`
5. **Create HTTP Routes** in `api/src/routes/`
6. **Add Tests** for each layer

### Code Quality

```bash
# Format code
cargo fmt

# Check for common mistakes
cargo clippy -- -D warnings

# Check dependencies for security vulnerabilities
cargo audit

# Generate documentation
cargo doc --open
```

## 🚀 Deployment

### Docker Deployment

```dockerfile
# Dockerfile
FROM rust:1.75 as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/api /app/api

EXPOSE 8080
CMD ["./api"]
```

Build and run:

```bash
docker build -t rust-api .
docker run -p 8080:8080 rust-api
```

### Kubernetes Deployment

Example Kubernetes manifests:

```yaml
# deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rust-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: rust-api
  template:
    metadata:
      labels:
        app: rust-api
    spec:
      containers:
      - name: api
        image: rust-api:latest
        ports:
        - containerPort: 8080
        env:
        - name: APP_DATABASE__URL
          valueFrom:
            secretKeyRef:
              name: db-secret
              key: url
        - name: APP_REDIS__URL
          valueFrom:
            configMapKeyRef:
              name: app-config
              key: redis-url
```

### Production Configuration

For production deployment:

1. **Use environment-specific configs**
2. **Enable TLS/SSL termination**
3. **Configure proper logging levels**
4. **Set up health checks and monitoring**
5. **Configure resource limits**
6. **Enable graceful shutdown**

## 🔧 Configuration

The application supports multiple configuration methods:

### 1. Environment Variables

All configuration can be set via environment variables with the `APP_` prefix:

```env
APP_SERVER__PORT=8080
APP_DATABASE__MAX_CONNECTIONS=20
APP_REDIS__DEFAULT_TTL_SECONDS=7200
```

### 2. Configuration Files

Create environment-specific config files:

```toml
# config/production.toml
[server]
port = 8080
workers = 4

[database]
max_connections = 20
min_connections = 5
connection_timeout = 30

[redis]
pool_size = 20
default_ttl_seconds = 7200

[logging]
level = "info"
format = "json"
```

### 3. Configuration Priority

1. Environment variables (highest)
2. Local config file (`config/local.toml`)
3. Environment-specific config (`config/production.toml`)
4. Default config (`config/default.toml`)
5. Built-in defaults (lowest)

## 🔍 Monitoring & Observability

### Structured Logging

The application uses structured logging with tracing:

```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self))]
pub async fn create_product(&self, product: Product) -> Result<Product> {
    info!(product_id = %product.id, "Creating new product");
    // ... business logic
    Ok(product)
}
```

### Health Checks

Multiple health check endpoints:

- `/health` - Overall application health
- `/metrics` - Application metrics
- Service-specific health checks

### Distributed Tracing

Ready for integration with:
- **Jaeger** for distributed tracing
- **Prometheus** for metrics collection
- **Grafana** for visualization
- **ELK Stack** for log aggregation

## 🧪 Testing Strategy

### Unit Tests

```bash
# Run unit tests
cargo test --lib

# Test specific module
cargo test product_service::tests
```

### Integration Tests

```bash
# Run integration tests
cargo test --test integration

# Test with real database (requires test DB)
TEST_DATABASE_URL=postgres://test cargo test
```

### Load Testing

Use tools like `wrk` or `hey` for load testing:

```bash
# Install wrk
brew install wrk  # macOS
sudo apt-get install wrk  # Ubuntu

# Load test
wrk -t12 -c400 -d30s http://localhost:8080/api/v1/products
```

## 🤝 Contributing

1. **Fork the repository**
2. **Create feature branch** (`git checkout -b feature/amazing-feature`)
3. **Write tests** for your changes
4. **Ensure all tests pass** (`cargo test`)
5. **Run code formatting** (`cargo fmt`)
6. **Run clippy** (`cargo clippy`)
7. **Commit changes** (`git commit -m 'Add amazing feature'`)
8. **Push to branch** (`git push origin feature/amazing-feature`)
9. **Open Pull Request**

### Code Standards

- Follow Rust naming conventions
- Write comprehensive tests
- Document public APIs
- Use meaningful commit messages
- Keep functions small and focused
- Follow the established architecture patterns

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Troubleshooting

### Common Issues

**Database Connection Issues:**
```bash
# Check PostgreSQL is running
pg_isready -h localhost -p 5432

# Check connection string
psql "postgresql://postgres:password@localhost:5432/products"
```

**Redis Connection Issues:**
```bash
# Check Redis is running
redis-cli ping

# Check Redis connection
redis-cli -h localhost -p 6379 info
```

**Port Already in Use:**
```bash
# Find process using port 8080
lsof -i :8080

# Kill process if needed
kill -9 <PID>
```

**Build Issues:**
```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

### Performance Tuning

1. **Database Connection Pool**: Adjust based on your load
2. **Redis Pool Size**: Scale with concurrent requests  
3. **Worker Threads**: Match your server's CPU cores
4. **Cache TTL**: Balance freshness vs performance
5. **Batch Size**: Optimize for your data patterns

### Debugging

Enable debug logging:
```env
APP_LOGGING__LEVEL=debug
RUST_LOG=debug
```

Use cargo with debug info:
```bash
cargo run --bin api -- --log-level debug
```

## 📞 Support

- **Documentation**: [Internal Wiki Link]
- **Issues**: [GitHub Issues](https://github.com/your-org/rust-backend-api/issues)
- **Discord**: [Team Discord Channel]
- **Email**: team@yourcompany.com

---

**Built with ❤️ using Rust and enterprise-grade design patterns**