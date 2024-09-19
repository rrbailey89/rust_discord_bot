use crate::emoji_reaction::handle_message;
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{ChannelId, Context, CreateEmbed, CreateEmbedFooter, CreateMessage, FullEvent, Guild, GuildId, MessageId, Message, Interaction, Reaction, ReactionType, CreateEmbedAuthor};
use poise::FrameworkContext;
use crate::commands::add_role_buttons::handle_role_button;
use regex::Regex;

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
                handle_message_for_leveling(ctx, new_message, data).await?;
                process_url_rule(ctx, data, new_message).await?;
            }
        }
        FullEvent::InteractionCreate { interaction } => {
            if let Interaction::Component(component) = interaction {
                if component.data.custom_id.starts_with("role_") || component.data.custom_id.starts_with("nested_") {
                    handle_role_button(ctx, component).await?;
                }
            }
        }
        FullEvent::ReactionAdd { add_reaction } => {
            handle_reaction_add(ctx, add_reaction).await?;
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

async fn handle_message_for_leveling(ctx: &Context, msg: &Message, data: &Data) -> Result<(), Error> {
    if msg.author.bot {
        return Ok(());
    }

    let guild_id = msg.guild_id.ok_or_else(|| Error::Unknown("Not a guild message".to_string()))?;
    let user_id = msg.author.id;

    let (mut current_level, current_exp) = data.database.get_user_level(guild_id.get() as i64, user_id.get() as i64).await?;
    let new_exp = current_exp + 1;

    // Check if the user should level up
    while new_exp >= calculate_required_exp(current_level + 1) {
        current_level += 1;

        if let Some(channel_id) = data.database.get_level_up_channel(guild_id.get() as i64).await? {
            let channel = ChannelId::new(channel_id as u64);
            channel.say(&ctx.http, format!(
                "🎉 Congratulations <@{}>! You've reached level {}!",
                user_id, current_level
            )).await?;
        }
    }

    // Update the database with the new level and total experience
    data.database.update_user_level_and_exp(guild_id.get() as i64, user_id.get() as i64, current_level, new_exp).await?;

    Ok(())
}
async fn handle_reaction_add(ctx: &Context, reaction: &Reaction) -> Result<(), Error> {
    if reaction.emoji == ReactionType::Unicode("⭐".to_string()) {
        handle_star_reaction(ctx, reaction).await?;
    }
    Ok(())
}
fn calculate_required_exp(level: i32) -> i32 {
    (10.0 * (1.5f64.powi(level - 1))).round() as i32
}

async fn handle_star_reaction(ctx: &Context, reaction: &Reaction) -> Result<(), Error> {
    let channel = reaction.channel_id;
    let message_id = reaction.message_id;

    // Fetch the original message
    let message = channel.message(&ctx.http, message_id).await?;

    // Create the embed
    let embed = CreateEmbed::default()
        .title("You're a Star! ⭐")
        .description(&message.content)
        .author(CreateEmbedAuthor::new(&message.author.name)
            .icon_url(message.author.face()))
        .footer(CreateEmbedFooter::new(format!("Original message ID: {}", message_id)))
        .timestamp(message.timestamp);

    // If the original message has an image, add it to the embed
    let embed = if let Some(attachment) = message.attachments.first() {
        if attachment.width.is_some() {  // This checks if it's an image
            embed.image(&attachment.url)
        } else {
            embed
        }
    } else {
        embed
    };

    // Send the embed in the same channel
    channel.send_message(&ctx.http, CreateMessage::default()
        .add_embed(embed)
    ).await?;

    Ok(())
}
async fn process_url_rule(ctx: &Context, data: &Data, message: &Message) -> Result<(), Error> {
    if let Some(guild_id) = message.guild_id {
        if let Some(rule) = data.database.get_url_rule(guild_id.get() as i64, message.channel_id.get() as i64).await? {
            let re = Regex::new(&rule.regex).map_err(|_| Error::Unknown("Invalid regex pattern".into()))?;
            if let Some(captures) = re.captures(&message.content) {
                let mut output = rule.output_template.clone();
                for (i, capture) in captures.iter().enumerate().skip(1) {
                    if let Some(c) = capture {
                        output = output.replace(&format!("${}", i), c.as_str());
                    }
                }
                message.channel_id.say(&ctx.http, &output).await?;
            }
        }
    }
    Ok(())
}