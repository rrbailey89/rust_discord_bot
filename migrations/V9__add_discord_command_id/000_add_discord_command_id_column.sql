-- Add discord_command_id column to guild_command_settings table
ALTER TABLE guild_command_settings ADD COLUMN discord_command_id TEXT;

-- Add an index for faster lookups
CREATE INDEX idx_guild_command_settings_discord_command_id ON guild_command_settings(discord_command_id);

-- Add comment explaining the column
COMMENT ON COLUMN guild_command_settings.discord_command_id IS 'Discord-assigned ID for the registered command';
