// src/web/services/analytics.rs
//! Analytics service for database operations related to analytics

use std::collections::HashMap;
use crate::error::Error;
use crate::services::database::DatabaseService;
use crate::web::models::analytics::{
    AnalyticsEvent, GuildAnalyticsSummary, CommandUsage,
    UserActivitySummary, GuildActivity, LogEventRequest,
    AnalyticsQueryParams
};
use chrono::{DateTime, Utc, Duration};
use tracing::info;
use serde_json::Value;

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

    /// Get analytics summary for a guild
    pub async fn get_guild_summary(
        &self,
        guild_id: i64,
        period: &str,
    ) -> Result<GuildAnalyticsSummary, Error> {
        let client = self.db.get_client().await?;

        // Calculate date range based on period
        let now = Utc::now();
        let start_date = match period {
            "day" => now - Duration::days(1),
            "week" => now - Duration::weeks(1),
            "month" => now - Duration::days(30),
            _ => return Err(Error::Unknown(format!("Invalid period: {}", period))),
        };

        // Get active users count
        let active_users_row = client
            .query_one(
                "SELECT COUNT(DISTINCT user_id) 
                 FROM analytics_events 
                 WHERE guild_id = $1 AND timestamp >= $2 AND user_id IS NOT NULL",
                &[&guild_id, &start_date],
            )
            .await?;
        
        let active_users: i32 = active_users_row.get(0);

        // Get command usage count
        let commands_row = client
            .query_one(
                "SELECT COUNT(*) 
                 FROM analytics_events 
                 WHERE guild_id = $1 AND timestamp >= $2 AND event_type = 'command_used'",
                &[&guild_id, &start_date],
            )
            .await?;
        
        let commands_used: i32 = commands_row.get(0);

        // Get message count
        let messages_row = client
            .query_one(
                "SELECT COUNT(*) 
                 FROM analytics_events 
                 WHERE guild_id = $1 AND timestamp >= $2 AND event_type = 'message_sent'",
                &[&guild_id, &start_date],
            )
            .await?;
        
        let message_count: i32 = messages_row.get(0);

        // Get top commands
        let top_commands_rows = client
            .query(
                "SELECT 
                    e.event_data->>'command_id' as command_id,
                    e.event_data->>'command_name' as command_name,
                    COUNT(*) as count
                 FROM analytics_events e
                 WHERE guild_id = $1 AND timestamp >= $2 AND event_type = 'command_used'
                 AND e.event_data->>'command_id' IS NOT NULL
                 GROUP BY e.event_data->>'command_id', e.event_data->>'command_name'
                 ORDER BY count DESC
                 LIMIT 5",
                &[&guild_id, &start_date],
            )
            .await?;
        
        let top_commands: Vec<CommandUsage> = top_commands_rows
            .iter()
            .map(|row| CommandUsage {
                command_id: row.get(0),
                command_name: row.get(1),
                count: row.get(2),
            })
            .collect();

        // Get event counts by type
        let event_types_rows = client
            .query(
                "SELECT 
                    event_type,
                    COUNT(*) as count
                 FROM analytics_events
                 WHERE guild_id = $1 AND timestamp >= $2
                 GROUP BY event_type
                 ORDER BY count DESC",
                &[&guild_id, &start_date],
            )
            .await?;
        
        let mut events_by_type = HashMap::new();
        for row in event_types_rows {
            events_by_type.insert(row.get::<_, String>(0), row.get::<_, i64>(1) as i32);
        }

        // Build and return the summary
        Ok(GuildAnalyticsSummary {
            guild_id,
            period: period.to_string(),
            active_users,
            commands_used,
            message_count,
            top_commands,
            events_by_type,
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
