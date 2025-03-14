use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};

type Context<'a> = poise::Context<'a, Data, Error>;

/// List your upcoming unavailabilities
#[poise::command(slash_command, guild_only)]
pub async fn listunavailable(
    ctx: Context<'_>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("This command can only be used in a server".to_string()))?;
    
    // Get the user's unavailability records
    let records = ctx.data().database.get_user_unavailability(
        guild_id.get() as i64,
        ctx.author().id.get() as i64,
    ).await?;
    
    if records.is_empty() {
        ctx.send(poise::CreateReply::default()
            .content("You don't have any upcoming unavailabilities.")
            .ephemeral(true)
        ).await?;
        return Ok(());
    }
    
    // Create a list of unavailabilities
    let mut description = String::new();
    for (id, date, reason) in records {
        let date_str = date.format("%Y-%m-%d").to_string();
        let reason_str = reason.as_deref().unwrap_or("No reason provided");
        description.push_str(&format!("**ID: {}** - Date: {} - Reason: {}\n\n", id, date_str, reason_str));
    }
    
    // Create and send the embed
    let embed = CreateEmbed::default()
        .title("Your Upcoming Unavailabilities")
        .description(description)
        .footer(CreateEmbedFooter::new("Use /cancelunavailable [ID] to cancel an unavailability"))
        .color(0x3498DB); // Blue color
    
    ctx.send(poise::CreateReply::default()
        .embed(embed)
        .ephemeral(true)
    ).await?;
    
    Ok(())
}
