use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::Channel;

type Context<'a> = poise::Context<'a, Data, Error>;

/// Set channels for reaction logs
#[poise::command(
    slash_command,
    subcommands("setstarschannel", "setreactionschannel"),
    default_member_permissions = "MANAGE_CHANNELS",
    guild_only
)]
pub async fn reactionslog(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// Set the channel for star messages
#[poise::command(slash_command)]
pub async fn setstarschannel(
    ctx: Context<'_>,
    #[description = "Channel to send star messages"] channel: Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("Failed to get guild ID".to_string()))?;

    ctx.data().database.set_reaction_log_channel(guild_id.get() as i64, channel.id().get() as i64, "star").await?;

    ctx.say(format!(
        "✅ Channel <#{}> has been set for star messages.",
        channel.id()
    )).await?;

    Ok(())
}

/// Set the channel for general reaction logs
#[poise::command(slash_command)]
pub async fn setreactionschannel(
    ctx: Context<'_>,
    #[description = "Channel to send reaction logs"] channel: Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("Failed to get guild ID".to_string()))?;

    ctx.data().database.set_reaction_log_channel(guild_id.get() as i64, channel.id().get() as i64, "reactions").await?;

    ctx.say(format!(
        "✅ Channel <#{}> has been set for general reaction logs.",
        channel.id()
    )).await?;

    Ok(())
}