// commands/reminder.rs

use crate::error::Error;
use crate::Data;
use chrono::{DateTime, NaiveTime, Utc, Weekday};
use poise::serenity_prelude::{ChannelId, CreateEmbed};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

type Context<'a> = poise::Context<'a, Data, Error>;

impl fmt::Display for Frequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Frequency::Daily => write!(f, "Daily"),
            Frequency::Weekly => write!(f, "Weekly"),
            Frequency::Monthly => write!(f, "Monthly"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reminder {
    pub id: i32,
    pub guild_id: i64,
    pub channel_id: i64,
    pub message: String,
    pub time: NaiveTime,
    pub days: Vec<Weekday>,
    pub frequency: Frequency,
    pub last_sent: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, poise::ChoiceParameter)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
}

impl FromStr for Frequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "daily" => Ok(Frequency::Daily),
            "weekly" => Ok(Frequency::Weekly),
            "monthly" => Ok(Frequency::Monthly),
            _ => Err(format!("Invalid frequency: {}", s)),
        }
    }
}

#[poise::command(slash_command, subcommands("create", "list", "delete"))]
pub async fn reminder(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn create(
    ctx: Context<'_>,
    #[description = "Channel to send the reminder"] channel: ChannelId,
    #[description = "Time of the reminder (HH:MM)"] time: String,
    #[description = "Days of the week (comma-separated, e.g., 'Mon,Wed,Fri')"] days: String,
    #[description = "Frequency of the reminder"] frequency: Frequency,
    #[description = "Message for the reminder"] message: String,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("This command can only be used in a server".to_string()))?;

    let parsed_time = NaiveTime::parse_from_str(&time, "%H:%M")
        .map_err(|_| Error::Unknown("Invalid time format. Please use HH:MM".to_string()))?;

    let parsed_days: Vec<Weekday> = days.split(',')
        .map(|day| match day.trim().to_lowercase().as_str() {
            "mon" => Ok(Weekday::Mon),
            "tue" => Ok(Weekday::Tue),
            "wed" => Ok(Weekday::Wed),
            "thu" => Ok(Weekday::Thu),
            "fri" => Ok(Weekday::Fri),
            "sat" => Ok(Weekday::Sat),
            "sun" => Ok(Weekday::Sun),
            _ => Err(Error::Unknown("Invalid day format".to_string())),
        })
        .collect::<Result<Vec<Weekday>, Error>>()?;

    let reminder = Reminder {
        id: 0,
        guild_id: guild_id.get() as i64,
        channel_id: channel.get() as i64,
        message,
        time: parsed_time,
        days: parsed_days,
        frequency,
        last_sent: None,
    };

    ctx.data().database.create_reminder(&reminder).await?;

    ctx.say("✅ Reminder created successfully!").await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("This command can only be used in a server".to_string()))?;

    let reminders = ctx.data().database.get_reminders(guild_id.get() as i64).await?;

    if reminders.is_empty() {
        ctx.say("No reminders set for this server.").await?;
        return Ok(());
    }

    let embed = CreateEmbed::default()
        .title("Reminders")
        .description(reminders.iter().map(|r| {
            format!("{}. <#{}> at {} on {} ({})\nMessage: {}",
                    r.id,
                    r.channel_id,
                    r.time.format("%H:%M"),
                    r.days.iter().map(|d| format!("{:?}", d)).collect::<Vec<_>>().join(", "),
                    r.frequency,
                    r.message
            )
        }).collect::<Vec<_>>().join("\n\n"))
        .color(0x00FF00);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

#[poise::command(slash_command)]
pub async fn delete(
    ctx: Context<'_>,
    #[description = "ID of the reminder to delete"] reminder_id: i32,
) -> Result<(), Error> {
    let guild_id = ctx.guild_id()
        .ok_or_else(|| Error::Unknown("This command can only be used in a server".to_string()))?;

    ctx.data().database.delete_reminder(guild_id.get() as i64, reminder_id).await?;

    ctx.say("✅ Reminder deleted successfully!").await?;

    Ok(())
}
