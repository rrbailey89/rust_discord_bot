-- V8: Migration diagnostic to help troubleshoot migration issues
-- This script logs information about the current state of command registrations

-- Create a schema_log table if it doesn't exist - for persistent diagnostic information
CREATE TABLE IF NOT EXISTS schema_log (
    id SERIAL PRIMARY KEY,
    version TEXT NOT NULL,
    applied_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    migration_name TEXT NOT NULL,
    message TEXT NOT NULL
);

-- Log information about the current command state
INSERT INTO schema_log (version, migration_name, message)
VALUES (
    'V8', 
    'migration_diagnostic',
    format(
        'Applied V8 migration. Command counts: Total=%s, Admin=%s, Fun=%s, Utility=%s, Availability=%s, Default Enabled=%s',
        (SELECT COUNT(*) FROM commands),
        (SELECT COUNT(*) FROM commands WHERE category = 'admin'),
        (SELECT COUNT(*) FROM commands WHERE category = 'fun'),
        (SELECT COUNT(*) FROM commands WHERE category = 'utility'),
        (SELECT COUNT(*) FROM commands WHERE category = 'availability'),
        (SELECT COUNT(*) FROM commands WHERE default_enabled = TRUE)
    )
);

-- Create additional validation checks for the schema_migrations table
DO $$
DECLARE
    v7_migrations_count INT;
    v8_migrations_count INT;
BEGIN
    -- Check if schema_migrations exists
    BEGIN
        SELECT COUNT(*) 
        INTO v7_migrations_count
        FROM schema_migrations 
        WHERE version = 7;
        
        SELECT COUNT(*) 
        INTO v8_migrations_count
        FROM schema_migrations 
        WHERE version = 8;
        
        -- Log information about schema_migrations table
        INSERT INTO schema_log (version, migration_name, message)
        VALUES (
            'V8', 
            'migration_diagnostic', 
            format('Found schema_migrations table: V7 migration count=%s, V8 migration count=%s', 
                   v7_migrations_count, v8_migrations_count)
        );
    EXCEPTION WHEN OTHERS THEN
        -- Log that schema_migrations table doesn't exist or other error
        INSERT INTO schema_log (version, migration_name, message)
        VALUES ('V8', 'migration_diagnostic', 'Error querying schema_migrations table: ' || SQLERRM);
    END;

    -- Log all existing commands for debugging
    BEGIN
        INSERT INTO schema_log (version, migration_name, message)
        SELECT 
            'V8',
            'command_inventory',
            format('Command: %s (category=%s, default_enabled=%s)', command_id, category, default_enabled)
        FROM commands
        ORDER BY category, command_id;
    EXCEPTION WHEN OTHERS THEN
        INSERT INTO schema_log (version, migration_name, message)
        VALUES ('V8', 'migration_diagnostic', 'Error logging command inventory: ' || SQLERRM);
    END;
END $$;

-- Log that the migration completed successfully
DO $$
BEGIN
    RAISE NOTICE 'V8 migration diagnostic complete. Check schema_log table for details.';
END $$;
