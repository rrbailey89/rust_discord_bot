use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{
    ButtonStyle as SerenityButtonStyle, ChannelId, CreateActionRow, CreateButton, MessageId, RoleId,
    Context as SerenityContext, ComponentInteraction, EditMessage, InteractionResponseFlags,
    CreateInteractionResponse, CreateInteractionResponseMessage, ActionRowComponent,
    ButtonKind,
};
use std::str::FromStr;
use poise::CreateReply;

type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(poise::ChoiceParameter, Clone, Copy)]
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
    #[description = "Comma-separated list of role IDs"] roles: String,
    #[description = "Comma-separated list of button labels"] labels: String,
    #[description = "Comma-separated list of button styles (Primary, Secondary, Success, Danger)"] styles: String,
) -> Result<(), Error> {
    let message = channel.message(&ctx.http(), message_id).await?;

    let role_ids: Vec<RoleId> = roles
        .split(',')
        .map(|id| RoleId::from_str(id.trim()).map_err(|_| Error::Unknown("Invalid role ID".to_string())))
        .collect::<Result<Vec<_>, _>>()?;

    let button_labels: Vec<String> = labels.split(',').map(|s| s.trim().to_string()).collect();
    let button_styles: Vec<ButtonStyle> = styles
        .split(',')
        .map(|s| match s.trim().to_lowercase().as_str() {
            "primary" => Ok(ButtonStyle::Primary),
            "secondary" => Ok(ButtonStyle::Secondary),
            "success" => Ok(ButtonStyle::Success),
            "danger" => Ok(ButtonStyle::Danger),
            _ => Err(Error::Unknown("Invalid button style".to_string())),
        })
        .collect::<Result<Vec<_>, _>>()?;

    if role_ids.len() != button_labels.len() || role_ids.len() != button_styles.len() {
        return Err(Error::Unknown("The number of roles, labels, and styles must be the same".to_string()));
    }

    let mut new_buttons = Vec::new();
    for ((role, label), style) in role_ids.iter().zip(button_labels.iter()).zip(button_styles.iter()) {
        let custom_id = format!("role_button:{}", role);
        let button = CreateButton::new(custom_id)
            .label(label)
            .style((*style).into());
        new_buttons.push(button);
    }

    let mut components = Vec::new();

    // Add existing components
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

    // Add the new buttons
    components.push(CreateActionRow::Buttons(new_buttons));

    channel.edit_message(&ctx.http(), message_id, EditMessage::new().components(components)).await?;

    ctx.say("Role assignment buttons added successfully!").await?;

    Ok(())
}
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
    if let Some(role_id_str) = custom_id.strip_prefix("role_button:") {
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
                    .content(message)
                    .flags(InteractionResponseFlags::EPHEMERAL)
            ))
            .await?;

        // Log the role change
        println!("Role {} {} for user {}", role_id, action, interaction.user.id);
    }

    Ok(())
}