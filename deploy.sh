#!/bin/bash

# Deploy script for Discord Bot with Web Frontend
# This script sets up the environment and deploys the application using Docker Compose

set -e

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${GREEN}Discord Bot with Web Frontend Deployment${NC}"
echo "========================================"

# Check if .env file exists
if [ ! -f .env ]; then
    echo -e "${YELLOW}No .env file found. Creating from .env.example...${NC}"
    if [ -f .env.example ]; then
        cp .env.example .env
        echo -e "${GREEN}.env file created. Please edit it with your actual values.${NC}"
        echo -e "${YELLOW}Edit the file now? (y/n)${NC}"
        read -r edit_env
        if [[ "$edit_env" =~ ^[Yy]$ ]]; then
            ${EDITOR:-nano} .env
        else
            echo -e "${RED}Please edit the .env file before continuing.${NC}"
            exit 1
        fi
    else
        echo -e "${RED}No .env.example file found. Cannot continue.${NC}"
        exit 1
    fi
fi

# Source the .env file to get environment variables
echo -e "${GREEN}Loading environment variables...${NC}"
set -a
source .env
set +a

# Validate required environment variables
echo -e "${GREEN}Validating environment variables...${NC}"
required_vars=(
    "BOT_TOKEN"
    "APPLICATION_ID"
    "DATABASE_URL"
    "TRAEFIK_DOMAIN"
)

missing_vars=()
for var in "${required_vars[@]}"; do
    if [ -z "${!var}" ]; then
        missing_vars+=("$var")
    fi
done

if [ ${#missing_vars[@]} -ne 0 ]; then
    echo -e "${RED}The following required environment variables are missing:${NC}"
    for var in "${missing_vars[@]}"; do
        echo "- $var"
    done
    echo -e "${RED}Please add them to your .env file and run this script again.${NC}"
    exit 1
fi

# Update OAuth redirect URI if needed
if [[ "$OAUTH_REDIRECT_URI" != *"$TRAEFIK_DOMAIN"* ]]; then
    echo -e "${YELLOW}OAuth redirect URI doesn't match the Traefik domain.${NC}"
    echo -e "${YELLOW}Updating OAUTH_REDIRECT_URI to use https://$TRAEFIK_DOMAIN/api/auth/callback${NC}"
    sed -i "s|OAUTH_REDIRECT_URI=.*|OAUTH_REDIRECT_URI=https://$TRAEFIK_DOMAIN/api/auth/callback|" .env
    # Re-source the updated .env file
    set -a
    source .env
    set +a
fi

# Create dynamic labels file for additional Traefik configuration
echo -e "${GREEN}Creating dynamic Traefik labels from environment variables...${NC}"
LABELS_FILE="traefik-dynamic-labels.env"

# Start with empty file
> "$LABELS_FILE"

# Add API key labels if they exist
api_keys=(
    "OPENAI_API_KEY"
    "OPENWEATHER_API_KEY"
    "API_NINJAS_KEY"
    "BFL_API_KEY"
)

for key in "${api_keys[@]}"; do
    if [ -n "${!key}" ]; then
        echo "TRAEFIK_LABEL_API_${key}=${!key}" >> "$LABELS_FILE"
    fi
done

# Add cache configuration
cache_vars=($(env | grep "^CACHE_" | cut -d= -f1))
for var in "${cache_vars[@]}"; do
    echo "TRAEFIK_LABEL_CACHE_${var}=${!var}" >> "$LABELS_FILE"
done

# Add logging configuration
log_vars=($(env | grep "^LOG_" | cut -d= -f1))
for var in "${log_vars[@]}"; do
    echo "TRAEFIK_LABEL_LOG_${var}=${!var}" >> "$LABELS_FILE"
done

echo -e "${GREEN}Deploying application with Docker Compose...${NC}"
# Use both .env and the dynamic labels file
docker compose --env-file .env --env-file "$LABELS_FILE" down
docker compose --env-file .env --env-file "$LABELS_FILE" up -d --build

echo -e "${GREEN}Checking container status...${NC}"
docker compose ps

echo -e "${GREEN}Deployment complete!${NC}"
echo "Your Discord bot and web frontend should now be accessible at:"
echo -e "${YELLOW}https://$TRAEFIK_DOMAIN${NC}"
echo ""
echo "To view logs:"
echo -e "${YELLOW}docker compose logs -f${NC}"
echo ""
echo "To stop the application:"
echo -e "${YELLOW}docker compose down${NC}"
