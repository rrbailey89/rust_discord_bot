use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};

type Context<'a> = poise::Context<'a, Data, Error>;

/// Check if California is currently on fire
#[poise::command(slash_command)]
pub async fn iscaliforniaonfire(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let client = reqwest::Client::new();
    let response = client
        .get("http://iscaliforniaonfire.com")
        .send()
        .await?
        .text()
        .await?;

    // Extract the answer (text between <h1> tags)
    let answer = response
        .split("<h1>")
        .nth(1)
        .and_then(|s| s.split("</h1>").next())
        .unwrap_or("Unknown");

    // Extract the update time (text after "updated: ")
    let updated = response
        .split("updated: ")
        .nth(1)
        .and_then(|s| s.split("</body>").next())
        .unwrap_or("Unknown time");

    // Create an embed with appropriate colors based on the answer
    let color = if answer == "Yes" { 0xFF0000 } else { 0x00FF00 }; // Red for Yes, Green for No

    let embed = CreateEmbed::default()
        .title("Is California On Fire?")
        .description(answer)
        .color(color)
        .footer(CreateEmbedFooter::new(format!("Last Updated: {}", updated)));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}