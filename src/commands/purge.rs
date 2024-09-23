use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::GetMessages;
use poise::CreateReply;

type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, guild_only, default_member_permissions = "MANAGE_MESSAGES")]
pub async fn purge(
    ctx: Context<'_>,
    #[description = "Number of messages to delete"]
    #[min = 2]
    #[max = 100]
    count: u64,
) -> Result<(), Error> {
    if count == 0 || count > 100 {
        return Err(Error::Unknown("Please provide a number between 1 and 100".to_string()));
    }

    let channel_id = ctx.channel_id();
    let messages = channel_id.messages(&ctx.http(), GetMessages::default().limit(count as u8)).await?;

    channel_id.delete_messages(&ctx.http(), &messages).await?;

    let reply = CreateReply::default()
        .content(format!("Successfully deleted {} messages.", messages.len()))
        .ephemeral(true);

    ctx.send(reply).await?;

    Ok(())
}
