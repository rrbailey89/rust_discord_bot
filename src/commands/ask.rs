// commands/ask.rs
use crate::error::Error;
use crate::Data;
use async_openai::{
    types::{
        ChatCompletionRequestSystemMessageArgs,
        ChatCompletionRequestUserMessageArgs,
        CreateChatCompletionRequestArgs,
    },
    Client as OpenAIClient,
};
use async_openai::config::OpenAIConfig;

type Context<'a> = poise::Context<'a, Data, Error>;

/// Ask a question to the AI assistant
#[poise::command(slash_command)]
pub async fn ask(
    ctx: Context<'_>,
    #[description = "Your question"] question: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let config = OpenAIConfig::new().with_api_key(&ctx.data().config.openai_api_key);
    let client = OpenAIClient::with_config(config);

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32)
        .model("gpt-3.5-turbo")
        .messages([
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful assistant.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(question)
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.chat().create(request).await?;

    if let Some(choice) = response.choices.first() {
        if let Some(content) = &choice.message.content {
            ctx.say(content).await?;
        } else {
            ctx.say("No response content received.").await?;
        }
    } else {
        ctx.say("No response received.").await?;
    }

    Ok(())
}