#!/bin/bash

# Railway Deployment Script
# Runs database migrations before starting the application

set -e

echo "🔄 Starting deployment..."

# Debug: List what binaries were built
echo "📋 Checking built binaries..."
echo "Contents of ./target/release/:"
ls -la ./target/release/ || echo "Main target directory not found"
echo "Contents of ./migration/target/release/:"
ls -la ./migration/target/release/ || echo "Migration target directory not found"

# Run database migrations using pre-built binary
echo "📊 Running database migrations..."
./migration/target/release/migration
migration_status=$?

# Check migration status
if [ $migration_status -eq 0 ]; then
    echo "✅ Database migrations completed successfully"
else
    echo "❌ Database migrations failed"
    exit 1
fi

# Start the application using pre-built binary
echo "🚀 Starting FreshAPI server..."
echo "Current working directory: $(pwd)"
echo "Looking for binary at: ./target/release/freshapi"

if [ -f "./target/release/freshapi" ]; then
    exec ./target/release/freshapi
else
    echo "❌ Binary not found, falling back to cargo run"
    exec cargo run --release --bin freshapi
fi