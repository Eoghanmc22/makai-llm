use llm::async_trait;
use serenity::all::{
    CommandInteraction, Context, InteractionContext, ResolvedOption, ResolvedValue,
};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;

use crate::ai;
use crate::commands::{CommandName, MakaiCommand};
use crate::context::MakaiContext;

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
        if let Some(ResolvedOption {
            value: ResolvedValue::String(prompt),
            ..
        }) = cmd.data.options().first()
        {
            ai::run_llm(
                cmd.user.global_name.as_ref().unwrap_or(&cmd.user.name),
                &prompt,
            )
            .await
        } else {
            Ok("ERROR: Got no prompt".to_string())
        }
    }
}
