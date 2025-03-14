use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::Channel;

type Context<'a> = poise::Context<'a, Data, Error>;

/// Set the channel for posting unavailability
#[poise::command(slash_command, default_member_permissions = "MANAGE_CHANNELS", guild_only)]
pub async fn setunavailabilitychannel(
    ctx: Context<'_>,
    #[description = "Channel for unavailability posts"] channel: Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("Failed to get guild ID".to_string()))?;

    ctx.data().database.store_unavailability_channel(guild_id.get() as i64, channel.id().get() as i64).await?;

    ctx.say(format!(
        "✅ Channel <#{}> has been set for unavailability posts.",
        channel.id()
    )).await?;

    Ok(())
}
