-- Create the commands table that's referenced but doesn't exist
CREATE TABLE IF NOT EXISTS commands (
    command_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL,
    category TEXT NOT NULL,
    default_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    options JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Create the guild_settings table that's referenced but doesn't exist
CREATE TABLE IF NOT EXISTS guild_settings (
    guild_id BIGINT PRIMARY KEY,
    prefix TEXT,
    mod_role_id BIGINT,
    admin_role_id BIGINT,
    welcome_channel_id BIGINT,
    welcome_message TEXT,
    auto_role_id BIGINT,
    log_channel_id BIGINT,
    settings JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Add indices for better performance
CREATE INDEX IF NOT EXISTS idx_commands_category ON commands(category);
CREATE INDEX IF NOT EXISTS idx_guild_settings_updated_at ON guild_settings(updated_at);
