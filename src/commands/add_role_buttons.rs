use crate::error::Error;
use crate::Data;
use poise::{
    serenity_prelude::{
        ActionRowComponent, ButtonKind, ButtonStyle as SerenityButtonStyle, ChannelId,
        ComponentInteraction, Context as SerenityContext, CreateActionRow, CreateButton,
        CreateInteractionResponse, CreateInteractionResponseMessage, EditMessage, EmojiId,
        InteractionResponseFlags, MessageId, ReactionType, RoleId,
    },
    CreateReply,
};
use std::str::FromStr;
use serde::{Serialize, Deserialize};
use crate::types::DataContainer;
use serde_json::Value;
use tracing::{info, debug, error};

type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(poise::ChoiceParameter, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ButtonStyle {
    Primary,
    Secondary,
    Success,
    Danger,
}

impl From<ButtonStyle> for SerenityButtonStyle {
    fn from(style: ButtonStyle) -> Self {
        match style {
            ButtonStyle::Primary => Self::Primary,
            ButtonStyle::Secondary => Self::Secondary,
            ButtonStyle::Success => Self::Success,
            ButtonStyle::Danger => Self::Danger,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ButtonInfo {
    label: String,
    style: ButtonStyle,
    emoji: Option<String>,
    role_id: Option<String>,
    nested_buttons: Option<Vec<ButtonInfo>>,
}

/// Add or remove role assignment buttons to a message
#[poise::command(slash_command, subcommands("add", "remove"))]
pub async fn rolebuttons(_ctx: poise::Context<'_, Data, Error>) -> Result<(), Error> {
    Ok(())
}

/// Add multiple role assignment buttons to a message
#[poise::command(slash_command, required_permissions = "MANAGE_ROLES")]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Channel where the message is located"] channel: ChannelId,
    #[description = "ID of the message to add buttons to"] message_id: MessageId,
    #[description = "JSON string of button configurations"] button_config: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("This command can only be used in a server".to_string()))?;

    let button_info: Vec<ButtonInfo> = serde_json::from_str(&button_config)
        .map_err(|e| Error::Unknown(format!("Invalid button configuration: {}", e)))?;

    // Validate JSON by parsing it
    let _: Value = serde_json::from_str(&button_config)
        .map_err(|e| Error::Unknown(format!("Invalid button configuration: {}", e)))?;

    ctx.data().database.create_button_config(
        guild_id.get() as i64,
        message_id.get() as i64,
        &button_config
    ).await?;

    let message = channel.message(&ctx.http(), message_id).await?;
    let mut components = Vec::new();

    for row in &message.components {
        let mut new_row_components = Vec::new();
        for component in &row.components {
            if let ActionRowComponent::Button(button) = component {
                if let ButtonKind::NonLink { custom_id, style } = &button.data {
                    let mut new_button = CreateButton::new(custom_id)
                        .label(button.label.clone().unwrap_or_default())
                        .style(*style)
                        .disabled(button.disabled);

                    if let Some(emoji) = &button.emoji {
                        new_button = new_button.emoji(emoji.clone());
                    }

                    new_row_components.push(new_button);
                }
            }
        }
        if !new_row_components.is_empty() {
            components.push(CreateActionRow::Buttons(new_row_components));
        }
    }

    let new_buttons = create_buttons(&button_info, 0)?;
    components.extend(new_buttons);

    channel.edit_message(&ctx.http(), message_id, EditMessage::new().components(components)).await?;

    ctx.say("Role assignment buttons added successfully!").await?;

    Ok(())
}

fn create_buttons(button_info: &[ButtonInfo], depth: usize) -> Result<Vec<CreateActionRow>, Error> {
    let mut rows = Vec::new();
    let mut current_row = Vec::new();

    for (i, button) in button_info.iter().enumerate() {
        let custom_id = if button.nested_buttons.is_some() {
            format!("nested_button:{}:{}", depth, i)
        } else if let Some(role_id) = &button.role_id {
            format!("role_button:{}:{}", depth, role_id)
        } else {
            return Err(Error::Unknown("Button must have either nested_buttons or role_id".to_string()));
        };

        let mut new_button = CreateButton::new(custom_id)
            .label(&button.label)
            .style(button.style.into());

        if let Some(emoji) = &button.emoji {
            new_button = new_button.emoji(parse_emoji(emoji)?);
        }

        current_row.push(new_button);

        if current_row.len() == 5 || i == button_info.len() - 1 {
            rows.push(CreateActionRow::Buttons(current_row.clone()));
            current_row.clear();
        }
    }

    Ok(rows)
}

/// Remove all buttons from a message
#[poise::command(slash_command)]
pub async fn remove(
    ctx: poise::Context<'_, Data, Error>,
    #[description = "Select the channel"] channel: ChannelId,
    #[description = "Message ID"] message_id: String,
) -> Result<(), Error> {
    let message_id = message_id.parse::<u64>().map_err(|e| Error::Unknown(e.to_string()))?;
    let message_id = MessageId::new(message_id);

    let mut message = ctx.http().get_message(channel, message_id).await?;

    message.edit(&ctx.http(), EditMessage::new().components(vec![])).await?;

    ctx.send(CreateReply::default().content("All buttons have been removed from the message.").ephemeral(true)).await?;

    Ok(())
}

pub async fn handle_role_button(
    ctx: &SerenityContext,
    interaction: &ComponentInteraction,
) -> Result<(), Error> {
    let custom_id = &interaction.data.custom_id;
    let parts: Vec<&str> = custom_id.split(':').collect();

    match parts[0] {
        "nested_button" => {
            let depth: usize = parts[1].parse().map_err(|_| Error::Unknown("Invalid depth".to_string()))?;
            let button_index: usize = parts[2].parse().map_err(|_| Error::Unknown("Invalid button index".to_string()))?;

            let nested_buttons = fetch_nested_buttons(ctx, interaction, depth, button_index).await?;

            if !nested_buttons.is_empty() {
                let buttons_to_display = if depth == 0 {
                    // If we're at the root level, we want to display the nested buttons of the first (and only) button
                    nested_buttons[0].nested_buttons.as_ref()
                        .ok_or_else(|| Error::Unknown("No nested buttons found".to_string()))?
                } else {
                    &nested_buttons
                };

                let new_buttons = create_buttons(buttons_to_display, depth + 1)?;

                interaction
                    .create_response(&ctx.http, CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("Here are your options:")
                            .components(new_buttons)
                            .flags(InteractionResponseFlags::EPHEMERAL)
                    ))
                    .await?;
            } else {
                // If no nested buttons are found, inform the user
                interaction
                    .create_response(&ctx.http, CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .content("No options available for this button.")
                            .flags(InteractionResponseFlags::EPHEMERAL)
                    ))
                    .await?;
            }
        }
        "role_button" => {
            handle_role_assignment(ctx, interaction, parts[2]).await?;
        }
        _ => return Err(Error::Unknown("Invalid button type".to_string())),
    }

    Ok(())
}

async fn handle_role_assignment(
    ctx: &SerenityContext,
    interaction: &ComponentInteraction,
    role_id_str: &str,
) -> Result<(), Error> {
    let role_id = RoleId::from_str(role_id_str).map_err(|_| Error::Unknown("Invalid role ID".to_string()))?;
    let guild_id = interaction.guild_id.ok_or_else(|| Error::Unknown("Not in a guild".to_string()))?;
    let member = guild_id.member(&ctx.http, interaction.user.id).await?;

    let (action, message) = if member.roles.contains(&role_id) {
        member.remove_role(&ctx.http, role_id).await?;
        ("removed", format!("Role <@&{}> removed", role_id))
    } else {
        member.add_role(&ctx.http, role_id).await?;
        ("added", format!("Role <@&{}> added", role_id))
    };

    interaction
        .create_response(&ctx.http, CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(&message)
                .flags(InteractionResponseFlags::EPHEMERAL)
        ))
        .await?;

    info!("Role {} {} for user {}", role_id, action, interaction.user.id);

    Ok(())
}

