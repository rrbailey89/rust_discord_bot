use poise::serenity_prelude as serenity;
use serenity::all::{ChannelType, RoleId};
use serenity::model::prelude::*;
use std::error::Error;
use std::fmt;
use tokio_postgres::NoTls;
use tracing::{debug, error, info};
use reqwest;
use serde_json::Value;
use reqwest::Client;
use serenity::builder::CreateEmbed;
use serenity::model::user::User;
use serenity::model::guild::Member;
use chrono::{NaiveDate, NaiveTime, NaiveDateTime, DateTime, Utc, TimeZone};
use chrono_tz;
use poise::CreateReply;
use poise::command;
use poise::serenity_prelude::{ChannelId, CreateAllowedMentions};
use serenity::builder::EditChannel;
use async_openai::{
    types::{
        ChatCompletionRequestAssistantMessageArgs,
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client as OpenAIClient,
};

const DATABASE_URL: &str = "postgres://serena:password4321@postgres/discord_bot_test_server";

struct Data {} // User data, which is stored and accessible in all command invocations
type ErrorType = Box<dyn Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, ErrorType>;

/// Function to fetch the bot token from PostgreSQL
async fn get_bot_token() -> Result<String, ErrorType> {
    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let row = client
        .query_one("SELECT token FROM bot_config WHERE name = 'bot_token'", &[])
        .await?;
    let bot_token: String = row.get(0);

    Ok(bot_token)
}

/// Handle the event when a guild is created or becomes available
async fn on_guild_create(guild: Guild) -> Result<(), ErrorType> {
    info!(
        "Guild Create event received for: {} (ID: {})",
        guild.name, guild.id
    );

    let (mut client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Database connection error: {}", e);
        }
    });

    // Start a transaction
    let transaction = client.transaction().await?;

    // Insert or update guild_info
    let guild_info_result = transaction
        .execute(
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
        )
        .await?;

    info!(
        "Successfully inserted/updated guild info for: {} (ID: {}). Rows affected: {}",
        guild.name, guild.id, guild_info_result
    );

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

        let channel_result = transaction
            .execute(
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
            )
            .await?;

        debug!("Successfully inserted/updated channel: {} (ID: {}, Type: {}) in guild: {}. Rows affected: {}",
               channel.name, channel_id, channel_type_str, guild.name, channel_result);
    }

    // Commit the transaction
    transaction.commit().await?;
    info!(
        "Successfully committed all changes for guild: {} (ID: {})",
        guild.name, guild.id
    );
    Ok(())
}

/// Handle the event when the bot leaves a guild
async fn on_guild_delete(guild_id: GuildId) -> Result<(), ErrorType> {
    info!("Bot has left the guild with ID: {}", guild_id);

    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    client
        .execute(
            "DELETE FROM guild_info WHERE guild_id = $1",
            &[&(guild_id.get() as i64)],
        )
        .await?;

    Ok(())
}

/// Warn a member
#[command(slash_command)]
async fn warn(
    ctx: Context<'_>,
    #[description = "Member to warn"] member: Member,
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
            let warn_channel = ChannelId::new(channel_id as u64);
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

#[command(slash_command)]
async fn setwarnchannel(
    ctx: Context<'_>,
    #[description = "Channel to log warnings"] channel: Channel,
) -> Result<(), ErrorType> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| Box::<dyn Error + Send + Sync>::from("Failed to get guild ID"))?;
    store_warn_channel(guild_id.get() as i64, channel.id().get() as i64).await?;
    ctx.say(format!(
        "✅ Channel <#{}> has been set for logging warnings.",
        channel.id()
    ))
    .await
    .map_err(|e| Box::new(e) as ErrorType)?;
    Ok(())
}

#[command(slash_command)]
async fn randomcatimage(ctx: Context<'_>) -> Result<(), ErrorType> {
    ctx.defer().await?;

    let client = Client::new();
    let response = client.get("https://api.thecatapi.com/v1/images/search")
        .send()
        .await?
        .json::<Vec<Value>>()
        .await?;

    let image_url = response[0]["url"].as_str().ok_or("Failed to get image URL")?;

    let embed = CreateEmbed::default()
        .title("Random Cat Image")
        .image(image_url);

    let reply = CreateReply::default()
        .embed(embed);

    ctx.send(reply).await?;

    Ok(())
}

