use crate::error::Error;
use crate::Data;
use poise::serenity_prelude::{Context, Message, ReactionType};
use poise::FrameworkContext;
use async_openai::{Client, types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessage,ChatCompletionRequestUserMessage, ChatCompletionRequestSystemMessage}};
use rand::Rng;

pub async fn handle_message(
    ctx: &Context,
    _framework: FrameworkContext<'_, Data, Error>,
    data: &Data,
    message: &Message,
) -> Result<(), Error> {
    let keywords = vec!["pierce brosnan", "yoshi p", "yoship", "yoshi-p", "japan", "raid", "shion", "saskia", "erik", "amia", "opal", "lief", "dyna", "bot", "cat", "broken", "lol", "lmao", "a8s"];
    let content = message.content.to_lowercase();

    let mut trigger_word_found = false;
    for keyword in keywords {
        if content.contains(keyword) {
            trigger_word_found = true;
            if rand::thread_rng().gen_range(0..100) <= 40 {
                let emoji = match keyword {
                    "cat" => "🐱",
                    "broken" => "🤔",
                    "lol" | "lmao" => "🤣",
                    "saskia" => "💜",
                    "yoship" | "yoshi p" | "yoshi-p" => "<:wine31:1223785508702781460>",
                    _ => "🤖",
                };
                message.react(ctx, ReactionType::Unicode(emoji.to_string())).await?;
            }
            break;
        }
    }

    if !trigger_word_found && rand::thread_rng().gen_range(0..100) <= 15 {
        let emoji = get_emoji_from_openai(&data.config.openai_api_key, &content).await?;
        message.react(ctx, ReactionType::Unicode(emoji)).await?;
    }

    Ok(())
}

async fn get_emoji_from_openai(api_key: &str, message: &str) -> Result<String, Error> {
    let client = Client::with_config(async_openai::config::OpenAIConfig::new().with_api_key(api_key));

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-3.5-turbo")
        .messages(vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: async_openai::types::ChatCompletionRequestSystemMessageContent::Text("You are only capable of responding to the message with a single emoji that best represents the message.".to_string()),
                name: None,

            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(message.to_string()),
                name: None,

            }),
        ])
        .build()?;

    let response = client.chat().create(request).await?;
    let emoji = response.choices[0].message.content.clone()
        .ok_or_else(|| Error::Unknown("OpenAI response did not contain content".to_string()))?;

    Ok(emoji)
}

