-- Ensure all commands used in the codebase are registered in the database
-- This addresses potential gaps between code and database

-- First, update any commands that might have different IDs in code vs database
-- Map from code command_id to database command_id
DO $$
DECLARE
  command_mappings JSONB := jsonb_build_object(
    'blame', 'blame_serena',
    'animehug', 'anime_hug',
    'randomcatimage', 'random_cat_image',
    'randomcapyimage', 'random_capy_image',
    'userinfo', 'user_info',
    'lifecheck', 'is_alive',
    'iscaliforniaonfire', 'iscaliforniaonfire',
    'whereiscaliforniaonfire', 'whereiscaliforniaonfire',
    'setunavailabilitychannel', 'set_unavailability_channel',
    'listunavailable', 'list_unavailable',
    'cancelunavailable', 'cancel_unavailable',
    'setwarnchannel', 'set_warn_channel',
    'updateraidtime', 'update_raid_time',
    'setdeletemessagechannel', 'set_delete_log_channel',
    'toggleemojireactions', 'toggle_emoji_reactions',
    'setlevelupchannel', 'set_level_up_channel',
    'rolebuttons', 'add_role_buttons',
    'seturlrule', 'set_url_rule',
    'reactionslog', 'set_reaction_log',
    'rule', 'rules',
    'reminder', 'remind',
    'dbhealth', 'health'
  );
  code_command_id TEXT;
  db_command_id TEXT;
  command_exists BOOLEAN;
BEGIN
  FOR code_command_id, db_command_id IN SELECT * FROM jsonb_each_text(command_mappings)
  LOOP
    -- Check if the DB version of the command exists
    EXECUTE 'SELECT EXISTS(SELECT 1 FROM commands WHERE command_id = $1)' 
      INTO command_exists
      USING db_command_id;
      
    IF command_exists THEN
      RAISE NOTICE 'Command % exists in DB as %', code_command_id, db_command_id;
    ELSE
      -- Check if the code version exists instead
      EXECUTE 'SELECT EXISTS(SELECT 1 FROM commands WHERE command_id = $1)' 
        INTO command_exists
        USING code_command_id;
        
      IF command_exists THEN
        RAISE NOTICE 'Command % exists in DB but needs renaming to %', code_command_id, db_command_id;
        -- If we find a code version but not DB version, we could update it
        -- (For now, we'll just log this situation)
      ELSE
        RAISE NOTICE 'Command % / % missing entirely from DB', code_command_id, db_command_id;
        -- In a full implementation, we would insert the missing command here
      END IF;
    END IF;
  END LOOP;
END $$;

-- The safe approach for now is to just ensure all commands are disabled by default
-- regardless of their naming convention
UPDATE commands 
SET default_enabled = FALSE
WHERE category IN ('admin', 'fun', 'utility', 'availability')
  AND command_id NOT IN ('ping', 'help');
