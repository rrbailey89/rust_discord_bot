-- Add indexes for frequently queried tables

-- Indexing basic lookup tables
CREATE INDEX IF NOT EXISTS idx_warn_channel_ids_guild_id ON warn_channel_ids(guild_id);
CREATE INDEX IF NOT EXISTS idx_user_hug_counts_user_id ON user_hug_counts(user_id);
CREATE INDEX IF NOT EXISTS idx_guild_info_guild_id ON guild_info(guild_id);
CREATE INDEX IF NOT EXISTS idx_guild_channels_guild_id ON guild_channels(guild_id);
CREATE INDEX IF NOT EXISTS idx_message_delete_channels_guild_id ON message_delete_channels(guild_id);

-- Indexing unavailability queries which filter by date
CREATE INDEX IF NOT EXISTS idx_user_unavailability_guild_id_user_id ON user_unavailability(guild_id, user_id);
CREATE INDEX IF NOT EXISTS idx_user_unavailability_date ON user_unavailability(unavailable_date);

-- Indexing for reminders which are queried by day and time
CREATE INDEX IF NOT EXISTS idx_reminders_guild_id ON reminders(guild_id);
CREATE INDEX IF NOT EXISTS idx_reminders_days ON reminders USING GIN(days);
CREATE INDEX IF NOT EXISTS idx_reminders_time ON reminders(time);

-- Indexing user levels for faster lookups
CREATE INDEX IF NOT EXISTS idx_user_levels_guild_id_user_id ON user_levels(guild_id, user_id);
