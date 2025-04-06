-- V8: Ensure all commands are correctly registered in the database
-- This migration adds missing commands and sets proper default values

-- Helper function to insert commands if they don't exist
CREATE OR REPLACE FUNCTION insert_command_if_not_exists(
    p_command_id TEXT,
    p_name TEXT,
    p_description TEXT,
    p_category TEXT,
    p_default_enabled BOOLEAN,
    p_discord_name TEXT DEFAULT NULL
) RETURNS VOID AS $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM commands WHERE command_id = p_command_id) THEN
        INSERT INTO commands (
            command_id, 
            name, 
            description, 
            category, 
            default_enabled, 
            options,
            discord_name,
            created_at
        ) VALUES (
            p_command_id,
            p_name,
            p_description,
            p_category,
            p_default_enabled,
            '{}',
            p_discord_name,
            NOW()
        );
        RAISE NOTICE 'Added missing command: %', p_command_id;
    ELSE
        -- Update the default_enabled flag for existing commands
        UPDATE commands
        SET default_enabled = p_default_enabled
        WHERE command_id = p_command_id;
        RAISE NOTICE 'Updated default_enabled for command: %', p_command_id;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Insert or update all commands from src/commands.rs
-- Admin commands (all disabled by default)
SELECT insert_command_if_not_exists('warn', 'Warn', 'Warn a user for breaking rules', 'admin', FALSE, 'warn');
SELECT insert_command_if_not_exists('setwarnchannel', 'Set Warn Channel', 'Set the warn channel', 'admin', FALSE, 'setwarnchannel');
SELECT insert_command_if_not_exists('updateraidtime', 'Update Raid Time', 'Update raid time', 'admin', FALSE, 'updateraidtime');
SELECT insert_command_if_not_exists('purge', 'Purge', 'Purge messages', 'admin', FALSE, 'purge');
SELECT insert_command_if_not_exists('setdeletemessagechannel', 'Set Delete Message Channel', 'Set delete message log channel', 'admin', FALSE, 'setdeletemessagechannel');
SELECT insert_command_if_not_exists('toggleemojireactions', 'Toggle Emoji Reactions', 'Toggle emoji reactions', 'admin', FALSE, 'toggleemojireactions');
SELECT insert_command_if_not_exists('setlevelupchannel', 'Set Level Up Channel', 'Set level up channel', 'admin', FALSE, 'setlevelupchannel');
SELECT insert_command_if_not_exists('rolebuttons', 'Role Buttons', 'Create role buttons', 'admin', FALSE, 'rolebuttons');
SELECT insert_command_if_not_exists('seturlrule', 'Set URL Rule', 'Set URL rule', 'admin', FALSE, 'seturlrule');
SELECT insert_command_if_not_exists('reactionslog', 'Reactions Log', 'Set reactions log channel', 'admin', FALSE, 'reactionslog');

-- Fun commands (all disabled by default)
SELECT insert_command_if_not_exists('randomcatimage', 'Random Cat Image', 'Get a random cat image', 'fun', FALSE, 'randomcatimage');
SELECT insert_command_if_not_exists('randomcapyimage', 'Random Capybara Image', 'Get a random capybara image', 'fun', FALSE, 'randomcapyimage');
SELECT insert_command_if_not_exists('animehug', 'Anime Hug', 'Send an anime hug', 'fun', FALSE, 'animehug');
SELECT insert_command_if_not_exists('createimage', 'Create Image', 'Create an AI image', 'fun', FALSE, 'createimage');
SELECT insert_command_if_not_exists('blame', 'Blame', 'Blame Serena for something', 'fun', FALSE, 'blame');
SELECT insert_command_if_not_exists('fluximage', 'Flux Image', 'Get a flux image', 'fun', FALSE, 'fluximage');

-- Also make sure the database version exists (we saw blame_serena in screenshots)
SELECT insert_command_if_not_exists('blame_serena', 'Blame Serena', 'Blame Serena for something', 'fun', FALSE, 'blameserena');

-- Utility commands (all disabled by default except ping and help)
SELECT insert_command_if_not_exists('userinfo', 'User Info', 'Get user information', 'utility', FALSE, 'userinfo');
SELECT insert_command_if_not_exists('ask', 'Ask', 'Ask a question', 'utility', FALSE, 'ask');
SELECT insert_command_if_not_exists('reminder', 'Reminder', 'Set a reminder', 'utility', FALSE, 'reminder');
SELECT insert_command_if_not_exists('weather', 'Weather', 'Get weather information', 'utility', FALSE, 'weather');
SELECT insert_command_if_not_exists('rule', 'Rule', 'Display server rules', 'utility', FALSE, 'rule');
SELECT insert_command_if_not_exists('lifecheck', 'Life Check', 'Check if the bot is alive', 'utility', FALSE, 'lifecheck');
SELECT insert_command_if_not_exists('relay', 'Relay', 'Relay a message', 'utility', FALSE, 'relay');
SELECT insert_command_if_not_exists('dbhealth', 'DB Health', 'Check database health', 'utility', FALSE, 'dbhealth');
SELECT insert_command_if_not_exists('dbschema', 'DB Schema', 'Show database schema', 'utility', FALSE, 'dbschema');

-- Global commands (enabled by default)
SELECT insert_command_if_not_exists('ping', 'Ping', 'Check bot latency', 'utility', TRUE, 'ping');
SELECT insert_command_if_not_exists('help', 'Help', 'Show command help', 'utility', TRUE, 'help');

-- Availability commands (all disabled by default)
SELECT insert_command_if_not_exists('setunavailabilitychannel', 'Set Unavailability Channel', 'Set unavailability channel', 'availability', FALSE, 'setunavailabilitychannel');
SELECT insert_command_if_not_exists('unavailable', 'Unavailable', 'Mark yourself as unavailable', 'availability', FALSE, 'unavailable');
SELECT insert_command_if_not_exists('listunavailable', 'List Unavailable', 'List unavailable users', 'availability', FALSE, 'listunavailable');
SELECT insert_command_if_not_exists('cancelunavailable', 'Cancel Unavailable', 'Cancel unavailability', 'availability', FALSE, 'cancelunavailable');

-- Additional utility commands
SELECT insert_command_if_not_exists('iscaliforniaonfire', 'Is California on Fire', 'Is California on fire?', 'utility', FALSE, 'iscaliforniaonfire');
SELECT insert_command_if_not_exists('whereiscaliforniaonfire', 'Where is California on Fire', 'Where is California on fire?', 'utility', FALSE, 'whereiscaliforniaonfire');

-- Also account for commands that might already exist with different IDs (from screenshots)
SELECT insert_command_if_not_exists('list_unavailable', 'List Unavailable', 'List all unavailable users', 'availability', FALSE, 'listunavailable');
SELECT insert_command_if_not_exists('remind', 'Reminder', 'Set a reminder', 'utility', FALSE, 'remind');
SELECT insert_command_if_not_exists('rules', 'Rules', 'Display server rules', 'utility', FALSE, 'rules');

-- Ensure existing commands are disabled if they're guild-scoped
UPDATE commands
SET default_enabled = FALSE
WHERE category IN ('admin', 'fun', 'utility', 'availability')
  AND command_id NOT IN ('ping', 'help');

-- Ensure global commands are enabled
UPDATE commands
SET default_enabled = TRUE
WHERE command_id IN ('ping', 'help');

-- Clean up the temporary function
DROP FUNCTION IF EXISTS insert_command_if_not_exists;

-- Final verification message
DO $$
BEGIN
    RAISE NOTICE 'Command registration complete. All commands set to default_enabled=FALSE except ping and help';
END $$;
