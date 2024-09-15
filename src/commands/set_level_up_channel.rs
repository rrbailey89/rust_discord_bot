use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{Channel};

type Context<'a> = poise::Context<'a, Data, Error>;

/// Set the channel for level-up announcements
#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn setlevelupchannel(
    ctx: Context<'_>,
    #[description = "Channel to send level-up messages"] channel: Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("Failed to get guild ID".to_string()))?;

    ctx.data().database.set_level_up_channel(guild_id.get() as i64, channel.id().get() as i64).await?;

    ctx.say(format!(
        "✅ Channel <#{}> has been set for level-up announcements.",
        channel.id()
    )).await?;

    Ok(())
}