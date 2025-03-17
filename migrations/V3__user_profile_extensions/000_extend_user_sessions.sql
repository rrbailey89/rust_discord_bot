-- Add new columns to user_sessions table for the additional OAuth scopes
ALTER TABLE user_sessions
    ADD COLUMN IF NOT EXISTS email TEXT,
    ADD COLUMN IF NOT EXISTS verified BOOLEAN,
    ADD COLUMN IF NOT EXISTS locale TEXT;

-- Create new table for guild member information
CREATE TABLE IF NOT EXISTS guild_members (
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    nickname TEXT,
    roles TEXT[],
    joined_at TIMESTAMP WITH TIME ZONE,
    PRIMARY KEY (guild_id, user_id)
);

-- Add indices for performance
CREATE INDEX IF NOT EXISTS idx_guild_members_user_id ON guild_members(user_id);
CREATE INDEX IF NOT EXISTS idx_guild_members_guild_id ON guild_members(guild_id);
