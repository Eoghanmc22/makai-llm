use anyhow::Context as _;
use async_trait::async_trait;
use serenity::all::{
    CommandInteraction, CommandType, Context, CreateInteractionResponse,
    CreateInteractionResponseMessage, InteractionContext,
};
use serenity::builder::CreateCommand;

use crate::ai::{self, MakaiMessage};
use crate::commands::{CommandName, MakaiCommand};
use crate::context::MakaiContext;

pub struct ReplyCommand;

#[async_trait]
impl MakaiCommand for ReplyCommand {
    fn name(&self) -> CommandName {
        "Reply"
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
        let defer = CreateInteractionResponse::Defer(CreateInteractionResponseMessage::default());
        cmd.create_response(&discord_ctx.http, defer)
            .await
            .context("Cannot defer command")?;

        let user = bot_ctx
            .user()
            .await
            .context("Got command before user is known")?;

        let message =
            MakaiMessage::from_message_command(user.id, cmd).context("Get message from command")?;

        let response = ai::run_llm(&*bot_ctx.channel(&cmd.channel_id).await, message)
            .await
            .context("Run LLM")?;
        response
            .send_follow_up(discord_ctx, cmd)
            .await
            .context("Send Follow up")?;

        Ok(())
    }
}
