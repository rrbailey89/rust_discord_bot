-- Populate the options column for commands with arguments/subcommands

-- Helper function to create a JSON option object
-- Note: This is illustrative; actual SQL might vary based on specific DB features
-- For simplicity, we'll construct the JSON strings directly in the UPDATE statements.

-- Function to create a basic option JSON string
-- CREATE OR REPLACE FUNCTION create_option(name TEXT, description TEXT, type TEXT, required BOOLEAN)
-- RETURNS JSONB AS $$
-- BEGIN
--     RETURN jsonb_build_object(
--         'name', name,
--         'description', description,
--         'option_type', type,
--         'required', required,
--         'default', null,
--         'enum_values', null,
--         'options', null
--     );
-- END;
-- $$ LANGUAGE plpgsql;

-- Function to create an option with choices
-- CREATE OR REPLACE FUNCTION create_choice_option(name TEXT, description TEXT, type TEXT, required BOOLEAN, choices JSONB)
-- RETURNS JSONB AS $$
-- BEGIN
--     RETURN jsonb_build_object(
--         'name', name,
--         'description', description,
--         'option_type', type,
--         'required', required,
--         'default', null,
--         'enum_values', choices,
--         'options', null
--     );
-- END;
-- $$ LANGUAGE plpgsql;

-- Function to create a subcommand option
-- CREATE OR REPLACE FUNCTION create_subcommand_option(name TEXT, description TEXT, sub_options JSONB)
-- RETURNS JSONB AS $$
-- BEGIN
--     RETURN jsonb_build_object(
--         'name', name,
--         'description', description,
--         'option_type', 'sub_command',
--         'required', false,
--         'default', null,
--         'enum_values', null,
--         'options', sub_options
--     );
-- END;
-- $$ LANGUAGE plpgsql;


-- Clear existing options first (optional, but safer)
-- UPDATE commands SET options = '{}'::jsonb;

-- === Admin Commands ===

-- rolebuttons (subcommands: add, remove)
UPDATE commands SET options = '{
  "options": [
    {
      "name": "add", "description": "Add multiple role assignment buttons to a message", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": [
        {"name": "channel", "description": "Channel where the message is located", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null},
        {"name": "message_id", "description": "ID of the message to add buttons to", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null},
        {"name": "button_config", "description": "JSON string of button configurations", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null}
      ]
    },
    {
      "name": "remove", "description": "Remove all buttons from a message", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": [
        {"name": "channel", "description": "Select the channel", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null},
        {"name": "message_id", "description": "Message ID", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null}
      ]
    }
  ]
}'::jsonb WHERE command_id = 'rolebuttons';

