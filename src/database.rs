// database.rs
use tokio_postgres::{Client, NoTls};
use crate::error::Error;

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
}