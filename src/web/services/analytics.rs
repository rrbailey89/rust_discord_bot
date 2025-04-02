// src/web/services/analytics.rs
//! Analytics service for database operations related to analytics

use std::collections::HashMap;
use crate::error::Error;
use crate::services::database::DatabaseService;
use crate::web::models::analytics::{
    AnalyticsEvent, GuildAnalyticsSummary, CommandUsage, TimeSeriesDataPoint, UserActivityDataPoint,
    UserActivitySummary, GuildActivity, LogEventRequest, AnalyticsQueryParams
};
use chrono::{DateTime, Utc, Duration, NaiveDate};
use tracing::{info, error}; // Added error
use serde_json::Value;
use tokio_postgres::types::ToSql; // Added ToSql

/// Analytics service for database operations
pub struct AnalyticsService {
    /// Database service
    db: DatabaseService,
}

impl AnalyticsService {
    /// Create a new analytics service
    pub fn new(db: DatabaseService) -> Self {
        Self { db }
    }

    /// Log a new analytics event
    pub async fn log_event(&self, request: &LogEventRequest) -> Result<i32, Error> {
        let client = self.db.get_client().await?;

        // Convert event_data to JSON if present
        let event_data = match &request.event_data {
            Some(data) => {
                let json = serde_json::to_value(data)
                    .map_err(|e| Error::Unknown(format!("Failed to serialize event data: {}", e)))?;
                Some(json)
            },
            None => None,
        };

        // Insert the new event
        let row = client
            .query_one(
                "INSERT INTO analytics_events
                (event_type, user_id, guild_id, event_data, timestamp)
                VALUES ($1, $2, $3, $4, NOW())
                RETURNING id",
                &[&request.event_type, &request.user_id, &request.guild_id, &event_data],
            )
            .await?;

        let event_id: i32 = row.get(0);
        info!("Logged analytics event: {}", event_id);
        
        Ok(event_id)
    }

    /// Get analytics events matching the query parameters
    pub async fn get_events(&self, params: &AnalyticsQueryParams) -> Result<Vec<AnalyticsEvent>, Error> {
        let client = self.db.get_client().await?;

        // Build the query based on parameters
        let mut query = String::from(
            "SELECT id, event_type, user_id, guild_id, event_data, timestamp
             FROM analytics_events
             WHERE 1=1"
        );
        
        let mut args = Vec::new();
        let mut arg_index = 1;
        
        // Add filters based on parameters
        if let Some(start_date) = params.start_date {
            query.push_str(&format!(" AND timestamp >= ${}", arg_index));
            args.push(Box::new(start_date) as Box<dyn tokio_postgres::types::ToSql + Sync>);
            arg_index += 1;
        }
        
        if let Some(end_date) = params.end_date {
            query.push_str(&format!(" AND timestamp <= ${}", arg_index));
            args.push(Box::new(end_date) as Box<dyn tokio_postgres::types::ToSql + Sync>);
            arg_index += 1;
        }
        
        if let Some(guild_id) = params.guild_id {
            query.push_str(&format!(" AND guild_id = ${}", arg_index));
            args.push(Box::new(guild_id) as Box<dyn tokio_postgres::types::ToSql + Sync>);
            arg_index += 1;
        }
        
        if let Some(user_id) = params.user_id {
            query.push_str(&format!(" AND user_id = ${}", arg_index));
            args.push(Box::new(user_id) as Box<dyn tokio_postgres::types::ToSql + Sync>);
            arg_index += 1;
        }
        
        if let Some(event_type) = &params.event_type {
            query.push_str(&format!(" AND event_type = ${}", arg_index));
            args.push(Box::new(event_type.clone()) as Box<dyn tokio_postgres::types::ToSql + Sync>);
            arg_index += 1;
        }
        
        // Add order and limit
        query.push_str(" ORDER BY timestamp DESC");
        
        if let Some(limit) = params.limit {
            query.push_str(&format!(" LIMIT ${}", arg_index));
            args.push(Box::new(limit) as Box<dyn tokio_postgres::types::ToSql + Sync>);
        }
        
        // Execute the query
        let rows = client
            .query(
                &query,
                &args.iter().map(|arg| arg.as_ref()).collect::<Vec<&(dyn tokio_postgres::types::ToSql + Sync)>>(),
            )
            .await?;

        // Map rows to AnalyticsEvent objects
        let events = rows
            .iter()
            .map(|row| AnalyticsEvent {
                id: row.get(0),
                event_type: row.get(1),
                user_id: row.get(2),
                guild_id: row.get(3),
                event_data: row.get::<_, Option<Value>>(4).map(|v| {
                    serde_json::from_value(v).unwrap_or_else(|_| HashMap::new())
                }),
                timestamp: row.get(5),
            })
            .collect();

        Ok(events)
    }

