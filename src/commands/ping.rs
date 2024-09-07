use crate::{error::Error, Data};
use poise::{CreateReply, serenity_prelude::CreateEmbed};
use std::time::Instant;
use psutil::process::Process;

#[poise::command(slash_command)]
pub async fn ping(ctx: poise::Context<'_, Data, Error>) -> Result<(), Error> {
    let start_time = Instant::now();

    // Get the number of guilds
    let guild_count = ctx.cache().guilds().len();

    // Measure API latency
    let api_latency = {
        let message = ctx.say("Measuring latency...").await?;
        let latency = start_time.elapsed().as_millis();
        message.edit(ctx, CreateReply::default().content("Pong!")).await?;
        latency
    };

    // Get application memory usage
    let memory_usage = {
        let process = Process::current().map_err(|e| Error::Unknown(format!("Failed to get current process: {}", e)))?;
        let memory_info = process.memory_info().map_err(|e| Error::Unknown(format!("Failed to get memory info: {}", e)))?;
        memory_info.rss() / 1024 / 1024 // Convert to MB
    };

    // Get cache size
    let cache_size = {
        let cache = ctx.cache();
        let guilds = cache.guilds().len();
        let channels = cache.guild_channel_count();
        let users = cache.users().len();
        guilds + channels + users
    };

    // Create and send embed
    let embed = CreateEmbed::default()
        .title("Pong! 🏓")
        .field("Guilds", guild_count.to_string(), true)
        .field("API Latency", format!("{}ms", api_latency), true)
        .field("Memory Usage", format!("{}MB", memory_usage), true)
        .field("Cache Size", cache_size.to_string(), true)
        .color(0x00FF00);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}