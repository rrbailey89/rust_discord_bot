// commands/update_raid_time.rs
use crate::error::Error;
use crate::utils::parse_datetime;
use crate::Data;
use poise::serenity_prelude::EditChannel;
use poise::serenity_prelude::{ChannelId, CreateAllowedMentions};

type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(poise::ChoiceParameter, Debug)]
pub enum Month {
    January, February, March, April, May, June, July,
    August, September, October, November, December
}

#[derive(poise::ChoiceParameter)]
pub enum Time {
    #[name = "12:00 AM"] T1200AM,
    #[name = "1:00 AM"] T0100AM,
    #[name = "2:00 AM"] T0200AM,
    #[name = "3:00 AM"] T0300AM,
    #[name = "4:00 AM"] T0400AM,
    #[name = "5:00 AM"] T0500AM,
    #[name = "6:00 AM"] T0600AM,
    #[name = "7:00 AM"] T0700AM,
    #[name = "8:00 AM"] T0800AM,
    #[name = "9:00 AM"] T0900AM,
    #[name = "10:00 AM"] T1000AM,
    #[name = "11:00 AM"] T1100AM,
    #[name = "12:00 PM"] T1200PM,
    #[name = "1:00 PM"] T0100PM,
    #[name = "2:00 PM"] T0200PM,
    #[name = "3:00 PM"] T0300PM,
    #[name = "4:00 PM"] T0400PM,
    #[name = "5:00 PM"] T0500PM,
    #[name = "6:00 PM"] T0600PM,
    #[name = "7:00 PM"] T0700PM,
    #[name = "8:00 PM"] T0800PM,
    #[name = "9:00 PM"] T0900PM,
    #[name = "10:00 PM"] T1000PM,
    #[name = "11:00 PM"] T1100PM,
}

#[derive(poise::ChoiceParameter)]
pub enum Year {
    #[name = "2024"] Y2024 = 2024,
    #[name = "2025"] Y2025 = 2025,
    #[name = "2026"] Y2026 = 2026,
    #[name = "2027"] Y2027 = 2027,
    #[name = "2028"] Y2028 = 2028,
    #[name = "2029"] Y2029 = 2029,
    #[name = "2030"] Y2030 = 2030,
}

#[derive(poise::ChoiceParameter)]
pub enum Timezone {
    #[name = "Eastern Time"] ET,
    #[name = "Central Time"] CT,
    #[name = "Mountain Time"] MT,
    #[name = "Pacific Time"] PT,
    #[name = "Alaska Time"] AKT,
    #[name = "Hawaii Standard Time"] HST,
}

#[derive(poise::ChoiceParameter)]
pub enum Raid {
    #[name = "Alexander - The Burden of the Son (Savage)"]
    AlexanderBurdenSavage,
    #[name = "Alexander - The Eyes of the Creator (Savage)"]
    AlexanderEyesSavage,
    #[name = "Alexander - The Breath of the Creator (Savage)"]
    AlexanderBreathSavage,
    #[name = "Alexander - The Heart of the Creator (Savage)"]
    AlexanderHeartSavage,
    #[name = "Alexander - The Soul of the Creator (Savage)"]
    AlexanderSoulSavage,
}

impl std::fmt::Display for Month {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Time {
    fn to_string(&self) -> String {
        match self {
            Time::T1200AM => "12:00 AM".to_string(),
            Time::T0100AM => "1:00 AM".to_string(),
            Time::T0200AM => "2:00 AM".to_string(),
            Time::T0300AM => "3:00 AM".to_string(),
            Time::T0400AM => "4:00 AM".to_string(),
            Time::T0500AM => "5:00 AM".to_string(),
            Time::T0600AM => "6:00 AM".to_string(),
            Time::T0700AM => "7:00 AM".to_string(),
            Time::T0800AM => "8:00 AM".to_string(),
            Time::T0900AM => "9:00 AM".to_string(),
            Time::T1000AM => "10:00 AM".to_string(),
            Time::T1100AM => "11:00 AM".to_string(),
            Time::T1200PM => "12:00 PM".to_string(),
            Time::T0100PM => "1:00 PM".to_string(),
            Time::T0200PM => "2:00 PM".to_string(),
            Time::T0300PM => "3:00 PM".to_string(),
            Time::T0400PM => "4:00 PM".to_string(),
            Time::T0500PM => "5:00 PM".to_string(),
            Time::T0600PM => "6:00 PM".to_string(),
            Time::T0700PM => "7:00 PM".to_string(),
            Time::T0800PM => "8:00 PM".to_string(),
            Time::T0900PM => "9:00 PM".to_string(),
            Time::T1000PM => "10:00 PM".to_string(),
            Time::T1100PM => "11:00 PM".to_string(),
        }
    }
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

/// Update the raid time in a channel's topic
#[poise::command(slash_command, guild_only)]
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
) -> Result<(), Error> {
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