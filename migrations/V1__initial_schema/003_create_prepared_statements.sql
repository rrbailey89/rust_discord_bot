-- Define prepared statements for frequently accessed queries

-- Create a function to register a named prepared statement
CREATE OR REPLACE FUNCTION register_prepared_statement(statement_name text, query text) RETURNS void AS $$
BEGIN
    EXECUTE 'PREPARE ' || statement_name || ' AS ' || query;
END;
$$ LANGUAGE plpgsql;

-- Prepare statements for guild-related operations
SELECT register_prepared_statement('get_warn_channel', 
    'SELECT channel_id FROM warn_channel_ids WHERE guild_id = $1');
    
SELECT register_prepared_statement('store_warn_channel', 
    'INSERT INTO warn_channel_ids (guild_id, channel_id) 
     VALUES ($1, $2) 
     ON CONFLICT (guild_id) DO UPDATE SET channel_id = EXCLUDED.channel_id');

-- Prepare statements for user-related operations
SELECT register_prepared_statement('get_user_level', 
    'SELECT level, experience FROM user_levels WHERE guild_id = $1 AND user_id = $2');
    
SELECT register_prepared_statement('update_user_experience', 
    'UPDATE user_levels SET experience = $3 
     WHERE guild_id = $1 AND user_id = $2 
     RETURNING level, experience');

-- Prepare statements for reminders
SELECT register_prepared_statement('get_due_reminders', 
    'SELECT id, guild_id, channel_id, message, time, days, frequency, last_sent
     FROM reminders
     WHERE $1 = ANY(days)
     AND $2::time >= time
     AND (
         last_sent IS NULL
         OR (
             CASE
                 WHEN frequency = ''Daily'' THEN
                     $3::date > last_sent::date
                 WHEN frequency = ''Weekly'' THEN
                     $3::date >= last_sent::date + INTERVAL ''7 days''
                 WHEN frequency = ''Monthly'' THEN
                     ($3::date >= last_sent::date + INTERVAL ''1 month'')
                     AND (EXTRACT(DAY FROM $3::date) = EXTRACT(DAY FROM last_sent::date))
             END
             AND $2::time >= time
         )
     )');

-- Prepare statements for unavailability
SELECT register_prepared_statement('get_user_unavailability', 
    'SELECT id, unavailable_date, reason 
     FROM user_unavailability 
     WHERE guild_id = $1 AND user_id = $2
     AND unavailable_date >= CURRENT_DATE
     ORDER BY unavailable_date ASC');
