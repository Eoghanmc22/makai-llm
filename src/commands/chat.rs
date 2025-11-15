use anyhow::{Context as _, bail};
use chrono::Utc;
use llm::async_trait;
use serenity::all::{
    CacheHttp, CommandInteraction, Context, CreateInteractionResponse,
    CreateInteractionResponseFollowup, CreateInteractionResponseMessage, InteractionContext,
    ResolvedOption, ResolvedValue,
};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;

use crate::ai::{self, MakaiMessage, MessageSender};
use crate::commands::{CommandName, MakaiCommand};
use crate::context::MakaiContext;
use crate::utils::user_to_name;

pub struct ChatCommand;

#[async_trait]
impl MakaiCommand for ChatCommand {
    fn name(&self) -> CommandName {
        "chat"
    }

    fn register(&self) -> CreateCommand {
        CreateCommand::new(self.name())
            .add_context(InteractionContext::BotDm)
            .add_context(InteractionContext::Guild)
            .add_context(InteractionContext::PrivateChannel)
            .description("Chat to Makai LLM")
            .add_option(
                CreateCommandOption::new(CommandOptionType::String, "prompt", "The prompt to send")
                    .required(true),
            )
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

        let options = cmd.data.options();
        let Some(ResolvedOption {
            value: ResolvedValue::String(prompt),
            ..
        }) = options.iter().find(|it| it.name == "prompt")
        else {
            bail!("Find prompy")
        };
        let message = MakaiMessage {
            message_id: None,
            timestamp: Utc::now(),
            sender: MessageSender::User(user_to_name(&cmd.user).to_string()),
            content: prompt.to_string(),
        };

        let response = ai::run_llm(&*bot_ctx.channel(&cmd.channel_id).await, message)
            .await
            .context("Run LLM")?;

        let follow_up = CreateInteractionResponseFollowup::default().content(response);
        cmd.create_followup(discord_ctx.http(), follow_up)
            .await
            .context("Cannot followup command")?;

        Ok(())
    }
}
