# Web Frontend Implementation Plan

## Technology Stack

1. **Backend:**
   - Actix Web for the API server
   - JWT for authentication tokens
   - Discord OAuth for user authentication
   - Rate limiting middleware

2. **Frontend:**
   - React with TypeScript
   - React Router for client-side routing
   - Axios for API requests
   - React Query for data fetching and caching
   - Styled Components or Tailwind CSS for styling

3. **Analytics:**
   - Server-side request logging
   - Client-side event tracking
   - Performance metrics collection
   - Admin dashboard for analytics visualization

## System Architecture

The system will consist of two main components running in the same container:
1. Discord Bot Process - The existing Discord bot
2. Actix Web Server - Serving the API and React frontend

These components will share the same database connection pool, configuration, and other resources.

## Implementation Phases

### Phase 1: Backend Foundation (2-3 days)

1. **Add Dependencies to Cargo.toml:**
   ```toml
   actix-web = "4.3"
   actix-files = "0.6"
   actix-cors = "0.6"
   jsonwebtoken = "8.3"
   oauth2 = "4.4"
   actix-web-lab = "0.19" # For rate limiting
   ```

2. **Create Web Server Structure:**
   - Create `src/web/mod.rs` as the entry point
   - Implement server configuration and startup
   - Set up shared state with bot application

3. **Implement Basic API Endpoints:**
   - Health check endpoint
   - Version information
   - Basic guild information retrieval

4. **Set Up Authentication:**
   - Discord OAuth flow
   - JWT token generation and validation
   - Authentication middleware

5. **Implement Rate Limiting:**
   - IP-based rate limiting for unauthenticated requests
   - User-based rate limiting for authenticated requests
   - Different limits for different endpoint categories

### Phase 2: Core API Development (3-4 days)

1. **Guild Management API:**
   - List guilds the user has access to
   - Get detailed guild information
   - Update guild settings

2. **Command Configuration API:**
   - List available commands
   - Enable/disable commands for guilds
   - Configure command settings

3. **Word Detection API:**
   - Create, read, update, delete word detection rules
   - Test word detection rules
   - Configure actions for rule triggers

4. **Settings API:**
   - Get and update global bot settings
   - Configure guild-specific settings
   - Manage user preferences

5. **Analytics API:**
   - Collect usage metrics
   - Track API performance
   - Store user interactions

### Phase 3: Frontend Setup (2-3 days)

1. **Create React Project:**
   - Set up React with TypeScript
   - Configure build system (Webpack/Vite)
   - Set up linting and formatting

2. **Implement Authentication UI:**
   - Login page with Discord OAuth
   - Authentication state management
   - Protected routes

3. **Create Core Layout:**
   - Header with navigation
   - Sidebar for guild selection
   - Main content area
   - Responsive design

4. **Implement API Client:**
   - Set up Axios with interceptors
   - Handle authentication tokens
   - Implement error handling

### Phase 4: Frontend Features (4-5 days)

1. **Guild Management UI:**
   - Guild selection interface
   - Guild settings configuration
   - Guild information display

2. **Command Management UI:**
   - Command list with toggle switches
   - Command configuration forms
   - Command usage statistics

3. **Word Detection UI:**
   - Rule creation and editing interface
   - Rule testing tool
   - Action configuration

4. **Settings UI:**
   - Global settings form
   - Guild-specific settings
   - User preferences

5. **Analytics Dashboard:**
   - Usage statistics charts
   - Performance metrics
   - User activity visualization

### Phase 5: Integration and Deployment (2-3 days)

1. **Update Dockerfile:**
   - Add Node.js for frontend building
   - Configure multi-stage build process
   - Include frontend assets in final image

2. **Update Docker Compose:**
   - Expose port 3000
   - Add environment variables for web configuration
   - Configure volume mounts if needed

3. **Integration Testing:**
   - Test API endpoints
   - Verify frontend functionality
   - Check authentication flow
   - Test rate limiting

4. **Documentation:**
   - API documentation
   - Frontend component documentation
   - Deployment instructions
   - User guide

## Database Schema Extensions

