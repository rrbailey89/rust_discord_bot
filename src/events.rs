// src/events.rs (Updated with Analytics Logging)
use crate::emoji_reaction::handle_message;
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{
    ChannelId, Context, CreateEmbed, CreateEmbedFooter, CreateMessage, FullEvent, Guild, GuildId,
    Member, Message, MessageId, Interaction, Reaction, ReactionType, CreateEmbedAuthor, User,
};
use poise::FrameworkContext;
use crate::commands::admin::add_role_buttons::handle_role_button;
use regex::Regex;
use crate::DataContainer;
use crate::web::services::AnalyticsService; // Added
use crate::web::models::analytics::LogEventRequest; // Added
use std::collections::HashMap; // Added
use tracing::{error, warn, info, debug}; // Added info, debug
use poise::serenity_prelude::{Ready, OnlineStatus, ActivityData}; // Added Ready, OnlineStatus, ActivityData
use std::sync::Arc; // Added Arc
use tokio::time::{sleep, Duration}; // Added sleep, Duration
use rand::{Rng, thread_rng}; // Added Rng, thread_rng

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
        // Revert GuildDelete pattern match to original
        FullEvent::GuildDelete { incomplete, .. } => {
            handle_guild_delete(ctx, incomplete.id, data).await?;
        }
        FullEvent::GuildMemberAddition { new_member } => {
            handle_guild_member_addition(ctx, new_member, data).await?;
        }
        FullEvent::GuildMemberRemoval { guild_id, user, member_data_if_available: _ } => {
            handle_guild_member_removal(ctx, guild_id, user, data).await?;
        }
        FullEvent::MessageDelete { channel_id, deleted_message_id, guild_id, .. } => {
            handle_message_delete(ctx, channel_id, *deleted_message_id, *guild_id, data).await?;
        }
        FullEvent::MessageDeleteBulk { channel_id, multiple_deleted_messages_ids, guild_id, .. } => {
            handle_message_delete_bulk(ctx, channel_id, multiple_deleted_messages_ids, *guild_id, data).await?;
        }
        FullEvent::Message { new_message } => {
            if !new_message.author.bot {
                // Log message_sent event
                let analytics_service = AnalyticsService::new(data.database.clone());
                let mut event_data = HashMap::new();
                event_data.insert("channel_id".to_string(), serde_json::Value::String(new_message.channel_id.to_string()));
                event_data.insert("message_length".to_string(), serde_json::Value::Number(serde_json::Number::from(new_message.content.len())));

                let log_request = LogEventRequest {
                    event_type: "message_sent".to_string(),
                    user_id: Some(new_message.author.id.get() as i64),
                    guild_id: new_message.guild_id.map(|g| g.get() as i64),
                    event_data: Some(event_data),
                };
                // Spawn task to avoid blocking event handler
                tokio::spawn(async move {
                    if let Err(e) = analytics_service.log_event(&log_request).await {
                        error!("Failed to log message_sent analytics event: {}", e);
                    }
                });

                // Existing message handling logic
                handle_message(ctx, framework, data, new_message).await?;
                handle_message_for_leveling(ctx, new_message, data).await?;
                process_url_rule(ctx, data, new_message).await?;

                // Check if this is the unavailability channel and delete non-bot messages
                if let Some(guild_id) = new_message.guild_id {
                    if let Some(unavailability_channel_id) = data.database.fetch_unavailability_channel(guild_id.get() as i64).await? {
                        if new_message.channel_id.get() as i64 == unavailability_channel_id {
                            // Delete the message if it's not from a bot
                            new_message.delete(&ctx.http).await?;
                        }
                    }
                }
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
        FullEvent::Ready { data_about_bot } => {
            info!("{} is connected!", data_about_bot.user.name);
            // Spawn the presence update task
            let ctx_clone = ctx.clone();
            // Get the Arc<Data> using the context's TypeMap and clone it
            let data_arc = {
                let data_read = ctx.data.read().await;
                data_read.get::<DataContainer>().expect("Expected DataContainer in TypeMap").clone()
            };
            tokio::spawn(async move {
                info!("Spawning presence update task...");
                // Pass the cloned Arc<Data> to the task
                if let Err(e) = update_presence(ctx_clone, data_arc).await {
                    error!("Presence update task failed: {}", e);
                } else {
                    // This part might not be reached if update_presence loops infinitely
                    info!("Presence update task finished unexpectedly.");
                }
            });
        }
        _ => {}
    }
    Ok(())
}

// --- Event Handlers ---

