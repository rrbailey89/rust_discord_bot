// commands/warn.rs
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{Member, ChannelId};

type Context<'a> = poise::Context<'a, Data, Error>;

/// Warn a member
#[poise::command(slash_command)]
pub async fn warn(
    ctx: Context<'_>,
    #[description = "Member to warn"] member: Member,
    #[description = "Reason for the warning"] reason: Option<String>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().ok_or_else(|| Error::Unknown("Failed to get guild ID".to_string()))?;
    let warn_channel_id = ctx.data().database.fetch_warn_channel(guild_id.get() as i64).await?;

    let reason_message = reason.unwrap_or_else(|| "No reason provided".to_string());
    let warn_message = format!("🚨 {} has been warned for: {}", member.user.name, reason_message);

    match warn_channel_id {
        Some(channel_id) => {
            let warn_channel = ChannelId::new(channel_id as u64);
            warn_channel.say(&ctx.http(), &warn_message).await?;
            ctx.say("✅ Warning has been issued successfully.").await?;
        }
        None => {
            ctx.channel_id().say(&ctx.http(), &warn_message).await?;
            ctx.say("Warning has been issued in this channel. Use /setwarnchannel to set a warning channel.").await?;
        }
    }

    Ok(())
}