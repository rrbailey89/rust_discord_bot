-- 000_create_tables.sql
-- Create tables for the Discord bot database

-- Create warn_channel_ids table
CREATE TABLE IF NOT EXISTS warn_channel_ids (
    guild_id BIGINT PRIMARY KEY,
    channel_id BIGINT NOT NULL
);

-- Create user_hug_counts table
CREATE TABLE IF NOT EXISTS user_hug_counts (
    user_id BIGINT PRIMARY KEY,
    hug_count INT NOT NULL DEFAULT 0
);

-- Create guild_info table
CREATE TABLE IF NOT EXISTS guild_info (
    guild_id BIGINT PRIMARY KEY,
    guild_name TEXT NOT NULL,
    owner_id BIGINT NOT NULL,
    member_count INT NOT NULL
);

-- Create guild_channels table
CREATE TABLE IF NOT EXISTS guild_channels (
    channel_id BIGINT PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    channel_name TEXT NOT NULL,
    channel_type TEXT NOT NULL
);

-- Create message_delete_channels table
CREATE TABLE IF NOT EXISTS message_delete_channels (
    guild_id BIGINT PRIMARY KEY,
    channel_id BIGINT NOT NULL,
    guild_name TEXT NOT NULL
);

-- Create user_unavailability table
CREATE TABLE IF NOT EXISTS user_unavailability (
    id SERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    unavailable_date DATE NOT NULL,
    reason TEXT,
    message_id BIGINT
);

-- Create reminders table
CREATE TABLE IF NOT EXISTS reminders (
    id SERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    message TEXT NOT NULL,
    time TIME NOT NULL,
    days INTEGER[] NOT NULL,
    frequency TEXT NOT NULL,
    last_sent TIMESTAMP WITH TIME ZONE
);

-- Create user_levels table
CREATE TABLE IF NOT EXISTS user_levels (
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    level INT NOT NULL DEFAULT 1,
    experience INT NOT NULL DEFAULT 0,
    PRIMARY KEY (guild_id, user_id)
);

-- Create unavailability_channels table
CREATE TABLE IF NOT EXISTS unavailability_channels (
    guild_id BIGINT PRIMARY KEY,
    channel_id BIGINT NOT NULL
);

-- Create guild_emoji_settings table
CREATE TABLE IF NOT EXISTS guild_emoji_settings (
    guild_id BIGINT PRIMARY KEY,
    emoji_reactions_enabled BOOLEAN NOT NULL DEFAULT TRUE
);

-- Create user_conversation_ids table
CREATE TABLE IF NOT EXISTS user_conversation_ids (
    id SERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    conversation_id TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL
);

-- Create blame_count table
CREATE TABLE IF NOT EXISTS blame_count (
    id INT PRIMARY KEY,
    count INT NOT NULL DEFAULT 0
);

-- Create user_blame_count table
CREATE TABLE IF NOT EXISTS user_blame_count (
    user_id BIGINT PRIMARY KEY,
    count INT NOT NULL DEFAULT 0
);

-- Create guild_rules table
CREATE TABLE IF NOT EXISTS guild_rules (
    id SERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    rule TEXT NOT NULL
);

-- Create level_up_channels table
CREATE TABLE IF NOT EXISTS level_up_channels (
    guild_id BIGINT PRIMARY KEY,
    channel_id BIGINT NOT NULL
);

-- Create url_rules table
CREATE TABLE IF NOT EXISTS url_rules (
    guild_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    regex TEXT NOT NULL,
    output_template TEXT NOT NULL,
    PRIMARY KEY (guild_id, channel_id)
);

-- Create button_configs table
CREATE TABLE IF NOT EXISTS button_configs (
    guild_id BIGINT NOT NULL,
    message_id BIGINT NOT NULL,
    config JSONB NOT NULL,
    PRIMARY KEY (guild_id, message_id)
);

-- Create star_channels table
CREATE TABLE IF NOT EXISTS star_channels (
    guild_id BIGINT PRIMARY KEY,
    channel_id BIGINT NOT NULL
);

-- Create reaction_log_channels table
CREATE TABLE IF NOT EXISTS reaction_log_channels (
    guild_id BIGINT NOT NULL,
    channel_id BIGINT NOT NULL,
    log_type TEXT NOT NULL,
    PRIMARY KEY (guild_id, log_type)
);
