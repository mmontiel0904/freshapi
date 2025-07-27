#!/bin/bash

# Railway Deployment Script
# Runs database migrations before starting the application

set -e

echo "🔄 Starting deployment..."

# Run database migrations - use cargo run since it works
echo "📊 Running database migrations..."
cd migration && cargo run --release
migration_status=$?
cd ..

# Check migration status
if [ $migration_status -eq 0 ]; then
    echo "✅ Database migrations completed successfully"
else
    echo "❌ Database migrations failed"
    exit 1
fi

# Start the application - use cargo run for reliability
echo "🚀 Starting FreshAPI server..."
exec cargo run --release --bin freshapi