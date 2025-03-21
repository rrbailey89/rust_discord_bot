# Environment Variables Configuration

This document outlines the environment variables used by the Discord bot and how to configure them in your Saltbox inventory file.

## Core Environment Variables

| Variable | Description | Required | Default |
|----------|-------------|----------|---------|
| BOT_TOKEN | Discord bot token | Yes | None |
| COMMAND_PREFIX | Command prefix for text commands | No | ! |
| SERENA_USER_ID | Discord user ID for Serena | Yes | 803867382447079485 |

## Saltbox Inventory Configuration

Add the following entry to your Saltbox inventory file to set the Serena user ID:

```yaml
# Discord Bot User IDs
discordbot_serena_user_id: "803867382447079485"  # Replace with Serena's actual Discord user ID
```

Example:
```yaml
# Discord Bot Configuration
discordbot_bot_token: "YOUR_BOT_TOKEN"
discordbot_application_id: "YOUR_APPLICATION_ID"
discordbot_client_id: "YOUR_CLIENT_ID"
discordbot_client_secret: "YOUR_CLIENT_SECRET"
discordbot_jwt_secret: "YOUR_JWT_SECRET"
discordbot_serena_user_id: "803867382447079485"  # Add this line
```

## How to Find a Discord User ID

1. Enable Developer Mode in Discord (User Settings > Advanced > Developer Mode)
2. Right-click on the user and select "Copy ID"

## Command Settings

The bot now automatically initializes command settings for new guilds. This ensures that command toggles in the web interface work correctly and reflect the actual state of commands in the guild.

When the bot joins a new guild, it will:
1. Create default entries in the `guild_command_settings` table for all available commands
2. Set all commands to enabled by default
3. Allow these settings to be modified through the web interface

## Frontend Improvements

The web interface now correctly displays command names and descriptions, even if they're missing or undefined. Command toggles also persist their state to the database, ensuring that command settings are properly saved.
