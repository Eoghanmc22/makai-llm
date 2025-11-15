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

impl Clone for MakaiContext {
    fn clone(&self) -> Self {
        tokio::task::block_in_place(|| Self {
            channels: RwLock::new(self.channels.blocking_read().clone()),
            bot_user: RwLock::new(self.bot_user.blocking_read().clone()),
        })
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

impl Clone for MakaiContextChannel {
    fn clone(&self) -> Self {
        tokio::task::block_in_place(|| Self {
            messages: RwLock::new(self.messages.blocking_read().clone()),
        })
    }
}

pub mod serde {
    use ::serde::{Deserialize, Serialize};

    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MakaiContextSerde {
        channels: HashMap<ChannelId, MakaiContextChannelSerde>,
        bot_user: Option<User>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MakaiContextChannelSerde {
        messages: BTreeMap<DateTime<Utc>, MakaiMessage>,
    }

    impl From<MakaiContextChannel> for MakaiContextChannelSerde {
        fn from(value: MakaiContextChannel) -> Self {
            let MakaiContextChannel { messages } = value;

            MakaiContextChannelSerde {
                messages: messages.into_inner(),
            }
        }
    }

    impl From<MakaiContextChannelSerde> for MakaiContextChannel {
        fn from(value: MakaiContextChannelSerde) -> Self {
            let MakaiContextChannelSerde { messages } = value;

            MakaiContextChannel {
                messages: messages.into(),
            }
        }
    }

    impl From<MakaiContext> for MakaiContextSerde {
        fn from(value: MakaiContext) -> Self {
            let MakaiContext { channels, bot_user } = value;

            MakaiContextSerde {
                channels: channels
                    .into_inner()
                    .into_iter()
                    .map(|(channel, ctx)| (channel, Arc::unwrap_or_clone(ctx).into()))
                    .collect(),
                bot_user: bot_user.into_inner(),
            }
        }
    }

    impl From<MakaiContextSerde> for MakaiContext {
        fn from(value: MakaiContextSerde) -> Self {
            let MakaiContextSerde { channels, bot_user } = value;

            MakaiContext {
                channels: channels
                    .into_iter()
                    .map(|(channel, ctx)| (channel, Arc::new(ctx.into())))
                    .collect::<HashMap<_, _>>()
                    .into(),
                bot_user: bot_user.into(),
            }
        }
    }
}