#[command(slash_command)]
async fn randomcapyimage(ctx: Context<'_>) -> Result<(), ErrorType> {
    ctx.defer().await?;

    let client = Client::new();
    let response = client.get("https://api.capy.lol/v1/capybara?json=true")
        .send()
        .await?
        .json::<Value>()
        .await?;

    let data = response["data"].as_object().ok_or("Failed to get data object")?;
    let image_url = data["url"].as_str().ok_or("Failed to get image URL")?;
    let alt_text = data["alt"].as_str().unwrap_or("Random Capybara");

    let embed = CreateEmbed::default()
        .title(alt_text)
        .image(image_url);

    let reply = CreateReply::default()
        .embed(embed);

    ctx.send(reply).await?;

    Ok(())
}

async fn fetch_warn_channel(guild_id: i64) -> Result<Option<i64>, ErrorType> {
    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });
    Ok(client
        .query_opt(
            "SELECT channel_id FROM warn_channel_ids WHERE guild_id = $1",
            &[&guild_id],
        )
        .await?
        .map(|r| r.get(0)))
}

async fn store_warn_channel(guild_id: i64, channel_id: i64) -> Result<(), ErrorType> {
    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });
    client
        .execute(
            "INSERT INTO warn_channel_ids (guild_id, channel_id) VALUES ($1, $2)
        ON CONFLICT (guild_id) DO UPDATE SET channel_id = EXCLUDED.channel_id",
            &[&guild_id, &channel_id],
        )
        .await?;
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

#[derive(Debug)]
struct UserInfo {
    discord_name: String,
    nickname: Option<String>,
    account_created: DateTime<Utc>,
    joined_server: DateTime<Utc>,
    roles: Vec<RoleId>,
    hug_count: i32,
}

#[command(context_menu_command = "User Information")]
async fn user_info(
    ctx: Context<'_>,
    user: User,
) -> Result<(), ErrorType> {
    ctx.defer().await?;

    let guild_id = ctx.guild_id().ok_or("Failed to get guild ID")?;
    let member = ctx.http().get_member(guild_id, user.id).await?;

    let user_info = fetch_user_info(&ctx, &user, &member, guild_id).await?;
    let embed = create_user_info_embed(&user_info, &user);

    let reply = CreateReply::default()
        .embed(embed);

    ctx.send(reply).await?;

    Ok(())
}

async fn fetch_user_info(
    _ctx: &Context<'_>,
    user: &User,
    member: &Member,
    _guild_id: GuildId
) -> Result<UserInfo, ErrorType> {
    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    let hug_count: i32 = client
        .query_opt(
            "SELECT hug_count FROM user_hug_counts WHERE user_id = $1",
            &[&(user.id.get() as i64)]
        )
        .await?
        .map(|row| row.get(0))
        .unwrap_or(0);

    Ok(UserInfo {
        discord_name: user.name.clone(),
        nickname: member.nick.clone(),
        account_created: Utc.timestamp_millis_opt(user.created_at().unix_timestamp() * 1000).unwrap(),
        joined_server: member.joined_at
            .map(|ts| Utc.timestamp_millis_opt(ts.unix_timestamp() * 1000).unwrap())
            .unwrap_or_else(|| Utc::now()),
        roles: member.roles.clone(),
        hug_count,
    })
}


fn create_user_info_embed(user_info: &UserInfo, user: &User) -> CreateEmbed {
    CreateEmbed::default()
        .title(format!("User Information for {}", user_info.nickname.as_deref().unwrap_or(&user.name)))
        .field("Discord Name", &user_info.discord_name, true)
        .field("Nickname", user_info.nickname.as_deref().unwrap_or("None"), true)
        .field("Account Created On", format!("<t:{}:f>", user_info.account_created.timestamp()), true)
        .field("Joined Server On", format!("<t:{}:f>", user_info.joined_server.timestamp()), true)
        .field("Roles", if user_info.roles.is_empty() { "None".to_string() } else { user_info.roles.iter().map(|r| format!("<@&{}>", r)).collect::<Vec<_>>().join(", ") }, true)
        .field("Total Hugs Received", user_info.hug_count.to_string(), true)
        .image(user.face())
        .color(0x00ff00)
}

