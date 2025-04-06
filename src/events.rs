// src/events.rs (Corrected ID Handling, Logging, and Structure)
use crate::emoji_reaction::handle_message;
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{
    ChannelId, Context, CreateEmbed, CreateEmbedFooter, CreateMessage, FullEvent, Guild, GuildId,
    Member, Message, MessageId, Interaction, Reaction, ReactionType, CreateEmbedAuthor, User, OnlineStatus, ActivityData,
};
use poise::FrameworkContext;
use crate::commands::admin::add_role_buttons::handle_role_button;
use regex::Regex;
use crate::DataContainer;
use crate::web::services::{AnalyticsService, GuildService};
use crate::web::models::analytics::LogEventRequest;
use std::collections::HashMap;
use tracing::{error, info, debug};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use rand::{Rng, rng};

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
                    // Assuming fetch_unavailability_channel returns Option<i64>
                    if let Some(unavailability_channel_id_i64) = data.database.fetch_unavailability_channel(guild_id.get() as i64).await? {
                        if new_message.channel_id.get() == unavailability_channel_id_i64 as u64 { // Cast for comparison
                            if let Err(e) = new_message.delete(&ctx.http).await {
                                error!("Failed to delete message in unavailability channel: {}", e);
                            }
                        }
                    }
                }
            }
        }
        FullEvent::InteractionCreate { interaction } => {
            // Log the raw interaction structure for debugging command parsing issues
            if let Interaction::Command(command) = interaction {
                debug!("Received Command interaction: {:?}", command);
            }

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
            let ctx_clone = ctx.clone();
            let data_arc = {
                let data_read = ctx.data.read().await;
                data_read.get::<DataContainer>().expect("Expected DataContainer in TypeMap").clone()
            };
            tokio::spawn(async move {
                info!("Spawning presence update task...");
                if let Err(e) = update_presence(ctx_clone, data_arc).await {
                    error!("Presence update task failed: {}", e);
                } else {
                    info!("Presence update task finished unexpectedly.");
                }
            });
        }
        _ => {}
    }
    Ok(())
}

// --- Event Handlers ---

pub async fn update_presence(ctx: Context, data: Arc<Data>) -> Result<(), Error> {
    info!("Presence update task started.");
    loop {
        let activity_help = ActivityData::custom("Use /help to learn more");
        ctx.set_presence(Some(activity_help), OnlineStatus::Online);
        debug!("Presence set to: Use /help to learn more");

        let sleep_duration_1 = { let mut rng = rng(); Duration::from_secs(rng.random_range(600..=900)) };
        debug!("Sleeping for {:?}", sleep_duration_1);
        sleep(sleep_duration_1).await;

        let activity_website = ActivityData::custom("https://blameserena.app/");
        ctx.set_presence(Some(activity_website), OnlineStatus::Online);
        debug!("Presence set to: https://blameserena.app/");

        let sleep_duration_2 = { let mut rng = rng(); Duration::from_secs(rng.random_range(600..=900)) };
        debug!("Sleeping for {:?}", sleep_duration_2);
        sleep(sleep_duration_2).await;

        match data.database.get_blame_count().await {
            Ok(blame_count) => {
                let activity_blame = ActivityData::custom(format!("Serena's blame count: {}", blame_count));
                ctx.set_presence(Some(activity_blame), OnlineStatus::Online);
                debug!("Presence set to: Serena's blame count: {}", blame_count);
            }
            Err(e) => error!("Failed to get blame count for presence update: {}", e),
        }

        let sleep_duration_3 = { let mut rng = rng(); Duration::from_secs(rng.random_range(600..=900)) };
        debug!("Sleeping for {:?}", sleep_duration_3);
        sleep(sleep_duration_3).await;
    }
    #[allow(unreachable_code)] // This function loops indefinitely
    Ok(())
}

async fn handle_guild_member_addition(_ctx: &Context, new_member: &Member, data: &Data) -> Result<(), Error> {
    info!("User {} joined guild {}", new_member.user.name, new_member.guild_id);
    let analytics_service = AnalyticsService::new(data.database.clone());
    let log_request = LogEventRequest {
        event_type: "user_joined".to_string(),
        user_id: Some(new_member.user.id.get() as i64),
        guild_id: Some(new_member.guild_id.get() as i64),
        event_data: None,
    };
    tokio::spawn(async move {
        if let Err(e) = analytics_service.log_event(&log_request).await {
            error!("Failed to log user_joined analytics event: {}", e);
        }
    });
    Ok(())
}

