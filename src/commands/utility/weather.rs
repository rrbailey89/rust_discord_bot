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

    // Create a cancellation source that will be cancelled if the command times out
    let cancellation_source = crate::utils::CancellationSource::new();
    let token = cancellation_source.token();
    
    // Create an async operation context
    let mut op_context = crate::utils::AsyncOpContext::new("weather_command")
        .with_timeout(std::time::Duration::from_secs(10));
    
    // Get weather data using the cached API service with retries, timeouts, and cancellation
    let weather_data = crate::utils::with_retry(
        || {
            crate::utils::cancellable(
                ctx.data().api.get_weather(&location),
                token.clone(),
                "weather_fetch"
            )
        },
        3, // max attempts
        std::time::Duration::from_millis(500), // base delay
        std::time::Duration::from_secs(2), // max delay
        true, // use jitter
        &mut op_context
    ).await?;
    
    // Extract necessary information from the API response
    let temp = weather_data["main"]["temp"].as_f64().unwrap_or(0.0);
    let feels_like = weather_data["main"]["feels_like"].as_f64().unwrap_or(0.0);
    let humidity = weather_data["main"]["humidity"].as_i64().unwrap_or(0);
    let pressure = weather_data["main"]["pressure"].as_i64().unwrap_or(0);
    let wind_speed = weather_data["wind"]["speed"].as_f64().unwrap_or(0.0);
    let description = weather_data["weather"][0]["description"].as_str().unwrap_or("Unknown");
    let city_name = weather_data["name"].as_str().unwrap_or(&location);
    let country = weather_data["sys"]["country"].as_str().unwrap_or("Unknown");
    let timestamp = weather_data["dt"].as_i64().unwrap_or(Utc::now().timestamp());
    
    // Can also fetch forecast data in a future enhancement

    // Create the embed to display weather data
    let embed = CreateEmbed::default()
        .title(format!("Weather in {}, {}", city_name, country))
        .field("Temperature", format!("{:.1}°C", temp), true)
        .field("Feels Like", format!("{:.1}°C", feels_like), true)
        .field("Humidity", format!("{}%", humidity), true)
        .field("Wind Speed", format!("{:.1} m/s", wind_speed), true)
        .field("Pressure", format!("{} hPa", pressure), true)
        .field("Description", description, false)
        .footer(CreateEmbedFooter::new("Powered by OpenWeatherMap"))
        .timestamp(DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap())
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
