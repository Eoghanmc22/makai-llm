use anyhow::Context as _;
use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, CommandType, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage, InteractionContext, InteractionResponseFlags,
};
use serenity::builder::CreateCommand;
use tracing::error;

use crate::ai::MakaiMessage;
use crate::commands::{CommandName, MakaiCommand};
use crate::context::MakaiContext;

pub struct ResetCommand;

#[async_trait]
impl MakaiCommand for ResetCommand {
    fn name(&self) -> CommandName {
        "reset"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .add_context(InteractionContext::BotDm)
            .add_context(InteractionContext::Guild)
            .add_context(InteractionContext::PrivateChannel)
            .description("Reset message memory")
    }

    async fn run(
        &self,
        bot_ctx: &MakaiContext,
        discord_ctx: Context,
        cmd: &CommandInteraction,
    ) -> anyhow::Result<()> {
        let message = CreateInteractionResponseMessage::default()
            .flags(InteractionResponseFlags::EPHEMERAL)
            .content("Memory for this channel has been cleared");
        let response = CreateInteractionResponse::Message(message);
        if let Err(err) = cmd.create_response(&discord_ctx.http, response).await {
            error!("Cannot ack command: {err:?}");
        }

        bot_ctx.channel(&cmd.channel_id).await.clear().await;

        Ok(())
    }
}
