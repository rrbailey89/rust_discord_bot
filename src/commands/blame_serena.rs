use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::UserId;

type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(poise::ChoiceParameter)]
pub enum BlameReason {
    #[name = "Being too cute"]
    TooCute,
    #[name = "Causing a wipe"]
    CausingWipe,
    #[name = "Being AFK"]
    BeingAfk,
    #[name = "Forgetting mechanics"]
    ForgettingMechanics,
    #[name = "Just because"]
    JustBecause,
}

impl BlameReason {
    fn format(&self) -> (&'static str, &'static str) {
        match self {
            BlameReason::TooCute => ("for ", "being too cute"),
            BlameReason::CausingWipe => ("for ", "causing a wipe"),
            BlameReason::BeingAfk => ("for ", "being AFK"),
            BlameReason::ForgettingMechanics => ("for ", "forgetting mechanics"),
            BlameReason::JustBecause => ("", "just because"),
        }
    }
}

/// Blame someone (or Serena) for something
#[poise::command(slash_command, guild_only)]
pub async fn blame(
    ctx: Context<'_>,
    #[description = "User to blame (defaults to Serena)"] user: Option<UserId>,
    #[description = "Reason for blaming"] reason: Option<BlameReason>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let serena_id = UserId::new(ctx.data().config.serena_user_id.parse().unwrap());
    let blamed_user = user.unwrap_or(serena_id);
    let (serena_blame_count, user_blame_count) = ctx.data().database.increment_blame_count(blamed_user.get() as i64).await?;

    let (prefix, reason_text) = reason.map(|r| r.format()).unwrap_or(("", ""));

    let response = if blamed_user == serena_id {
        format!(
            "Serena has been blamed {}{}! Total blame count: {}",
            prefix, reason_text, serena_blame_count
        )
    } else {
        format!(
            "<@{}> has been blamed {}{}, but it was probably meant for <@{}>!\n\
            <@{}>'s blame count: {} | Serena's blame count: {}",
            blamed_user, prefix, reason_text, serena_id,
            blamed_user, user_blame_count, serena_blame_count
        )
    };

    ctx.say(response).await?;

    // Update the bot's status with the new total blame count
    let activity = poise::serenity_prelude::ActivityData::custom(format!("Serena's blame count: {}", serena_blame_count));
    ctx.serenity_context().set_activity(Some(activity));

    Ok(())
}