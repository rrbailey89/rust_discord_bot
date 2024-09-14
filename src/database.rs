use crate::commands::reminder::{Frequency, Reminder};
use crate::error::Error;
use chrono::{Datelike, Utc, Weekday};
use chrono_tz::America::Los_Angeles;
use poise::serenity_prelude::{ChannelType, Guild, UserId};
use std::str::FromStr;
use std::sync::Arc;
use tokio_postgres::{Client, NoTls};

#[derive(Clone)]
pub struct Database {
    client: Arc<Client>,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(url, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });

        Ok(Self { client: client.into() })
    }

    pub async fn fetch_warn_channel(&self, guild_id: i64) -> Result<Option<i64>, Error> {
        let row = self.client
            .query_opt(
                "SELECT channel_id FROM warn_channel_ids WHERE guild_id = $1",
                &[&guild_id],
            )
            .await?;

        Ok(row.map(|r| r.get(0)))
    }

    pub async fn store_warn_channel(&self, guild_id: i64, channel_id: i64) -> Result<(), Error> {
        self.client
            .execute(
                "INSERT INTO warn_channel_ids (guild_id, channel_id) VALUES ($1, $2)
                ON CONFLICT (guild_id) DO UPDATE SET channel_id = EXCLUDED.channel_id",
                &[&guild_id, &channel_id],
            )
            .await?;
        Ok(())
    }

    pub async fn get_hug_count(&self, user_id: i64) -> Result<i32, Error> {
        let row = self.client
            .query_opt(
                "SELECT hug_count FROM user_hug_counts WHERE user_id = $1",
                &[&user_id],
            )
            .await?;

        Ok(row.map(|r| r.get(0)).unwrap_or(0))
    }

    pub async fn increment_hug_count(&self, user_id: i64) -> Result<i32, Error> {
        let row = self.client
            .query_one(
                "INSERT INTO user_hug_counts (user_id, hug_count)
                 VALUES ($1, 1)
                 ON CONFLICT (user_id)
                 DO UPDATE SET hug_count = user_hug_counts.hug_count + 1
                 RETURNING hug_count",
                &[&user_id],
            )
            .await?;

        Ok(row.get(0))
    }

    pub async fn store_guild_info(&self, guild: &Guild) -> Result<(), Error> {
        self.client
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
        Ok(())
    }


    pub async fn store_guild_channels(&self, guild: &Guild) -> Result<(), Error> {
        for (channel_id, channel) in &guild.channels {
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
                ChannelType::Directory => "guilddirectory",
                ChannelType::Forum => "forum",
                _ => "unknown",
            };

            self.client
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
        }
        Ok(())
    }

    pub async fn remove_guild_info(&self, guild_id: i64) -> Result<(), Error> {
        self.client
            .execute(
                "DELETE FROM guild_info WHERE guild_id = $1",
                &[&guild_id],
            )
            .await?;
        Ok(())
    }

    pub async fn store_delete_log_channel(&self, guild_id: i64, channel_id: i64, guild_name: String) -> Result<(), Error> {
        self.client
            .execute(
                "INSERT INTO message_delete_channels (guild_id, channel_id, guild_name) VALUES ($1, $2, $3)
                ON CONFLICT (guild_id) DO UPDATE SET channel_id = EXCLUDED.channel_id, guild_name = EXCLUDED.guild_name",
                &[&guild_id, &channel_id, &guild_name],
            )
            .await?;
        Ok(())
    }

    pub async fn fetch_delete_log_channel(&self, guild_id: i64) -> Result<Option<i64>, Error> {
        let row = self.client
            .query_opt(
                "SELECT channel_id FROM message_delete_channels WHERE guild_id = $1",
                &[&guild_id],
            )
            .await?;

        Ok(row.map(|r| r.get(0)))
    }
    pub async fn get_conversation_thread_id_for_user(&self, user_id: UserId) -> Result<Option<String>, Error> {
        let row = self.client
            .query_opt(
                "SELECT conversation_id FROM user_conversation_ids WHERE user_id = $1 ORDER BY created_at DESC LIMIT 1",
                &[&(user_id.get() as i64)],
            )
            .await?;

        Ok(row.map(|r| r.get(0)))
    }

    pub async fn store_conversation_thread_id_for_user(&self, user_id: UserId, thread_id: String) -> Result<(), Error> {
        self.client
            .execute(
                "INSERT INTO user_conversation_ids (user_id, conversation_id, created_at) VALUES ($1, $2, NOW())",
                &[&(user_id.get() as i64), &thread_id],
            )
            .await?;
        Ok(())
    }

    // Fetch whether emoji reactions are enabled for a specific guild
    pub async fn fetch_emoji_reactions_enabled(&self, guild_id: i64) -> Result<bool, Error> {
        let row = self.client
            .query_opt(
                "SELECT emoji_reactions_enabled FROM guild_emoji_settings WHERE guild_id = $1",
                &[&guild_id],
            )
            .await?;

        Ok(row.map(|r| r.get(0)).unwrap_or(true)) // Explicitly return true if no record is found
    }

    // Store or update the emoji reactions enabled/disabled setting for a guild
    pub async fn store_emoji_reactions_enabled(&self, guild_id: i64, enabled: bool) -> Result<(), Error> {
        self.client
            .execute(
                "INSERT INTO guild_emoji_settings (guild_id, emoji_reactions_enabled)
                 VALUES ($1, $2)
                 ON CONFLICT (guild_id) DO UPDATE SET emoji_reactions_enabled = EXCLUDED.emoji_reactions_enabled",
                &[&guild_id, &enabled],
            )
            .await?;
        Ok(())
    }

    pub async fn create_reminder(&self, reminder: &Reminder) -> Result<(), Error> {
        self.client
            .execute(
                "INSERT INTO reminders (guild_id, channel_id, message, time, days, frequency)
             VALUES ($1, $2, $3, $4, $5, $6)",
                &[
                    &reminder.guild_id,
                    &reminder.channel_id,
                    &reminder.message,
                    &reminder.time,
                    &reminder.days.iter().map(|d| d.num_days_from_sunday() as i32).collect::<Vec<i32>>(),
                    &reminder.frequency.to_string(),
                ],
            )
            .await?;
        Ok(())
    }


    pub async fn get_reminders(&self, guild_id: i64) -> Result<Vec<Reminder>, Error> {
        let rows = self.client
            .query(
                "SELECT id, channel_id, message, time, days, frequency FROM reminders WHERE guild_id = $1",
                &[&guild_id],
            )
            .await?;

        let reminders = rows.iter().map(|row| {
            Reminder {
                id: row.get(0),  // Make sure to get the actual ID
                guild_id,
                channel_id: row.get(1),
                message: row.get(2),
                time: row.get(3),
                days: row.get::<_, Vec<i32>>(4)
                    .into_iter()
                    .map(|d| match d {
                        0 => Weekday::Sun,
                        1 => Weekday::Mon,
                        2 => Weekday::Tue,
                        3 => Weekday::Wed,
                        4 => Weekday::Thu,
                        5 => Weekday::Fri,
                        6 => Weekday::Sat,
                        _ => Weekday::Sun, // Default to Sunday for any unexpected values
                    })
                    .collect(),
                frequency: Frequency::from_str(&row.get::<_, String>(5)).unwrap_or(Frequency::Daily),
                last_sent: None,
            }
        }).collect();

        Ok(reminders)
    }

    pub async fn delete_reminder(&self, guild_id: i64, reminder_id: i32) -> Result<(), Error> {
        self.client
            .execute(
                "DELETE FROM reminders WHERE guild_id = $1 AND id = $2",
                &[&guild_id, &reminder_id],
            )
            .await?;
        Ok(())
    }

    pub async fn get_due_reminders(&self) -> Result<Vec<Reminder>, Error> {
        let now = Utc::now().with_timezone(&Los_Angeles);
        let current_time = now.time();
        let current_day = now.weekday().num_days_from_sunday() as i32;

        let query = "
            SELECT id, guild_id, channel_id, message, time, days, frequency, last_sent
            FROM reminders
            WHERE $1 = ANY(days)
            AND $2::time >= time
            AND (
                last_sent IS NULL
                OR (
                    CASE
                        WHEN frequency = 'Daily' THEN
                            $3::date > last_sent::date
                        WHEN frequency = 'Weekly' THEN
                            $3::date >= last_sent::date + INTERVAL '7 days'
                        WHEN frequency = 'Monthly' THEN
                            ($3::date >= last_sent::date + INTERVAL '1 month')
                            AND (EXTRACT(DAY FROM $3::date) = EXTRACT(DAY FROM last_sent::date))
                    END
                    AND $2::time >= time
                )
            )";

        let rows = self.client
            .query(query, &[&current_day, &current_time, &now.date_naive()])
            .await?;

        let reminders: Vec<_> = rows.iter().map(|row| {
            Reminder {
                id: row.get(0),
                guild_id: row.get(1),
                channel_id: row.get(2),
                message: row.get(3),
                time: row.get(4),
                days: row.get::<_, Vec<i32>>(5)
                    .into_iter()
                    .map(|d| match d {
                        0 => Weekday::Sun,
                        1 => Weekday::Mon,
                        2 => Weekday::Tue,
                        3 => Weekday::Wed,
                        4 => Weekday::Thu,
                        5 => Weekday::Fri,
                        6 => Weekday::Sat,
                        _ => Weekday::Sun
                    })
                    .collect(),
                frequency: Frequency::from_str(&row.get::<_, String>(6)).unwrap_or(Frequency::Daily),
                last_sent: row.get(7),
            }
        }).collect();

        Ok(reminders)
    }

    pub async fn update_reminder_last_sent(&self, reminder_id: i32) -> Result<(), Error> {
        self.client
            .execute(
                "UPDATE reminders SET last_sent = CURRENT_TIMESTAMP AT TIME ZONE 'America/Los_Angeles' WHERE id = $1",
                &[&reminder_id],
            )
            .await?;
        Ok(())
    }

    pub async fn increment_blame_count(&self, user_id: i64) -> Result<(i32, i32), Error> {
        let row = self.client
            .query_one(
                "WITH serena_blame AS (
                    INSERT INTO blame_count (id, count) VALUES (1, 1)
                    ON CONFLICT (id) DO UPDATE SET count = blame_count.count + 1
                    RETURNING count AS serena_count
                ),
                user_blame AS (
                    INSERT INTO user_blame_count (user_id, count) VALUES ($1, 1)
                    ON CONFLICT (user_id) DO UPDATE SET count = user_blame_count.count + 1
                    RETURNING count AS user_count
                )
                SELECT serena_count, user_count FROM serena_blame, user_blame",
                &[&user_id],
            )
            .await?;

        Ok((row.get(0), row.get(1)))
    }
    pub async fn add_rule(&self, guild_id: i64, rule: &str) -> Result<(), Error> {
        self.client
            .execute(
                "INSERT INTO guild_rules (guild_id, rule) VALUES ($1, $2)",
                &[&guild_id, &rule],
            )
            .await?;
        Ok(())
    }

    pub async fn remove_rule(&self, guild_id: i64, rule_number: i64) -> Result<(), Error> {
        self.client
            .execute(
                "DELETE FROM guild_rules WHERE guild_id = $1 AND id = (
                    SELECT id FROM (
                        SELECT id, ROW_NUMBER() OVER (ORDER BY id) as row_num
                        FROM guild_rules
                        WHERE guild_id = $1
                    ) t
                    WHERE t.row_num = $2
                )",
                &[&guild_id, &rule_number],
            )
            .await?;
        Ok(())
    }

    pub async fn get_rules(&self, guild_id: i64) -> Result<Vec<String>, Error> {
        let rows = self.client
            .query(
                "SELECT rule FROM guild_rules WHERE guild_id = $1 ORDER BY id",
                &[&guild_id],
            )
            .await?;

        Ok(rows.iter().map(|row| row.get(0)).collect())
    }
}
