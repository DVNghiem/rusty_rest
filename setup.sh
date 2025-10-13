#!/bin/bash

# Development setup script for Rust Backend API
set -e

echo "🚀 Setting up Rust Backend API development environment..."

# Check if required tools are installed
check_tool() {
    if ! command -v $1 &> /dev/null; then
        echo "❌ $1 is not installed. Please install it first."
        exit 1
    else
        echo "✅ $1 is available"
    fi
}

echo "🔍 Checking required tools..."
check_tool "docker"
check_tool "docker-compose"
check_tool "cargo"
check_tool "psql"

# Copy environment file if it doesn't exist
if [ ! -f ".env" ]; then
    echo "📋 Creating .env file from template..."
    cp .env.example .env
    echo "⚠️  Please update .env with your configuration before running the application"
else
    echo "✅ .env file already exists"
fi

# Start Docker services
echo "🐳 Starting Docker services..."
docker-compose up -d postgres redis

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
until docker-compose exec postgres pg_isready -U postgres &> /dev/null; do
    sleep 1
done
echo "✅ PostgreSQL is ready"

# Wait for Redis to be ready
echo "⏳ Waiting for Redis to be ready..."
until docker-compose exec redis redis-cli ping &> /dev/null; do
    sleep 1
done
echo "✅ Redis is ready"

# Install SQLx CLI if not present
if ! command -v sqlx &> /dev/null; then
    echo "📦 Installing SQLx CLI..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Run database migrations
echo "🗄️  Running database migrations..."
cd crates/infrastructure
sqlx migrate run
cd ../..

# Build the project
echo "🔨 Building the project..."
cargo build

echo "✨ Setup complete! You can now:"
echo "  1. Run the API server: cargo run --bin api"
echo "  2. View PostgreSQL admin at http://localhost:5050 (admin@example.com / admin123)"
echo "  3. View Redis admin at http://localhost:8081"
echo "  4. API will be available at http://localhost:3000"
echo ""
echo "📚 Check the README.md for more information!"