-- Create table for storing the non-availability channel for each guild
CREATE TABLE IF NOT EXISTS unavailability_channels (
    guild_id BIGINT PRIMARY KEY,
    channel_id BIGINT NOT NULL
);

-- Create table for storing user unavailability records
CREATE TABLE IF NOT EXISTS user_unavailability (
    id SERIAL PRIMARY KEY,
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    unavailable_date DATE NOT NULL,
    reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Add indices for better query performance
CREATE INDEX IF NOT EXISTS idx_user_unavailability_guild_id ON user_unavailability(guild_id);
CREATE INDEX IF NOT EXISTS idx_user_unavailability_user_id ON user_unavailability(user_id);
CREATE INDEX IF NOT EXISTS idx_user_unavailability_date ON user_unavailability(unavailable_date);
