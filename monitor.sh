#!/bin/bash
# monitor.sh

# Print header
echo "======================================"
echo "Discord Bot Monitoring - $(date)"
echo "======================================"

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

# Check container status
echo -e "\n## Container Status ##"
docker ps -a | grep -E 'discord_bot|discord_bot_db'

# Check container logs (last 20 lines)
echo -e "\n## Bot Recent Logs ##"
docker logs --tail=20 discord_bot 2>&1 | grep -v '^$' || echo "Bot container not found or not running"

echo -e "\n## Database Recent Logs ##"
docker logs --tail=10 discord_bot_db 2>&1 | grep -v '^$' || echo "Database container not found or not running"

# Check disk space
echo -e "\n## Disk Space ##"
df -h | grep -E '(Filesystem|/$)'

# Check memory usage
echo -e "\n## Memory Usage ##"
free -h

# Check database size
echo -e "\n## Database Size ##"
if [ -f .env ]; then
    # Export variables from .env file, ignore comments and empty lines
    export $(grep -v '^#' .env | xargs)
    echo "Database size:"
    docker exec discord_bot_db psql -U $POSTGRES_USER -d $POSTGRES_DB -c "SELECT pg_size_pretty(pg_database_size('$POSTGRES_DB'));"
    
    echo -e "\nTable sizes:"
    docker exec discord_bot_db psql -U $POSTGRES_USER -d $POSTGRES_DB -c "SELECT relname as table_name, pg_size_pretty(pg_total_relation_size(relid)) as size FROM pg_catalog.pg_statio_user_tables ORDER BY pg_total_relation_size(relid) DESC LIMIT 10;"
fi

# Check docker stats
echo -e "\n## Container Resource Usage ##"
docker stats --no-stream discord_bot discord_bot_db

echo -e "\nMonitoring completed at $(date)"
