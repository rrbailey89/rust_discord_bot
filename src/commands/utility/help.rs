// commands/help.rs
use crate::error::Error;
use crate::Data;

/// Show help for commands
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    category = "Utility"
)]
pub async fn help(
    ctx: poise::Context<'_, Data, Error>,
    #[description = "Specific command to show help about"]
    #[rest]
    command: Option<String>,
) -> Result<(), Error> {
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "Type /help command for more info on a command.",
        show_context_menu_commands: true,
        ephemeral: true,
        ..Default::default()
    };
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    Ok(())
}