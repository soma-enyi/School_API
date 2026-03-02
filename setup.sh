#!/bin/bash

echo "Setting up School Backend..."

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    echo "PostgreSQL is not installed. Please install it first."
    exit 1
fi

# Create database
echo "Creating database..."
createdb school_db 2>/dev/null || echo "Database already exists"

# Run migrations
echo "Running migrations..."
psql -d school_db -f migrations/001_init.sql

echo "Setup complete! You can now run: cargo run"
