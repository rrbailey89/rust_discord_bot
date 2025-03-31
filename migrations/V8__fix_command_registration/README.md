# Command Registration Fix (V8 Migration)

This migration fixes issues with command registration in the database, addressing the discrepancy between command IDs used in code and those stored in the database.

## Problem

We identified several issues in the command registration system:

1. **Command ID mismatch**: The code uses command IDs like `blame` while the database has `blame_serena`
2. **Missing commands**: Many commands defined in `src/commands.rs` were missing from the database
3. **Inconsistent default settings**: Guild commands need to be disabled by default

## Migration Files

### 000_ensure_all_commands_registered.sql

This file:
- Creates a helper function to safely insert or update commands
- Explicitly adds all commands from `src/commands.rs` with proper settings
- Handles both code-style IDs (`blame`) and database-style IDs (`blame_serena`)
- Sets all guild commands to `default_enabled = FALSE`
- Ensures global commands (`ping`, `help`) remain `default_enabled = TRUE`

### 001_migration_diagnostic.sql

This migration:
- Creates a `schema_log` table for persistent diagnostic information
- Logs detailed information about the state of commands after migration
- Checks schema_migrations counts for V7 and V8
- Creates a complete inventory of commands for troubleshooting

## Troubleshooting

If the migrations don't seem to be running:

1. Check the logs for any errors during migration
2. Verify that the schema version in `schema_migrations` is being incremented
3. Query the `schema_log` table for diagnostic information:
   ```sql
   SELECT * FROM schema_log ORDER BY applied_at DESC;
   ```
4. Check that commands are properly registered:
   ```sql
   SELECT command_id, category, default_enabled FROM commands ORDER BY category, command_id;
   ```

## Expected Results

After this migration:
- All commands from `src/commands.rs` will exist in the database
- All guild commands will be set to `default_enabled = FALSE`
- Only global commands (`ping`, `help`) will be set to `default_enabled = TRUE`
- Existing guild settings will remain unchanged
