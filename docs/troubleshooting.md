# Troubleshooting Guide

This guide covers common issues and solutions when working with the Rust Backend API.

## Common Issues

### 1. Database Connection Issues

#### Symptom
```
Error: Failed to connect to database
```

#### Solutions
1. **Check PostgreSQL is running:**
```bash
docker-compose ps postgres
# Should show postgres container as "Up"
```

2. **Verify DATABASE_URL:**
```bash
# Check .env file
cat .env | grep DATABASE_URL

# Test connection manually
psql "postgresql://postgres:password@localhost:5432/products"
```

3. **Reset database:**
```bash
docker-compose down postgres
docker-compose up -d postgres
# Wait a moment, then run migrations
cd crates/infrastructure && sqlx migrate run
```

### 2. Redis Connection Issues

#### Symptom
```
Error: Redis connection failed
```

#### Solutions
1. **Check Redis is running:**
```bash
docker-compose ps redis
redis-cli -h localhost -p 6379 ping
```

2. **Verify REDIS_URL:**
```bash
cat .env | grep REDIS_URL
```

### 3. Build Errors

#### SQLx Compile-Time Verification Fails

```
error: error occurred while checking query
```

**Solution:**
```bash
# Set DATABASE_URL for compile-time checks
export DATABASE_URL="postgresql://postgres:password@localhost:5432/products"

# Or create .sqlx directory for offline mode
cargo sqlx prepare
```

#### Missing System Dependencies

```
error: linking with `cc` failed
```

**Solution (Ubuntu/Debian):**
```bash
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev libpq-dev
```

**Solution (macOS):**
```bash
brew install postgresql openssl
export LDFLAGS="-L$(brew --prefix openssl)/lib"
export CPPFLAGS="-I$(brew --prefix openssl)/include"
```

### 4. Performance Issues

#### High Memory Usage

**Check connection pools:**
```bash
# View current connections
SELECT count(*) FROM pg_stat_activity;
```

**Solutions:**
- Reduce `DATABASE_MAX_CONNECTIONS` in .env
- Reduce `REDIS_MAX_CONNECTIONS` in .env
- Check for connection leaks in logs

#### Slow Queries

**Enable query logging:**
```env
RUST_LOG=debug,sqlx::query=debug
```

**Analyze queries:**
```sql
-- In PostgreSQL
EXPLAIN ANALYZE SELECT * FROM products WHERE category = 'electronics';
```

### 5. Docker Issues

#### Container Won't Start

```
Error: port is already allocated
```

**Solution:**
```bash
# Check what's using the port
lsof -i :3000

# Stop conflicting processes or change port
SERVER_PORT=3001 cargo run --bin api
```

#### Out of Disk Space

```
Error: no space left on device
```

**Solution:**
```bash
# Clean up Docker
docker system prune -a
docker volume prune
```

### 6. Migration Issues

#### Migration Out of Order

```
error: migration 002 was applied before migration 001
```

**Solution:**
```bash
# Reset migrations (CAUTION: This drops data)
sqlx migrate revert
sqlx migrate run
```

#### Failed Migration

```
error: migration failed to apply
```

**Solution:**
```bash
# Check migration status
sqlx migrate info

# Force mark as applied (if you know it's correct)
sqlx migrate resolve <migration_number>
```

## Environment-Specific Issues

### Development

1. **Hot Reload Not Working:**
   - Use `cargo watch -x run` instead of `cargo run`
   - Install: `cargo install cargo-watch`

2. **Test Database Conflicts:**
   - Use separate test database
   - Set `TEST_DATABASE_URL` in .env

### Production

1. **Health Check Failures:**
   - Verify all dependencies are accessible
   - Check `/health/detailed` for component status
   - Ensure proper resource limits

2. **Memory Leaks:**
   - Monitor with `htop` or similar
   - Check connection pool sizes
   - Review async task spawning

## Logging and Debugging

### Enable Debug Logging
```bash
RUST_LOG=debug cargo run --bin api
```

### Structured Logging Query
```bash
# Filter by request ID
cat logs/app.log | jq 'select(.request_id == "uuid-here")'

# Filter by level
cat logs/app.log | jq 'select(.level == "ERROR")'
```

### Performance Profiling
```bash
# Install flamegraph
cargo install flamegraph

# Profile the application
cargo flamegraph --bin api
```

## Getting Help

1. **Check the logs first:**
   ```bash
   docker-compose logs api
   ```

2. **Verify configuration:**
   ```bash
   ./setup.sh --verify
   ```

3. **Run health checks:**
   ```bash
   curl http://localhost:3000/health/detailed
   ```

4. **Create minimal reproduction:**
   - Isolate the problem
   - Document steps to reproduce
   - Include relevant logs and configuration

## Preventive Measures

1. **Regular Maintenance:**
   ```bash
   # Update dependencies
   cargo update
   
   # Check for security issues
   cargo audit
   
   # Clean build artifacts
   cargo clean
   ```

2. **Monitoring Setup:**
   - Set up health check alerts
   - Monitor resource usage
   - Track error rates and response times

3. **Backup Procedures:**
   - Regular database backups
   - Configuration file version control
   - Document recovery procedures