/// Task to periodically update the bot's presence.
pub async fn update_presence(ctx: Context, data: Arc<Data>) -> Result<(), Error> {
    info!("Presence update task started.");
    loop {
        // Set initial/default presence
        let activity_help = ActivityData::custom("Use /help to learn more");
        ctx.set_presence(Some(activity_help), OnlineStatus::Online);
        debug!("Presence set to: Use /help to learn more");

        // Sleep for a random duration
        let sleep_duration_1 = {
            let mut rng = thread_rng();
            Duration::from_secs(rng.gen_range(600..=900)) // 10-15 minutes
        };
        debug!("Sleeping for {:?}", sleep_duration_1);
        sleep(sleep_duration_1).await;

        // Fetch blame count and set presence
        match data.database.get_blame_count().await {
            Ok(blame_count) => {
                let activity_blame = ActivityData::custom(format!("Serena's blame count: {}", blame_count));
                ctx.set_presence(Some(activity_blame), OnlineStatus::Online);
                debug!("Presence set to: Serena's blame count: {}", blame_count);
            }
            Err(e) => {
                error!("Failed to get blame count for presence update: {}", e);
                // Optionally, set a default status or just skip this update cycle
            }
        }

        // Sleep again before looping
        let sleep_duration_2 = {
            let mut rng = thread_rng();
            Duration::from_secs(rng.gen_range(600..=900)) // 10-15 minutes
        };
        debug!("Sleeping for {:?}", sleep_duration_2);
        sleep(sleep_duration_2).await;
    }
    // Note: This loop is infinite, so Ok(()) is technically unreachable unless the loop breaks.
    // If we wanted it to be stoppable, we'd need a different mechanism (e.g., checking an AtomicBool).
    #[allow(unreachable_code)]
    Ok(())
}


async fn handle_guild_member_addition(ctx: &Context, new_member: &Member, data: &Data) -> Result<(), Error> {
    info!("User {} joined guild {}", new_member.user.name, new_member.guild_id);

    // Log analytics event
    let analytics_service = AnalyticsService::new(data.database.clone());
    let log_request = LogEventRequest {
        event_type: "user_joined".to_string(),
        user_id: Some(new_member.user.id.get() as i64),
        guild_id: Some(new_member.guild_id.get() as i64),
        event_data: None, // No extra data needed for this event type
    };
    tokio::spawn(async move {
        if let Err(e) = analytics_service.log_event(&log_request).await {
            error!("Failed to log user_joined analytics event: {}", e);
        }
    });

    // Potentially send a welcome message, etc.

    Ok(())
}

async fn handle_guild_member_removal(ctx: &Context, guild_id: &GuildId, user: &User, data: &Data) -> Result<(), Error> {
    info!("User {} left guild {}", user.name, guild_id);

    // Log analytics event
    let analytics_service = AnalyticsService::new(data.database.clone());
    let log_request = LogEventRequest {
        event_type: "user_left".to_string(),
        user_id: Some(user.id.get() as i64),
        guild_id: Some(guild_id.get() as i64),
        event_data: None, // No extra data needed for this event type
    };
    tokio::spawn(async move {
        if let Err(e) = analytics_service.log_event(&log_request).await {
            error!("Failed to log user_left analytics event: {}", e);
        }
    });

    Ok(())
}


async fn handle_guild_create(ctx: &Context, guild: &Guild, data: &Data) -> Result<(), Error> {
    // Log guild creation
    info!("Guild Create event received for: {} (ID: {})", guild.name, guild.id);

    // Store guild info in the database
    data.database.store_guild_info(guild).await?;

    // Store guild channels in the database
    data.database.store_guild_channels(guild).await?;

    // Initialize command settings for this guild
    let command_service = crate::web::services::CommandService::new(data.database.clone());
    match command_service.initialize_guild_command_settings(guild.id.get() as i64).await {
        Ok(_) => {
            info!("Initialized command settings for guild {}", guild.id);
        },
        Err(e) => {
            error!("Failed to initialize command settings for guild {}: {}", guild.id, e);
        }
    }

    // Additionally, fetch and store the guild members.
    // We'll do this in a separate task to avoid blocking.
    let ctx_clone = ctx.clone();
    let guild_id_clone = guild.id;
    let guild_id_i64 = guild.id.get() as i64;
    let database = data.database.clone();
    let guild_roles = guild.roles.clone();

    tokio::spawn(async move {
        info!("Starting background task to fetch members for guild {}", guild_id_i64);

        // Fetch up to 1000 members
        match guild_id_clone.members(&ctx_clone.http, None, None).await {
            Ok(members) => {
                info!("Fetched {} members for guild {}", members.len(), guild_id_i64);

                // Call the refactored store_guild_members directly with the Vec<Member>
                match database.store_guild_members(guild_id_i64, &members).await {
                    Ok(count) => {
                        // This count now reflects attempts to store in guild_members,
                        // not necessarily successful user inserts.
                        info!("Processed {} members for guild {}", count, guild_id_i64);
                    },
                    Err(e) => {
                        error!("Failed to store members for guild {}: {}", guild_id_i64, e);
                    }
                }
            },
            Err(e) => {
                error!("Failed to fetch members for guild {}: {}", guild_id_i64, e);
            }
        }
    });

    Ok(())
}

