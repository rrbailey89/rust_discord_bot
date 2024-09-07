// database.rs
use tokio_postgres::{Client, NoTls};
use crate::error::Error;
use poise::serenity_prelude::{Guild, UserId, ChannelType};

pub struct Database {
    client: Client,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self, Error> {
        let (client, connection) = tokio_postgres::connect(url, NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });

        Ok(Self { client })
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
}