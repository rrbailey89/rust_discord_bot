-- Add specific columns to guild_settings for channel-related settings
-- This avoids misusing the guild_channels table

ALTER TABLE guild_settings
ADD COLUMN IF NOT EXISTS level_up_channel_id BIGINT,
ADD COLUMN IF NOT EXISTS warn_channel_id BIGINT,
ADD COLUMN IF NOT EXISTS delete_log_channel_id BIGINT,
ADD COLUMN IF NOT EXISTS reaction_log_channel_id BIGINT;

-- Note: We are not adding constraints here, just the columns.
-- Existing data in these columns (if any from previous attempts) will be preserved.
-- The application logic will handle updating these specific columns.
