-- Update all guild-scoped commands to be disabled by default
-- This ensures new guilds need to explicitly enable these commands

-- First, update the existing commands table
UPDATE commands 
SET default_enabled = FALSE
WHERE command_id IN (
    'warn', 'setwarnchannel', 'updateraidtime', 'purge', 'setdeletemessagechannel',
    'toggleemojireactions', 'setlevelupchannel', 'rolebuttons', 'seturlrule',
    'reactionslog', 'randomcatimage', 'randomcapyimage', 'animehug', 'createimage',
    'blame', 'fluximage', 'userinfo', 'ask', 'reminder', 'weather', 'rule',
    'lifecheck', 'relay', 'dbhealth', 'dbschema', 'setunavailabilitychannel',
    'unavailable', 'listunavailable', 'cancelunavailable', 'iscaliforniaonfire',
    'whereiscaliforniaonfire'
);

-- Keep global commands enabled by default
UPDATE commands
SET default_enabled = TRUE
WHERE command_id IN ('ping', 'help');
