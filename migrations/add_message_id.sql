-- Add message_id column to user_unavailability table to store Discord message IDs
ALTER TABLE user_unavailability 
ADD COLUMN message_id BIGINT;
