#!/bin/bash
set -e

echo "========================================"
echo "  CourseFlow Backend Setup"
echo "========================================"

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    echo "ERROR: PostgreSQL is not installed."
    echo "Install it with: sudo apt install postgresql postgresql-contrib"
    exit 1
fi

# Check if PostgreSQL service is running
if ! pg_isready &> /dev/null; then
    echo "Starting PostgreSQL service..."
    sudo systemctl start postgresql
fi

# Database name
DB_NAME="course_flow_db"
DB_USER="postgres"

# Create database (as postgres user)
echo "Creating database '$DB_NAME'..."
sudo -u postgres createdb "$DB_NAME" 2>/dev/null || echo "Database '$DB_NAME' already exists"

# Ensure postgres user has a password set
echo "Setting postgres user password to 'postgres'..."
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'postgres';" 2>/dev/null

# Run migrations in order via sqlx (cargo run will auto-migrate)
# But we can also run them manually:
echo "Running migrations..."
for migration in migrations/*.sql; do
    echo "  -> Running $migration"
    sudo -u postgres psql -d "$DB_NAME" -f "$migration" 2>/dev/null || true
done

echo ""
echo "========================================"
echo "  Setup complete!"
echo "========================================"
echo ""
echo "Your .env should contain:"
echo "  DATABASE_URL=postgres://postgres:postgres@localhost:5432/$DB_NAME"
echo ""
echo "Run the server with:"
echo "  cargo run"
echo ""
echo "Then open: http://127.0.0.1:3000/docs  (Swagger UI)"
echo "Health check: http://127.0.0.1:3000/health"
