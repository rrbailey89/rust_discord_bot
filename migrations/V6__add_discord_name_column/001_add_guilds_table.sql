-- Create guilds table for tracking bot membership
CREATE TABLE IF NOT EXISTS guilds (
    guild_id BIGINT PRIMARY KEY,
    bot_joined BOOLEAN NOT NULL DEFAULT TRUE,
    joined_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    name TEXT,
    icon TEXT,
    owner_id BIGINT,
    member_count INTEGER
);

-- Add index for performance
CREATE INDEX IF NOT EXISTS idx_guilds_bot_joined ON guilds(bot_joined);

-- Populate the guilds table from existing guild data
INSERT INTO guilds (guild_id, name, owner_id, member_count)
SELECT guild_id, guild_name, owner_id, member_count 
FROM guild_info
ON CONFLICT (guild_id) DO NOTHING;
