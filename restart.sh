#!/bin/bash
# restart.sh

echo "Restarting Discord bot containers..."

# Check if option to pull latest changes is provided
if [ "$1" == "--pull" ] || [ "$1" == "-p" ]; then
    echo "Pulling latest changes from repository..."
    git pull
    echo "Changes pulled successfully."
fi

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

# Restart containers
${DOCKER_COMPOSE} restart
echo "Containers restart initiated"

# Wait for containers to be fully up
sleep 5

# Check container status
echo "Current container status:"
${DOCKER_COMPOSE} ps

echo "Recent logs from bot container:"
docker logs --tail=10 discord_bot 2>/dev/null || echo "Bot container not found or not running"

echo "Restart completed at $(date)"
