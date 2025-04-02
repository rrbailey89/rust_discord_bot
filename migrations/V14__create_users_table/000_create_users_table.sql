-- migrations/V14__create_users_table/000_create_users_table.sql
-- Create the users table if it doesn't exist, needed for storing user profile info synced from Discord

-- Attempt to create the table with the ideal schema if it doesn't exist.
-- This defines the primary key and NOT NULL constraints for a new table.
CREATE TABLE IF NOT EXISTS users (
    user_id BIGINT PRIMARY KEY,
    username TEXT NOT NULL,
    discriminator TEXT, -- Keep for now, Discord is phasing out but code uses it
    avatar TEXT,
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Ensure the core columns exist, even if the table was created manually before
-- or has a slightly different schema. This won't add constraints like NOT NULL
-- or PRIMARY KEY if the table already exists.
ALTER TABLE users ADD COLUMN IF NOT EXISTS user_id BIGINT; -- Add user_id if missing (though unlikely if PK was set)
ALTER TABLE users ADD COLUMN IF NOT EXISTS username TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS discriminator TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS avatar TEXT;
ALTER TABLE users ADD COLUMN IF NOT EXISTS last_updated TIMESTAMP WITH TIME ZONE;

-- Note: This migration prioritizes ensuring the table and columns exist
-- for the application code to reference. It does not attempt to fully reconcile
-- potentially different schemas between environments if the table pre-existed.
-- If 'username' or 'last_updated' columns are added to a table with existing rows,
-- they might initially be NULL, which could cause issues later if the code expects NOT NULL.
-- However, the INSERT/UPDATE logic in store_guild_members should populate them.
