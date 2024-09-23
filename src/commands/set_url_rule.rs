use crate::{error::Error, Data, types::UrlRule};
use poise::serenity_prelude::ChannelId;
use regex::Regex;

#[poise::command(slash_command, guild_only, default_member_permissions = "MANAGE_CHANNELS")]
pub async fn seturlrule(
    ctx: poise::Context<'_, Data, Error>,
    #[description = "Channel to apply the rule"] channel: ChannelId,
    #[description = "Regex pattern to match URLs"] regex: String,
    #[description = "Output template (use $1, $2, etc. for capture groups)"] output_template: String,
) -> Result<(), Error> {
    // Validate the regex
    Regex::new(&regex).map_err(|_| Error::Unknown("Invalid regex pattern".into()))?;

    let guild_id = ctx.guild_id().unwrap().get() as i64;
    let channel_id = channel.get() as i64;

    // Create the UrlRule struct
    let rule = UrlRule {
        guild_id,
        channel_id,
        regex,
        output_template,
    };

    // Store the rule in the database
    ctx.data().database.store_url_rule(&rule).await?;

    ctx.say("URL rule set successfully!").await?;

    Ok(())
}