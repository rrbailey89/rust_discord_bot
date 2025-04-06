-- Create a users table to store basic user information
CREATE TABLE IF NOT EXISTS users (
    user_id BIGINT PRIMARY KEY,
    username TEXT NOT NULL,
    discriminator TEXT NOT NULL,
    avatar TEXT,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create guild_members table to store detailed member information
CREATE TABLE IF NOT EXISTS guild_members (
    guild_id BIGINT NOT NULL,
    user_id BIGINT NOT NULL,
    nickname TEXT,
roles text[] NOT NULL,
    joined_at TEXT,
    PRIMARY KEY (guild_id, user_id)
);

-- Index for faster lookups
CREATE INDEX IF NOT EXISTS guild_members_guild_idx ON guild_members(guild_id);
CREATE INDEX IF NOT EXISTS guild_members_user_idx ON guild_members(user_id);