#[command(slash_command)]
async fn animehug(
    ctx: Context<'_>,
    #[description = "User to hug"] user: User,
) -> Result<(), ErrorType> {
    ctx.defer().await?;

    let client = Client::new();
    let response = client.get("https://nekos.best/api/v2/hug")
        .send()
        .await?
        .json::<Value>()
        .await?;

    let result = &response["results"][0];
    let anime_name = result["anime_name"].as_str().unwrap_or("Unknown");
    let image_url = result["url"].as_str().unwrap_or("");

    let embed = CreateEmbed::default()
        .title(format!("From the Anime: {}", anime_name))
        .image(image_url)
        .color(0x5865F2);

    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("connection error: {}", e);
        }
    });

    let hug_count: i32 = client
        .query_one(
            "INSERT INTO user_hug_counts (user_id, hug_count)
             VALUES ($1, 1)
             ON CONFLICT (user_id)
             DO UPDATE SET hug_count = user_hug_counts.hug_count + 1
             RETURNING hug_count",
            &[&(user.id.get() as i64)],
        )
        .await?
        .get(0);

    let response_text = format!(
        "{} was hugged by {}. They have been hugged {} times.",
        user.id.mention(),
        ctx.author().id.mention(),
        hug_count
    );

    let reply = CreateReply::default()
        .content(response_text)
        .embed(embed)
        .allowed_mentions(CreateAllowedMentions::new().users([user.id]));

    ctx.send(reply).await?;

    Ok(())
}

#[command(slash_command)]
pub async fn updateraidtime(
    ctx: Context<'_>,
    #[description = "Select the month"] month: Month,
    #[description = "Enter the day"] day: i64,
    #[description = "Select the year"] year: Year,
    #[description = "Select the time"] time: Time,
    #[description = "Select the timezone"] timezone: Timezone,
    #[description = "Select the raid"] raid: Raid,
    #[description = "Select the channel"] channel: ChannelId,
    #[description = "Is this M.I.N.E. or not?"] mine: Option<bool>,
) -> Result<(), ErrorType> {
    ctx.defer().await?;

    let datetime = parse_datetime(&month.to_string(), day, year as i64, &time.to_string(), timezone.as_ref())?;
    let unix_timestamp = datetime.timestamp();

    let mut topic = format!("Next Meet Is: {} | Time: <t:{}:f>", raid.as_ref(), unix_timestamp);
    if mine.unwrap_or(false) {
        topic += " M.I.N.E.";
    }

    ctx.http().edit_channel(channel, &EditChannel::new().topic(&topic), None).await?;

    let response = poise::CreateReply::default()
        .content(format!("Updated channel topic: {}", topic))
        .allowed_mentions(CreateAllowedMentions::new());

    ctx.send(response).await?;

    Ok(())
}


#[derive(poise::ChoiceParameter, Debug)]
pub enum Month {
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(poise::ChoiceParameter, Debug)]
pub enum Time {
    #[name = "1:00 AM"]
    T0100AM,
    #[name = "2:00 AM"]
    T0200AM,
    // ... (add all other time options)
    #[name = "11:00 PM"]
    T1100PM,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let time_str = match self {
            Time::T0100AM => "1:00 AM",
            Time::T0200AM => "2:00 AM",
            // ... (add all other time options)
            Time::T1100PM => "11:00 PM",
        };
        write!(f, "{}", time_str)
    }
}

#[derive(poise::ChoiceParameter, Debug)]
pub enum Year {
    Y2024 = 2024,
    Y2025 = 2025,
    Y2026 = 2026,
    Y2027 = 2027,
    Y2028 = 2028,
    Y2029 = 2029,
    Y2030 = 2030,
}

#[derive(poise::ChoiceParameter, Debug)]
pub enum Timezone {
    #[name = "Eastern Time"]
    ET,
    #[name = "Central Time"]
    CT,
    #[name = "Mountain Time"]
    MT,
    #[name = "Pacific Time"]
    PT,
    #[name = "Alaska Time"]
    AKT,
    #[name = "Hawaii Standard Time"]
    HST,
}

