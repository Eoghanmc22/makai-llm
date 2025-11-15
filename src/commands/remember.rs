use anyhow::bail;
use async_trait::async_trait;
use serenity::all::{CommandInteraction, CommandType, Context, InteractionContext, Message};
use serenity::builder::CreateCommand;

use crate::commands::{CommandName, MakaiCommand};
use crate::context::MakaiContext;

pub struct RememberCommand;

#[async_trait]
impl MakaiCommand for RememberCommand {
    fn name(&self) -> CommandName {
        "Remember"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .kind(CommandType::Message)
            .add_context(InteractionContext::BotDm)
            .add_context(InteractionContext::Guild)
            .add_context(InteractionContext::PrivateChannel)
    }

    async fn run(
        &self,
        bot_ctx: &MakaiContext,
        discord_ctx: Context,
        cmd: &CommandInteraction,
    ) -> anyhow::Result<()> {
        if let Some(Message {
            author,
            content,
            embeds,
            ..
        }) = cmd.data.resolved.messages.values().next()
        {
            bail!("TODO");
        } else {
            Ok("ERROR: Got no prompt".to_string())
        }
    }
}
