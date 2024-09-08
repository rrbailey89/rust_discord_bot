// commands/user_info.rs
use crate::error::Error;
use crate::Data;
use chrono::{DateTime, Utc};
use poise::serenity_prelude::{CreateEmbed, Member, RoleId, User};
use poise::CreateReply;

type Context<'a> = poise::Context<'a, Data, Error>;

struct UserInfo {
    discord_name: String,
    nickname: Option<String>,
    account_created: DateTime<Utc>,
    joined_server: DateTime<Utc>,
    roles: Vec<RoleId>,
    hug_count: i32,
}

/// Get information about a user
#[poise::command(context_menu_command = "User Information")]
pub async fn userinfo(
    ctx: Context<'_>,
    user: User,
) -> Result<(), Error> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("Failed to get guild ID".to_string()))?;
    let member = ctx.http().get_member(guild_id, user.id).await?;

    let user_info = fetch_user_info(&ctx, &user, &member).await?;
    let embed = create_user_info_embed(&user_info, &user);

    ctx.send(CreateReply::default().embed(embed)).await?;

    Ok(())
}

async fn fetch_user_info(ctx: &Context<'_>, user: &User, member: &Member) -> Result<UserInfo, Error> {
    let hug_count = ctx.data().database.get_hug_count(user.id.get() as i64).await?;

    Ok(UserInfo {
        discord_name: user.name.clone(),
        nickname: member.nick.clone(),
        account_created: *user.created_at(),
        joined_server: member.joined_at
            .map(|ts| *ts)
            .unwrap_or_else(Utc::now),
        roles: member.roles.clone(),
        hug_count,
    })
}

fn create_user_info_embed(user_info: &UserInfo, user: &User) -> CreateEmbed {
    CreateEmbed::new()
        .title(format!("User Information for {}", user_info.nickname.as_deref().unwrap_or(&user.name)))
        .field("Discord Name", &user_info.discord_name, true)
        .field("Nickname", user_info.nickname.as_deref().unwrap_or("None"), true)
        .field("Account Created", format!("<t:{}:F>", user_info.account_created.timestamp()), true)
        .field("Joined Server", format!("<t:{}:F>", user_info.joined_server.timestamp()), true)
        .field("Roles", if user_info.roles.is_empty() {
            "None".to_string()
        } else {
            user_info.roles.iter().map(|r| format!("<@&{}>", r)).collect::<Vec<_>>().join(", ")
        }, true)
        .field("Total Hugs Received", user_info.hug_count.to_string(), true)
        .image(user.face())
        .color(0x00ff00)
}