use crate::commands::Commands;
use anyhow::{anyhow, Result};
use configs::constants::{
    AI_MODEL, CHAT_API_KEY, CHAT_API_URL, EMBEDDING_MODEL, SYSTEM_PROMPT_PATH,
};
use dialoguer::{console::Term, theme::ColorfulTheme, Confirm, Input, Select};

pub fn interactive_cli() -> Result<Commands> {
    let theme = ColorfulTheme::default();
    let commands = vec!["Load", "LanceQuery", "RagQuery", "Generate", "Version"];

    let command_index = Select::with_theme(&theme)
        .with_prompt("Select the CLI command to run")
        .items(&commands)
        .interact_on(&Term::stdout())?;

    match commands[command_index] {
        "Load" => Ok(Commands::Load {
            path: Input::with_theme(&theme)
                .with_prompt("Enter file path")
                .interact_text()?,
            chunk_size: Input::with_theme(&theme)
                .with_prompt("Enter chunk size")
                .default("1024".to_string())
                .interact_text()?,
            llm_provider: Input::with_theme(&theme)
                .with_prompt("Enter LLM provider")
                .default("ollama".to_string())
                .interact_text()?,
            embed_model: Input::with_theme(&theme)
                .with_prompt("Embedding model")
                .default(EMBEDDING_MODEL.to_string())
                .interact_text()?,
            api_url: Input::with_theme(&theme)
                .with_prompt("API URL")
                .default(CHAT_API_URL.to_string())
                .interact_text()?,
            api_key: Input::with_theme(&theme)
                .with_prompt("API Key")
                .default(CHAT_API_KEY.to_string())
                .interact_text()?,
        }),

        "LanceQuery" => Ok(Commands::LanceQuery {
            input: Input::<String>::with_theme(&theme)
                .with_prompt("Enter your query")
                .interact_text()?
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            llm_provider: Input::with_theme(&theme)
                .with_prompt("LLM provider")
                .default("ollama".to_string())
                .interact_text()?,
            api_url: Input::with_theme(&theme)
                .with_prompt("API URL")
                .default(CHAT_API_URL.to_string())
                .interact_text()?,
            api_key: Input::with_theme(&theme)
                .with_prompt("API Key")
                .default(CHAT_API_KEY.to_string())
                .interact_text()?,
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
        }),

        "RagQuery" => Ok(Commands::RagQuery {
            input: Input::<String>::with_theme(&theme)
                .with_prompt("Enter your query")
                .interact_text()?
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            llm_provider: Input::with_theme(&theme)
                .with_prompt("LLM provider")
                .default("ollama".to_string())
                .interact_text()?,
            embed_model: Input::with_theme(&theme)
                .with_prompt("Embedding model")
                .default(EMBEDDING_MODEL.to_string())
                .interact_text()?,
            api_url: Input::with_theme(&theme)
                .with_prompt("API URL")
                .default(CHAT_API_URL.to_string())
                .interact_text()?,
            api_key: Input::with_theme(&theme)
                .with_prompt("API Key")
                .default(CHAT_API_KEY.to_string())
                .interact_text()?,
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
        }),

        "Generate" => Ok(Commands::Generate {
            prompt: Input::<String>::with_theme(&theme)
                .with_prompt("Enter your prompt")
                .interact_text()?
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            llm_provider: Input::with_theme(&theme)
                .with_prompt("LLM provider")
                .default("ollama".to_string())
                .interact_text()?,
            api_url: Input::with_theme(&theme)
                .with_prompt("API URL")
                .default(CHAT_API_URL.to_string())
                .interact_text()?,
            api_key: Input::with_theme(&theme)
                .with_prompt("API Key")
                .default(CHAT_API_KEY.to_string())
                .interact_text()?,
            ai_model: Input::with_theme(&theme)
                .with_prompt("AI Model")
                .default(AI_MODEL.to_string())
                .interact_text()?,
        }),

        "Version" => Ok(Commands::Version {
            version: Input::with_theme(&theme)
                .with_prompt("Specify version")
                .default("0.1.0".into())
                .interact_text()?,
        }),

        _ => Err(anyhow!("Selected invalid command")),
    }
}
