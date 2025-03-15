// services/api.rs
use crate::error::Error;
use crate::config::ApiConfig;
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Clone)]
pub struct ApiService {
    client: Client,
    config: Arc<ApiConfig>,
}

impl ApiService {
    pub fn new(config: Arc<ApiConfig>) -> Self {
        Self {
            client: Client::new(),
            config,
        }
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

    // Weather API service methods
    pub async fn get_weather(&self, location: &str) -> Result<Value, Error> {
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
