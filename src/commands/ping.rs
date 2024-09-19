use crate::{error::Error, Data};
use poise::{serenity_prelude::CreateEmbed, CreateReply};
use psutil::process::Process;
use crate::types::ShardManagerContainer;

/// Ping command to measure bot latency and other metrics.
#[poise::command(slash_command)]
pub async fn ping(ctx: poise::Context<'_, Data, Error>) -> Result<(), Error> {
    let cache = ctx.serenity_context().cache.clone();
    let guild_count = cache.guilds().len();

    let memory_usage = {
        let process = Process::current().map_err(|e| Error::Unknown(format!("Failed to get current process: {}", e)))?;
        let memory_info = process.memory_info().map_err(|e| Error::Unknown(format!("Failed to get memory info: {}", e)))?;
        memory_info.rss() / 1024 / 1024
    };

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

    let embed = CreateEmbed::default()
        .title("Pong! 🏓")
        .field("Guilds", guild_count.to_string(), true)
        .field("Gateway Latency", format!("{}ms", gateway_latency), true)
        .field("Memory Usage", format!("{}MB", memory_usage), true)
        .color(0x00FF00);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}