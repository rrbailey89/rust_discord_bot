// events.rs
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{Context, FullEvent, Guild, GuildId};
use poise::FrameworkContext;

pub async fn handle_event(
    ctx: &Context,
    event: &FullEvent,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::GuildCreate { guild, .. } => {
            handle_guild_create(ctx, guild, data).await?;
        }
        FullEvent::GuildDelete { incomplete, .. } => {
            handle_guild_delete(ctx, incomplete.id, data).await?;
        }
        _ => {}
    }
    Ok(())
}

async fn handle_guild_create(ctx: &Context, guild: &Guild, data: &Data) -> Result<(), Error> {
    // Log guild creation
    tracing::info!("Guild Create event received for: {} (ID: {})", guild.name, guild.id);

    // Store guild info in the database
    // Note: You'll need to implement this method in your Database struct
    data.database.store_guild_info(guild).await?;

    // Store guild channels in the database
    // Note: You'll need to implement this method in your Database struct
    data.database.store_guild_channels(guild).await?;

    Ok(())
}

async fn handle_guild_delete(_ctx: &Context, guild_id: GuildId, data: &Data) -> Result<(), Error> {
    // Log guild deletion
    tracing::info!("Bot has left the guild with ID: {}", guild_id);

    // Remove guild info from the database
    // Note: You'll need to implement this method in your Database struct
    data.database.remove_guild_info(guild_id.get() as i64).await?;

    Ok(())
}