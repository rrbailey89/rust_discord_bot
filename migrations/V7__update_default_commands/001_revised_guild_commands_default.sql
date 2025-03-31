-- Revised migration to set all guild commands disabled by default
-- This addresses the issues with command ID naming conventions

-- First, set all non-global commands to disabled by default
UPDATE commands 
SET default_enabled = FALSE
WHERE category IN ('admin', 'fun', 'utility', 'availability')
  AND command_id NOT IN ('ping', 'help');

-- Keep global commands enabled by default
UPDATE commands
SET default_enabled = TRUE
WHERE command_id IN ('ping', 'help');

-- Note: This migration changes default_enabled to FALSE for all non-global commands,
-- ensuring that new guilds must explicitly enable commands before they appear.
-- Existing guild settings are NOT affected by this change.
