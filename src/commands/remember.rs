use anyhow::Context as _;
use async_trait::async_trait;
use serenity::all::{
    CacheHttp, CommandInteraction, CommandType, Context, CreateInteractionResponse,
    InteractionContext,
};
use serenity::builder::CreateCommand;

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
        let defer = CreateInteractionResponse::Acknowledge;
        if let Err(err) = cmd.create_response(&discord_ctx.http, defer).await {
            println!("Cannot ack command: {err}");
        }

        let user = bot_ctx
            .user()
            .await
            .context("Got command before user is known")?;

        let message =
            MakaiMessage::from_message_command(user.id, cmd).context("Get message from command")?;

        discord_ctx
            .http()
            .create_reaction(
                cmd.channel_id,
                message.message_id.context("Get message id")?,
                &'👍'.into(),
            )
            .await
            .context("Add reaction")?;

        bot_ctx.channel(&cmd.channel_id).await.add_message(message);

        Ok(())
    }
}