async fn handle_guild_delete(_ctx: &Context, guild_id: GuildId, data: &Data) -> Result<(), Error> {
    // Log guild deletion
    info!("Bot has left the guild with ID: {}", guild_id);

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
    if msg.author.bot || msg.guild_id.is_none() || msg.content.is_empty() {
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

fn calculate_required_exp(level: i32) -> i32 {
    (10.0 * (1.5f64.powi(level - 1))).round() as i32
}

async fn handle_reaction_add(ctx: &Context, reaction: &Reaction) -> Result<(), Error> {
    // Get the user who added the reaction
    let user = reaction.user(&ctx.http).await?;

    // Check if the reaction is from a bot
    if user.bot {
        return Ok(()); // Ignore reactions from bots
    }

    // Log reaction_added event
    let analytics_service = {
        let data_read = ctx.data.read().await;
        AnalyticsService::new(data_read.get::<DataContainer>().expect("Expected DataContainer in TypeMap").database.clone())
    };
    let mut event_data = HashMap::new();
    event_data.insert("channel_id".to_string(), serde_json::Value::String(reaction.channel_id.to_string()));
    event_data.insert("message_id".to_string(), serde_json::Value::String(reaction.message_id.to_string()));
    event_data.insert("emoji".to_string(), serde_json::Value::String(reaction.emoji.to_string()));

    let log_request = LogEventRequest {
        event_type: "reaction_added".to_string(),
        user_id: Some(user.id.get() as i64),
        guild_id: reaction.guild_id.map(|g| g.get() as i64),
        event_data: Some(event_data),
    };
    tokio::spawn(async move {
        if let Err(e) = analytics_service.log_event(&log_request).await {
            error!("Failed to log reaction_added analytics event: {}", e);
        }
    });


    if let Some(guild_id) = reaction.guild_id {
        let data = {
            let data_read = ctx.data.read().await;
            data_read.get::<DataContainer>().expect("Expected DataContainer in TypeMap").clone()
        };

        let guild_name = guild_id.name(&ctx.cache).unwrap_or_else(|| "Unknown Guild".to_string());

        if reaction.emoji == ReactionType::Unicode("⭐".to_string()) {
            if let Some(star_channel_id) = data.database.get_reaction_log_channel(guild_id.get() as i64, "star").await? {
                let star_channel = ChannelId::new(star_channel_id as u64);

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

                // Send the embed in the star channel
                star_channel.send_message(&ctx.http, CreateMessage::default()
                    .add_embed(embed)
                ).await?;
            }
        } else {
            // Log non-star reactions to the reaction log channel
            if let Some(reaction_log_channel_id) = data.database.get_reaction_log_channel(guild_id.get() as i64, "reactions").await? {
                let reaction_log_channel = ChannelId::new(reaction_log_channel_id as u64);

                let emoji_name = match &reaction.emoji {
                    ReactionType::Custom { animated: _, id, name } => name.as_ref().map_or_else(|| id.to_string(), |s| s.clone()),
                    ReactionType::Unicode(s) => s.clone(),
                    _ => "Unknown Emoji".to_string(),
                };

                // We already fetched the user above, no need to fetch again
                // let user = reaction.user(&ctx.http).await?;
                let timestamp = chrono::Utc::now().timestamp();

                let embed = CreateEmbed::default()
                    .title("Reaction Added")
                    .description(format!("{} reacted with {} at <t:{}:F>", user.name, emoji_name, timestamp))
                    .footer(CreateEmbedFooter::new(format!("Guild: {}", guild_name)))
                    .color(0x00FF00);

                reaction_log_channel.send_message(&ctx.http, CreateMessage::default()
                    .add_embed(embed)
                ).await?;
            }
        }
    }
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