async fn fetch_nested_buttons(
    ctx: &SerenityContext,
    interaction: &ComponentInteraction,
    depth: usize,
    button_index: usize,
) -> Result<Vec<ButtonInfo>, Error> {
    let guild_id = interaction.guild_id.ok_or_else(|| Error::Unknown("Not in a guild".to_string()))?;
    let message_id = interaction.message.id;

    let data_lock = ctx.data.read().await;
    let data = data_lock
        .get::<DataContainer>()
        .ok_or_else(|| Error::Unknown("Failed to get bot data".to_string()))?
        .clone();

    let nested_buttons_json = data
        .database
        .get_nested_buttons(
            guild_id.get() as i64,
            message_id.get() as i64,
            depth as i32,
            button_index as i32,
        )
        .await?;

    debug!("Retrieved nested buttons JSON: {:?}", nested_buttons_json);

    match nested_buttons_json {
        Some(json_value) => {
            let nested_buttons: Vec<ButtonInfo> = if depth == 0 {
                // For root level, parse the entire config
                json_value.as_array()
                    .ok_or_else(|| Error::Unknown("Root config is not an array".to_string()))?
                    .iter()
                    .filter_map(|button| serde_json::from_value(button.clone()).ok())
                    .collect()
            } else {
                // For nested levels, parse the nested buttons
                json_value.as_array()
                    .ok_or_else(|| Error::Unknown("Nested buttons are not an array".to_string()))?
                    .iter()
                    .filter_map(|button| serde_json::from_value(button.clone()).ok())
                    .collect()
            };
            debug!("Parsed nested buttons: {:?}", nested_buttons);
            Ok(nested_buttons)
        }
        None => {
            error!("No nested buttons found for depth {} and index {}", depth, button_index);
            Ok(vec![])
        }
    }
}

fn parse_emoji(emoji_str: &str) -> Result<ReactionType, Error> {
    if emoji_str.starts_with('<') && emoji_str.ends_with('>') {
        let content = &emoji_str[1..emoji_str.len()-1];
        let animated = content.starts_with("a:");
        let parts: Vec<&str> = content.split(':').collect();
        if animated {
            if parts.len() != 4 || parts[0] != "a" {
                return Err(Error::Unknown("Invalid animated emoji format".to_string()));
            }
            let name = parts[1].to_string();
            let id = parts[2].parse::<u64>().map_err(|_| Error::Unknown("Invalid emoji ID".to_string()))?;
            Ok(ReactionType::Custom {
                animated: true,
                id: EmojiId::new(id),
                name: Some(name),
            })
        } else {
            if parts.len() != 3 {
                return Err(Error::Unknown("Invalid emoji format".to_string()));
            }
            let name = parts[1].to_string();
            let id = parts[2].parse::<u64>().map_err(|_| Error::Unknown("Invalid emoji ID".to_string()))?;
            Ok(ReactionType::Custom {
                animated: false,
                id: EmojiId::new(id),
                name: Some(name),
            })
        }
    } else {
        Ok(ReactionType::Unicode(emoji_str.to_string()))
    }
}