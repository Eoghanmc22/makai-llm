use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use chrono::{DateTime, Utc};
use llm::chat::ChatMessage;
use serenity::all::{ChannelId, User};
use tokio::sync::RwLock;

use crate::ai::MakaiMessage;

#[derive(Default)]
pub struct MakaiContext {
    channels: RwLock<HashMap<ChannelId, Arc<MakaiContextChannel>>>,
    bot_user: RwLock<Option<User>>,
}

impl MakaiContext {
    pub async fn channel(&self, channel: &ChannelId) -> Arc<MakaiContextChannel> {
        let read_lock = self.channels.read().await;

        if let Some(channel) = read_lock.get(channel) {
            channel.clone()
        } else {
            drop(read_lock);

            self.channels
                .write()
                .await
                .entry(*channel)
                .or_default()
                .clone()
        }
    }

    pub async fn user(&self) -> Option<User> {
        self.bot_user.read().await.clone()
    }

    pub async fn set_user(&self, user: User) {
        *self.bot_user.write().await = Some(user);
    }
}

#[derive(Default)]
pub struct MakaiContextChannel {
    messages: RwLock<BTreeMap<DateTime<Utc>, MakaiMessage>>,
}

impl MakaiContextChannel {
    pub async fn add_message(&self, message: MakaiMessage) {
        self.messages
            .write()
            .await
            .insert(message.timestamp, message);
    }

    pub async fn chat_messages(&self) -> Vec<ChatMessage> {
        self.messages
            .read()
            .await
            .values()
            .map(MakaiMessage::to_chat_message)
            .collect()
    }
}
