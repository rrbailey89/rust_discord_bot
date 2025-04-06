-- Add discord_name column to commands table
ALTER TABLE commands ADD COLUMN discord_name VARCHAR(32);

-- Update existing commands with valid Discord names
UPDATE commands 
SET discord_name = 
    CASE 
        WHEN length(regexp_replace(lower(name), '[^a-z0-9_]', '', 'g')) > 0 
        THEN substr(regexp_replace(lower(name), '[^a-z0-9_]', '', 'g'), 1, 32)
        ELSE 'cmd_' || command_id
    END;
