# Discord Bot Web Frontend API Documentation

This document provides detailed information about the API endpoints available in the Discord Bot Web Frontend.

## Base URL

All API endpoints are prefixed with `/api`.

## Authentication

Most API endpoints require authentication via JWT token.

### Authentication Endpoints

#### POST /api/auth/login

Authenticates a user with Discord OAuth and returns a JWT token.

**Request:**
- No request body required. Redirects to Discord OAuth flow.

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400,
  "user": {
    "id": "12345678901234567",
    "username": "username",
    "avatar_url": "https://cdn.discordapp.com/avatars/...",
    "guilds": [...]
  }
}
```

#### POST /api/auth/refresh

Refreshes an expired JWT token.

**Request:**
```json
{
  "refresh_token": "refresh_token_here"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 86400
}
```

#### POST /api/auth/logout

Logs out the current user by invalidating their token.

**Request:**
- No request body required.

**Response:**
```json
{
  "success": true
}
```

## Guild Management API

### GET /api/guilds

Lists all guilds the user has access to.

**Request:**
- No request body required.

**Response:**
```json
[
  {
    "id": "12345678901234567",
    "name": "Server Name",
    "icon": "icon_hash",
    "owner": false,
    "permissions": 2147483647,
    "botJoined": true,
    "memberCount": 120,
    "commandsEnabled": 15,
    "wordRules": 5
  },
  ...
]
```

### GET /api/guilds/:guildId

Gets detailed information about a specific guild.

**Request:**
- No request body required.

**Response:**
```json
{
  "id": "12345678901234567",
  "name": "Server Name",
  "icon": "icon_hash",
  "owner": false,
  "permissions": 2147483647,
  "botJoined": true,
  "memberCount": 120,
  "roles": [...],
  "channels": [...],
  "commandsEnabled": 15,
  "wordRules": 5
}
```

### PUT /api/guilds/:guildId/settings

Updates guild settings.

**Request:**
```json
{
  "prefix": "!",
  "logChannelId": "12345678901234567",
  "moderationEnabled": true,
  "autoModeration": {
    "enabled": true,
    "filterLinks": true,
    "filterInvites": true,
    "filterProfanity": false
  },
  "welcomeMessage": {
    "enabled": true,
    "channelId": "12345678901234567",
    "message": "Welcome {user} to {server}!"
  }
}
```

**Response:**
```json
{
  "success": true,
  "guild_id": "12345678901234567",
  "prefix": "!",
  "logChannelId": "12345678901234567",
  "moderationEnabled": true,
  "autoModeration": {
    "enabled": true,
    "filterLinks": true,
    "filterInvites": true,
    "filterProfanity": false
  },
  "welcomeMessage": {
    "enabled": true,
    "channelId": "12345678901234567",
    "message": "Welcome {user} to {server}!"
  }
}
```

## Command Configuration API

### GET /api/guilds/:guildId/commands

Gets all commands available for a guild.

**Request:**
- No request body required.

**Response:**
```json
[
  {
    "id": "help",
    "name": "Help",
    "description": "Shows help information",
    "enabled": true,
    "category": "utility",
    "settings": {}
  },
  {
    "id": "ban",
    "name": "Ban",
    "description": "Bans a user from the server",
    "enabled": true,
    "category": "moderation",
    "settings": {
      "requireReason": true
    }
  },
  ...
]
```

### PUT /api/guilds/:guildId/commands/:commandId

Updates a command's configuration for a guild.

**Request:**
```json
{
  "enabled": true,
  "settings": {
    "requireReason": true
  }
}
```

**Response:**
```json
{
  "success": true,
  "id": "ban",
  "name": "Ban",
  "description": "Bans a user from the server",
  "enabled": true,
  "category": "moderation",
  "settings": {
    "requireReason": true
  }
}
```

### PUT /api/guilds/:guildId/commands/bulk

Updates multiple commands at once.

**Request:**
```json
{
  "commands": [
    {
      "id": "ban",
      "enabled": true
    },
    {
      "id": "kick",
      "enabled": false
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "updated": 2
}
```

## Word Detection API

### GET /api/guilds/:guildId/word-detection

Gets all word detection rules for a guild.

**Request:**
- No request body required.

**Response:**
```json
[
  {
    "id": 1,
    "guild_id": "12345678901234567",
    "pattern": "bad_word",
    "action": "delete",
    "action_params": {},
    "created_at": "2023-01-01T00:00:00Z",
    "updated_at": "2023-01-01T00:00:00Z"
  },
  ...
]
```

### POST /api/guilds/:guildId/word-detection

Creates a new word detection rule.

**Request:**
```json
{
  "pattern": "bad_word",
  "action": "delete",
  "action_params": {
    "notify": true
  }
}
```

**Response:**
```json
{
  "id": 1,
  "guild_id": "12345678901234567",
  "pattern": "bad_word",
  "action": "delete",
  "action_params": {
    "notify": true
  },
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-01T00:00:00Z"
}
```

### PUT /api/guilds/:guildId/word-detection/:ruleId

Updates an existing word detection rule.

**Request:**
```json
{
  "pattern": "updated_bad_word",
  "action": "delete",
  "action_params": {
    "notify": true
  }
}
```

**Response:**
```json
{
  "id": 1,
  "guild_id": "12345678901234567",
  "pattern": "updated_bad_word",
  "action": "delete",
  "action_params": {
    "notify": true
  },
  "created_at": "2023-01-01T00:00:00Z",
  "updated_at": "2023-01-02T00:00:00Z"
}
```

### DELETE /api/guilds/:guildId/word-detection/:ruleId

Deletes a word detection rule.

**Request:**
- No request body required.

**Response:**
```json
{
  "success": true
}
```

### POST /api/guilds/:guildId/word-detection/test

Tests a pattern against a sample text.

**Request:**
```json
{
  "pattern": "bad_word",
  "text": "This is a test with a bad_word in it."
}
```

**Response:**
```json
{
  "matches": true,
  "count": 1,
  "positions": [
    {
      "start": 20,
      "end": 28
    }
  ]
}
```

## Analytics API

### GET /api/analytics/data

Gets analytics data based on specified parameters.

**Request Parameters:**
- `guildId` (optional): Filter by guild ID
- `startDate` (optional): Start date for date range (ISO format)
- `endDate` (optional): End date for date range (ISO format)
- `eventType` (optional): Filter by event type

**Response:**
```json
{
  "commandUsage": [
    { "name": "January", "help": 120, "ban": 45, "kick": 30 },
    { "name": "February", "help": 132, "ban": 42, "kick": 25 }
  ],
  "userActivity": [
    { "name": "Monday", "messages": 430, "commands": 86 },
    { "name": "Tuesday", "messages": 520, "commands": 104 }
  ]
}
```

### GET /api/analytics/summary

Gets analytics summary statistics.

**Request Parameters:**
- `guildId` (optional): Filter by guild ID

**Response:**
```json
[
  { "title": "Total Commands Used", "value": 12453, "change": 5.2 },
  { "title": "Active Users", "value": 387, "change": 2.1 },
  { "title": "Messages Processed", "value": 24789, "change": -1.3 },
  { "title": "Rules Triggered", "value": 126, "change": 7.8 }
]
```

## Error Responses

All API endpoints return a standard error format when an error occurs:

```json
{
  "error": "Error message",
  "code": "ERROR_CODE",
  "status": 400
}
```

Common status codes:

- `400`: Bad Request - Invalid parameters
- `401`: Unauthorized - Authentication required
- `403`: Forbidden - Insufficient permissions
- `404`: Not Found - Resource not found
- `429`: Too Many Requests - Rate limit exceeded
- `500`: Internal Server Error - Server error

## Rate Limiting

API endpoints are subject to rate limiting. The following headers are included in all responses:

- `X-RateLimit-Limit`: Maximum requests allowed in the time window
- `X-RateLimit-Remaining`: Remaining requests in the current time window
- `X-RateLimit-Reset`: Time in seconds until the rate limit resets

If the rate limit is exceeded, a `429 Too Many Requests` status code is returned with the following response:

```json
{
  "error": "Rate limit exceeded",
  "code": "RATE_LIMIT_EXCEEDED",
  "status": 429,
  "retry_after": 60
}
