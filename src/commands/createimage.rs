use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{CreateAttachment};
use async_openai::{Client, types::{CreateImageRequestArgs, ImageModel, ImageSize, ImageStyle, ImageResponseFormat, Image, ImageQuality}, };


type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(poise::ChoiceParameter)]
pub enum ImageSizeChoice {
    #[name = "1024x1024"]
    Size1024,
    #[name = "1792x1024"]
    Size1792x1024,
    #[name = "1024x1792"]
    Size1024x1792,
}

#[derive(poise::ChoiceParameter)]
pub enum ImageStyleChoice {
    #[name = "vivid"]
    Vivid,
    #[name = "natural"]
    Natural,
}

#[derive(poise::ChoiceParameter)]
pub enum ImageQualityChoice {
    #[name = "standard"]
    Standard,
    #[name = "hd"]
    HD,
}

/// Create an image using DALL-E 3
#[poise::command(slash_command)]
pub async fn createimage(
    ctx: Context<'_>,
    #[description = "Description of the image you want to create"]
    prompt: String,
    #[description = "Image size"]
    size: Option<ImageSizeChoice>,
    #[description = "Image style"]
    style: Option<ImageStyleChoice>,
    #[description = "Image quality (HD or standard)"]
    quality: Option<ImageQualityChoice>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let openai_api_key = &ctx.data().config.openai_api_key;
    let client = Client::with_config(async_openai::config::OpenAIConfig::new().with_api_key(openai_api_key));

    let size = match size {
        Some(ImageSizeChoice::Size1024) => ImageSize::S1024x1024,
        Some(ImageSizeChoice::Size1792x1024) => ImageSize::S1792x1024,
        Some(ImageSizeChoice::Size1024x1792) => ImageSize::S1024x1792,
        None => ImageSize::S1024x1024,
    };

    let style = match style {
        Some(ImageStyleChoice::Vivid) => ImageStyle::Vivid,
        Some(ImageStyleChoice::Natural) => ImageStyle::Natural,
        None => ImageStyle::Vivid,
    };

    let quality = match quality {
        Some(ImageQualityChoice::HD) => ImageQuality::HD,
        Some(ImageQualityChoice::Standard) | None => ImageQuality::Standard,
    };

    let request = CreateImageRequestArgs::default()
        .prompt(prompt)
        .model(ImageModel::DallE3)
        .n(1)
        .size(size)
        .style(style)
        .quality(quality)
        .response_format(ImageResponseFormat::Url)
        .build()?;

    let response = client.images().create(request).await?;

    if let Some(image) = response.data.first() {
        if let Image::Url { url, .. } = image.as_ref() {
            let image_data = reqwest::get(url).await?.bytes().await?;
            let attachment = CreateAttachment::bytes(image_data, "generated_image.png");
            let reply = poise::CreateReply::default()
                .content("Here's your generated image:")
                .attachment(attachment);
            ctx.send(reply).await?;
        } else {
            ctx.say("Unexpected image format received. Please try again.").await?;
        }
    } else {
        ctx.say("Failed to generate the image. Please try again.").await?;
    }

    Ok(())
}