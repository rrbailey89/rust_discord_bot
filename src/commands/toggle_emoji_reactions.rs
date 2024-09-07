//toggle_emoji_reactions.rs
use crate::error::Error;
use crate::Data;


type Context<'a> = poise::Context<'a, Data, Error>;

/// Toggle emoji reactions for this server
#[poise::command(slash_command, guild_only, required_permissions = "MANAGE_GUILD")]
pub async fn toggleemojireactions(
    ctx: Context<'_>,
    #[description = "Enable or disable emoji reactions"] enable: Option<bool>,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get() as i64;

    // Fetch the current state
    let current_state = ctx.data().database.fetch_emoji_reactions_enabled(guild_id).await?;

    match enable {
        Some(new_state) => {
            // Update the state only if it's different from the current state
            if new_state != current_state {
                ctx.data().database.store_emoji_reactions_enabled(guild_id, new_state).await?;
                let status = if new_state { "enabled" } else { "disabled" };
                ctx.say(format!("Emoji reactions have been {} for this server.", status)).await?;
            } else {
                let status = if new_state { "enabled" } else { "disabled" };
                ctx.say(format!("Emoji reactions were already {} for this server.", status)).await?;
            }
        },
        None => {
            // If no argument is provided, just report the current state
            let status = if current_state { "enabled" } else { "disabled" };
            ctx.say(format!("Emoji reactions are currently {} for this server.", status)).await?;
        }
    }

    Ok(())
}