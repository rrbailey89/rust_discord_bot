use crate::error::Error;
use crate::Data;

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

/// Blame Serena for something
#[poise::command(slash_command)]
pub async fn blameserena(
    ctx: Context<'_>,
    #[description = "Reason for blaming Serena"] reason: Option<BlameReason>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let blame_count = ctx.data().database.increment_blame_count().await?;

    let response = if let Some(reason) = reason {
        let (prefix, reason_text) = reason.format();
        format!(
            "Serena has been blamed {}{}! Total blame count: {}",
            prefix, reason_text, blame_count
        )
    } else {
        format!("Serena has been blamed! Total blame count: {}", blame_count)
    };

    ctx.say(response).await?;

    Ok(())
}
