use std::env;

use anyhow::Context;
use llm::{
    builder::{LLMBackend, LLMBuilder},
    chat::ChatMessage,
};
use rand::seq::SliceRandom;

pub async fn run_llm(user: &str, message: &str) -> anyhow::Result<String> {
    let url = env::var("LLM_API").context("Expected a llm api url in env")?;
    let model = env::var("LLM_MODEL").context("Expected a llm model in env")?;
    let prompt_file = env::var("LLM_PROMPT_FILE").context("Expected a prompt file in env")?;
    let words_file = env::var("LLM_WORDS_FILE").context("Expected a words file in env")?;

    let system = tokio::fs::read_to_string(prompt_file)
        .await
        .context("Read prompt file")?;
    let words = tokio::fs::read_to_string(words_file)
        .await
        .context("Read words file")?;
    let mut words = words.lines().collect::<Vec<&str>>();
    words.shuffle(&mut rand::rng());
    let system = system.replace(
        "{WORDS}",
        &words.iter().fold(String::new(), |mut acc, it| {
            acc.push_str(&format!("- {it}\n"));
            acc
        }),
    );

    println!("{system}");

    let llm = LLMBuilder::new()
        .backend(LLMBackend::OpenAI)
        .api_key("funny-api-key")
        .base_url(url)
        .model(model)
        .max_tokens(512)
        .system(system)
        .build()
        .context("Failed to build LLM")?;

    let messages = vec![
        ChatMessage::user()
            .content(format!("User '{user}' says: {message}"))
            .build(),
        // ChatMessage::assistant()
        //     .content(format!("MakAI said in response: "))
        //     .build(),
    ];

    let response = llm.chat(&messages).await.context("LLM Error")?;

    println!("AI responded: `{:?}`", response.text());

    // Get rid of thinking stuff
    let text = response.text().unwrap_or_default();
    let text = text.split("▷").last();

    Ok(text.unwrap_or_default().to_string())
}
