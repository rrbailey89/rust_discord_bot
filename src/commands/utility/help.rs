// commands/help.rs
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{CreateEmbed, CreateMessage};

/// Show help for commands
#[poise::command(
    prefix_command,
    track_edits,
    slash_command,
    category = "Utility"
)]
pub async fn help(
    ctx: poise::Context<'_, Data, Error>,
    #[description = "Specific command or category to show help about"]
    #[rest]
    command: Option<String>,
) -> Result<(), Error> {
    // If a specific command is provided, use Poise's built-in help
    if let Some(cmd) = &command {
        let config = poise::builtins::HelpConfiguration {
            extra_text_at_bottom: "Type /help command for more info on a command.",
            show_context_menu_commands: true,
            ephemeral: true,
            ..Default::default()
        };
        
        // Try to show help for the specific command
        if let Ok(_) = poise::builtins::help(ctx, Some(cmd), config).await {
            return Ok(());
        }
        
        // If command help failed, check if it's a category
        let category = cmd.to_lowercase();
        return show_category_help(ctx, &category).await;
    }
    
    // For general help without a specific command, create a custom embed
    let mut embed = CreateEmbed::default()
        .title("Bot Commands")
        .description("Here are the available command categories. Type `/help <category>` for more details on a specific category.")
        .field("Admin", "Administrative commands\n`/help admin`", true)
        .field("Fun", "Fun and entertainment commands\n`/help fun`", true)
        .field("Utility", "Utility and informational commands\n`/help utility`", true)
        .field("Availability", "Availability and status commands\n`/help availability`", true)
        .footer(|f| f.text("Type /help <command> for detailed info on a specific command"));
    
    ctx.send(|m| m
        .embed(|e| {
            *e = embed;
            e
        })
        .ephemeral(true)
    ).await?;
    
    Ok(())
}

// Helper function to show help for a specific category
async fn show_category_help(ctx: poise::Context<'_, Data, Error>, category: &str) -> Result<(), Error> {
    let framework = &ctx.framework();
    let commands = framework.options().commands.clone();
    
    // Filter commands by category
    let filtered_commands: Vec<_> = commands.iter()
        .filter(|cmd| {
            cmd.category.as_deref().unwrap_or("").to_lowercase() == category
        })
        .collect();
    
    if filtered_commands.is_empty() {
        ctx.send(|m| m
            .content(format!("No commands found in category '{}'", category))
            .ephemeral(true)
        ).await?;
        return Ok(());
    }
    
    // Build command list with descriptions
    let command_list = filtered_commands.iter()
        .map(|cmd| format!("**/{0}** - {1}", cmd.name, cmd.description.as_deref().unwrap_or("No description")))
        .collect::<Vec<_>>()
        .join("\n");
    
    let mut embed = CreateEmbed::default()
        .title(format!("{} Commands", capitalize(category)))
        .description(command_list)
        .footer(|f| f.text("Type /help <command> for more details on a specific command"));
    
    ctx.send(|m| m
        .embed(|e| {
            *e = embed;
            e
        })
        .ephemeral(true)
    ).await?;
    
    Ok(())
}

// Helper function to capitalize first letter of a string
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().chain(chars).collect(),
    }
}
