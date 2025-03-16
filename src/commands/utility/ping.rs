use crate::{error::Error, Data, utils::get_memory_usage};
use poise::{serenity_prelude::CreateEmbed, CreateReply};
use crate::types::ShardManagerContainer;
use std::time::Duration;

/// Ping command to measure bot latency and other metrics.
#[poise::command(slash_command)]
pub async fn ping(ctx: poise::Context<'_, Data, Error>) -> Result<(), Error> {
    let cache = ctx.serenity_context().cache.clone();
    let guild_count = cache.guilds().len();

    let shard_manager = {
        let data = ctx.serenity_context().data.read().await;
        data.get::<ShardManagerContainer>()
            .cloned()
            .ok_or_else(|| Error::Unknown("Failed to retrieve Shard Manager.".into()))?
    };

    let shard_id = ctx.serenity_context().shard_id;

    let gateway_latency = {
        let runners = shard_manager.runners.lock().await;
        runners.get(&shard_id)
            .and_then(|runner_info| runner_info.latency)
            .map(|latency| latency.as_millis())
            .unwrap_or(0)
    };

    let uptime = ctx.data().start_time.elapsed();
    let uptime_str = format_duration(uptime);

    // Get application's memory usage in MB
    let memory_mb = get_memory_usage();
    let memory_usage = format!("{}MB", memory_mb);

    let embed = CreateEmbed::default()
        .title("Pong! 🏓")
        .field("Guilds", guild_count.to_string(), true)
        .field("Gateway Latency", format!("{}ms", gateway_latency), true)
        .field("Uptime", uptime_str, true)
        .field("Memory Usage", memory_usage, true)
        .color(0x00FF00);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m {}s", days, hours, minutes, seconds)
    } else if hours > 0 {
        format!("{}h {}m {}s", hours, minutes, seconds)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, seconds)
    } else {
        format!("{}s", seconds)
    }
}
