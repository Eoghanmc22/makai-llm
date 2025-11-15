use std::{borrow::Cow, env};

use anyhow::Context as _;
use chrono::{DateTime, Utc};
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::{ChatMessage, Usage},
};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serenity::all::{
    CacheHttp, CommandInteraction, Context, CreateAttachment, CreateInteractionResponseFollowup,
    Message, MessageId, UserId,
};

use crate::{context::MakaiContextChannel, utils::user_to_name};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakaiMessage {
    pub message_id: Option<MessageId>,
    pub timestamp: DateTime<Utc>,
    pub sender: MessageSender,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageSender {
    MakaiBot,
    User(String),
}

impl MakaiMessage {
    pub fn from_message_command(bot_id: UserId, interaction: &CommandInteraction) -> Option<Self> {
        if let Some(Message {
            id,
            author,
            timestamp,
            content,
            embeds,
            ..
        }) = interaction.data.resolved.messages.values().next()
        {
            let timestamp = timestamp.to_utc();

            let sender = if bot_id == author.id {
                MessageSender::MakaiBot
            } else {
                MessageSender::User(user_to_name(author).to_string())
            };

            let mut content = content.clone();

            for embed in embeds {
                content.push_str(&format!(
                    "\nThe user's message included a link: Title: `{}`, Description: `{}`",
                    embed
                        .title
                        .as_ref()
                        .map(|it| it.as_str())
                        .unwrap_or("Unknown"),
                    embed
                        .description
                        .as_ref()
                        .map(|it| it.as_str())
                        .unwrap_or("Unknown")
                ));
            }

            Some(Self {
                message_id: Some(*id),
                timestamp,
                sender,
                content,
            })
        } else {
            None
        }
    }

    pub fn from_assistant_response(content: String) -> Self {
        Self {
            message_id: None,
            timestamp: Utc::now(),
            sender: MessageSender::MakaiBot,
            content,
        }
    }

    pub fn to_chat_message(&self) -> ChatMessage {
        match &self.sender {
            MessageSender::MakaiBot => ChatMessage::assistant()
                .content(format!("You (MakAI) said: {}", self.content))
                .build(),
            MessageSender::User(sender) => ChatMessage::user()
                .content(format!("User `{sender}` said: {}", self.content))
                .build(),
        }
    }
}

pub async fn run_llm(
    ctx: &MakaiContextChannel,
    message: MakaiMessage,
) -> anyhow::Result<LlmResponse> {
    let url = env::var("LLM_API").context("Expected a llm api url in env")?;
    let model = env::var("LLM_MODEL").context("Expected a llm model in env")?;
    let prompt_file = env::var("LLM_PROMPT_FILE").context("Expected a prompt file in env")?;
    let words_file = env::var("LLM_WORDS_FILE").context("Expected a words file in env")?;

    let system = tokio::fs::read_to_string(prompt_file)
        .await
        .context("Read prompt file")?;
    let words = tokio::fs::read_to_string(words_file)
        .await
        .context("Read words file")?;
    let mut words = words.lines().collect::<Vec<&str>>();
    words.shuffle(&mut rand::rng());
    let system = system.replace(
        "{WORDS}",
        &words.iter().fold(String::new(), |mut acc, it| {
            acc.push_str(&format!("- {it}\n"));
            acc
        }),
    );

    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key("funny-api-key")
        .base_url(url)
        .model(model)
        .system(system)
        .build()
        .context("Failed to build LLM")?;

    let mut messages = ctx.chat_messages(20).await;
    messages.push(message.to_chat_message());
    messages.push(
        ChatMessage::user()
            .content("Generate a makian reply to the previous message.")
            .build(),
    );

    let response = llm.chat(&messages).await.context("LLM Error")?;
    let text = response.text().unwrap_or_default();

    // Update stored context
    ctx.add_message(message).await;
    ctx.add_message(MakaiMessage::from_assistant_response(text.clone()))
        .await;

    Ok(LlmResponse {
        response: text,
        usage: response.usage(),
    })
}

pub struct LlmResponse {
    pub response: String,
    pub usage: Option<Usage>,
}

impl LlmResponse {
    pub async fn send_follow_up(
        &self,
        discord_ctx: Context,
        cmd: &CommandInteraction,
    ) -> anyhow::Result<()> {
        let content = if let Some(usage) = &self.usage {
            &format!(
                "{}\n-# Generated {} tokens",
                self.response.trim(),
                usage.completion_tokens
            )
        } else {
            &self.response
        };
        let follow_up = CreateInteractionResponseFollowup::default().content(content);
        let res1 = cmd
            .create_followup(discord_ctx.http(), follow_up)
            .await
            .context("Cannot followup command");

        if let Err(_) = res1 {
            let word_wrapped = self
                .response
                .lines()
                .map(|it| {
                    if it.len() > 100 {
                        let mut cumlative_buf = String::new();
                        let mut line_buf = String::new();

                        for word in it.split_whitespace() {
                            line_buf.push_str(word);
                            if line_buf.len() > 70 {
                                line_buf.push('\n');
                                cumlative_buf.push_str(&line_buf);
                                line_buf.clear();
                            } else {
                                line_buf.push(' ');
                            }
                        }
                        cumlative_buf.push_str(&line_buf);
                        Cow::Owned(cumlative_buf)
                    } else {
                        Cow::Borrowed(it)
                    }
                })
                .fold(String::new(), |mut acc, chunk| {
                    acc.push_str(chunk.trim());
                    acc.push('\n');
                    acc
                });

            let follow_up = CreateInteractionResponseFollowup::default()
                .add_file(CreateAttachment::bytes(word_wrapped.as_bytes(), "raw.txt"));

            let follow_up = if let Some(usage) = &self.usage {
                follow_up.content(format!("-# Generated {} tokens", usage.completion_tokens))
            } else {
                follow_up
            };

            cmd.create_followup(discord_ctx.http(), follow_up)
                .await
                .context("Cannot followup command")?;
        }

        Ok(())
    }
}
