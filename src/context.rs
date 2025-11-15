use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Utc};
use llm::chat::ChatMessage;
use serenity::all::ChannelId;

#[derive(Default)]
pub struct MakaiContext {
    channels: HashMap<ChannelId, MakaiContextChannel>,
}

impl MakaiContext {
    pub fn channel(&mut self, channel: &ChannelId) -> &mut MakaiContextChannel {
        self.channels.entry(*channel).or_default()
    }
}

#[derive(Default)]
pub struct MakaiContextChannel {
    messages: BTreeMap<DateTime<Utc>, StoredMessage>,
}

impl MakaiContextChannel {
    pub fn add_message(&mut self, message: StoredMessage) {
        self.messages.insert(message.timestamp, message);
    }

    pub fn chat_messages(&self) -> impl Iterator<Item = ChatMessage> {
        self.messages.values().map(StoredMessage::to_chat_message)
    }
}

pub struct StoredMessage {
    pub timestamp: DateTime<Utc>,
    pub sender: MessageSender,
    pub content: String,
}

pub enum MessageSender {
    MakaiBot,
    User(String),
}

impl StoredMessage {
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