async fn handle_guild_member_removal(_ctx: &Context, guild_id: &GuildId, user: &User, data: &Data) -> Result<(), Error> {
    info!("User {} left guild {}", user.name, guild_id);
    let analytics_service = AnalyticsService::new(data.database.clone());
    let log_request = LogEventRequest {
        event_type: "user_left".to_string(),
        user_id: Some(user.id.get() as i64),
        guild_id: Some(guild_id.get() as i64),
        event_data: None,
    };
    tokio::spawn(async move {
        if let Err(e) = analytics_service.log_event(&log_request).await {
            error!("Failed to log user_left analytics event: {}", e);
        }
    });
    Ok(())
}

async fn handle_guild_create(ctx: &Context, guild: &Guild, data: &Data) -> Result<(), Error> {
    info!("Guild Create event received for: {} (ID: {})", guild.name, guild.id);
    data.database.store_guild_info(guild).await?;
    data.database.store_guild_channels(guild).await?;

    let command_service = crate::web::services::CommandService::new(data.database.clone());
    match command_service.initialize_guild_command_settings(guild.id.get() as i64).await {
        Ok(_) => info!("Initialized command settings for guild {}", guild.id),
        Err(e) => error!("Failed to initialize command settings for guild {}: {}", guild.id, e),
    }

    let ctx_clone = ctx.clone();
    let guild_id_clone = guild.id;
    let guild_id_i64 = guild.id.get() as i64;
    let database = data.database.clone();

    tokio::spawn(async move {
        info!("Starting background task to fetch members for guild {}", guild_id_i64);
        match guild_id_clone.members(&ctx_clone.http, None, None).await {
            Ok(members) => {
                info!("Fetched {} members for guild {}", members.len(), guild_id_i64);
                match database.store_guild_members(guild_id_i64, &members).await {
                    Ok(count) => info!("Processed {} members for guild {}", count, guild_id_i64),
                    Err(e) => error!("Failed to store members for guild {}: {}", guild_id_i64, e),
                }
            },
            Err(e) => error!("Failed to fetch members for guild {}: {}", guild_id_i64, e),
        }
    });
    Ok(())
}

