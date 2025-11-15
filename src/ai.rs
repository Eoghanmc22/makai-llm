use std::env;

use anyhow::Context;
use chrono::{DateTime, Utc};
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use serenity::all::{CommandInteraction, Message, MessageId, UserId};

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

pub async fn run_llm(ctx: &MakaiContextChannel, message: MakaiMessage) -> anyhow::Result<String> {
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
        .max_tokens(512)
        .system(system)
        .build()
        .context("Failed to build LLM")?;

    let mut messages = ctx.chat_messages().await;
    messages.push(message.to_chat_message());

    let response = llm.chat(&messages).await.context("LLM Error")?;

    // Get rid of thinking stuff
    let text = response.text().unwrap_or_default();
    let text = text.split("▷").last();
    let text = text.unwrap_or_default().to_string();

    // Update stored context
    ctx.add_message(message).await;
    ctx.add_message(MakaiMessage::from_assistant_response(text.clone()))
        .await;

    Ok(text)
}