    /// Get analytics summary for a specific guild or all guilds
    pub async fn get_guild_summary(
        &self,
        guild_id_opt: Option<i64>, // Changed to Option<i64>
        period: &str,
    ) -> Result<GuildAnalyticsSummary, Error> {
        let client = self.db.get_client().await?;

        // Calculate date range based on period
        let now = Utc::now();
        let start_date = match period {
            "day" => now - Duration::days(1),
            "week" => now - Duration::weeks(1),
            "month" => now - Duration::days(30),
            "month" => now - Duration::days(30), // Approx month
            "90d" => now - Duration::days(90), // Added 90d
            _ => now - Duration::weeks(1), // Default to week if invalid
        };

        // Build base query and arguments, handling optional guild_id
        let mut base_where_clause = String::from("timestamp >= $1");
        let mut query_args: Vec<&(dyn ToSql + Sync)> = vec![&start_date];

        if let Some(guild_id) = &guild_id_opt {
            base_where_clause.push_str(" AND guild_id = $2");
            query_args.push(guild_id);
        }

        // --- Aggregate Stats ---

        // Get active users count
        let active_users_query = format!(
            "SELECT COUNT(DISTINCT user_id) FROM analytics_events WHERE {} AND user_id IS NOT NULL",
            base_where_clause
        );
        let active_users_row = client.query_one(&active_users_query, &query_args[..]).await?;
        let active_users: i64 = active_users_row.get(0); // Use i64 from DB

        // Get command usage count
        let commands_query = format!(
            "SELECT COUNT(*) FROM analytics_events WHERE {} AND event_type = 'command_used'",
            base_where_clause
        );
        let commands_row = client.query_one(&commands_query, &query_args[..]).await?;
        let commands_used: i64 = commands_row.get(0); // Use i64 from DB

        // Get message count
        let messages_query = format!(
            "SELECT COUNT(*) FROM analytics_events WHERE {} AND event_type = 'message_sent'",
            base_where_clause
        );
        let messages_row = client.query_one(&messages_query, &query_args[..]).await?;
        let message_count: i64 = messages_row.get(0); // Use i64 from DB

        // Get top commands
        let top_commands_query = format!(
            "SELECT
                e.event_data->>'command_id' as command_id,
                e.event_data->>'command_name' as command_name,
                COUNT(*) as count
             FROM analytics_events e
             WHERE {} AND event_type = 'command_used'
             AND e.event_data->>'command_id' IS NOT NULL
             GROUP BY e.event_data->>'command_id', e.event_data->>'command_name'
             ORDER BY count DESC
             LIMIT 5",
            base_where_clause
        );
        let top_commands_rows = client.query(&top_commands_query, &query_args[..]).await?;
        let top_commands: Vec<CommandUsage> = top_commands_rows
            .iter()
            .map(|row| {
                let count_i64: i64 = row.get(2); // Get count as i64
                CommandUsage {
                    command_id: row.get(0),
                    command_name: row.get(1),
                    count: count_i64 as i32, // Convert to i32 for the struct
                }
            })
            .collect();

        // Get event counts by type
        let event_types_query = format!(
            "SELECT
                event_type,
                COUNT(*) as count
             FROM analytics_events
             WHERE {}
             GROUP BY event_type
             ORDER BY count DESC",
            base_where_clause
        );
        let event_types_rows = client.query(&event_types_query, &query_args[..]).await?;
        let mut events_by_type = HashMap::new();
        for row in event_types_rows {
            let count_i64: i64 = row.get(1); // Get count as i64
            events_by_type.insert(row.get::<_, String>(0), count_i64 as i32); // Convert to i32
        }

        // --- Time Series Data ---

        // Get command usage over time (daily)
        let command_usage_daily_query = format!(
            "SELECT
                DATE(timestamp) as date,
                COUNT(*) as count
             FROM analytics_events
             WHERE {} AND event_type = 'command_used'
             GROUP BY DATE(timestamp)
             ORDER BY date ASC",
            base_where_clause
        );
        let command_usage_daily_rows = client.query(&command_usage_daily_query, &query_args[..]).await?;
        let command_usage_over_time: Vec<TimeSeriesDataPoint> = command_usage_daily_rows
            .iter()
            .map(|row| {
                let count_i64: i64 = row.get(1);
                TimeSeriesDataPoint {
                    date: row.get(0),
                    value: count_i64 as i32,
                }
            })
            .collect();

        // Get user activity over time (daily)
        let user_activity_daily_query = format!(
            "SELECT
                DATE(timestamp) as date,
                COUNT(CASE WHEN event_type = 'message_sent' THEN 1 END) as messages,
                COUNT(CASE WHEN event_type = 'command_used' THEN 1 END) as commands
             FROM analytics_events
             WHERE {} AND (event_type = 'message_sent' OR event_type = 'command_used')
             GROUP BY DATE(timestamp)
             ORDER BY date ASC",
            base_where_clause
        );
        let user_activity_daily_rows = client.query(&user_activity_daily_query, &query_args[..]).await?;
        let user_activity_over_time: Vec<UserActivityDataPoint> = user_activity_daily_rows
            .iter()
            .map(|row| {
                let messages_i64: i64 = row.get(1);
                let commands_i64: i64 = row.get(2);
                UserActivityDataPoint {
                    date: row.get(0),
                    messages: messages_i64 as i32,
                    commands: commands_i64 as i32,
                }
            })
            .collect();


        // Build and return the summary
        Ok(GuildAnalyticsSummary {
            // Use guild_id_opt.unwrap_or(0) for the ID field, or adjust model if ID should be optional
            guild_id: guild_id_opt.unwrap_or(0), // Use 0 or another indicator for "All Guilds"
            period: period.to_string(),
            active_users: active_users as i32, // Convert final aggregates to i32
            commands_used: commands_used as i32,
            message_count: message_count as i32,
            top_commands,
            events_by_type,
            command_usage_over_time, // Add new field
            user_activity_over_time, // Add new field
        })
    }


