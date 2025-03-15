# Docker Deployment Guide

This guide will help you deploy the Discord bot on a headless Ubuntu machine using Docker.

## Prerequisites

- Docker and Docker Compose installed on your Ubuntu server
- A Discord bot token
- API keys for the various services used by the bot

## Quick Start

1. **Transfer Files to Server**:
   ```bash
   # Clone the repository
   git clone https://your-repository-url.git
   cd discord-bot

   # Or use SCP from your local machine
   scp -r /path/to/local/project/* user@remote-server:~/discord-bot/
   ```

2. **Configure Environment**:
   ```bash
   # Copy the example environment file
   cp .env.docker .env

   # Edit the .env file with your actual credentials
   nano .env
   ```

3. **Deploy the Application**:
   ```bash
   # Make scripts executable
   chmod +x *.sh

   # Run the deployment script
   ./deploy.sh
   ```

## Deployment Files

- **Dockerfile**: Defines how to build the Discord bot image
- **docker-compose.yml**: Orchestrates the bot and database containers
- **.env.docker**: Template for environment variables
- **deploy.sh**: Script to deploy the application
- **backup.sh**: Script to backup the database
- **monitor.sh**: Script to monitor the application
- **restart.sh**: Script to restart the containers
- **discord-bot.service**: Systemd service file for auto-starting the application

## Maintenance Commands

### Deploying the Application

```bash
./deploy.sh
```

### Monitoring the Application

```bash
./monitor.sh
```

### Backing Up the Database

```bash
./backup.sh
```

### Restarting the Application

```bash
# Simple restart
./restart.sh

# Restart and pull latest changes
./restart.sh --pull
```

### Setting Up as a System Service

1. **Edit the service file**:
   ```bash
   # Update the WorkingDirectory path in the service file
   nano discord-bot.service
   ```

2. **Install the service**:
   ```bash
   sudo cp discord-bot.service /etc/systemd/system/
   sudo systemctl daemon-reload
   sudo systemctl enable discord-bot.service
   sudo systemctl start discord-bot.service
   ```

3. **Check service status**:
   ```bash
   sudo systemctl status discord-bot.service
   ```

## Troubleshooting

### Container Logs

```bash
# View bot container logs
docker logs discord_bot

# View database container logs
docker logs discord_bot_db

# View real-time logs
docker-compose logs -f
```

### Database Connection Issues

If the bot cannot connect to the database:

1. Verify the database container is running:
   ```bash
   docker ps | grep discord_bot_db
   ```

2. Check database logs for errors:
   ```bash
   docker logs discord_bot_db
   ```

3. Verify the DATABASE_URL in .env has the correct format:
   ```
   DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@db:5432/${POSTGRES_DB}
   ```

### Container Won't Start

If a container fails to start:

1. Check container status:
   ```bash
   docker ps -a | grep discord_bot
   ```

2. Check container logs:
   ```bash
   docker logs discord_bot
   ```

3. Try rebuilding the containers:
   ```bash
   docker-compose down
   docker-compose up -d --build
   ```

## Security Considerations

- The .env file contains sensitive information. Make sure it has restricted permissions:
  ```bash
  chmod 600 .env
  ```

- Database backups contain sensitive data. Secure the backup directory:
  ```bash
  chmod 700 db_backups
  ```

- Consider using Docker secrets for production deployments.