-- purge
UPDATE commands SET options = '{
  "options": [
    {"name": "count", "description": "Number of messages to delete", "option_type": "integer", "required": true, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'purge';

-- setdeletemessagechannel
UPDATE commands SET options = '{
  "options": [
    {"name": "channel", "description": "Channel to log deleted messages", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'setdeletemessagechannel';

-- setlevelupchannel
UPDATE commands SET options = '{
  "options": [
    {"name": "channel", "description": "Channel to send level-up messages", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'setlevelupchannel';

-- reactionslog (subcommands: setstarschannel, setreactionschannel)
UPDATE commands SET options = '{
  "options": [
    {
      "name": "setstarschannel", "description": "Set the channel for star messages", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": [
        {"name": "channel", "description": "Channel to send star messages", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null}
      ]
    },
    {
      "name": "setreactionschannel", "description": "Set the channel for general reaction logs", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": [
        {"name": "channel", "description": "Channel to send reaction logs", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null}
      ]
    }
  ]
}'::jsonb WHERE command_id = 'reactionslog';

-- seturlrule
UPDATE commands SET options = '{
  "options": [
    {"name": "channel", "description": "Channel to apply the rule", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null},
    {"name": "regex", "description": "Regex pattern to match URLs", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null},
    {"name": "output_template", "description": "Output template (use $1, $2, etc. for capture groups)", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'seturlrule';

-- setwarnchannel
UPDATE commands SET options = '{
  "options": [
    {"name": "channel", "description": "Channel to log warnings", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'setwarnchannel';

-- toggleemojireactions
UPDATE commands SET options = '{
  "options": [
    {"name": "enable", "description": "Enable or disable emoji reactions", "option_type": "boolean", "required": false, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'toggleemojireactions';

-- updateraidtime
UPDATE commands SET options = '{
  "options": [
    {"name": "month", "description": "Select the month", "option_type": "string", "required": true, "default": null, "enum_values": [
        {"value": "January", "label": "January"}, {"value": "February", "label": "February"}, {"value": "March", "label": "March"}, {"value": "April", "label": "April"},
        {"value": "May", "label": "May"}, {"value": "June", "label": "June"}, {"value": "July", "label": "July"}, {"value": "August", "label": "August"},
        {"value": "September", "label": "September"}, {"value": "October", "label": "October"}, {"value": "November", "label": "November"}, {"value": "December", "label": "December"}
    ], "options": null},
    {"name": "day", "description": "Enter the day", "option_type": "integer", "required": true, "default": null, "enum_values": null, "options": null},
    {"name": "year", "description": "Select the year", "option_type": "integer", "required": true, "default": null, "enum_values": [
        {"value": "2024", "label": "2024"}, {"value": "2025", "label": "2025"}, {"value": "2026", "label": "2026"}, {"value": "2027", "label": "2027"},
        {"value": "2028", "label": "2028"}, {"value": "2029", "label": "2029"}, {"value": "2030", "label": "2030"}
    ], "options": null},
    {"name": "time", "description": "Select the time", "option_type": "string", "required": true, "default": null, "enum_values": [
        {"value": "12:00 AM", "label": "12:00 AM"}, {"value": "1:00 AM", "label": "1:00 AM"}, {"value": "2:00 AM", "label": "2:00 AM"}, {"value": "3:00 AM", "label": "3:00 AM"},
        {"value": "4:00 AM", "label": "4:00 AM"}, {"value": "5:00 AM", "label": "5:00 AM"}, {"value": "6:00 AM", "label": "6:00 AM"}, {"value": "7:00 AM", "label": "7:00 AM"},
        {"value": "8:00 AM", "label": "8:00 AM"}, {"value": "9:00 AM", "label": "9:00 AM"}, {"value": "10:00 AM", "label": "10:00 AM"}, {"value": "11:00 AM", "label": "11:00 AM"},
        {"value": "12:00 PM", "label": "12:00 PM"}, {"value": "1:00 PM", "label": "1:00 PM"}, {"value": "2:00 PM", "label": "2:00 PM"}, {"value": "3:00 PM", "label": "3:00 PM"},
        {"value": "4:00 PM", "label": "4:00 PM"}, {"value": "5:00 PM", "label": "5:00 PM"}, {"value": "6:00 PM", "label": "6:00 PM"}, {"value": "7:00 PM", "label": "7:00 PM"},
        {"value": "8:00 PM", "label": "8:00 PM"}, {"value": "9:00 PM", "label": "9:00 PM"}, {"value": "10:00 PM", "label": "10:00 PM"}, {"value": "11:00 PM", "label": "11:00 PM"}
    ], "options": null},
    {"name": "timezone", "description": "Select the timezone", "option_type": "string", "required": true, "default": null, "enum_values": [
        {"value": "ET", "label": "Eastern Time"}, {"value": "CT", "label": "Central Time"}, {"value": "MT", "label": "Mountain Time"},
        {"value": "PT", "label": "Pacific Time"}, {"value": "AKT", "label": "Alaska Time"}, {"value": "HST", "label": "Hawaii Standard Time"}
    ], "options": null},
    {"name": "raid", "description": "Select the raid", "option_type": "string", "required": true, "default": null, "enum_values": [
        {"value": "AlexanderBurdenSavage", "label": "Alexander - The Burden of the Son (Savage)"},
        {"value": "AlexanderEyesSavage", "label": "Alexander - The Eyes of the Creator (Savage)"},
        {"value": "AlexanderBreathSavage", "label": "Alexander - The Breath of the Creator (Savage)"},
        {"value": "AlexanderHeartSavage", "label": "Alexander - The Heart of the Creator (Savage)"},
        {"value": "AlexanderSoulSavage", "label": "Alexander - The Soul of the Creator (Savage)"}
    ], "options": null},
    {"name": "channel", "description": "Select the channel", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null},
    {"name": "mine", "description": "Is this M.I.N.E. or not?", "option_type": "boolean", "required": false, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'updateraidtime';

-- warn
UPDATE commands SET options = '{
  "options": [
    {"name": "member", "description": "Member to warn", "option_type": "user", "required": true, "default": null, "enum_values": null, "options": null},
    {"name": "reason", "description": "Reason for the warning", "option_type": "string", "required": false, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'warn';

-- === Availability Commands ===

-- cancelunavailable
UPDATE commands SET options = '{
  "options": [
    {"name": "unavailability_id", "description": "ID of the unavailability to cancel", "option_type": "integer", "required": true, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'cancelunavailable';

-- setunavailabilitychannel
UPDATE commands SET options = '{
  "options": [
    {"name": "channel", "description": "Channel for unavailability posts", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'setunavailabilitychannel';

-- unavailable
UPDATE commands SET options = '{
  "options": [
    {"name": "month", "description": "Select the month", "option_type": "string", "required": true, "default": null, "enum_values": [
        {"value": "January", "label": "January"}, {"value": "February", "label": "February"}, {"value": "March", "label": "March"}, {"value": "April", "label": "April"},
        {"value": "May", "label": "May"}, {"value": "June", "label": "June"}, {"value": "July", "label": "July"}, {"value": "August", "label": "August"},
        {"value": "September", "label": "September"}, {"value": "October", "label": "October"}, {"value": "November", "label": "November"}, {"value": "December", "label": "December"}
    ], "options": null},
    {"name": "day", "description": "Enter the day (1-31)", "option_type": "integer", "required": true, "default": null, "enum_values": null, "options": null},
    {"name": "year", "description": "Select the year", "option_type": "integer", "required": true, "default": null, "enum_values": [
        {"value": "2025", "label": "2025"}, {"value": "2026", "label": "2026"}, {"value": "2027", "label": "2027"}, {"value": "2028", "label": "2028"}
    ], "options": null},
    {"name": "reason", "description": "Reason for unavailability (optional)", "option_type": "string", "required": false, "default": null, "enum_values": null, "options": null},
    {"name": "mention_role", "description": "Mention Role (optional)", "option_type": "role", "required": false, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'unavailable';

-- === Fun Commands ===

-- animehug
UPDATE commands SET options = '{
  "options": [
    {"name": "user", "description": "User to hug", "option_type": "user", "required": true, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'animehug';

-- blame
UPDATE commands SET options = '{
  "options": [
    {"name": "user", "description": "User to blame (defaults to Serena)", "option_type": "user", "required": false, "default": null, "enum_values": null, "options": null},
    {"name": "reason", "description": "Reason for blaming", "option_type": "string", "required": false, "default": null, "enum_values": [
        {"value": "TooCute", "label": "Being too cute"}, {"value": "CausingWipe", "label": "Causing a wipe"}, {"value": "BeingAfk", "label": "Being AFK"},
        {"value": "ForgettingMechanics", "label": "Forgetting mechanics"}, {"value": "JustBecause", "label": "Just because"}
    ], "options": null}
  ]
}'::jsonb WHERE command_id = 'blame';

-- createimage
UPDATE commands SET options = '{
  "options": [
    {"name": "prompt", "description": "Description of the image you want to create", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null},
    {"name": "size", "description": "Image size", "option_type": "string", "required": false, "default": null, "enum_values": [
        {"value": "Size1024", "label": "1024x1024"}, {"value": "Size1792x1024", "label": "1792x1024"}, {"value": "Size1024x1792", "label": "1024x1792"}
    ], "options": null},
    {"name": "style", "description": "Image style", "option_type": "string", "required": false, "default": null, "enum_values": [
        {"value": "Vivid", "label": "vivid"}, {"value": "Natural", "label": "natural"}
    ], "options": null},
    {"name": "quality", "description": "Image quality (HD or standard)", "option_type": "string", "required": false, "default": null, "enum_values": [
        {"value": "Standard", "label": "standard"}, {"value": "HD", "label": "hd"}
    ], "options": null}
  ]
}'::jsonb WHERE command_id = 'createimage';

-- fluximage
UPDATE commands SET options = '{
  "options": [
    {"name": "prompt", "description": "Description of the image you want to create", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null},
    {"name": "width", "description": "Image width", "option_type": "integer", "required": false, "default": null, "enum_values": [
        {"value": "256", "label": "256"}, {"value": "512", "label": "512"}, {"value": "768", "label": "768"}, {"value": "1024", "label": "1024"}, {"value": "1280", "label": "1280"}, {"value": "1440", "label": "1440"}
    ], "options": null},
    {"name": "height", "description": "Image height", "option_type": "integer", "required": false, "default": null, "enum_values": [
        {"value": "256", "label": "256"}, {"value": "512", "label": "512"}, {"value": "768", "label": "768"}, {"value": "1024", "label": "1024"}, {"value": "1280", "label": "1280"}, {"value": "1440", "label": "1440"}
    ], "options": null},
    {"name": "steps", "description": "Number of steps (1-50)", "option_type": "integer", "required": false, "default": null, "enum_values": null, "options": null},
    {"name": "prompt_upsampling", "description": "Enable prompt upsampling", "option_type": "boolean", "required": false, "default": null, "enum_values": null, "options": null},
    {"name": "seed", "description": "Seed for reproducibility", "option_type": "integer", "required": false, "default": null, "enum_values": null, "options": null},
    {"name": "guidance", "description": "Guidance scale (1.5-5.0)", "option_type": "number", "required": false, "default": null, "enum_values": null, "options": null},
    {"name": "safety_tolerance", "description": "Safety tolerance (0-6, 0 most strict, 6 least strict)", "option_type": "integer", "required": false, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'fluximage';

-- === Utility Commands ===

-- ask
UPDATE commands SET options = '{
  "options": [
    {"name": "question", "description": "Your question", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'ask';

-- help
UPDATE commands SET options = '{
  "options": [
    {"name": "command", "description": "Specific command to show help about", "option_type": "string", "required": false, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'help';

-- reminder (subcommands: create, list, delete)
UPDATE commands SET options = '{
  "options": [
    {
      "name": "create", "description": "Create a reminder", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": [
        {"name": "channel", "description": "Channel to send the reminder", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null},
        {"name": "time", "description": "Time of the reminder (HH:MM)", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null},
        {"name": "days", "description": "Days of the week (comma-separated, e.g., ''Mon,Wed,Fri'')", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null},
        {"name": "frequency", "description": "Frequency of the reminder", "option_type": "string", "required": true, "default": null, "enum_values": [
            {"value": "Daily", "label": "Daily"}, {"value": "Weekly", "label": "Weekly"}, {"value": "Monthly", "label": "Monthly"}
        ], "options": null},
        {"name": "message", "description": "Message for the reminder", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null}
      ]
    },
    {
      "name": "list", "description": "List reminders", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": []
    },
    {
      "name": "delete", "description": "Delete a reminder", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": [
        {"name": "reminder_id", "description": "ID of the reminder to delete", "option_type": "integer", "required": true, "default": null, "enum_values": null, "options": null}
      ]
    }
  ]
}'::jsonb WHERE command_id = 'reminder';

-- rule (subcommands: add, remove, list, post)
UPDATE commands SET options = '{
  "options": [
    {
      "name": "add", "description": "Add a rule", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": [
        {"name": "rule", "description": "The rule to add", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null}
      ]
    },
    {
      "name": "remove", "description": "Remove a rule", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": [
        {"name": "rule_number", "description": "The rule number to remove", "option_type": "integer", "required": true, "default": null, "enum_values": null, "options": null}
      ]
    },
    {
      "name": "list", "description": "List all rules", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": []
    },
    {
      "name": "post", "description": "Post rules to a channel", "option_type": "sub_command", "required": false, "default": null, "enum_values": null,
      "options": [
        {"name": "channel", "description": "The channel to post rules in", "option_type": "channel", "required": true, "default": null, "enum_values": null, "options": null}
      ]
    }
  ]
}'::jsonb WHERE command_id = 'rule';

-- weather
UPDATE commands SET options = '{
  "options": [
    {"name": "location", "description": "City name (and state if in the US, e.g., ''Austin, TX'')", "option_type": "string", "required": true, "default": null, "enum_values": null, "options": null}
  ]
}'::jsonb WHERE command_id = 'weather';

-- Commands without arguments (options remain '{}'):
-- dbhealth, dbschema, lifecheck, iscaliforniaonfire, ping, listunavailable, randomcapyimage, randomcatimage, whereiscaliforniaonfire
