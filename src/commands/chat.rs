use serenity::all::{CommandInteraction, InteractionContext, ResolvedOption, ResolvedValue};
use serenity::builder::{CreateCommand, CreateCommandOption};
use serenity::model::application::CommandOptionType;

use crate::ai;

pub async fn run(cmd: &CommandInteraction) -> anyhow::Result<String> {
    if let Some(ResolvedOption {
        value: ResolvedValue::String(prompt),
        ..
    }) = cmd.data.options().first()
    {
        ai::run_llm(&cmd.user.name, &prompt).await
    } else {
        Ok("ERROR: Got no prompt".to_string())
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("chat")
        .add_context(InteractionContext::BotDm)
        .add_context(InteractionContext::Guild)
        .add_context(InteractionContext::PrivateChannel)
        .description("Chat to Makai LLM")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "prompt", "The prompt to send")
                .required(true),
        )
}
