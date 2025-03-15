// services/api.rs
use crate::error::Error;
use crate::config::ApiConfig;
use crate::services::cache::{CacheService, CacheResult};
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

#[derive(Clone, Debug)]
pub struct ApiService {
    client: Client,
    config: Arc<ApiConfig>,
    cache: Option<Arc<CacheService>>,
}

impl ApiService {
    pub fn new(config: Arc<ApiConfig>) -> Self {
        Self {
            client: Client::new(),
            config,
            cache: None,
        }
    }
    
    pub fn with_cache(mut self, cache: Arc<CacheService>) -> Self {
        self.cache = Some(cache);
        self
    }

    // OpenAI API service methods
    pub async fn create_chat_completion(&self, system: &str, messages: Vec<(String, String)>) -> Result<String, Error> {
        let mut formatted_messages = vec![
            json!({
                "role": "system",
                "content": system
            })
        ];

        for (role, content) in messages {
            formatted_messages.push(json!({
                "role": role,
                "content": content
            }));
        }

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.config.openai_api_key))
            .json(&json!({
                "model": "gpt-3.5-turbo",
                "messages": formatted_messages,
                "temperature": 0.7
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Unknown(format!("OpenAI API error: {}", error_text)));
        }

        let response_json: Value = response.json().await?;
        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| Error::Unknown("Invalid response from OpenAI API".to_string()))?;

        Ok(content.to_string())
    }

    // Weather API service methods with caching
    pub async fn get_weather(&self, location: &str) -> Result<Value, Error> {
        let cache_key = format!("weather:location:{}", location);
        
        // Create async operation context
        let op_context = crate::utils::AsyncOpContext::new("get_weather")
            .with_timeout(Duration::from_secs(5));
        
        // Try to get from cache first if caching is enabled
        if let Some(cache) = &self.cache {
            match cache.get::<Value, _>(&cache_key).await {
                Ok(result) => {
                    debug!("Cache hit for weather data: {}", location);
                    if result.is_stale {
                        // If stale, fetch fresh data in the background but return cached data immediately
                        let self_clone = self.clone();
                        let location_clone = location.to_string();
                        let cache_key_clone = cache_key.clone();
                        
                        // Use task manager to track this background refresh task
                        let task_name = format!("refresh_weather_cache:{}", location);
                        if let Some(data) = crate::types::DATA.get() {
                            let _ = data.task_manager.spawn_task_with_priority(
                                &task_name,
                                crate::services::TaskPriority::Low,
                                async move {
                                    // Use backoff retry logic for this background task
                                    let mut op_context = crate::utils::AsyncOpContext::new(
                                        format!("refresh_weather_cache:{}", location_clone)
                                    );
                                    if let Ok(fresh_data) = crate::utils::with_retry(
                                        || self_clone.fetch_weather_data(&location_clone),
                                        3, // max retries
                                        Duration::from_millis(100), // base delay
                                        Duration::from_secs(1), // max delay
                                        true, // use jitter
                                        &mut op_context
                                    ).await {
                                        if let Some(cache) = &self_clone.cache {
                                            let ttl = Some(Duration::from_secs(1800)); // 30 minutes
                                            let _ = cache.set_serialized(&cache_key_clone, &fresh_data, ttl, false).await;
                                            debug!("Refreshed cache for weather data: {}", location_clone);
                                        }
                                    }
                                }
                            );
                        } else {
                            // Fallback to normal tokio::spawn if task manager not available
                            tokio::spawn(async move {
                                if let Ok(fresh_data) = self_clone.fetch_weather_data(&location_clone).await {
                                    if let Some(cache) = &self_clone.cache {
                                        let ttl = Some(Duration::from_secs(1800)); // 30 minutes
                                        let _ = cache.set_serialized(&cache_key_clone, &fresh_data, ttl, false).await;
                                    }
                                }
                            });
                        }
                    }
                    return Ok(result.value);
                },
                Err(e) => {
                    debug!("Cache miss for weather data: {} - {}", location, e);
                    // Continue to fetch from API
                }
            }
        }
        
        // Fetch fresh data from API with retry logic
        let mut fetch_context = crate::utils::AsyncOpContext::new(format!("fetch_weather:{}", location));
        let weather_data = crate::utils::with_retry(
            || self.fetch_weather_data(location),
            2, // max attempts
            Duration::from_millis(100), // base delay
            Duration::from_secs(1), // max delay
            true, // use jitter
            &mut fetch_context
        ).await?;
        
        // Apply rate limiting if configured
        if let Some(data) = crate::types::DATA.get() {
            let _ = data.rate_limiter.with_rate_limit(
                "openweather_api", 
                5, // max concurrent requests
                async {
                    // Cache the result if caching is enabled
                    if let Some(cache) = &self.cache {
                        let ttl = Some(Duration::from_secs(1800)); // 30 minutes
                        let _ = cache.set_serialized(&cache_key, &weather_data, ttl, false).await;
                        debug!("Cached weather data for location: {}", location);
                    }
                    Ok::<_, Error>(())
                }
            ).await;
        } else {
            // Fallback if DATA is not available
            if let Some(cache) = &self.cache {
                let ttl = Some(Duration::from_secs(1800)); // 30 minutes
                let _ = cache.set_serialized(&cache_key, &weather_data, ttl, false).await;
                debug!("Cached weather data for location: {}", location);
            }
        }
        
        Ok(weather_data)
    }
    
    // Fetch weather data directly from API without caching
    async fn fetch_weather_data(&self, location: &str) -> Result<Value, Error> {
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
            location, self.config.openweather_api_key
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Unknown(format!("Weather API error: {}", error_text)));
        }

        let weather_data: Value = response.json().await?;
        Ok(weather_data)
    }

    // Image generation API methods
    pub async fn create_image(&self, prompt: &str) -> Result<String, Error> {
        let response = self.client
            .post("https://api.openai.com/v1/images/generations")
            .header("Authorization", format!("Bearer {}", self.config.openai_api_key))
            .json(&json!({
                "prompt": prompt,
                "n": 1,
                "size": "1024x1024"
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Unknown(format!("Image generation API error: {}", error_text)));
        }

        let response_json: Value = response.json().await?;
        let image_url = response_json["data"][0]["url"]
            .as_str()
            .ok_or_else(|| Error::Unknown("Invalid response from image generation API".to_string()))?;

        Ok(image_url.to_string())
    }

    // API Ninjas service methods
    pub async fn get_random_fact(&self, category: &str) -> Result<String, Error> {
        let url = format!(
            "https://api.api-ninjas.com/v1/facts?category={}",
            category
        );

        let response = self.client
            .get(&url)
            .header("X-Api-Key", &self.config.api_ninjas_key)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Unknown(format!("API Ninjas error: {}", error_text)));
        }

        let facts: Value = response.json().await?;
        if let Some(fact) = facts[0]["fact"].as_str() {
            Ok(fact.to_string())
        } else {
            Err(Error::Unknown("No facts found".to_string()))
        }
    }

    // California Fire API service method
    pub async fn get_california_fire_data(&self) -> Result<Value, Error> {
        let url = format!(
            "https://www.fire.ca.gov/umbraco/api/IncidentApi/GetIncidents?year=2022",
        );

        let response = self.client.get(&url).send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(Error::Unknown(format!("California Fire API error: {}", error_text)));
        }

        let fire_data: Value = response.json().await?;
        Ok(fire_data)
    }

    // Additional API methods can be added here as needed
}