async fn handle_guild_delete(_ctx: &Context, guild_id: GuildId, data: &Data) -> Result<(), Error> {
    info!("Bot has left the guild with ID: {}", guild_id);
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
    if let Some(gid) = guild_id {
        let guild_service = GuildService::new(data.database.clone());
        match guild_service.get_guild_settings(gid.get() as i64).await {
            Ok(settings) => {
                if let Some(log_channel_id_str) = settings.delete_log_channel_id {
                    match log_channel_id_str.parse::<u64>() {
                        Ok(log_channel_id_u64) => {
                            debug!("Found delete log channel {} for guild {}", log_channel_id_u64, gid);
                            let log_channel = ChannelId::new(log_channel_id_u64);
                            let message_content = ctx.cache.message(channel_id, deleted_message_id).map(|msg| (msg.content.clone(), msg.author.id, msg.timestamp));

                            if let Some((content, author_id, timestamp)) = message_content {
                                let embed = CreateEmbed::default()
                                    .title("Message Deleted")
                                    .description(format!("A message from <@{}> was deleted in <#{}>", author_id, channel_id))
                                    .field("Content", content, false)
                                    .field("Message ID", deleted_message_id.to_string(), true)
                                    .field("Author ID", author_id.to_string(), true)
                                    .timestamp(timestamp)
                                    .footer(CreateEmbedFooter::new(format!("Message sent at {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC"))))
                                    .color(0xFF0000);
                                let message = CreateMessage::default().embed(embed);
                                if let Err(e) = log_channel.send_message(&ctx.http, message).await {
                                    error!("Failed to send delete log message: {}", e);
                                }
                            } else {
                                if let Err(e) = log_channel.say(&ctx.http, format!("A message was deleted in <#{}>\nMessage ID: {}", channel_id, deleted_message_id)).await {
                                    error!("Failed to send simple delete log message: {}", e);
                                }
                            }
                        },
                        Err(_) => {
                            error!("Failed to parse delete_log_channel_id '{}' as u64 for guild {}", log_channel_id_str, gid);
                        }
                    }
                } else {
                    debug!("Delete log channel not configured for guild {}", gid);
                }
            },
            Err(e) => {
                error!("Failed to fetch guild settings for delete log in guild {}: {}", gid, e);
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
    if let Some(gid) = guild_id {
        let guild_service = GuildService::new(data.database.clone());
        match guild_service.get_guild_settings(gid.get() as i64).await {
            Ok(settings) => {
                if let Some(log_channel_id_str) = settings.delete_log_channel_id {
                    match log_channel_id_str.parse::<u64>() {
                        Ok(log_channel_id_u64) => {
                            debug!("Found delete log channel {} for bulk delete in guild {}", log_channel_id_u64, gid);
                            let log_channel = ChannelId::new(log_channel_id_u64);
                            if let Err(e) = log_channel.say(&ctx.http, format!("Bulk message deletion in <#{}>\nNumber of messages deleted: {}", channel_id, multiple_deleted_messages_ids.len())).await {
                                 error!("Failed to send bulk delete initial log message: {}", e);
                            }

                            for message_id in multiple_deleted_messages_ids {
                                let message_content = ctx.cache.message(channel_id, message_id).map(|msg| (msg.content.clone(), msg.author.id, msg.timestamp));
                                if let Some((content, author_id, timestamp)) = message_content {
                                    let embed = CreateEmbed::default()
                                        .title("Deleted Message (Bulk)")
                                        .description(format!("Author: <@{}>", author_id))
                                        .field("Content", content, false)
                                        .field("Message ID", message_id.to_string(), true)
                                        .field("Author ID", author_id.to_string(), true)
                                        .timestamp(timestamp)
                                        .footer(CreateEmbedFooter::new(format!("Message sent at {}", timestamp.format("%Y-%m-%d %H:%M:%S UTC"))))
                                        .color(0xFF0000);
                                    let message = CreateMessage::default().embed(embed);
                                    if let Err(e) = log_channel.send_message(&ctx.http, message).await {
                                        error!("Failed to send bulk delete detail message for {}: {}", message_id, e);
                                    }
                                }
                            }
                        },
                        Err(_) => {
                            error!("Failed to parse delete_log_channel_id '{}' as u64 for bulk delete in guild {}", log_channel_id_str, gid);
                        }
                    }
                } else {
                    debug!("Delete log channel not configured for bulk delete in guild {}", gid);
                }
            },
            Err(e) => {
                error!("Failed to fetch guild settings for bulk delete log in guild {}: {}", gid, e);
            }
        }
    }
    Ok(())
}

async fn handle_message_for_leveling(ctx: &Context, msg: &Message, data: &Data) -> Result<(), Error> {
    if msg.author.bot || msg.guild_id.is_none() || msg.content.is_empty() { return Ok(()); }
    let guild_id = msg.guild_id.unwrap();
    let user_id = msg.author.id;

    let (mut current_level, current_exp) = data.database.get_user_level(guild_id.get() as i64, user_id.get() as i64).await?;
    let new_exp = current_exp + 1;

    while new_exp >= calculate_required_exp(current_level + 1) {
        current_level += 1;
        // Fetch level up channel using GuildService
        let guild_service = GuildService::new(data.database.clone());
        match guild_service.get_guild_settings(guild_id.get() as i64).await {
            Ok(settings) => {
                if let Some(channel_id_str) = settings.level_up_channel_id {
                    match channel_id_str.parse::<u64>() {
                        Ok(channel_id_u64) => {
                            let channel = ChannelId::new(channel_id_u64);
                            if let Err(e) = channel.say(&ctx.http, format!("🎉 Congratulations <@{}>! You've reached level {}!", user_id, current_level)).await {
                                error!("Failed to send level up message to channel {}: {}", channel_id_u64, e);
                            }
                        },
                        Err(_) => {
                            error!("Failed to parse level_up_channel_id '{}' as u64 for guild {}", channel_id_str, guild_id);
                        }
                    }
                }
            },
            Err(e) => {
                 error!("Failed to fetch guild settings for level up message in guild {}: {}", guild_id, e);
            }
        }
    }
    data.database.update_user_level_and_exp(guild_id.get() as i64, user_id.get() as i64, current_level, new_exp).await?;
    Ok(())
}

fn calculate_required_exp(level: i32) -> i32 {
    (10.0 * (1.5f64.powi(level - 1))).round() as i32
}

async fn handle_reaction_add(ctx: &Context, reaction: &Reaction) -> Result<(), Error> {
    let user = match reaction.user(&ctx.http).await {
        Ok(u) => u,
        Err(e) => {
            error!("Failed to get user for reaction: {}", e);
            return Ok(());
        }
    };
    if user.bot { return Ok(()); }

    // Log analytics event
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

        // Handle star reaction
        if reaction.emoji == ReactionType::Unicode("⭐".to_string()) {
            // Assuming get_reaction_log_channel returns Option<i64>
            if let Some(star_channel_id_i64) = data.database.get_reaction_log_channel(guild_id.get() as i64, "star").await? {
                let star_channel = ChannelId::new(star_channel_id_i64 as u64); // Cast i64 to u64
                match reaction.channel_id.message(&ctx.http, reaction.message_id).await {
                    Ok(message) => {
                        let embed = CreateEmbed::default()
                            .title("You're a Star! ⭐")
                            .description(&message.content)
                            .author(CreateEmbedAuthor::new(&message.author.name).icon_url(message.author.face()))
                            .footer(CreateEmbedFooter::new(format!("Original message ID: {}", reaction.message_id)))
                            .timestamp(message.timestamp);
                        let embed = if let Some(att) = message.attachments.first() { if att.width.is_some() { embed.image(&att.url) } else { embed } } else { embed };
                        if let Err(e) = star_channel.send_message(&ctx.http, CreateMessage::default().add_embed(embed)).await {
                            error!("Failed to send star message to channel {}: {}", star_channel_id_i64, e); // Log i64
                        }
                    },
                    Err(e) => error!("Failed to fetch original message {} for star reaction: {}", reaction.message_id, e),
                }
            }
        } else {
            // Log other reactions using the correct guild_settings column
            let guild_service = GuildService::new(data.database.clone());
            match guild_service.get_guild_settings(guild_id.get() as i64).await {
                Ok(settings) => {
                    // reaction_log_channel_id is Option<String> in GuildSettings
                    if let Some(log_channel_id_str) = settings.reaction_log_channel_id {
                        // Parse the string ID to u64
                        match log_channel_id_str.parse::<u64>() {
                            Ok(log_channel_id_u64) => {
                                debug!("Found reaction log channel {} for guild {}", log_channel_id_u64, guild_id);
                                let reaction_log_channel = ChannelId::new(log_channel_id_u64);
                                let emoji_name = match &reaction.emoji {
                                    ReactionType::Custom { id, name, .. } => name.as_ref().map_or_else(|| id.to_string(), |s| s.clone()),
                                    ReactionType::Unicode(s) => s.clone(),
                                    _ => "Unknown Emoji".to_string(),
                                };
                                let timestamp = chrono::Utc::now().timestamp();
                                let embed = CreateEmbed::default()
                                    .title("Reaction Added")
                                    .description(format!("{} reacted with {} at <t:{}:F>", user.name, emoji_name, timestamp))
                                    .footer(CreateEmbedFooter::new(format!("Guild: {}", guild_name)))
                                    .color(0x00FF00);
                                match reaction_log_channel.send_message(&ctx.http, CreateMessage::default().add_embed(embed)).await {
                                    Ok(_) => debug!("Successfully sent reaction log to channel {}", log_channel_id_u64),
                                    Err(e) => error!("Failed to send reaction log to channel {}: {}", log_channel_id_u64, e),
                                }
                            },
                            Err(_) => {
                                error!("Failed to parse reaction_log_channel_id '{}' as u64 for guild {}", log_channel_id_str, guild_id);
                            }
                        }
                    } else {
                        debug!("Reaction log channel not configured for guild {}", guild_id);
                    }
                },
                Err(e) => error!("Failed to fetch guild settings for reaction log in guild {}: {}", guild_id, e),
            }
        }
    }
    Ok(())
}

async fn process_url_rule(ctx: &Context, data: &Data, message: &Message) -> Result<(), Error> {
    if let Some(guild_id) = message.guild_id {
        // TODO: This still uses the old get_url_rule which might query the wrong table/column
        // Need to confirm if url_rule is stored in guild_settings.settings JSONB or a dedicated column
        if let Some(rule) = data.database.get_url_rule(guild_id.get() as i64, message.channel_id.get() as i64).await? {
            let re = Regex::new(&rule.regex).map_err(|_| Error::Unknown("Invalid regex pattern".into()))?;
            if let Some(captures) = re.captures(&message.content) {
                let mut output = rule.output_template.clone();
                for (i, capture) in captures.iter().enumerate().skip(1) {
                    if let Some(c) = capture {
                        output = output.replace(&format!("${}", i), c.as_str());
                    }
                }
                if let Err(e) = message.channel_id.say(&ctx.http, &output).await {
                    error!("Failed to send URL rule message: {}", e);
                }
            }
        }
    }
    Ok(())
}
