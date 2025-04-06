-- Add missing commands from src/commands.rs to the database
-- This ensures all commands defined in the codebase have corresponding database entries

-- Helper function to insert commands if they don't exist
CREATE OR REPLACE FUNCTION insert_command_if_not_exists(
    p_command_id TEXT,
    p_name TEXT,
    p_description TEXT,
    p_category TEXT,
    p_default_enabled BOOLEAN
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
            created_at
        ) VALUES (
            p_command_id,
            p_name,
            p_description,
            p_category,
            p_default_enabled,
            '{}',
            NOW()
        );
        RAISE NOTICE 'Added missing command: %', p_command_id;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Insert missing commands from src/commands.rs
-- Admin commands
SELECT insert_command_if_not_exists('warn', 'Warn', 'Warn a user for breaking rules', 'admin', FALSE);
SELECT insert_command_if_not_exists('setwarnchannel', 'Set Warn Channel', 'Set the warn channel', 'admin', FALSE);
SELECT insert_command_if_not_exists('updateraidtime', 'Update Raid Time', 'Update raid time', 'admin', FALSE);
SELECT insert_command_if_not_exists('purge', 'Purge', 'Purge messages', 'admin', FALSE);
SELECT insert_command_if_not_exists('setdeletemessagechannel', 'Set Delete Message Channel', 'Set delete message log channel', 'admin', FALSE);
SELECT insert_command_if_not_exists('toggleemojireactions', 'Toggle Emoji Reactions', 'Toggle emoji reactions', 'admin', FALSE);
SELECT insert_command_if_not_exists('setlevelupchannel', 'Set Level Up Channel', 'Set level up channel', 'admin', FALSE);
SELECT insert_command_if_not_exists('rolebuttons', 'Role Buttons', 'Create role buttons', 'admin', FALSE);
SELECT insert_command_if_not_exists('seturlrule', 'Set URL Rule', 'Set URL rule', 'admin', FALSE);
SELECT insert_command_if_not_exists('reactionslog', 'Reactions Log', 'Set reactions log channel', 'admin', FALSE);

-- Fun commands
SELECT insert_command_if_not_exists('randomcatimage', 'Random Cat Image', 'Get a random cat image', 'fun', FALSE);
SELECT insert_command_if_not_exists('randomcapyimage', 'Random Capybara Image', 'Get a random capybara image', 'fun', FALSE);
SELECT insert_command_if_not_exists('animehug', 'Anime Hug', 'Send an anime hug', 'fun', FALSE);
SELECT insert_command_if_not_exists('createimage', 'Create Image', 'Create an AI image', 'fun', FALSE);
SELECT insert_command_if_not_exists('blame', 'Blame', 'Blame Serena for something', 'fun', FALSE);
SELECT insert_command_if_not_exists('fluximage', 'Flux Image', 'Get a flux image', 'fun', FALSE);

-- Utility commands
SELECT insert_command_if_not_exists('userinfo', 'User Info', 'Get user information', 'utility', FALSE);
SELECT insert_command_if_not_exists('ask', 'Ask', 'Ask a question', 'utility', FALSE);
SELECT insert_command_if_not_exists('reminder', 'Reminder', 'Set a reminder', 'utility', FALSE);
SELECT insert_command_if_not_exists('weather', 'Weather', 'Get weather information', 'utility', FALSE);
SELECT insert_command_if_not_exists('rule', 'Rule', 'Display server rules', 'utility', FALSE);
SELECT insert_command_if_not_exists('lifecheck', 'Life Check', 'Check if the bot is alive', 'utility', FALSE);
SELECT insert_command_if_not_exists('relay', 'Relay', 'Relay a message', 'utility', FALSE);
SELECT insert_command_if_not_exists('dbhealth', 'DB Health', 'Check database health', 'utility', FALSE);
SELECT insert_command_if_not_exists('dbschema', 'DB Schema', 'Show database schema', 'utility', FALSE);
SELECT insert_command_if_not_exists('ping', 'Ping', 'Check bot latency', 'utility', TRUE);
SELECT insert_command_if_not_exists('help', 'Help', 'Show command help', 'utility', TRUE);

-- Availability commands
SELECT insert_command_if_not_exists('setunavailabilitychannel', 'Set Unavailability Channel', 'Set unavailability channel', 'availability', FALSE);
SELECT insert_command_if_not_exists('unavailable', 'Unavailable', 'Mark yourself as unavailable', 'availability', FALSE);
SELECT insert_command_if_not_exists('listunavailable', 'List Unavailable', 'List unavailable users', 'availability', FALSE);
SELECT insert_command_if_not_exists('cancelunavailable', 'Cancel Unavailable', 'Cancel unavailability', 'availability', FALSE);

-- Additional utility commands
SELECT insert_command_if_not_exists('iscaliforniaonfire', 'Is California on Fire', 'Is California on fire?', 'utility', FALSE);
SELECT insert_command_if_not_exists('whereiscaliforniaonfire', 'Where is California on Fire', 'Where is California on fire?', 'utility', FALSE);

-- Clean up the temporary function
DROP FUNCTION IF EXISTS insert_command_if_not_exists;

-- Final verification
DO $$
DECLARE
    code_commands TEXT[] := ARRAY[
        'ping', 'help', 'warn', 'setwarnchannel', 'updateraidtime', 'purge', 
        'setdeletemessagechannel', 'toggleemojireactions', 'setlevelupchannel', 
        'rolebuttons', 'seturlrule', 'reactionslog', 'randomcatimage', 
        'randomcapyimage', 'animehug', 'createimage', 'blame', 'fluximage', 
        'userinfo', 'ask', 'reminder', 'weather', 'rule', 'lifecheck', 'relay', 
        'dbhealth', 'dbschema', 'setunavailabilitychannel', 'unavailable', 
        'listunavailable', 'cancelunavailable', 'iscaliforniaonfire', 
        'whereiscaliforniaonfire'
    ];
    
    cmd TEXT;
    missing_count INT := 0;
    db_cmd_count INT;
BEGIN
    -- Check each command
    FOREACH cmd IN ARRAY code_commands
    LOOP
        EXECUTE 'SELECT COUNT(*) FROM commands WHERE command_id = $1 OR command_id = $2'
        INTO db_cmd_count
        USING cmd, REPLACE(cmd, '_', '');
        
        IF db_cmd_count = 0 THEN
            RAISE WARNING 'Command still missing after migration: %', cmd;
            missing_count := missing_count + 1;
        END IF;
    END LOOP;
    
    IF missing_count = 0 THEN
        RAISE NOTICE 'All commands from src/commands.rs are now in the database!';
    ELSE
        RAISE WARNING '% commands from src/commands.rs are still missing from the database', missing_count;
    END IF;
END $$;
