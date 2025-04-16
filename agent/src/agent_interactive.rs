use crate::ai_agent::{AIModel, Agent, EmbeddingModel, LLMProvider};
use anyhow::{anyhow, Result};
use configs::constants::{
    AI_MODEL, CHAT_API_KEY, CHAT_API_URL, EMBEDDING_MODEL, OPEN_AI_URL, SYSTEM_PROMPT_PATH,
};
use dialoguer::{console::Term, theme::ColorfulTheme, Confirm, Input, Select};

pub fn interactive_cli(rt: tokio::runtime::Runtime) -> Result<()> {
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
            let (llm_provider, _, _) = fetch_llm_config(&theme)?;
            let embedding_model = EmbeddingModel {
                model: Input::with_theme(&theme)
                    .with_prompt("Embedding model")
                    .default(EMBEDDING_MODEL.to_string())
                    .interact_text()?,
            };
            let ai_model = AIModel {
                model: String::new(),
            };

            let agent = Agent::new(llm_provider, embedding_model, ai_model);
            let embedding_store = rt.block_on(
                agent.load_embeddings(
                    &Input::with_theme(&theme)
                        .with_prompt("Enter file path")
                        .interact_text()?,
                    Input::with_theme(&theme)
                        .with_prompt("Enter chunk size")
                        .default("1024".to_string())
                        .interact_text()?
                        .parse::<usize>()?,
                ),
            )?;
            println!("Embedding store: {:?}", embedding_store);
        }

        "LanceQuery" => {
            let (llm_provider, _, _) = fetch_llm_config(&theme)?;
            let embedding_model = EmbeddingModel {
                model: Input::with_theme(&theme)
                    .with_prompt("Embedding model")
                    .default(EMBEDDING_MODEL.to_string())
                    .interact_text()?,
            };
            let ai_model = AIModel {
                model: String::new(),
            };

            let table = Input::with_theme(&theme)
                .with_prompt("Table name")
                .default("rag_table".to_string())
                .interact_text()?;
            let database = Input::with_theme(&theme)
                .with_prompt("Database URI")
                .default("rag_db".to_string())
                .interact_text()?;

            let agent = Agent::new(llm_provider, embedding_model, ai_model);
            let embedding_store = vectordb::EmbeddingStore::new(&table, &database);

            let content = rt.block_on(
                agent.query_embeddings(
                    Input::<String>::with_theme(&theme)
                        .with_prompt("Enter your query")
                        .interact_text()?
                        .split(',')
                        .map(|s| s.to_string())
                        .collect(),
                    Confirm::with_theme(&theme)
                        .with_prompt("Use whole query embedding?")
                        .default(false)
                        .interact()?,
                    Confirm::with_theme(&theme)
                        .with_prompt("Use file context?")
                        .default(false)
                        .interact()?,
                    &embedding_store,
                ),
            )?;
            println!("Query content: {:?}", content);
        }

        "RagQuery" => {
            let (llm_provider, _, _) = fetch_llm_config(&theme)?;
            let embedding_model = EmbeddingModel {
                model: Input::with_theme(&theme)
                    .with_prompt("Embedding model")
                    .default(EMBEDDING_MODEL.to_string())
                    .interact_text()?,
            };
            let ai_model = AIModel {
                model: Input::with_theme(&theme)
                    .with_prompt("AI Model")
                    .default(AI_MODEL.to_string())
                    .interact_text()?,
            };

            let agent = Agent::new(llm_provider, embedding_model, ai_model);
            agent.rag_query(
                &Input::with_theme(&theme)
                    .with_prompt("Enter file path")
                    .interact_text()?,
                Input::with_theme(&theme)
                    .with_prompt("Enter chunk size")
                    .default("1024".to_string())
                    .interact_text()?
                    .parse::<usize>()?,
                rt,
                Input::<String>::with_theme(&theme)
                    .with_prompt("Enter your query")
                    .interact_text()?
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                Confirm::with_theme(&theme)
                    .with_prompt("Use whole query embedding?")
                    .default(false)
                    .interact()?,
                Confirm::with_theme(&theme)
                    .with_prompt("Use file context?")
                    .default(false)
                    .interact()?,
                &Input::with_theme(&theme)
                    .with_prompt("System prompt file path")
                    .default(SYSTEM_PROMPT_PATH.into())
                    .interact_text()?,
                Confirm::with_theme(&theme)
                    .with_prompt("Continue chat?")
                    .default(true)
                    .interact()?,
            )?;
        }

        "Generate" => {
            let (llm_provider, _, _) = fetch_llm_config(&theme)?;
            let embedding_model = EmbeddingModel {
                model: String::new(),
            };
            let ai_model = AIModel {
                model: Input::with_theme(&theme)
                    .with_prompt("AI Model")
                    .default(AI_MODEL.to_string())
                    .interact_text()?,
            };

            let agent = Agent::new(llm_provider, embedding_model, ai_model);
            agent.generate(
                rt,
                &Input::<String>::with_theme(&theme)
                    .with_prompt("Enter your prompt")
                    .interact_text()?,
                SYSTEM_PROMPT_PATH,
            )?;
        }

        "Version" => {
            println!("Version: {}", configs::constants::VERSION);
        }

        "Exit" => {
            println!("Exiting application");
            std::process::exit(0);
        }

        "Help" => {
            println!("Help command is not implemented yet");
        }

        _ => return Err(anyhow!("Selected invalid command")),
    }

    Ok(())
}

fn fetch_llm_config(theme: &ColorfulTheme) -> Result<(LLMProvider, String, String)> {
    let provider = Input::with_theme(theme)
        .with_prompt("LLM provider")
        .validate_with(|input: &String| -> core::result::Result<(), &str> {
            match LLMProvider::get_provider(&input.to_lowercase()) {
                Ok(_) => Ok(()),
                _ => Err("Please input one of the supported providers: ollama, openai"),
            }
        })
        .default("ollama".to_string())
        .interact_text()?;

    let llm_provider = LLMProvider {
        provider,
        api_url: Input::with_theme(theme)
            .with_prompt("Chat API Url")
            .default(get_chat_api_url(provider.as_str())?.to_string())
            .interact_text()?,
        api_key: Input::with_theme(theme)
            .with_prompt("API Key")
            .default(CHAT_API_KEY.to_string())
            .interact_text()?,
    };

    Ok((
        llm_provider,
        llm_provider.api_url.clone(),
        llm_provider.api_key.clone(),
    ))
}

fn get_chat_api_url(provider: &str) -> Result<String> {
    let provider =
        LLMProvider::get_provider(provider).map_err(|_| anyhow!("Unsupported provider"))?;
    match provider {
        LLMProvider::OpenAI => Ok(OPEN_AI_URL.to_string()),
        LLMProvider::Ollama => Ok(CHAT_API_URL.to_string()),
    }
}
