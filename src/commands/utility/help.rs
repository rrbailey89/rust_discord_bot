// commands/help.rs
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude as serenity;
use std::collections::HashMap;
use std::time::Duration;

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
    if let Some(cmd) = command.as_deref() {
        // Use the built-in help command with specific command
        let config = poise::builtins::HelpConfiguration {
            show_context_menu_commands: true,
            ephemeral: true,
            ..Default::default()
        };
        
        poise::builtins::help(ctx, Some(cmd), config).await?;
        return Ok(());
    }

    // For general help, use pagination
    let pages = generate_help_pages(ctx);
    paginate_help(ctx, pages).await?;
    
    Ok(())
}

// Generate help pages from commands
fn generate_help_pages(ctx: poise::Context<'_, Data, Error>) -> Vec<String> {
    // Group commands by category
    let commands = &ctx.framework().options().commands;
    let mut categories: HashMap<String, Vec<&poise::Command<Data, Error>>> = HashMap::new();
    
    for cmd in commands {
        let category = cmd.category.clone().unwrap_or_else(|| "Uncategorized".to_string());
        categories.entry(category).or_insert_with(Vec::new).push(cmd);
    }
    
    // Sort categories for consistent ordering
    let mut category_names: Vec<String> = categories.keys().cloned().collect();
    category_names.sort();
    
    // Create pages - one page per category
    let mut pages = Vec::new();

    // First page - overview
    let mut overview = String::from("# Bot Help\n\n");
    overview.push_str("Use the navigation buttons below to browse through command categories.\n\n");
    overview.push_str("**Available Categories:**\n");
    
    for category in &category_names {
        let cmd_count = categories.get(category).map_or(0, |cmds| cmds.len());
        overview.push_str(&format!("• **{}** - {} commands\n", category, cmd_count));
    }
    
    overview.push_str("\nTip: Use `/help <category>` to see details for a specific category or `/help <command>` for a specific command.");
    pages.push(overview);
    
    // Generate a page for each category
    for category in category_names {
        if let Some(cmds) = categories.get(&category) {
            let mut page = format!("# {} Commands\n\n", category);
            
            for cmd in cmds {
                let description = cmd.description.as_deref().unwrap_or("No description");
                page.push_str(&format!("• **`/{}`** - {}\n", cmd.name, description));
            }
            
            page.push_str("\nUse `/help <command>` for more details about a specific command.");
            pages.push(page);
        }
    }
    
    pages
}

// Paginated help implementation, adapted from the provided example
async fn paginate_help(
    ctx: poise::Context<'_, Data, Error>,
    pages: Vec<String>,
) -> Result<(), Error> {
    // Define unique identifiers for the navigation buttons
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    // Send the embed with the first page as content
    let reply = {
        let components = serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&prev_button_id)
                .emoji('◀')
                .style(serenity::ButtonStyle::Secondary),
            serenity::CreateButton::new(&next_button_id)
                .emoji('▶')
                .style(serenity::ButtonStyle::Secondary),
        ]);

        poise::CreateReply::default()
            .embed(
                serenity::CreateEmbed::new()
                    .description(&pages[0])
                    .color(0x3498DB)
            )
            .components(vec![components])
            .ephemeral(true)
    };

    ctx.send(reply).await?;

    // Loop through incoming interactions with the navigation buttons
    let mut current_page = 0;
    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        // Filter for our specific button IDs
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        // Timeout after 5 minutes of inactivity
        .timeout(Duration::from_secs(300))
        .await
    {
        // Determine which button was pressed
        if press.data.custom_id == next_button_id {
            current_page += 1;
            if current_page >= pages.len() {
                current_page = 0;
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or(pages.len() - 1);
        } else {
            // This is an unrelated button interaction
            continue;
        }

        // Update the message with the new page contents
        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new()
                        .embed(
                            serenity::CreateEmbed::new()
                                .description(&pages[current_page])
                                .color(0x3498DB)
                                .footer(
                                    serenity::CreateEmbedFooter::new(
                                        format!("Page {}/{}", current_page + 1, pages.len())
                                    )
                                )
                        ),
                ),
            )
            .await?;
    }

    Ok(())
}
