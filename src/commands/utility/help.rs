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
    // Use Poise's simple built-in help
    let config = poise::builtins::HelpConfiguration {
        extra_text_at_bottom: "\nTip: You can also use `/help <category>` to see all commands in a category.",
        show_context_menu_commands: true,
        ephemeral: true,
        ..Default::default()
    };
    
    // For simplicity, always use the built-in help command
    poise::builtins::help(ctx, command.as_deref(), config).await?;
    
    Ok(())
}
