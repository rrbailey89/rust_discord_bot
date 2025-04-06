-- Create user_sessions table for storing Discord OAuth tokens
CREATE TABLE user_sessions (
    user_id BIGINT PRIMARY KEY,
    discord_token TEXT,
    token_expires_at TIMESTAMP WITH TIME ZONE,
    refresh_token TEXT,
    last_login TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create guild_command_settings table
CREATE TABLE guild_command_settings (
    guild_id BIGINT NOT NULL,
    command_id TEXT NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    settings JSONB,
    PRIMARY KEY (guild_id, command_id)
);

-- Create word_detection_rules table
CREATE TABLE word_detection_rules (
    id SERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    pattern TEXT NOT NULL,
    action TEXT NOT NULL,
    action_params JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create analytics_events table
CREATE TABLE analytics_events (
    id SERIAL PRIMARY KEY,
    event_type TEXT NOT NULL,
    user_id BIGINT,
    guild_id BIGINT,
    event_data JSONB,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Add indices for performance
CREATE INDEX idx_user_sessions_last_login ON user_sessions(last_login);
CREATE INDEX idx_word_detection_rules_guild_id ON word_detection_rules(guild_id);
CREATE INDEX idx_analytics_events_event_type ON analytics_events(event_type);
CREATE INDEX idx_analytics_events_timestamp ON analytics_events(timestamp);
