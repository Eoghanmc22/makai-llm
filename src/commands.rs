use anyhow::Context as _;
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::error;

use serenity::all::{
    Command, CommandInteraction, Context, CreateCommand, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, InteractionResponseFlags,
};

use crate::{
    commands::{
        chat::ChatCommand, remember::RememberCommand, reply::ReplyCommand, reset::ResetCommand,
    },
    context::MakaiContext,
};

pub mod chat;
pub mod remember;
pub mod reply;
pub mod reset;

pub type CommandName = &'static str;

#[async_trait]
pub trait MakaiCommand {
    fn name(&self) -> CommandName;
    fn register(&self) -> CreateCommand;
    async fn run(
        &self,
        bot_ctx: &MakaiContext,
        discord_ctx: Context,
        cmd: &CommandInteraction,
    ) -> anyhow::Result<()>;
}

pub struct MakaiCommandRegistry<'a> {
    commands: HashMap<CommandName, Box<dyn MakaiCommand + Send + Sync + 'a>>,
}

impl<'a> MakaiCommandRegistry<'a> {
    pub fn empty() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    pub fn add_command(&mut self, command: impl MakaiCommand + Send + Sync + 'a) {
        self.commands.insert(command.name(), Box::new(command));
    }

    pub async fn register_command(&self, discord_ctx: Context) -> anyhow::Result<()> {
        for command in self.commands.values() {
            Command::create_global_command(&discord_ctx.http, command.register())
                .await
                .with_context(|| format!("Register `{}` command", command.name()))?;
        }

        Ok(())
    }

    pub async fn handle_command(
        &self,
        bot_ctx: &MakaiContext,
        discord_ctx: Context,
        interaction: &CommandInteraction,
    ) -> anyhow::Result<()> {
        let Some(cmd) = self.commands.get(interaction.data.name.as_str()) else {
            let builder = CreateInteractionResponseMessage::default()
                .flags(InteractionResponseFlags::EPHEMERAL)
                .content("not implemented :(");
            let defer = CreateInteractionResponse::Message(builder);
            if let Err(err) = interaction.create_response(&discord_ctx.http, defer).await {
                error!("Cannot defer to slash command: {err:?}");
            }

            return Ok(());
        };

        let res = cmd.run(bot_ctx, discord_ctx.clone(), interaction).await;

        if let Err(_) = res {
            let follow_up = CreateInteractionResponseFollowup::default()
                .content("An error occoured while processing your command!");
            interaction
                .create_followup(&discord_ctx.http, follow_up)
                .await
                .context("Cannot followup command")?;
        }

        res
    }
}

impl Default for MakaiCommandRegistry<'_> {
    fn default() -> Self {
        let mut reg = Self::empty();

        reg.add_command(ReplyCommand);
        reg.add_command(ChatCommand);
        reg.add_command(RememberCommand);
        reg.add_command(ResetCommand);

        reg
    }
}
