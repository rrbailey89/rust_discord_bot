use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{CreateEmbed, Mentionable, User};
use serde_json::Value;
use tracing::{error};
use reqwest::Client;

type Context<'a> = poise::Context<'a, Data, Error>;

lazy_static::lazy_static! {
    static ref CLIENT: Client = Client::new();
}

/// Hug another user with an anime gif
#[poise::command(slash_command, category = "Fun")]
pub async fn animehug(
    ctx: Context<'_>,
    #[description = "User to hug"] user: User,
) -> Result<(), Error> {
    ctx.defer().await?;

    let response = CLIENT.get("https://nekos.best/api/v2/hug")
        .header("User-Agent", "Blame_Serena Discord Bot (https://github.com/rrbailey89/rust_discord_bot)")
        .send()
        .await?;

    if !response.status().is_success() {
        error!("API request failed with status: {}", response.status());
        return Err(Error::Unknown("Failed to fetch hug gif. Please try again later.".to_string()));
    }

    let json: Value = response.json().await.map_err(|e| {
        error!("Failed to parse JSON: {}", e);
        Error::Unknown("Failed to parse API response. Please try again later.".to_string())
    })?;

    let anime_name = json["results"][0]["anime_name"].as_str().unwrap_or("Unknown Anime");
    let image_url = json["results"][0]["url"].as_str().unwrap_or("");

    if image_url.is_empty() {
        error!("Failed to get image URL from API response");
        return Err(Error::Unknown("Failed to get image URL from API response".to_string()));
    }

    let embed = CreateEmbed::default()
        .title(format!("From the Anime: {}", anime_name))
        .image(image_url)
        .color(0x5865F2);

    let hug_count = ctx.data().database.increment_hug_count(user.id.get() as i64).await?;

    let response_text = format!(
        "{} was hugged by {}. They have been hugged {} times.",
        user.mention(),
        ctx.author().mention(),
        hug_count
    );

    ctx.send(
        poise::CreateReply::default()
            .content(response_text)
            .embed(embed)
            .allowed_mentions(poise::serenity_prelude::CreateAllowedMentions::default().users(vec![user.id]))
    ).await?;

    Ok(())
}