use crate::commands::Commands;
use anyhow::{anyhow, Context, Result};
use configs::LLMProvider;
use configs::constants::{
    AI_MODEL, CHAT_API_KEY, CHAT_API_URL, EMBEDDING_MODEL, OPEN_AI_URL, SYSTEM_PROMPT_PATH,
};
use dialoguer::{console::Term, theme::ColorfulTheme, Confirm, Input, Select};

pub fn interactive_cli() -> Result<Commands> {
    let theme = ColorfulTheme::default();
    let commands = vec![
        "Load",
        "LanceQuery",
        "RagQuery",
        "Generate",
        "Version",
        "Exit",
        "Help",
    ];

    let command_index = Select::with_theme(&theme)
        .with_prompt("Select the CLI command to run")
        .items(&commands)
        .interact_on(&Term::stdout())?;

    match commands[command_index] {
        "Load" => {
            let (llm_provider, api_url, api_key) = fetch_llm_config(&theme)?;

            Ok(Commands::Load {
                path: Input::with_theme(&theme)
                    .with_prompt("Enter file path")
                    .interact_text()?,
                chunk_size: Input::with_theme(&theme)
                    .with_prompt("Enter chunk size")
                    .default("1024".to_string())
                    .interact_text()?,
                llm_provider,
                embed_model: Input::with_theme(&theme)
                    .with_prompt("Embedding model")
                    .default(EMBEDDING_MODEL.to_string())
                    .interact_text()?,
                api_url,
                api_key,
            })
        }

        "LanceQuery" => {
            let (llm_provider, api_url, api_key) = fetch_llm_config(&theme)?;
            Ok(Commands::LanceQuery {
                input: Input::<String>::with_theme(&theme)
                    .with_prompt("Enter your query")
                    .interact_text()?
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                llm_provider,
                api_url,
                api_key,
                model: Input::with_theme(&theme)
                    .with_prompt("Embedding model")
                    .default(EMBEDDING_MODEL.to_string())
                    .interact_text()?,
                table: Input::with_theme(&theme)
                    .with_prompt("Table name")
                    .default("rag_table".to_string())
                    .interact_text()?,
                database: Input::with_theme(&theme)
                    .with_prompt("Database URI")
                    .default("rag_db".to_string())
                    .interact_text()?,
                whole_query: Confirm::with_theme(&theme)
                    .with_prompt("Use whole query embedding?")
                    .default(false)
                    .interact()?
                    .to_string(),
                file_context: Confirm::with_theme(&theme)
                    .with_prompt("Use file context?")
                    .default(false)
                    .interact()?
                    .to_string(),
            })
        }

        "RagQuery" => {
            let (llm_provider, api_url, api_key) = fetch_llm_config(&theme)?;

            Ok(Commands::RagQuery {
                input: Input::<String>::with_theme(&theme)
                    .with_prompt("Enter your query")
                    .interact_text()?
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                llm_provider,
                embed_model: Input::with_theme(&theme)
                    .with_prompt("Embedding model")
                    .default(EMBEDDING_MODEL.to_string())
                    .interact_text()?,
                api_url,
                api_key,
                ai_model: Input::with_theme(&theme)
                    .with_prompt("AI Model")
                    .default(AI_MODEL.to_string())
                    .interact_text()?,
                table: Input::with_theme(&theme)
                    .with_prompt("VectorDB Table")
                    .default("rag_table".to_string())
                    .interact_text()?,
                database: Input::with_theme(&theme)
                    .with_prompt("Database URI")
                    .default("rag_db".to_string())
                    .interact_text()?,
                whole_query: Confirm::with_theme(&theme)
                    .with_prompt("Use whole query embedding?")
                    .default(false)
                    .interact()?
                    .to_string(),
                file_context: Confirm::with_theme(&theme)
                    .with_prompt("Use file context?")
                    .default(false)
                    .interact()?
                    .to_string(),
                system_prompt: Input::with_theme(&theme)
                    .with_prompt("System prompt file path")
                    .default(SYSTEM_PROMPT_PATH.into())
                    .interact_text()?,
                continue_chat: Confirm::with_theme(&theme)
                    .with_prompt("Continue chat?")
                    .default(true)
                    .interact()?
                    .to_string(),
            })
        }

        "Generate" => {
            let (llm_provider, api_url, api_key) = fetch_llm_config(&theme)?;
            Ok(Commands::Generate {
                prompt: Input::<String>::with_theme(&theme)
                    .with_prompt("Enter your prompt")
                    .interact_text()?
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                llm_provider,
                api_url,
                api_key,
                ai_model: Input::with_theme(&theme)
                    .with_prompt("AI Model")
                    .default(AI_MODEL.to_string())
                    .interact_text()?,
            })
        }

        "Version" => Ok(Commands::Version {
            version: configs::constants::VERSION.to_string(),
        }),

        "Exit" => Ok(Commands::Exit),

        "Help" => Ok(Commands::Man),

        _ => Err(anyhow!("Selected invalid command")),
    }
}

fn fetch_llm_config(theme: &ColorfulTheme) -> Result<(String, String, String)> {
    let llm_provider = Input::with_theme(theme)
        .with_prompt("LLM provider")
        .validate_with(|input: &String| -> core::result::Result<(), &str> {
            match LLMProvider::get_provider(&input.to_lowercase()) {
                Ok(_) => Ok(()),
                _ => Err("Please input one of the supported providers: ollama, openai"),
            }
        })
        .default("ollama".to_string())
        .interact_text()?;

    let chat_api_url =
        get_chat_api_url(&llm_provider).context("Provided LLm Provider not supported")?;

    let api_url = Input::with_theme(theme)
        .with_prompt("Chat API Url")
        .default(chat_api_url)
        .interact_text()?;

    let api_key = Input::with_theme(theme)
        .with_prompt("API Key")
        .default(CHAT_API_KEY.to_string())
        .interact_text()?;

    Ok((llm_provider, api_url, api_key))
}

fn get_chat_api_url(provider: &str) -> Result<String> {
    let provider =
        LLMProvider::get_provider(provider).map_err(|_| anyhow!("Unsupported provider"))?;
    match provider {
        LLMProvider::OpenAI => Ok(OPEN_AI_URL.to_string()),
        LLMProvider::Ollama => Ok(CHAT_API_URL.to_string()),
    }
}
