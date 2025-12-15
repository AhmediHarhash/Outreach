#!/bin/bash
# Run database migrations against Neon PostgreSQL
# Usage: ./scripts/run_migrations.sh

set -e

# Load environment variables
if [ -f .env ]; then
    export $(cat .env | grep -v '^#' | xargs)
fi

if [ -z "$DATABASE_URL" ]; then
    echo "ERROR: DATABASE_URL not set"
    exit 1
fi

echo "Running migrations against database..."
echo "Database: ${DATABASE_URL%%@*}@***"

# Run the migration
psql "$DATABASE_URL" -f src/db/migrations/001_init.sql

echo "Migrations completed successfully!"