We'll need to add the following tables to the existing database:

1. **guild_command_settings:**
   ```sql
   CREATE TABLE guild_command_settings (
       guild_id BIGINT NOT NULL,
       command_id TEXT NOT NULL,
       enabled BOOLEAN NOT NULL DEFAULT TRUE,
       settings JSONB,
       PRIMARY KEY (guild_id, command_id)
   );
   ```

2. **word_detection_rules:**
   ```sql
   CREATE TABLE word_detection_rules (
       id SERIAL PRIMARY KEY,
       guild_id BIGINT NOT NULL,
       pattern TEXT NOT NULL,
       action TEXT NOT NULL,
       action_params JSONB,
       created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
       updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
   );
   ```

3. **user_sessions:**
   ```sql
   CREATE TABLE user_sessions (
       user_id BIGINT PRIMARY KEY,
       discord_token TEXT,
       token_expires_at TIMESTAMP WITH TIME ZONE,
       refresh_token TEXT,
       last_login TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
   );
   ```

4. **analytics_events:**
   ```sql
   CREATE TABLE analytics_events (
       id SERIAL PRIMARY KEY,
       event_type TEXT NOT NULL,
       user_id BIGINT,
       guild_id BIGINT,
       event_data JSONB,
       timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
   );
   ```

## Code Structure

### Backend Structure

```
src/
  web/
    mod.rs                 # Web server entry point
    server.rs              # Server configuration
    state.rs               # Shared application state
    middleware/
      auth.rs              # Authentication middleware
      rate_limit.rs        # Rate limiting middleware
      logging.rs           # Request logging middleware
    routes/
      mod.rs               # Route registration
      auth.rs              # Authentication routes
      guilds.rs            # Guild management routes
      commands.rs          # Command configuration routes
      word_detection.rs    # Word detection routes
      settings.rs          # Settings routes
      analytics.rs         # Analytics routes
    handlers/
      auth.rs              # Authentication handlers
      guilds.rs            # Guild handlers
      commands.rs          # Command handlers
      word_detection.rs    # Word detection handlers
      settings.rs          # Settings handlers
      analytics.rs         # Analytics handlers
    models/
      guild.rs             # Guild data models
      command.rs           # Command data models
      rule.rs              # Word detection rule models
      settings.rs          # Settings models
      analytics.rs         # Analytics models
    services/
      discord.rs           # Discord API service
      analytics.rs         # Analytics service
```

### Frontend Structure

```
frontend/
  src/
    components/
      layout/
        Header.tsx
        Sidebar.tsx
        Footer.tsx
      auth/
        LoginButton.tsx
        AuthGuard.tsx
      guilds/
        GuildList.tsx
        GuildCard.tsx
        GuildSettings.tsx
      commands/
        CommandList.tsx
        CommandToggle.tsx
        CommandSettings.tsx
      word-detection/
        RuleList.tsx
        RuleEditor.tsx
        RuleTester.tsx
      settings/
        SettingsForm.tsx
      analytics/
        Dashboard.tsx
        UsageChart.tsx
        ActivityLog.tsx
      common/
        Button.tsx
        Card.tsx
        Modal.tsx
        Table.tsx
    hooks/
      useAuth.ts
      useGuilds.ts
      useCommands.ts
      useRules.ts
      useSettings.ts
      useAnalytics.ts
    services/
      api.ts
      auth.ts
      analytics.ts
    pages/
      Home.tsx
      Login.tsx
      GuildManagement.tsx
      CommandConfig.tsx
      WordDetection.tsx
      Settings.tsx
      Analytics.tsx
    App.tsx
    index.tsx
    routes.tsx
```

## Integration with Main Application

The web server will be integrated with the main application by:

1. **Shared State:**
   - The bot's data structures will be wrapped in Arc for thread-safe sharing
   - Database connections will be shared via a connection pool
   - Configuration will be shared between bot and web server

2. **Startup Process:**
   - Main function will initialize shared components
   - Bot will be started in its own thread
   - Web server will be started in a separate thread
   - Graceful shutdown will be coordinated between both
