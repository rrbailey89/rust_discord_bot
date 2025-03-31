-- Add scope column to commands table
ALTER TABLE commands ADD COLUMN scope TEXT NOT NULL DEFAULT 'Guild';

-- Add comment explaining the column
COMMENT ON COLUMN commands.scope IS 'Scope of the command (Global or Guild)';

-- Update existing commands based on their intended scope
-- Default is 'Guild', so we only need to explicitly set 'Global'

UPDATE commands SET scope = 'Global' WHERE command_id = 'ping';
UPDATE commands SET scope = 'Global' WHERE command_id = 'help';

-- Add a check constraint to ensure valid values (optional but recommended)
ALTER TABLE commands ADD CONSTRAINT check_command_scope CHECK (scope IN ('Global', 'Guild'));
