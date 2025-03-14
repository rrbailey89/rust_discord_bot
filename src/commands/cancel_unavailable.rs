use crate::error::Error;
use crate::Data;
use chrono::Utc;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, CreateMessage};

type Context<'a> = poise::Context<'a, Data, Error>;

/// Cancel a previously posted unavailability
#[poise::command(slash_command, guild_only)]
pub async fn cancelunavailable(
    ctx: Context<'_>,
    #[description = "ID of the unavailability to cancel"] unavailability_id: i32,
) -> Result<(), Error> {
    // Defer the response to avoid timeout
    ctx.defer_ephemeral().await?;
    
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("This command can only be used in a server".to_string()))?;
    
    // Attempt to delete the unavailability record
    let success = ctx.data().database.delete_user_unavailability(
        guild_id.get() as i64,
        ctx.author().id.get() as i64,
        unavailability_id,
    ).await?;
    
    if success {
        // Get the unavailability channel
        let unavailability_channel_id = match ctx.data().database.fetch_unavailability_channel(guild_id.get() as i64).await? {
            Some(channel_id) => channel_id,
            None => return Err(Error::Unknown("No unavailability channel has been set for this server.".to_string())),
        };
        
        // Get the user's nickname or username
        let member = ctx.author_member().await.ok_or_else(|| Error::Unknown("Failed to get member data".to_string()))?;
        let display_name = member.nick.as_deref().unwrap_or(&ctx.author().name);
        
        // Create cancellation notice embed
        let embed = CreateEmbed::default()
            .title("Unavailability Cancelled")
            .author(CreateEmbedAuthor::new(display_name).icon_url(ctx.author().face()))
            .description(format!("<@{}> has cancelled their unavailability.", ctx.author().id))
            .color(0x00FF00) // Green color
            .footer(CreateEmbedFooter::new(format!("Cancelled on {}", Utc::now().format("%Y-%m-%d"))));
        
        // Send the embed to the unavailability channel
        let unavailability_channel = poise::serenity_prelude::ChannelId::new(unavailability_channel_id as u64);
        unavailability_channel.send_message(&ctx.serenity_context().http, 
            CreateMessage::default().add_embed(embed)
        ).await?;
        
        // Send ephemeral confirmation to the user
        ctx.send(poise::CreateReply::default()
            .content("✅ Your unavailability has been cancelled.")
            .ephemeral(true)
        ).await?;
        
        Ok(())
    } else {
        // If no record was deleted, return an error
        Err(Error::Unknown("Could not find an unavailability with that ID, or you don't have permission to cancel it.".to_string()))
    }
}
