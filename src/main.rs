// TODO: Load/Save context
//
// TODO: Move to a vision model (llama 4?, qwen? gemma?), and add image and url support
// TODO: Remember Context
// TODO: add to context command
// TODO: Show thoughts option

pub mod ai;
pub mod commands;
pub mod context;
pub mod utils;

use std::env;

use anyhow::Context as _;
use tracing::error;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};

use serenity::async_trait;
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::commands::MakaiCommandRegistry;
use crate::context::MakaiContext;

struct Handler<'a> {
    commands: MakaiCommandRegistry<'a>,
    context: MakaiContext,
}

#[async_trait]
impl EventHandler for Handler<'_> {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {command:#?}");

            let res = self
                .commands
                .handle_command(&self.context, ctx, &command)
                .await;

            if let Err(err) = res {
                error!("Error while handeling command: {err}");
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        self.commands
            .register_command(ctx)
            .await
            .expect("Register Commands");

        self.context.set_user(ready.user.into());

        println!("Commands registered");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv().context("Load env vars");

    let subscriber = tracing_subscriber::Registry::default().with(
        tracing_subscriber::fmt::layer().with_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        ),
    );
    tracing::subscriber::set_global_default(subscriber)?;

    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler(Handler {
            commands: MakaiCommandRegistry::default(),
            context: MakaiContext::default(),
        })
        .await
        .expect("Error creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(())
}
