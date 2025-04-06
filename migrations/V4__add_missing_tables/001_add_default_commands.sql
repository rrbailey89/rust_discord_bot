-- Add default commands to the commands table
-- This ensures the web interface knows about available commands

-- Only insert if not already present
INSERT INTO commands (command_id, name, description, category, default_enabled, options, created_at)
VALUES 
    ('ping', 'Ping', 'Check if the bot is alive', 'utility', TRUE, '{}', NOW()),
    ('help', 'Help', 'Show help information about available commands', 'utility', TRUE, '{}', NOW()),
    ('blame_serena', 'Blame Serena', 'Blame Serena for something', 'fun', TRUE, '{}', NOW()),
    ('warn', 'Warn', 'Warn a user for breaking rules', 'admin', TRUE, '{}', NOW()),
    ('rules', 'Rules', 'Display server rules', 'utility', TRUE, '{}', NOW()),
    ('weather', 'Weather', 'Get weather information for a location', 'utility', TRUE, '{}', NOW()),
    ('remind', 'Remind', 'Set a reminder', 'utility', TRUE, '{}', NOW()),
    ('unavailable', 'Unavailable', 'Mark yourself as unavailable', 'availability', TRUE, '{}', NOW()),
    ('list_unavailable', 'List Unavailable', 'List all unavailable users', 'availability', TRUE, '{}', NOW()),
    ('anime_hug', 'Anime Hug', 'Send a virtual anime hug', 'fun', TRUE, '{}', NOW()),
    ('createimage', 'Create Image', 'Generate an image', 'fun', TRUE, '{}', NOW())
ON CONFLICT (command_id) DO NOTHING;
