use poise::serenity_prelude as serenity;
use serenity::all::ChannelType;
use tokio_postgres::NoTls;
use tracing::{info, error, debug};
use std::error::Error;

const DATABASE_URL: &str = "postgres://serena:password4321@postgres/discord_bot_test_server";

struct Data {} // User data, which is stored and accessible in all command invocations
type ErrorType = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, ErrorType>;

/// Function to fetch the bot token from PostgreSQL
async fn get_bot_token() -> Result<String, ErrorType> {
    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let row = client.query_one("SELECT token FROM bot_config WHERE name = 'bot_token'", &[]).await?;
    let bot_token: String = row.get(0);

    Ok(bot_token)
}

/// Handle the event when a guild is created or becomes available
async fn on_guild_create(guild: serenity::Guild) -> Result<(), ErrorType> {
    info!("Guild Create event received for: {} (ID: {})", guild.name, guild.id);

    let (mut client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {}", e);
        }
    });

    // Start a transaction
    let transaction = client.transaction().await?;

    // Insert or update guild_info
    let guild_info_result = transaction.execute(
        "INSERT INTO guild_info (guild_id, guild_name, owner_id, member_count)
         VALUES ($1, $2, $3, $4)
         ON CONFLICT (guild_id) DO UPDATE SET
         guild_name = EXCLUDED.guild_name,
         owner_id = EXCLUDED.owner_id,
         member_count = EXCLUDED.member_count",
        &[
            &(guild.id.get() as i64),
            &guild.name,
            &(guild.owner_id.get() as i64),
            &(guild.member_count as i32),
        ],
    ).await?;

    info!("Successfully inserted/updated guild info for: {} (ID: {}). Rows affected: {}", guild.name, guild.id, guild_info_result);

    // Insert or update guild_channels
    for (channel_id, channel) in guild.channels {
        let channel_type_str = match channel.kind {
            ChannelType::Text => "text",
            ChannelType::Private => "private",
            ChannelType::Voice => "voice",
            ChannelType::GroupDm => "group",
            ChannelType::Category => "category",
            ChannelType::News => "news",
            ChannelType::NewsThread => "news_thread",
            ChannelType::PublicThread => "public_thread",
            ChannelType::PrivateThread => "private_thread",
            ChannelType::Stage => "stage",
            ChannelType::Directory => "guild_directory",
            ChannelType::Forum => "forum",
            _ => "unknown",
        };

        let channel_result = transaction.execute(
            "INSERT INTO guild_channels (channel_id, guild_id, channel_name, channel_type)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (channel_id) DO UPDATE SET
             guild_id = EXCLUDED.guild_id,
             channel_name = EXCLUDED.channel_name,
             channel_type = EXCLUDED.channel_type",
            &[
                &(channel_id.get() as i64),
                &(guild.id.get() as i64),
                &channel.name,
                &channel_type_str,
            ],
        ).await?;

        debug!("Successfully inserted/updated channel: {} (ID: {}, Type: {}) in guild: {}. Rows affected: {}",
               channel.name, channel_id, channel_type_str, guild.name, channel_result);
    }

    // Commit the transaction
    transaction.commit().await?;
    info!("Successfully committed all changes for guild: {} (ID: {})", guild.name, guild.id);
    Ok(())
}

/// Handle the event when the bot leaves a guild
async fn on_guild_delete(guild_id: serenity::GuildId) -> Result<(), ErrorType> {
    info!("Bot has left the guild with ID: {}", guild_id);

    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client.execute(
        "DELETE FROM guild_info WHERE guild_id = $1",
        &[&(guild_id.get() as i64)]
    ).await?;

    Ok(())
}

/// Warn a member
#[poise::command(slash_command)]
async fn warn(
    ctx: Context<'_>,
    #[description = "Member to warn"] member: serenity::Member,
    #[description = "Reason for the warning"] reason: Option<String>,
) -> Result<(), ErrorType> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().ok_or_else(|| "Failed to get guild ID")?;
    let warn_channel_id = fetch_warn_channel(guild_id.get() as i64).await?;

    let reason_message = reason.unwrap_or_else(|| "No reason provided".to_string());
    let warn_message = format!(
        "🚨 {} has been warned for: {}",
        member.user.name, reason_message
    );

    match warn_channel_id {
        Some(channel_id) => {
            let warn_channel = serenity::ChannelId::new(channel_id as u64);
            warn_channel.say(&ctx.http(), &warn_message).await?;
            ctx.say("✅ Warning has been issued successfully.").await?;
        }
        None => {
            ctx.channel_id().say(&ctx.http(), &warn_message).await?;
            ctx.say("Warning has been issued in this channel. Use /setwarnchannel to set a warning channel.").await?;
        }
    }

    Ok(())
}

#[poise::command(slash_command)]
async fn setwarnchannel(
    ctx: Context<'_>,
    #[description = "Channel to log warnings"] channel: serenity::Channel,
) -> Result<(), ErrorType> {
    let guild_id = ctx.guild_id().ok_or_else(|| Box::<dyn Error + Send + Sync>::from("Failed to get guild ID"))?;
    store_warn_channel(guild_id.get() as i64, channel.id().get() as i64).await?;
    ctx.say(format!("✅ Channel <#{}> has been set for logging warnings.", channel.id())).await.map_err(|e| Box::new(e) as ErrorType)?;
    Ok(())
}

async fn fetch_warn_channel(guild_id: i64) -> Result<Option<i64>, ErrorType> {
    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;
    tokio::spawn(async move { if let Err(e) = connection.await { error!("connection error: {}", e); }});
    Ok(client.query_opt("SELECT channel_id FROM warn_channel_ids WHERE guild_id = $1", &[&guild_id]).await?.map(|r| r.get(0)))
}

async fn store_warn_channel(guild_id: i64, channel_id: i64) -> Result<(), ErrorType> {
    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;
    tokio::spawn(async move { if let Err(e) = connection.await { error!("connection error: {}", e); }});
    client.execute(
        "INSERT INTO warn_channel_ids (guild_id, channel_id) VALUES ($1, $2)
        ON CONFLICT (guild_id) DO UPDATE SET channel_id = EXCLUDED.channel_id",
        &[&guild_id, &channel_id]
    ).await?;
    Ok(())
}

async fn event_handler(
    _ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, ErrorType>,
    _data: &Data,
) -> Result<(), ErrorType> {
    match event {
        serenity::FullEvent::GuildCreate { guild, .. } => {
            on_guild_create(guild.clone()).await?;
        }
        serenity::FullEvent::GuildDelete { incomplete, .. } => {
            on_guild_delete(incomplete.id).await?;
        }
        _ => {}
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ErrorType> {
    tracing_subscriber::fmt::init();
    let token = get_bot_token().await?;
    let intents = serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![warn(), setwarnchannel()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| Box::pin(async move {
            poise::builtins::register_globally(ctx, &framework.options().commands).await?;
            Ok(Data {})
        }))
        .build();

    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .await?;

    client.start().await.map_err(|e| Box::new(e) as ErrorType)
}