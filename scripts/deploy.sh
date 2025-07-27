#!/bin/bash

# Railway Deployment Script
# Runs database migrations before starting the application

set -e

echo "🔄 Starting deployment..."

# Run database migrations
echo "📊 Running database migrations..."
cargo run --release --bin migration

# Check migration status
if [ $? -eq 0 ]; then
    echo "✅ Database migrations completed successfully"
else
    echo "❌ Database migrations failed"
    exit 1
fi

# Start the application
echo "🚀 Starting FreshAPI server..."
exec cargo run --release --bin freshapi