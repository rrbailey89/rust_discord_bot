// commands/random_capy_image.rs
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{CreateEmbed};
use poise::CreateReply;

type Context<'a> = poise::Context<'a, Data, Error>;

/// Get a random capybara image
#[poise::command(slash_command)]
pub async fn randomcapyimage(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let client = reqwest::Client::new();
    let response = client.get("https://api.capy.lol/v1/capybara?json=true")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let data = response["data"].as_object()
        .ok_or_else(|| Error::Unknown("Failed to get data object".to_string()))?;
    let image_url = data["url"].as_str()
        .ok_or_else(|| Error::Unknown("Failed to get image URL".to_string()))?;
    let alt_text = data["alt"].as_str().unwrap_or("Random Capybara");

    let embed = CreateEmbed::new()
        .title(alt_text)
        .image(image_url);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}