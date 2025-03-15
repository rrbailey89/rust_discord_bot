use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::CreateAttachment;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{error, info};

type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Serialize)]
struct FluxRequest {
    prompt: String,
    width: u32,
    height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    steps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt_upsampling: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    seed: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    guidance: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    safety_tolerance: Option<u32>,
}

#[derive(Deserialize)]
struct FluxResponse {
    id: String,
}

#[derive(Deserialize)]
struct FluxResult {
    status: String,
    result: Option<FluxResultData>,
}

#[derive(Deserialize)]
struct FluxResultData {
    sample: String,
}

#[derive(poise::ChoiceParameter)]
enum Width {
    #[name = "256"]
    W256 = 256,
    #[name = "512"]
    W512 = 512,
    #[name = "768"]
    W768 = 768,
    #[name = "1024"]
    W1024 = 1024,
    #[name = "1280"]
    W1280 = 1280,
    #[name = "1440"]
    W1440 = 1440,
}

impl Default for Width {
    fn default() -> Self {
        Width::W1024
    }
}

#[derive(poise::ChoiceParameter)]
enum Height {
    #[name = "256"]
    H256 = 256,
    #[name = "512"]
    H512 = 512,
    #[name = "768"]
    H768 = 768,
    #[name = "1024"]
    H1024 = 1024,
    #[name = "1280"]
    H1280 = 1280,
    #[name = "1440"]
    H1440 = 1440,
}

impl Default for Height {
    fn default() -> Self {
        Height::H768
    }
}

/// Generate an image using Flux 1.1 Pro
#[poise::command(slash_command)]
pub async fn fluximage(
    ctx: Context<'_>,
    #[description = "Description of the image you want to create"] prompt: String,
    #[description = "Image width"] width: Option<Width>,
    #[description = "Image height"] height: Option<Height>,
    #[description = "Number of steps (1-50)"] steps: Option<u32>,
    #[description = "Enable prompt upsampling"] prompt_upsampling: Option<bool>,
    #[description = "Seed for reproducibility"] seed: Option<u32>,
    #[description = "Guidance scale (1.5-5.0)"] guidance: Option<f32>,
    #[description = "Safety tolerance (0-6, 0 most strict, 6 least strict)"] safety_tolerance: Option<u32>
) -> Result<(), Error> {
    ctx.defer().await?;

    let api_key = &ctx.data().config.api.bfl_api_key;
    let client = reqwest::Client::new();

    let mut headers = HeaderMap::new();
    headers.insert("accept", HeaderValue::from_static("application/json"));
    headers.insert("x-key", HeaderValue::from_str(api_key)?);
    headers.insert("Content-Type", HeaderValue::from_static("application/json"));

    let request = FluxRequest {
        prompt,
        width: width.unwrap_or_default() as u32,
        height: height.unwrap_or_default() as u32,
        steps: steps.map(|s| s.clamp(1, 50)),
        prompt_upsampling,
        seed,
        guidance: guidance.map(|g| g.clamp(1.5, 5.0)),
        safety_tolerance: safety_tolerance.map(|s| s.clamp(0, 6)),
    };

    let response: FluxResponse = client
        .post("https://api.bfl.ml/v1/flux-pro-1.1")
        .headers(headers.clone())
        .json(&request)
        .send()
        .await?
        .json()
        .await?;

    let request_id = response.id;

    let mut result: FluxResult;
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;
        result = client
            .get(format!("https://api.bfl.ml/v1/get_result?id={}", request_id))
            .headers(headers.clone())
            .send()
            .await?
            .json()
            .await?;

        match result.status.as_str() {
            "Ready" => break,
            "Pending" => continue,
            "Task not found" => return Err(Error::Unknown("Task not found".to_string())),
            "Request Moderated" => return Err(Error::Unknown("Request was moderated".to_string())),
            "Content Moderated" => return Err(Error::Unknown("Generated content was moderated".to_string())),
            "Error" => return Err(Error::Unknown("An error occurred during image generation".to_string())),
            _ => return Err(Error::Unknown("Unknown status received".to_string())),
        }
    }

    if let Some(result_data) = result.result {
        match client.get(&result_data.sample).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let image_data = response.bytes().await?;
                    let attachment = CreateAttachment::bytes(image_data, "generated_image.jpg");

                    ctx.send(poise::CreateReply::default()
                        .attachment(attachment))
                        .await?;

                    info!("Image successfully generated and sent");
                } else {
                    error!("Failed to fetch image. Status: {}", response.status());
                    ctx.say("Failed to generate the image. Please try again.").await?;
                }
            }
            Err(e) => {
                error!("Error fetching image: {:?}", e);
                ctx.say("An error occurred while generating the image. Please try again.").await?;
            }
        }
    } else {
        ctx.say("Failed to generate the image. Please try again.").await?;
    }

    Ok(())
}
