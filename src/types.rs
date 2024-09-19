use bytes::BytesMut;
use serenity::prelude::TypeMapKey;
use serenity::gateway::ShardManager;
use std::sync::Arc;
use chrono::NaiveTime;
use postgres_types::{accepts, to_sql_checked, FromSql, IsNull, ToSql, Type};
use std::error::Error;
use serde::{Deserialize, Serialize};
use crate::Data;
#[derive(Debug, Clone, Copy)]
pub struct Time(pub NaiveTime);
pub struct ShardManagerContainer;

pub struct DataContainer;
impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<ShardManager>;
}
impl TypeMapKey for DataContainer {
    type Value = Data;
}

impl<'a> FromSql<'a> for Time {
    fn from_sql(ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        let time: &str = FromSql::from_sql(ty, raw)?;
        Ok(Time(NaiveTime::parse_from_str(time, "%H:%M:%S").map_err(|e| Box::new(e) as Box<dyn Error + Sync + Send>)?))
    }

    accepts!(TIME);
}

impl ToSql for Time {
    fn to_sql(&self, _ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        let time_str = self.0.format("%H:%M:%S").to_string();
        out.extend_from_slice(time_str.as_bytes());
        Ok(IsNull::No)
    }

    accepts!(TIME);
    to_sql_checked!();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlRule {
    pub guild_id: i64,
    pub channel_id: i64,
    pub regex: String,
    pub output_template: String,
}