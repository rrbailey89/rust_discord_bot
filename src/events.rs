use crate::emoji_reaction::handle_message;
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{ChannelId, Context, CreateEmbed, CreateEmbedFooter, CreateMessage, FullEvent, Guild, GuildId, MessageId};
use poise::FrameworkContext;

pub async fn handle_event(
    ctx: &Context,
    event: &FullEvent,
    framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        FullEvent::GuildCreate { guild, .. } => {
            handle_guild_create(ctx, guild, data).await?;
        }
        FullEvent::GuildDelete { incomplete, .. } => {
            handle_guild_delete(ctx, incomplete.id, data).await?;
        }
        FullEvent::MessageDelete { channel_id, deleted_message_id, guild_id, .. } => {
            handle_message_delete(ctx, channel_id, *deleted_message_id, *guild_id, data).await?;
        }
        FullEvent::MessageDeleteBulk { channel_id, multiple_deleted_messages_ids, guild_id, .. } => {
            handle_message_delete_bulk(ctx, channel_id, multiple_deleted_messages_ids, *guild_id, data).await?;
        }
        FullEvent::Message { new_message } => {
            if !new_message.author.bot {
                handle_message(ctx, framework, data, new_message).await?;
            }
        }
        _ => {}
    }
    Ok(())
}

async fn handle_guild_create(_ctx: &Context, guild: &Guild, data: &Data) -> Result<(), Error> {
    // Log guild creation
    tracing::info!("Guild Create event received for: {} (ID: {})", guild.name, guild.id);

    // Store guild info in the database
    data.database.store_guild_info(guild).await?;

    // Store guild channels in the database
    data.database.store_guild_channels(guild).await?;

    Ok(())
}

async fn handle_guild_delete(_ctx: &Context, guild_id: GuildId, data: &Data) -> Result<(), Error> {
    // Log guild deletion
    tracing::info!("Bot has left the guild with ID: {}", guild_id);

    // Remove guild info from the database
    data.database.remove_guild_info(guild_id.get() as i64).await?;

    Ok(())
}
async fn handle_message_delete(
    ctx: &Context,
    channel_id: &ChannelId,
    deleted_message_id: MessageId,
    guild_id: Option<GuildId>,
    data: &Data,
) -> Result<(), Error> {
    if let Some(guild_id) = guild_id {
        if let Some(log_channel_id) = data.database.fetch_delete_log_channel(guild_id.get() as i64).await? {
            let log_channel = ChannelId::new(log_channel_id as u64);

            let message_content = ctx.cache.message(channel_id, deleted_message_id).map(|message| {
                (message.content.clone(), message.author.id, message.timestamp)
            });

            if let Some((content, author_id, timestamp)) = message_content {
                let embed = CreateEmbed::default()
                    .title("Message Deleted")
                    .description(format!("A message from <@{}> was deleted in <#{}>", author_id, channel_id))
                    .field("Content", content, false)
                    .field("Message ID", deleted_message_id.to_string(), true)
                    .field("Author ID", author_id.to_string(), true)
                    .timestamp(timestamp)
                    .footer(CreateEmbedFooter::new(format!("Message sent at {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC"))))
                    .color(0xFF0000); // Red color

                let message = CreateMessage::default().embed(embed);
                log_channel.send_message(&ctx.http, message).await?;
            } else {
                // If the message is not in the cache, we can't retrieve its content
                log_channel.say(&ctx.http, format!(
                    "A message was deleted in <#{}>\nMessage ID: {}",
                    channel_id, deleted_message_id
                )).await?;
            }
        }
    }
    Ok(())
}

async fn handle_message_delete_bulk(
    ctx: &Context,
    channel_id: &ChannelId,
    multiple_deleted_messages_ids: &Vec<MessageId>,
    guild_id: Option<GuildId>,
    data: &Data,
) -> Result<(), Error> {
    if let Some(guild_id) = guild_id {
        if let Some(log_channel_id) = data.database.fetch_delete_log_channel(guild_id.get() as i64).await? {
            let log_channel = ChannelId::new(log_channel_id as u64);

            // Send an initial message about the bulk deletion
            log_channel.say(&ctx.http, format!(
                "Bulk message deletion in <#{}>\nNumber of messages deleted: {}",
                channel_id, multiple_deleted_messages_ids.len()
            )).await?;

            // Create an embed for each deleted message
            for message_id in multiple_deleted_messages_ids {
                let message_content = ctx.cache.message(channel_id, message_id).map(|message| {
                    (message.content.clone(), message.author.id, message.timestamp)
                });

                if let Some((content, author_id, timestamp)) = message_content {
                    let embed = CreateEmbed::default()
                        .title("Deleted Message")
                        .description(format!("Author: <@{}>", author_id))
                        .field("Content", content, false)
                        .field("Message ID", message_id.to_string(), true)
                        .field("Author ID", author_id.to_string(), true)
                        .timestamp(timestamp)
                        .footer(CreateEmbedFooter::new(format!("Message sent at {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC"))))
                        .color(0xFF0000); // Red color

                    let message = CreateMessage::default().embed(embed);
                    log_channel.send_message(&ctx.http, message).await?;
                }
            }
        }
    }
    Ok(())
}
