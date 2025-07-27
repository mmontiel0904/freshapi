#!/bin/bash

# Railway Deployment Script
# Runs database migrations before starting the application

set -e

echo "🔄 Starting deployment..."

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
exec ./target/release/freshapi