    /// Get activity summary for a user
    pub async fn get_user_summary(
        &self,
        user_id: i64,
        period: &str,
    ) -> Result<UserActivitySummary, Error> {
        let client = self.db.get_client().await?;

        // Calculate date range based on period
        let now = Utc::now();
        let start_date = match period {
            "day" => now - Duration::days(1),
            "week" => now - Duration::weeks(1),
            "month" => now - Duration::days(30),
            _ => return Err(Error::Unknown(format!("Invalid period: {}", period))),
        };

        // Get user info
        let user_info_row = client
            .query_opt(
                "SELECT username FROM users WHERE id = $1",
                &[&user_id],
            )
            .await?;
        
        let username = match user_info_row {
            Some(row) => row.get::<_, String>(0),
            None => "Unknown User".to_string(),
        };

        // Get command usage
        let commands_rows = client
            .query(
                "SELECT 
                    e.event_data->>'command_id' as command_id,
                    e.event_data->>'command_name' as command_name,
                    COUNT(*) as count
                 FROM analytics_events e
                 WHERE user_id = $1 AND timestamp >= $2 AND event_type = 'command_used'
                 AND e.event_data->>'command_id' IS NOT NULL
                 GROUP BY e.event_data->>'command_id', e.event_data->>'command_name'
                 ORDER BY count DESC",
                &[&user_id, &start_date],
            )
            .await?;
        
        let commands_used: Vec<CommandUsage> = commands_rows
            .iter()
            .map(|row| CommandUsage {
                command_id: row.get(0),
                command_name: row.get(1),
                count: row.get(2),
            })
            .collect();

        // Get guild activity
        let guilds_rows = client
            .query(
                "SELECT 
                    e.guild_id,
                    g.name as guild_name,
                    COUNT(CASE WHEN e.event_type = 'message_sent' THEN 1 END) as message_count,
                    COUNT(CASE WHEN e.event_type = 'command_used' THEN 1 END) as commands_used
                 FROM analytics_events e
                 JOIN guilds g ON e.guild_id = g.id
                 WHERE e.user_id = $1 AND e.timestamp >= $2 AND e.guild_id IS NOT NULL
                 GROUP BY e.guild_id, g.name
                 ORDER BY message_count + commands_used DESC",
                &[&user_id, &start_date],
            )
            .await?;
        
        let active_guilds: Vec<GuildActivity> = guilds_rows
            .iter()
            .map(|row| GuildActivity {
                guild_id: row.get(0),
                guild_name: row.get(1),
                message_count: row.get(2),
                commands_used: row.get(3),
            })
            .collect();

        // Get message count
        let messages_row = client
            .query_one(
                "SELECT COUNT(*) 
                 FROM analytics_events 
                 WHERE user_id = $1 AND timestamp >= $2 AND event_type = 'message_sent'",
                &[&user_id, &start_date],
            )
            .await?;
        
        let message_count: i32 = messages_row.get(0);

        // Get first and last activity
        let activity_row = client
            .query_one(
                "SELECT 
                    MIN(timestamp) as first_activity,
                    MAX(timestamp) as last_activity
                 FROM analytics_events 
                 WHERE user_id = $1 AND timestamp >= $2",
                &[&user_id, &start_date],
            )
            .await?;
        
        let first_activity: DateTime<Utc> = activity_row.get(0);
        let last_activity: DateTime<Utc> = activity_row.get(1);

        // Build and return the summary
        Ok(UserActivitySummary {
            user_id,
            username,
            period: period.to_string(),
            commands_used,
            active_guilds,
            message_count,
            first_activity,
            last_activity,
        })
    }
}
