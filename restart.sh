#!/bin/bash
# restart.sh

echo "Restarting Discord bot containers..."

# Check if option to pull latest changes is provided
if [ "$1" == "--pull" ] || [ "$1" == "-p" ]; then
    echo "Pulling latest changes from repository..."
    git pull
    echo "Changes pulled successfully."
fi

# Restart containers
docker-compose restart
echo "Containers restart initiated"

# Wait for containers to be fully up
sleep 5

# Check container status
echo "Current container status:"
docker-compose ps

echo "Recent logs from bot container:"
docker logs --tail=10 discord_bot

echo "Restart completed at $(date)"
