use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{ChannelId, GuildId, Context as SerenityContext};

/// Relay a message to a specific channel in a guild
#[poise::command(prefix_command, dm_only, hide_in_help)]
pub async fn relay(
    ctx: poise::Context<'_, Data, Error>,
    #[description = "Guild ID"] guild_id: String,
    #[description = "Channel ID"] channel_id: String,
    #[description = "Message to relay"] #[rest] message: String,
) -> Result<(), Error> {
    // Parse the guild_id and channel_id
    let guild_id = guild_id.parse::<u64>()
        .map_err(|_| Error::Unknown("Invalid guild ID".to_string()))?;
    let channel_id = channel_id.parse::<u64>()
        .map_err(|_| Error::Unknown("Invalid channel ID".to_string()))?;

    let guild_id = GuildId::new(guild_id);
    let channel_id = ChannelId::new(channel_id);

    // Use the serenity context directly
    let serenity_ctx = ctx.serenity_context();

    // Check if the bot is in the specified guild and if the channel exists
    match is_bot_in_guild_and_channel_exists(serenity_ctx, guild_id, channel_id) {
        Ok(_) => {
            // Send the message to the specified channel
            channel_id.say(&ctx.http(), &message).await?;

            // Confirm to the user that the message was sent
            ctx.say("Message relayed successfully!").await?;
            Ok(())
        },
        Err(Error::NotInGuild) => {
            ctx.say("Bot is not a member of the specified guild.").await?;
            Ok(()) // Return Ok to prevent poise from handling the error
        },
        Err(Error::ChannelNotFound) => {
            ctx.say("The specified channel was not found in the guild.").await?;
            Ok(()) // Return Ok to prevent poise from handling the error
        },
        Err(e) => {
            ctx.say(format!("An unexpected error occurred: {}", e)).await?;
            Ok(()) // Return Ok to prevent poise from handling the error
        },
    }
}

fn is_bot_in_guild_and_channel_exists(ctx: &SerenityContext, guild_id: GuildId, channel_id: ChannelId) -> Result<bool, Error> {
    if let Some(guild) = ctx.cache.guild(guild_id) {
        if guild.channels.contains_key(&channel_id) {
            Ok(true)
        } else {
            Err(Error::ChannelNotFound)
        }
    } else {
        Err(Error::NotInGuild)
    }
}