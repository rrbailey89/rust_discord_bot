#!/bin/bash
# backup.sh

# Exit on error
set -e

# Set variables
BACKUP_DIR="./db_backups"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_FILE="$BACKUP_DIR/discord_bot_db_$TIMESTAMP.sql"

# Load environment variables from .env
if [ -f .env ]; then
    # Export variables from .env file, ignore comments and empty lines
    export $(grep -v '^#' .env | xargs)
else
    echo "Error: .env file not found."
    exit 1
fi

# Create backup directory if it doesn't exist
mkdir -p $BACKUP_DIR
echo "Backup directory: $BACKUP_DIR"

echo "Starting database backup..."

# Check for docker-compose command
if command -v docker-compose &> /dev/null; then
    DOCKER_COMPOSE="docker-compose"
elif command -v docker &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
else
    echo "Error: Neither docker-compose nor docker command found. Please install Docker and Docker Compose."
    exit 1
fi

echo "Using ${DOCKER_COMPOSE} command"

# Backup database from the PostgreSQL container in the saltbox network
# Note: We're accessing the existing PostgreSQL container instead of our own
docker exec postgres pg_dump -U $POSTGRES_USER $POSTGRES_DB > $BACKUP_FILE
echo "Database backup created: $BACKUP_FILE"

# Compress backup
echo "Compressing backup..."
gzip $BACKUP_FILE
echo "Compressed backup: ${BACKUP_FILE}.gz"

# Remove backups older than 30 days
echo "Removing backups older than 30 days..."
find $BACKUP_DIR -name "discord_bot_db_*.sql.gz" -type f -mtime +30 -delete

echo "Backup completed successfully"
echo "Backup size: $(du -h ${BACKUP_FILE}.gz | cut -f1)"
