-- Update the blame_serena command to be disabled by default
-- This ensures new guilds need to explicitly enable the command

UPDATE commands 
SET default_enabled = FALSE
WHERE command_id = 'blame_serena';
