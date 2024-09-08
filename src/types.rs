use bytes::BytesMut;

use chrono::NaiveTime;
use postgres_types::{accepts, to_sql_checked, FromSql, IsNull, ToSql, Type};
use std::error::Error;

#[derive(Debug, Clone, Copy)]
pub struct Time(pub NaiveTime);

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
