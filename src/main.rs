// TODO: Move to qwen-vl-4b, and add image and url support

pub mod ai;
mod commands;

use std::env;

use anyhow::Context as _;
use serenity::all::CreateInteractionResponseFollowup;
use tracing::error;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer, layer::SubscriberExt};

use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{Command, Interaction};
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {command:#?}");

            let data = CreateInteractionResponseMessage::new().content("Processing...");
            let builder = CreateInteractionResponse::Defer(data);
            if let Err(why) = command.create_response(&ctx.http, builder).await {
                println!("Cannot respond to slash command 1: {why}");
            }

            let content = match command.data.name.as_str() {
                "chat" => commands::chat::run(&command).await,
                "Reply" => commands::reply::run(&command).await,
                _ => Ok("not implemented :(".to_string()),
            };

            let content = match content {
                Ok(content) => content,
                Err(err) => {
                    error!("Hit an error: {err:?}");
                    "An error occoured while processing your command!".to_string()
                }
            };

            let builder = CreateInteractionResponseFollowup::new().content(content);
            if let Err(why) = command.create_followup(&ctx.http, builder).await {
                println!("Cannot respond to slash command 2: {why}");
            }
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        // let old_commands = Command::get_global_commands(&ctx.http).await;
        // for old_cmd in old_commands.iter().flatten() {
        //     let _ = Command::delete_global_command(&ctx.http, old_cmd.id).await;
        // }

        let guild_command1 =
            Command::create_global_command(&ctx.http, commands::chat::register()).await;
        let guild_command2 =
            Command::create_global_command(&ctx.http, commands::reply::register()).await;

        println!("I created the following global slash command: {guild_command1:#?}");
        println!("I created the following global slash command: {guild_command2:#?}");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().context("Load env vars")?;

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
        .event_handler(Handler)
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
