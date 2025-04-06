# Deployment Guide

This document provides instructions for deploying the Discord Bot with Web Frontend in various environments.

## Requirements

- Docker and Docker Compose
- PostgreSQL 13+
- Node.js 18+ (for frontend development only)
- Rust 1.70+ (for backend development only)
- Discord Application with Bot created at [Discord Developer Portal](https://discord.com/developers/applications)

## Environment Variables

Configure the following environment variables before deploying:

### Core Bot Configuration

```
# Bot token from Discord Developer Portal
DISCORD_TOKEN=your_discord_bot_token

# Discord Application ID
APPLICATION_ID=your_discord_application_id

# Discord CDN Image Hash Format
CDN_AVATAR_URL=https://cdn.discordapp.com/avatars

# Bot Mode (development or production)
BOT_MODE=production

# Default command prefix
DEFAULT_PREFIX=!
```

### Database Configuration

```
# PostgreSQL connection information
DATABASE_URL=postgres://username:password@hostname:5432/database

# Connection pool settings
DATABASE_MIN_CONNECTIONS=5
DATABASE_MAX_CONNECTIONS=20
DATABASE_ACQUIRE_TIMEOUT=30000
DATABASE_IDLE_TIMEOUT=10000
```

### Web Server Configuration

```
# JWT secret for authentication
JWT_SECRET=your_secret_key_at_least_32_characters

# Web server port
WEB_PORT=3000

# Discord OAuth settings
DISCORD_CLIENT_ID=your_discord_client_id
DISCORD_CLIENT_SECRET=your_discord_client_secret
DISCORD_REDIRECT_URI=https://your-domain.com/api/auth/callback

# CORS settings
CORS_ALLOWED_ORIGINS=https://your-domain.com
```

### Logging Configuration

```
# Log level (error, warn, info, debug, trace)
RUST_LOG=info

# Log file path (if not using stdout)
LOG_FILE_PATH=/app/logs/bot.log

# Whether to log to stdout
LOG_TO_STDOUT=true
```

## Docker Deployment

The recommended way to deploy is using Docker with Docker Compose.

### 1. Create a `.env` file

Create a `.env` file in the project root with all the required environment variables.

### 2. Deploy with Docker Compose

```bash
# Build and start the containers
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the service
docker-compose down
```

### Docker Compose Configuration

The `docker-compose.yml` file included in the project configures the following:

- Discord bot container with web frontend
- PostgreSQL database container
- Volume for persistent database storage
- Volume for logs
- Proper network configuration

## Manual Deployment

If you prefer to deploy without Docker, follow these steps:

### 1. Set up PostgreSQL

```bash
# Install PostgreSQL
sudo apt update && sudo apt install postgresql

# Create database and user
sudo -u postgres psql
postgres=# CREATE USER botuser WITH PASSWORD 'your_password';
postgres=# CREATE DATABASE botdb;
postgres=# GRANT ALL PRIVILEGES ON DATABASE botdb TO botuser;
postgres=# \q
```

### 2. Build the Frontend

```bash
# Navigate to frontend directory
cd frontend

# Install dependencies
npm install

# Build for production
npm run build
```

### 3. Build the Backend

```bash
# Build the rust application
cargo build --release
```

### 4. Set up Environment Variables

Set all the required environment variables in your system.

### 5. Run Database Migrations

```bash
# Copy migrations directory
cp -r migrations /path/to/deployment/

# Run the application once to execute migrations
./target/release/discord_bot
```

### 6. Configure as a System Service

Create a systemd service file `/etc/systemd/system/discord-bot.service`:

```ini
[Unit]
Description=Discord Bot with Web Frontend
After=network.target postgresql.service

[Service]
Type=simple
User=your_user
WorkingDirectory=/path/to/deployment
ExecStart=/path/to/deployment/discord_bot
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info
Environment=LOG_TO_STDOUT=true
# Add all other environment variables here

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl enable discord-bot
sudo systemctl start discord-bot
sudo systemctl status discord-bot
```

## Nginx Configuration for Web Frontend

When deploying to production, it's recommended to use Nginx as a reverse proxy.

Create a new Nginx configuration file `/etc/nginx/sites-available/discord-bot`:

```nginx
server {
    listen 80;
    server_name your-domain.com;

    # Redirect HTTP to HTTPS
    location / {
        return 301 https://$host$request_uri;
    }
}

server {
    listen 443 ssl;
    server_name your-domain.com;

    # SSL configuration
    ssl_certificate /path/to/fullchain.pem;
    ssl_certificate_key /path/to/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_prefer_server_ciphers on;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Content-Type-Options nosniff;
    add_header X-Frame-Options SAMEORIGIN;
    add_header X-XSS-Protection "1; mode=block";

    # Proxy to application
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;
    }
}
```

Enable the configuration:

```bash
sudo ln -s /etc/nginx/sites-available/discord-bot /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx
```

## SSL Configuration with Let's Encrypt

To secure your web frontend with HTTPS, use Let's Encrypt:

```bash
# Install Certbot
sudo apt update && sudo apt install certbot python3-certbot-nginx

# Get SSL certificate
sudo certbot --nginx -d your-domain.com

# Set up auto-renewal
sudo systemctl status certbot.timer
```

## Monitoring

For production deployments, consider setting up:

1. Prometheus for metrics collection
2. Grafana for dashboards and visualization
3. Alertmanager for notifications

## Scaling Considerations

For high-traffic deployments:

1. Use a separate PostgreSQL server with optimized configuration
2. Deploy multiple bot instances behind a load balancer
3. Implement proper caching strategies
4. Consider using Redis for rate-limiting and session storage

## Troubleshooting

Common issues and their solutions:

### Connection Refused to Database

- Check that PostgreSQL is running and accessible
- Verify that the DATABASE_URL environment variable is correct
- Check network configuration and firewall rules

### Bot Not Connecting to Discord

- Verify that the DISCORD_TOKEN is correct
- Check that the bot has proper intents enabled in the Discord Developer Portal
- Look for rate limiting messages in the logs

### Web Frontend Not Loading

- Check that the web server port is accessible
- Verify that the frontend was built correctly
- Check Nginx configuration (if using)
- Look for JavaScript errors in the browser console

### Discord OAuth Not Working

- Verify DISCORD_CLIENT_ID and DISCORD_CLIENT_SECRET
- Check that the redirect URI matches exactly what's configured in the Discord Developer Portal
- Make sure the app has the correct OAuth scopes
