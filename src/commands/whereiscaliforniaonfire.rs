use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};

type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
struct FireInfo {
    name: String,
    url: String,
    counties: String,
    acres: String,
    containment: String,
}

/// Check where California is currently on fire
#[poise::command(slash_command)]
pub async fn whereiscaliforniaonfire(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    let client = reqwest::Client::new();
    let response = client
        .get("http://whereiscaliforniaonfire.com")
        .send()
        .await?
        .text()
        .await?;

    // Extract update time
    let updated = response
        .split("Updated on ")
        .nth(1)
        .and_then(|s| s.split("<p>").next())
        .unwrap_or("Unknown time");

    // Parse fire information from the table
    let fires: Vec<FireInfo> = response
        .split("<tr>")
        .skip(3) // Skip header rows
        .filter(|row| row.contains("</td>"))
        .map(|row| {
            let cols: Vec<&str> = row.split("</td>").collect();

            // Extract URL and name from first column
            let name_cell = cols[0];
            let url = name_cell
                .split("href=\"")
                .nth(1)
                .and_then(|s| s.split("\"").next())
                .unwrap_or("");
            let name = name_cell
                .split("\">")
                .nth(1)
                .and_then(|s| s.split("</a>").next())
                .unwrap_or("");

            // Extract other columns
            let counties = cols[1]
                .split(">")
                .last()
                .unwrap_or("")
                .trim();
            let acres = cols[2]
                .split(">")
                .last()
                .unwrap_or("")
                .trim();
            let containment = cols[3]
                .split(">")
                .last()
                .unwrap_or("")
                .trim();

            FireInfo {
                name: name.to_string(),
                url: url.to_string(),
                counties: counties.to_string(),
                acres: acres.to_string(),
                containment: containment.to_string(),
            }
        })
        .collect();

    // Create the embed description
    let mut description = String::new();
    for fire in fires {
        description.push_str(&format!(
            "**[{}](<{}>)**\n• Counties: {}\n• Acres Burned: {}\n• Containment: {}\n\n",
            fire.name,
            fire.url,
            fire.counties,
            fire.acres,
            if fire.containment.is_empty() { "Unknown" } else { &fire.containment }
        ));
    }

    let embed = CreateEmbed::default()
        .title("Active Fires in California")
        .description(description)
        .color(0xFF4500) // Orange-red color
        .footer(CreateEmbedFooter::new(format!("Last Updated: {}", updated)));

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}