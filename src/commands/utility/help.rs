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
    // Split the help command by categories to avoid "Message too large" error
    let config = poise::builtins::HelpConfiguration {
        show_context_menu_commands: true,
        ephemeral: true,
        extra_text_at_bottom: "\nUse `/help <category>` to see commands in a specific category, or `/help <command>` for detailed command help.",
        ..Default::default()
    };
    
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    
    Ok(())
}
