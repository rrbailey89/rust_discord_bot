use crate::error::Error;
use crate::Data;
use chrono::{NaiveDate, Utc};
use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};

type Context<'a> = poise::Context<'a, Data, Error>;

/// Mark yourself as unavailable on a specific date
#[poise::command(slash_command, guild_only)]
pub async fn unavailable(
    ctx: Context<'_>,
    #[description = "Month (e.g., January, February)"] month: String,
    #[description = "Day (1-31)"] day: i64,
    #[description = "Year (e.g., 2025)"] year: i64,
    #[description = "Reason for unavailability (optional)"] reason: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("This command can only be used in a server".to_string()))?;
    
    // Validate the date
    let month_num = match month.to_lowercase().as_str() {
        "january" | "jan" => 1,
        "february" | "feb" => 2,
        "march" | "mar" => 3,
        "april" | "apr" => 4,
        "may" => 5,
        "june" | "jun" => 6,
        "july" | "jul" => 7,
        "august" | "aug" => 8,
        "september" | "sep" => 9,
        "october" | "oct" => 10,
        "november" | "nov" => 11,
        "december" | "dec" => 12,
        _ => return Err(Error::Unknown("Invalid month. Please use full month name or 3-letter abbreviation.".to_string())),
    };
    
    // Validate the date is valid
    let unavailable_date = NaiveDate::from_ymd_opt(year as i32, month_num, day as u32)
        .ok_or_else(|| Error::Unknown("Invalid date. Please check that the day exists for the given month and year.".to_string()))?;
    
    // Get the unavailability channel
    let unavailability_channel_id = match ctx.data().database.fetch_unavailability_channel(guild_id.get() as i64).await? {
        Some(channel_id) => channel_id,
        None => return Err(Error::Unknown("No unavailability channel has been set for this server. Please ask an admin to set one using /setunavailabilitychannel.".to_string())),
    };
    
    // Store the unavailability record
    ctx.data().database.store_user_unavailability(
        guild_id.get() as i64,
        ctx.author().id.get() as i64,
        unavailable_date,
        reason.clone(),
    ).await?;
    
    // Get the user's nickname or username
    let guild = ctx.guild().ok_or_else(|| Error::Unknown("Failed to get guild".to_string()))?;
    let member = guild.member(&ctx.serenity_context().http, ctx.author().id).await?;
    let display_name = member.nick.as_deref().unwrap_or(&ctx.author().name);
    
    // Create the embed
    let unix_timestamp = unavailable_date.and_hms_opt(0, 0, 0)
        .ok_or_else(|| Error::Unknown("Failed to create timestamp".to_string()))?
        .timestamp();
    
    let mut embed = CreateEmbed::default()
        .title("Unavailability Notice")
        .author(CreateEmbedAuthor::new(display_name).icon_url(ctx.author().face()))
        .field("User", format!("<@{}>", ctx.author().id), true)
        .field("Date", format!("<t:{}:D>", unix_timestamp), true)
        .color(0xFF9900) // Orange color
        .footer(CreateEmbedFooter::new(format!("Posted on {}", Utc::now().format("%Y-%m-%d"))));
    
    if let Some(reason_text) = reason {
        embed = embed.field("Reason", reason_text, false);
    }
    
    // Send the embed to the unavailability channel
    let unavailability_channel = poise::serenity_prelude::ChannelId::new(unavailability_channel_id as u64);
    unavailability_channel.send_message(&ctx.serenity_context().http, 
        CreateMessage::default().add_embed(embed)
    ).await?;
    
    // Send ephemeral confirmation to the user
    ctx.send(poise::CreateReply::default()
        .content("✅ Your unavailability has been posted.")
        .ephemeral(true)
    ).await?;
    
    Ok(())
}
