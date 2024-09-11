// commands/anime_hug.rs
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{CreateEmbed, Mentionable, User};

type Context<'a> = poise::Context<'a, Data, Error>;

/// Hug another user with an anime gif
#[poise::command(slash_command, category = "Fun")]
pub async fn animehug(
    ctx: Context<'_>,
    #[description = "User to hug"] user: User,
) -> Result<(), Error> {
    ctx.defer().await?;

    let client = reqwest::Client::new();
    let response = client.get("https://nekos.best/api/v2/hug")
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    let result = &response["results"][0];
    let anime_name = result["anime_name"].as_str().unwrap_or("Unknown");
    let image_url = result["url"].as_str().unwrap_or("");

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

    ctx.send(poise::CreateReply::default()
        .content(response_text)
        .embed(embed)
        .allowed_mentions(poise::serenity_prelude::CreateAllowedMentions::default().users(vec![user.id])))
        .await?;

    Ok(())
}