-- migrations/V13__migrate_emoji_settings/000_migrate_emoji_settings.sql

-- Ensure the settings column exists and has a default value if needed
-- (Assuming the table and column already exist based on provided info)
-- ALTER TABLE guild_settings ALTER COLUMN settings SET DEFAULT '{}'::jsonb;

-- Update existing settings JSONB, setting a default if NULL
UPDATE guild_settings
SET settings = COALESCE(settings, '{}'::jsonb);

-- Migrate data from guild_emoji_settings to guild_settings.settings JSONB
-- This uses jsonb_set to add or update the key within the JSONB column.
-- It handles cases where the guild might not exist in guild_settings yet
-- by using INSERT ... ON CONFLICT.
INSERT INTO guild_settings (guild_id, settings)
SELECT
    ges.guild_id,
    jsonb_set('{}'::jsonb, '{emoji_reactions_enabled}', to_jsonb(ges.emoji_reactions_enabled))
FROM
    guild_emoji_settings ges
ON CONFLICT (guild_id) DO UPDATE
SET
    -- Merge the existing settings JSONB with the new key from the EXCLUDED row.
    -- EXCLUDED.settings already contains '{"emoji_reactions_enabled": true/false}'
    settings = COALESCE(guild_settings.settings, '{}'::jsonb) || EXCLUDED.settings;

-- Optional: Drop the old table after successful migration
DROP TABLE IF EXISTS guild_emoji_settings;

-- Add a comment indicating the migration purpose
COMMENT ON TABLE guild_settings IS 'Migrated emoji_reactions_enabled setting into settings JSONB column from legacy guild_emoji_settings table.';
