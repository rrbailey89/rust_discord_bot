use crate::error::Error;
use crate::utils::parse_datetime;
use crate::Data;
use chrono::Utc;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage, Role};

type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(poise::ChoiceParameter, Debug)]
pub enum Month {
    January, February, March, April, May, June, July,
    August, September, October, November, December
}

#[derive(poise::ChoiceParameter)]
pub enum Year {
    #[name = "2025"] Y2025 = 2025,
    #[name = "2026"] Y2026 = 2026,
    #[name = "2027"] Y2027 = 2027,
    #[name = "2028"] Y2028 = 2028,
}

impl std::fmt::Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Mark yourself as unavailable on a specific date
#[poise::command(slash_command, guild_only)]
pub async fn unavailable(
    ctx: Context<'_>,
    #[description = "Select the month"] month: Month,
    #[description = "Enter the day (1-31)"] day: i64,
    #[description = "Select the year"] year: Year,
    #[description = "Reason for unavailability (optional)"] reason: Option<String>,
    #[description = "Mention Role (optional)"] role_to_mention: Option<Role>,
) -> Result<(), Error> {
    // Defer the response to avoid timeout
    ctx.defer_ephemeral().await?;
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("This command can only be used in a server".to_string()))?;
    
    // We'll use the parse_datetime function to create a timestamp
    // Since we only care about the date (not time), we'll use a fixed time and timezone
    let datetime = parse_datetime(&month.to_string(), day, year as i64, "12:00 PM", "America/Los_Angeles")?;
    let unix_timestamp = datetime.timestamp();
    let unavailable_date = datetime.date_naive();
    
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
    let member = ctx.author_member().await.ok_or_else(|| Error::Unknown("Failed to get member data".to_string()))?;
    let display_name = member.nick.as_deref().unwrap_or(&ctx.author().name);
    
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
    
    // Create message with embed and optional role mention
    let mut message = CreateMessage::default().add_embed(embed);
    
    // Add role mention to the message content if provided
    if let Some(role) = role_to_mention {
        message = message.content(format!("<@&{}>", role.id));
    }
    
    unavailability_channel.send_message(&ctx.serenity_context().http, message).await?;
    
    // Send ephemeral confirmation to the user
    ctx.send(poise::CreateReply::default()
        .content("✅ Your unavailability has been posted.")
        .ephemeral(true)
    ).await?;
    
    Ok(())
}
