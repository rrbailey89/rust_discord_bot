-- Set default enabled state for commands in guild_command_settings

-- Step 1: Disable all commands by default for all guilds
UPDATE guild_command_settings SET enabled = false;

-- Step 2: Enable 'ping' and 'help' commands specifically for all guilds where they exist
UPDATE guild_command_settings
SET enabled = true
WHERE command_id IN ('ping', 'help');

-- Note: The initialize_guild_command_settings function in command.rs
-- should also be updated to reflect this new default logic for future guilds/commands.
