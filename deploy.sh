#!/bin/bash
# deploy.sh

# Exit on error
set -e

echo "Starting deployment process..."

# Check if .env file exists
if [ ! -f .env ]; then
    if [ -f .env.docker ]; then
        echo "No .env file found but .env.docker exists. Creating .env from .env.docker..."
        cp .env.docker .env
        echo "Please edit .env with your actual configuration values before proceeding."
        exit 1
    else
        echo "Error: No .env or .env.docker file found. Please create an .env file before deploying."
        exit 1
    fi
fi

# Create logs directory if it doesn't exist
mkdir -p logs
echo "Created logs directory"

# Build and start containers
echo "Building and starting containers..."
docker-compose up -d --build

# Check if containers are running
echo "Checking container status..."
if [ "$(docker ps -q -f name=discord_bot)" ] && [ "$(docker ps -q -f name=discord_bot_db)" ]; then
    echo "Deployment successful! Containers are running."
    echo "Use 'docker-compose logs -f' to view logs."
else
    echo "Error: Containers failed to start. Checking logs..."
    docker-compose logs
    exit 1
fi
