use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{Channel, CreateEmbed, CreateMessage};

type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, subcommands("add", "remove", "list", "post"), guild_only)]
pub async fn rule(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, guild_only)]
async fn add(
    ctx: Context<'_>,
    #[description = "The rule to add"] rule: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get() as i64;
    ctx.data().database.add_rule(guild_id, &rule).await?;
    ctx.say("Rule added successfully!").await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
async fn remove(
    ctx: Context<'_>,
    #[description = "The rule number to remove"] rule_number: i64,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get() as i64;
    ctx.data().database.remove_rule(guild_id, rule_number).await?;
    ctx.say("Rule removed successfully!").await?;
    Ok(())
}

#[poise::command(slash_command, guild_only)]
async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get() as i64;
    let rules = ctx.data().database.get_rules(guild_id).await?;

    if rules.is_empty() {
        ctx.say("No rules have been set for this server.").await?;
    } else {
        let rules_list = rules.iter().enumerate()
            .map(|(i, rule)| format!("{}. {}", i + 1, rule))
            .collect::<Vec<String>>()
            .join("\n");

        let embed = CreateEmbed::default()
            .title("Server Rules")
            .description(rules_list)
            .color(0x00FF00);

        ctx.send(poise::CreateReply::default().embed(embed)).await?;
    }
    Ok(())
}

#[poise::command(slash_command, guild_only)]
async fn post(
    ctx: Context<'_>,
    #[description = "The channel to post rules in"] channel: Channel,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap().get() as i64;
    let rules = ctx.data().database.get_rules(guild_id).await?;

    if rules.is_empty() {
        ctx.say("No rules have been set for this server.").await?;
    } else {
        let rules_list = rules.iter().enumerate()
            .map(|(i, rule)| format!("{}. {}", i + 1, rule))
            .collect::<Vec<String>>()
            .join("\n");

        let embed = CreateEmbed::default()
            .title("Server Rules")
            .description(rules_list)
            .color(0x00FF00);

        let message = CreateMessage::default().add_embed(embed);
        channel.id().send_message(&ctx.http(), message).await?;
        ctx.say("Rules have been posted in the specified channel.").await?;
    }
    Ok(())
}
