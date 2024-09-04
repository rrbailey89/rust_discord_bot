// commands/random_cat_image.rs
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{CreateEmbed};

type Context<'a> = poise::Context<'a, Data, Error>;

/// Get a random cat image
#[poise::command(slash_command)]
pub async fn randomcatimage(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let client = reqwest::Client::new();
    let response = client.get("https://api.thecatapi.com/v1/images/search")
        .send()
        .await?
        .json::<Vec<serde_json::Value>>()
        .await?;

    let image_url = response[0]["url"].as_str()
        .ok_or_else(|| Error::Unknown("Failed to get image URL".to_string()))?;

    let embed = CreateEmbed::default()
        .title("Random Cat Image")
        .image(image_url);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}