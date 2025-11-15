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
        let message = CreateInteractionResponseMessage::default()
            .flags(InteractionResponseFlags::EPHEMERAL)
            .content("Added to memory");
        let response = CreateInteractionResponse::Message(message);
        if let Err(err) = cmd.create_response(&discord_ctx.http, response).await {
            error!("Cannot ack command: {err:?}");
        }

        let user = bot_ctx
            .user()
            .await
            .context("Got command before user is known")?;

        let message =
            MakaiMessage::from_message_command(user.id, cmd).context("Get message from command")?;

        bot_ctx
            .channel(&cmd.channel_id)
            .await
            .add_message(message)
            .await;

        Ok(())
    }
}
