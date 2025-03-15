use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::Channel;

type Context<'a> = poise::Context<'a, Data, Error>;

/// Set the channel for logging warnings
#[poise::command(slash_command, default_member_permissions = "MANAGE_CHANNELS", guild_only)]
pub async fn setwarnchannel(
    ctx: Context<'_>,
    #[description = "Channel to log warnings"] channel: Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("Failed to get guild ID".to_string()))?;

    ctx.data().database.store_warn_channel(guild_id.get() as i64, channel.id().get() as i64).await?;

    ctx.say(format!(
        "✅ Channel <#{}> has been set for logging warnings.",
        channel.id()
    )).await?;

    Ok(())
}