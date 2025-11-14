use serenity::all::{CommandInteraction, CommandType, InteractionContext, Message};
use serenity::builder::CreateCommand;

use crate::ai;

pub async fn run(cmd: &CommandInteraction) -> anyhow::Result<String> {
    if let Some(Message {
        author, content, ..
    }) = cmd.data.resolved.messages.values().next()
    {
        ai::run_llm(&author.name, &content).await
    } else {
        Ok("ERROR: Got no prompt".to_string())
    }
}

pub fn register() -> CreateCommand {
    CreateCommand::new("Reply")
        .kind(CommandType::Message)
        .add_context(InteractionContext::BotDm)
        .add_context(InteractionContext::Guild)
        .add_context(InteractionContext::PrivateChannel)
}
