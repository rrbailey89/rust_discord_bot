-- Add requires_admin column to commands table
ALTER TABLE commands ADD COLUMN requires_admin BOOLEAN NOT NULL DEFAULT false;

-- Update any existing commands that should require admin privileges
-- For example, setting requires_admin=true for administrative commands
UPDATE commands SET requires_admin = true WHERE category = 'admin';

-- Add an index on the requires_admin column for faster filtering
CREATE INDEX idx_commands_requires_admin ON commands(requires_admin);

-- Set a comment explaining the column purpose
COMMENT ON COLUMN commands.requires_admin IS 'Whether the command requires administrator privileges';
