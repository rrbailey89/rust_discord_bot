// commands/weather.rs
use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter};
use serde::Deserialize;
use chrono::{DateTime, Utc};

type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Deserialize)]
struct GeocodingResponse {
    name: String,
    lat: f64,
    lon: f64,
    country: String,
    state: Option<String>,
}

#[derive(Deserialize)]
struct WeatherResponse {
    timezone: String,
    current: CurrentWeather,
    daily: Vec<DailyForecast>,
}

#[derive(Deserialize)]
struct CurrentWeather {
    dt: i64,
    temp: f32,
    feels_like: f32,
    pressure: i32,
    humidity: i32,
    wind_speed: f32,
    weather: Vec<WeatherInfo>,
}

#[derive(Deserialize)]
struct DailyForecast {
    dt: i64,
    temp: Temperature,
    weather: Vec<WeatherInfo>,
}

#[derive(Deserialize)]
struct Temperature {
    min: f32,
    max: f32,
}

#[derive(Deserialize)]
struct WeatherInfo {
    description: String,
}

/// Get the current weather and forecast for a city
#[poise::command(slash_command)]
pub async fn weather(
    ctx: Context<'_>,
    #[description = "City name (and state if in the US, e.g., 'Austin, TX')"] location: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let api_key = &ctx.data().config.openweather_api_key;

    // Parse the location input
    let (city, state) = parse_location(&location);

    // Construct the query string for the geocoding API
    let query = if let Some(state) = state {
        format!("{},{},US", city, state)
    } else {
        city
    };

    // First, get coordinates using the geocoding API
    let geocoding_url = format!(
        "http://api.openweathermap.org/geo/1.0/direct?q={}&limit=1&appid={}",
        query, api_key
    );

    let client = reqwest::Client::new();
    let geocoding_response: Vec<GeocodingResponse> = client.get(&geocoding_url).send().await?.json().await?;

    if geocoding_response.is_empty() {
        return Err(Error::Unknown("Location not found".to_string()));
    }

    let location = &geocoding_response[0];

    // Now, get the weather data using the coordinates
    let weather_url = format!(
        "https://api.openweathermap.org/data/3.0/onecall?lat={}&lon={}&exclude=minutely,hourly,alerts&units=imperial&appid={}",
        location.lat, location.lon, api_key
    );

    let weather_response: WeatherResponse = client.get(&weather_url).send().await?.json().await?;

    let current = &weather_response.current;
    let daily = &weather_response.daily;

    let location_name = if let Some(state) = &location.state {
        format!("{}, {}", location.name, state)
    } else {
        location.name.clone()
    };

    let embed = CreateEmbed::default()
        .title(format!("Weather in {}, {}", location_name, location.country))
        .field("Temperature", format!("{:.1}°F", current.temp), true)
        .field("Feels Like", format!("{:.1}°F", current.feels_like), true)
        .field("Humidity", format!("{}%", current.humidity), true)
        .field("Wind Speed", format!("{:.1} mph", current.wind_speed), true)
        .field("Pressure", format!("{} hPa", current.pressure), true)
        .field("Description", &current.weather[0].description, false)
        .field("Forecast", format_forecast(daily), false)
        .footer(CreateEmbedFooter::new(format!("Timezone: {}", weather_response.timezone)))
        .timestamp(DateTime::<Utc>::from_timestamp(current.dt, 0).unwrap())
        .color(0x00BFFF);

    ctx.send(poise::CreateReply::default().embed(embed)).await?;

    Ok(())
}

fn format_forecast(daily: &[DailyForecast]) -> String {
    daily.iter().take(3).map(|day| {
        let date = DateTime::<Utc>::from_timestamp(day.dt, 0).unwrap();
        format!(
            "{}: {:.1}°F to {:.1}°F, {}",
            date.format("%A"),
            day.temp.min,
            day.temp.max,
            day.weather[0].description
        )
    }).collect::<Vec<String>>().join("\n")
}

fn parse_location(input: &str) -> (String, Option<String>) {
    let parts: Vec<&str> = input.split(',').map(str::trim).collect();
    match parts.len() {
        1 => (parts[0].to_string(), None),
        2 => (parts[0].to_string(), Some(parts[1].to_string())),
        _ => (input.to_string(), None), // If there are more than 2 parts, treat the whole input as the city
    }
}