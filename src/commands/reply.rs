use serenity::all::{CommandInteraction, CommandType, InteractionContext, Message};
use serenity::builder::CreateCommand;

use crate::ai;

pub async fn run(cmd: &CommandInteraction) -> anyhow::Result<String> {
    if let Some(Message {
        author,
        content,
        embeds,
        ..
    }) = cmd.data.resolved.messages.values().next()
    {
        let mut message = content.clone();

        for embed in embeds {
            message.push_str(&format!(
                "\nThe user's message included a link: Title: `{}`, Description: `{}`",
                embed
                    .title
                    .as_ref()
                    .map(|it| it.as_str())
                    .unwrap_or("Unknown"),
                embed
                    .description
                    .as_ref()
                    .map(|it| it.as_str())
                    .unwrap_or("Unknown")
            ));
        }

        ai::run_llm(
            author.global_name.as_ref().unwrap_or(&author.name),
            &message,
        )
        .await
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
