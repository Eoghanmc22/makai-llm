= MakAI LLM

An inside joke discord bot that behaves like some of my friends

== Local Development

- Install a recent version of rust/cargo
- Create a `.env` file with the following content:
  ```env
  DISCORD_TOKEN=your-discord-bot-token-here
  LLM_API=openai-compatable-llm-api-endpoint-here
  LLM_API_KEY=your-api-key-for-the-llm-provider
  LLM_MODEL=a-llm-model-provided-by-that-api # Tested with gpt-oss-20b

  # Optionally env vars
  LLM_PROMPT_FILE=./prompt.txt
  LLM_WORDS_FILE=./words.txt
  ```
- Run the bot with
  ```sh
  $ cargo run
  ```

Note: Edits to the prompt files are reflected immediately, no need to restart the bot.

For an inference provider for testing I'd recommend the [Groq free tier](https://console.groq.com/home)
they have respectable rate limits and really fast inference.
