use crate::{error::Error, Data};
use poise::serenity_prelude::{CreateActionRow, CreateSelectMenu, CreateSelectMenuOption, CreateSelectMenuKind,
    ComponentInteractionDataKind, CreateInteractionResponse, CreateInteractionResponseMessage};
use reqwest;
use serde_json::Value;

fn format_name(name: &str) -> String {
    name.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

fn format_occupation(occupation: &[String]) -> String {
    occupation.iter()
        .map(|job| job.replace('_', " "))
        .collect::<Vec<String>>()
        .join(", ")
}

/// Check if a selected person is alive
#[poise::command(slash_command, guild_only)]
pub async fn lifecheck(ctx: poise::Context<'_, Data, Error>) -> Result<(), Error> {
    let people = vec![
        "Pierce Brosnan",
        "Keanu Reeves",
    ];

    let options: Vec<CreateSelectMenuOption> = people
        .iter()
        .map(|&name| CreateSelectMenuOption::new(name, name))
        .collect();

    let select_menu = CreateSelectMenu::new(
        "person_select",
        CreateSelectMenuKind::String { options }
    ).placeholder("Select a person");

    let action_row = CreateActionRow::SelectMenu(select_menu);

    let message = ctx.send(poise::CreateReply::default()
        .content("Select a person to check if they're alive:")
        .components(vec![action_row]))
        .await?;

    if let Some(interaction) = message
        .message()
        .await?
        .await_component_interaction(ctx)
        .author_id(ctx.author().id)
        .timeout(std::time::Duration::from_secs(60))
        .await
    {
        let selected_name = match &interaction.data.kind {
            ComponentInteractionDataKind::StringSelect { values } => values.first(),
            _ => None,
        };

        if let Some(selected_name) = selected_name {
            let api_key = &ctx.data().config.api_ninjas_key;
            let client = reqwest::Client::new();
            let url = format!("https://api.api-ninjas.com/v1/celebrity?name={}", selected_name);

            let response = client.get(&url)
                .header("X-Api-Key", api_key)
                .send()
                .await?;

            let status = response.status();
            let body = response.text().await?;

            if !status.is_success() {
                println!("API Error: Status: {}, Body: {}", status, body);
                interaction.create_response(ctx.serenity_context(), CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content(format!("Sorry, I couldn't retrieve information for {}. Please try again later.", selected_name))
                )).await?;
                return Ok(());
            }

            let json: Vec<Value> = serde_json::from_str(&body)
                .map_err(|e| Error::Unknown(format!("Failed to parse JSON: {}. Body: {}", e, body)))?;

            if let Some(celebrity) = json.first() {
                let name = format_name(celebrity["name"].as_str().unwrap_or(selected_name));
                let is_alive = celebrity["is_alive"].as_bool().unwrap_or(false);
                let age = celebrity["age"].as_u64().unwrap_or(0);
                let occupation = celebrity["occupation"].as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<String>>())
                    .unwrap_or_else(Vec::new);

                let status = if is_alive { "alive" } else { "not alive" };
                let response = format!(
                    "{} is currently {}. They are {} years old and known as {}.",
                    name,
                    status,
                    age,
                    format_occupation(&occupation)
                );

                interaction.create_response(ctx.serenity_context(), CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().content(response)
                )).await?;
            } else {
                interaction.create_response(ctx.serenity_context(), CreateInteractionResponse::UpdateMessage(
                    CreateInteractionResponseMessage::new().content(format!("Sorry, I couldn't find any information for {}.", selected_name))
                )).await?;
            }
        } else {
            interaction.create_response(ctx.serenity_context(), CreateInteractionResponse::UpdateMessage(
                CreateInteractionResponseMessage::new().content("Error: No selection made.")
            )).await?;
        }

        // Remove the components (modal) from the original message
        message.edit(ctx, poise::CreateReply::default().components(Vec::new())).await?;
    } else {
        ctx.say("You didn't select a person in time.").await?;
    }

    Ok(())
}