use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc, TimeZone};
use chrono_tz::Tz;
use crate::error::Error;

pub fn parse_datetime(month: &str, day: i64, year: i64, time: &str, timezone: &str) -> Result<DateTime<Utc>, Error> {
    let month_num = match month.to_lowercase().as_str() {
        "january" => 1, "february" => 2, "march" => 3, "april" => 4,
        "may" => 5, "june" => 6, "july" => 7, "august" => 8,
        "september" => 9, "october" => 10, "november" => 11, "december" => 12,
        _ => return Err(Error::Unknown("Invalid month".to_string())),
    };

    let (hour, minute) = parse_time(time)?;

    let naive_date = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(year as i32, month_num, day as u32).ok_or_else(|| Error::Unknown("Invalid date".to_string()))?,
        NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| Error::Unknown("Invalid time".to_string()))?,
    );

    let tz: Tz = timezone.parse()?;
    Ok(tz.from_local_datetime(&naive_date)
        .single() // This replaces `unwrap()` and handles ambiguous times
        .ok_or_else(|| Error::Unknown("Ambiguous or non-existent local time".to_string()))?
        .with_timezone(&Utc))
}

fn parse_time(time: &str) -> Result<(u32, u32), Error> {
    let parts: Vec<&str> = time.split(':').collect();
    if parts.len() != 2 {
        return Err(Error::Unknown("Invalid time format".to_string()));
    }

    let hour: u32 = parts[0].parse().map_err(|_| Error::Unknown("Invalid hour".to_string()))?;
    let minute: u32 = parts[1].split_whitespace().next().unwrap().parse().map_err(|_| Error::Unknown("Invalid minute".to_string()))?;
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