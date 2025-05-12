 #!/bin/bash
set -e

echo "Starting database migration setup..."

# Check if DATABASE_URL exists
if [ -z "$DATABASE_URL" ]; then
    echo "ERROR: DATABASE_URL environment variable is not set"
    exit 1
fi

echo "Creating a temporary psql script to run migrations..."
cat > migration_runner.sql << EOF
\i migrations/01_migration_script.sql
EOF

echo "Running migration script directly with psql..."
psql "$DATABASE_URL" -f migration_runner.sql

# Check if migration succeeded
if [ $? -eq 0 ]; then
    echo ":white_check_mark: Migration completed successfully"
    # Clean up
    rm migration_runner.sql
else
    echo ":x: Migration failed"
    exit 1
fi

echo "Database setup complete!"