// TODO: Move to a vision model (llama 4?, qwen? gemma?), and add image and url support
// TODO: Show thoughts option

pub mod ai;
pub mod commands;
pub mod context;
pub mod utils;

use std::env;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context as _;
use futures::FutureExt;
use serenity::all::ShardManager;
use tracing::level_filters::LevelFilter;
use tracing::{debug, error, info, warn};
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};

use serenity::async_trait;
use serenity::model::application::Interaction;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::commands::MakaiCommandRegistry;
use crate::context::MakaiContext;
use crate::context::serde::MakaiContextSerde;

const STATE_PATH: &str = "./makai_state.json";
const SAVE_INTERVAL: Duration = Duration::from_secs(5 * 60);

struct Handler {
    commands: MakaiCommandRegistry<'static>,
    context: MakaiContext,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            debug!("Received command interaction: {command:#?}");

            let res = self
                .commands
                .handle_command(&self.context, ctx, &command)
                .await;

            if let Err(err) = res {
                error!("Error while handeling command: {err:?}");
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);

        self.commands
            .register_command(ctx)
            .await
            .expect("Register Commands");

        self.context.set_user(ready.user.into()).await;

        info!("Commands registered");
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

    let handler = Handler {
        commands: MakaiCommandRegistry::default(),
        context: load_state().await.context("Load State")?,
    };
    let handler = Arc::new(handler);

    // Build our client.
    let mut client = Client::builder(token, GatewayIntents::empty())
        .event_handler_arc(handler.clone())
        .await
        .expect("Error creating client");

    start_auto_save_and_shutdown_hooks(handler.clone(), client.shard_manager.clone());

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }

    save_state(handler.context.clone())
        .await
        .context("Save state")?;

    Ok(())
}

async fn load_state() -> anyhow::Result<MakaiContext> {
    let exists = tokio::fs::try_exists(&STATE_PATH)
        .await
        .context("Check if state exists")?;

    if !exists {
        info!("No saved state found, Starting with clean state!");

        return Ok(MakaiContext::default());
    }

    let state = tokio::fs::read_to_string(&STATE_PATH)
        .await
        .context("Read State")?;
    let state: MakaiContextSerde = serde_json::from_str(&state).context("Parse State")?;

    Ok(state.into())
}

async fn save_state(state: MakaiContext) -> anyhow::Result<()> {
    info!("Saving");

    let state: MakaiContextSerde = state.into();
    let state = serde_json::to_vec(&state).context("Encode State")?;
    tokio::fs::write(&STATE_PATH, state)
        .await
        .context("Write State")?;

    Ok(())
}

fn start_auto_save_and_shutdown_hooks(state: Arc<Handler>, shard_manager: Arc<ShardManager>) {
    let (tx_needs_save, rx_needs_save) = tokio::sync::oneshot::channel::<()>();
    let (tx_save_done, rx_save_done) = tokio::sync::oneshot::channel::<()>();

    {
        let mut channels = Some((tx_needs_save, rx_save_done));

        ctrlc::set_handler(move || {
            if let Some((tx_needs_save, rx_save_done)) = channels.take() {
                info!("Got ctrlc");
                tx_needs_save.send(()).unwrap();
                let _ = rx_save_done.blocking_recv();
                info!("Exiting");
            } else {
                warn!("Got multiple ctrlc");
            }
        })
        .expect("Set term handler");
    }

    let mut tx_save_done = Some(tx_save_done);
    let mut rx_needs_save = rx_needs_save.fuse();

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(SAVE_INTERVAL);

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let res = save_state(state.context.clone()).await;
                    if let Err(err) = res {
                        error!("Error while saving: {err:?}");
                    }
                }
                _ = &mut rx_needs_save => {
                    let res = save_state(state.context.clone()).await;
                    if let Err(err) = res {
                        error!("Error while saving: {err:?}");
                    }

                    if let Some(tx_save_done) = tx_save_done.take() {
                        info!("Shutting Down");
                        shard_manager.shutdown_all().await;

                        let _ = tx_save_done.send(());
                    }
                }
            }
        }
    });
}
