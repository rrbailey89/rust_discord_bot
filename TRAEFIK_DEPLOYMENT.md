# Deploying with Traefik

This guide explains how to deploy your Discord bot with web frontend using Traefik as a reverse proxy on your Ubuntu server.

## Prerequisites

- Ubuntu server with Docker and Docker Compose installed
- Existing Traefik instance running on the server
- DNS record for `blameserena.rayplex.life` pointing to your server
- Cloudflare configured (if applicable)

## Deployment Steps

### 1. Configure Environment Variables

Create a `.env` file in the project root:

```bash
cp .env.example .env
nano .env
```

Fill in all the required environment variables, especially:

- `DISCORD_TOKEN`: Your Discord bot token
- `APPLICATION_ID`: Your Discord application ID
- `DATABASE_URL`: Connection string for your PostgreSQL database
- `DISCORD_CLIENT_ID` and `DISCORD_CLIENT_SECRET`: Get these from the Discord Developer Portal
- `TRAEFIK_DOMAIN`: The domain name for your bot (e.g., `blameserena.rayplex.life`)
- `OAUTH_REDIRECT_URI`: Set to `https://${TRAEFIK_DOMAIN}/api/auth/callback`
- `JWT_SECRET`: A secure random string (at least 32 characters)

### 2. Deploy Using the Deployment Script

The project includes a deployment script that automates the configuration and deployment process. The script:

- Creates/validates the `.env` file
- Ensures all required environment variables are set
- Updates the OAuth redirect URI to match your domain
- Creates dynamic Traefik labels from environment variables
- Deploys the application with Docker Compose

Run the deployment script:

```bash
./deploy.sh
```

Alternatively, you can deploy manually with Docker Compose:

```bash
docker-compose up -d
```

### 3. Verify Deployment

Check the container logs:

```bash
docker-compose logs -f
```

Ensure that:
- The bot connects to Discord successfully
- The web server starts on port 3000 (internal to the container)
- No errors appear in the logs

### 4. Access the Web Frontend

Visit `https://blameserena.rayplex.life` in your browser. You should see the login page.

## Troubleshooting

### Container fails to start

Check the logs for detailed error messages:

```bash
docker-compose logs -f
```

### Web frontend not accessible

1. Verify the container is running:
   ```bash
   docker-compose ps
   ```

2. Check Traefik logs for routing issues:
   ```bash
   docker logs traefik
   ```

3. Verify DNS is correctly configured for `blameserena.rayplex.life`

4. Make sure your Traefik instance is properly configured for TLS

### Database connection issues

1. Check that the database container is running
2. Verify the `DATABASE_URL` in your `.env` file is correct
3. Try connecting to the database manually to ensure credentials are working

## Maintenance

### Updating the Application

To update the application:

```bash
git pull
docker-compose down
docker-compose up -d --build
```

### Backing Up the Database

```bash
docker exec -t postgres_container_name pg_dump -U username database_name > backup_$(date +%Y-%m-%d).sql
```

### Viewing Logs

```bash
docker-compose logs -f
```

## Security Considerations

- The Discord OAuth flow requires HTTPS to work properly
- Ensure your `.env` file has restricted permissions (`chmod 600 .env`)
- Regularly update your application and dependencies
- Consider adding authentication middleware for additional security