impl Timezone {
    fn as_ref(&self) -> &'static str {
        match self {
            Timezone::ET => "America/New_York",
            Timezone::CT => "America/Chicago",
            Timezone::MT => "America/Denver",
            Timezone::PT => "America/Los_Angeles",
            Timezone::AKT => "America/Anchorage",
            Timezone::HST => "America/Honolulu",
        }
    }
}

#[derive(poise::ChoiceParameter, Debug)]
pub enum Raid {
    AlexanderBurdenSavage,
    AlexanderEyesSavage,
    AlexanderBreathSavage,
    AlexanderHeartSavage,
    AlexanderSoulSavage,
}

impl Raid {
    fn as_ref(&self) -> &'static str {
        match self {
            Raid::AlexanderBurdenSavage => "Alexander - The Burden of the Son (Savage)",
            Raid::AlexanderEyesSavage => "Alexander - The Eyes of the Creator (Savage)",
            Raid::AlexanderBreathSavage => "Alexander - The Breath of the Creator (Savage)",
            Raid::AlexanderHeartSavage => "Alexander - The Heart of the Creator (Savage)",
            Raid::AlexanderSoulSavage => "Alexander - The Soul of the Creator (Savage)",
        }
    }
}

fn parse_datetime(month: &str, day: i64, year: i64, time: &str, timezone: &str) -> Result<DateTime<Utc>, ErrorType> {
    let month_num = match month.to_lowercase().as_str() {
        "january" => 1, "february" => 2, "march" => 3, "april" => 4,
        "may" => 5, "june" => 6, "july" => 7, "august" => 8,
        "september" => 9, "october" => 10, "november" => 11, "december" => 12,
        _ => return Err("Invalid month".into()),
    };

    let (hour, minute) = parse_time(time)?;

    let naive_date = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(year as i32, month_num, day as u32).ok_or("Invalid date")?,
        NaiveTime::from_hms_opt(hour, minute, 0).ok_or("Invalid time")?,
    );

    let tz: chrono_tz::Tz = timezone.parse()?;
    Ok(tz.from_local_datetime(&naive_date).unwrap().with_timezone(&Utc))
}

fn parse_time(time: &str) -> Result<(u32, u32), ErrorType> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid time format".into());
    }

    let hour: u32 = parts[0].parse()?;
    let minute: u32 = parts[1].split_whitespace().next().unwrap().parse()?;
    let period = time.to_lowercase();

    let hour = if period.contains("pm") && hour != 12 {
        hour + 12
    } else if period.contains("am") && hour == 12 {
        0
    } else {
        hour
    };

    Ok((hour, minute))
}

async fn get_openai_api_key() -> Result<String, ErrorType> {
    let (client, connection) = tokio_postgres::connect(DATABASE_URL, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let row = client
        .query_one("SELECT token FROM bot_config WHERE name = 'openai_api_key'", &[])
        .await?;

    let api_key: String = row.get(0);
    Ok(api_key)
}

#[command(slash_command)]
async fn ask(
    ctx: Context<'_>,
    #[description = "Your question"] question: String,
) -> Result<(), ErrorType> {
    ctx.defer().await?;

    let api_key = get_openai_api_key().await?;
    let config = async_openai::config::OpenAIConfig::new().with_api_key(api_key);
    let client = OpenAIClient::with_config(config);

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32)
        .model("gpt-3.5-turbo")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful assistant.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(question)
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.chat().create(request).await?;

    if let Some(choice) = response.choices.first() {
        if let Some(content) = &choice.message.content {
            ctx.say(content).await?;
        } else {
            ctx.say("No response content received.").await?;
        }
    } else {
        ctx.say("No response received.").await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), ErrorType> {
    tracing_subscriber::fmt::init();
    let token = get_bot_token().await?;
    let intents =
        GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![warn(), setwarnchannel(), randomcatimage(), randomcapyimage(), user_info(), animehug(), updateraidtime(), ask()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(&token, intents)
        .framework(framework)
        .await?;

    client.start().await.map_err(|e| Box::new(e) as ErrorType)
}
