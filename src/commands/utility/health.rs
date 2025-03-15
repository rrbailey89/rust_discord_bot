use crate::Data;
use crate::error::Error;
use poise::serenity_prelude as serenity;
use std::time::{Duration, Instant};

/// Show the database connection pool health status
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    category = "Utility"
)]
pub async fn dbhealth(
    ctx: poise::Context<'_, Data, Error>,
) -> Result<(), Error> {
    // Perform a health check
    let status = ctx.data().database.check_health().await?;
    
    // Create a nice formatted response for the admin
    let response = format!(
        "## Database Connection Pool Status\n\
        - **Available Connections**: {}/{}\n\
        - **Tasks Waiting**: {}\n\
        - **Status**: {}",
        status.available,
        status.max_size,
        status.waiting,
        if status.available > 0 { "✅ Healthy" } else { "❌ No Available Connections" }
    );
    
    // Send the response
    ctx.say(response).await?;
    
    Ok(())
}

/// Show database schema information
#[poise::command(
    slash_command,
    guild_only,
    required_permissions = "ADMINISTRATOR",
    category = "Utility"
)]
pub async fn dbschema(
    ctx: poise::Context<'_, Data, Error>,
) -> Result<(), Error> {
    // Defer the response while we fetch information
    ctx.defer().await?;
    
    // Get the current schema version
    let version = match ctx.data().database.get_schema_version().await {
        Ok(Some(v)) => v.to_string(),
        Ok(None) => "Unknown".to_string(),
        Err(_) => "Error fetching version".to_string(),
    };
    
    // Measure query performance
    let start = Instant::now();
    let _result = ctx.data().database.get_due_reminders_with_timeout(Some(1000)).await;
    let query_time = start.elapsed();
    
    // Create a response with schema information
    let response = format!(
        "## Database Schema Information\n\
        - **Schema Version**: {}\n\
        - **Query Performance**: {}ms\n\
        - **Connection Pool Status**: {}",
        version,
        query_time.as_millis(),
        if query_time < Duration::from_millis(500) { "✅ Good" } else { "⚠️ Slow" }
    );
    
    // Send the response
    ctx.say(response).await?;
    
    Ok(())
}
