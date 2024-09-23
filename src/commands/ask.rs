use crate::{error::Error, Data};
use async_openai::{
    types::{
        AssistantStreamEvent, CreateMessageRequest, CreateMessageRequestContent,
        CreateRunRequest, CreateThreadAndRunRequest,
        CreateThreadRequest, MessageDeltaContent,
        MessageRole,
    },
    Client,
};
use futures::StreamExt;

#[poise::command(slash_command, guild_only)]
pub async fn ask(
    ctx: poise::Context<'_, Data, Error>,
    #[description = "Your question"] question: String,
) -> Result<(), Error> {
    ctx.defer().await?;
    let user_id = ctx.author().id;
    let openai_api_key = &ctx.data().config.openai_api_key;
    let client = Client::with_config(async_openai::config::OpenAIConfig::new().with_api_key(openai_api_key));
    let assistant_id = "asst_u0ap3h8EWbamvzjZVmZJxrfj";

    let thread_id = ctx.data().database.get_conversation_thread_id_for_user(user_id).await?;

    let mut stream = if let Some(ref thread_id) = thread_id {
        let request = CreateRunRequest {
            assistant_id: assistant_id.to_string(),
            model: None,
            instructions: None,
            tools: None,
            metadata: None,
            stream: Some(true),
            temperature: None,
            top_p: None,
            max_completion_tokens: None,
            max_prompt_tokens: None,
            parallel_tool_calls: None,
            response_format: None,
            tool_choice: None,
            truncation_strategy: None,
            additional_instructions: None,
            additional_messages: None,
        };

        client.threads().messages(&thread_id).create(CreateMessageRequest {
            role: MessageRole::User,
            content: CreateMessageRequestContent::Content(question),
            attachments: None,
            metadata: None,
        }).await?;

        client.threads().runs(&thread_id).create_stream(request).await?
    } else {
        let request = CreateThreadAndRunRequest {
            assistant_id: assistant_id.to_string(),
            thread: Some(CreateThreadRequest {
                messages: Some(vec![CreateMessageRequest {
                    role: MessageRole::User,
                    content: CreateMessageRequestContent::Content(question),
                    attachments: None,
                    metadata: None,
                }]),
                metadata: None,
                tool_resources: None,
            }),
            model: None,
            instructions: None,
            tools: None,
            metadata: None,
            stream: Some(true),
            temperature: None,
            top_p: None,
            max_completion_tokens: None,
            max_prompt_tokens: None,
            parallel_tool_calls: None,
            response_format: None,
            tool_choice: None,
            truncation_strategy: None,
            tool_resources: None,
        };

        client.threads().create_and_run_stream(request).await?
    };

    if thread_id.is_none() {
        if let Some(Ok(AssistantStreamEvent::TreadCreated(thread))) = stream.next().await {
            let new_thread_id = thread.id.clone();
            ctx.data().database.store_conversation_thread_id_for_user(user_id, new_thread_id).await?;
        }
    }

    let mut response = String::new();
    while let Some(event) = stream.next().await {
        match event {
            Ok(event) => {
                match event {
                    AssistantStreamEvent::ThreadMessageDelta(delta) => {
                        if let Some(content) = delta.delta.content {
                            for content_part in content {
                                if let MessageDeltaContent::Text(text_object) = content_part {
                                    if let Some(text_value) = text_object.text.unwrap().value {
                                        response.push_str(&text_value);
                                    }
                                }
                            }
                        }
                    }
                    AssistantStreamEvent::ThreadRunCompleted(_) => {
                        break;
                    }
                    _ => {}
                }
            },
            Err(e) => {
                return Err(Error::OpenAI(e));
            }
        }
    }

    ctx.send(poise::CreateReply::default().content(response)).await?;
    Ok(())
}