-- This script is for querying only, not for actual migration
-- It helps us understand what commands currently exist in the database

SELECT command_id, name, category, default_enabled 
FROM commands 
ORDER BY category, name;
