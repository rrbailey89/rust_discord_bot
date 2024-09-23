use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::Channel;

type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, guild_only)]
pub async fn setdeletemessagechannel(
    ctx: Context<'_>,
    #[description = "Channel to log deleted messages"] channel: Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("Failed to get guild ID".to_string()))?;

    let guild_name = guild_id.name(&ctx.cache())
        .ok_or_else(|| Error::Unknown("Failed to get guild name".to_string()))?
        .to_string();

    ctx.data().database.store_delete_log_channel(
        guild_id.get() as i64,
        channel.id().get() as i64,
        guild_name.clone()
    ).await?;

    ctx.say(format!(
        "✅ Channel <#{}> has been set for logging deleted messages in {}.",
        channel.id(),
        guild_name
    )).await?;

    Ok(())
}
