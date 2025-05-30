use agent::ai_agent::{EmbedAgent, EmbeddingProvider, LLMAgent, ModelAPIProvider, RagAgent};
use anyhow::{anyhow, Context, Ok, Result};
use chat::chat_config::LLMProvider;
use configs::constants::{
    AI_MODEL, CHAT_API_KEY, EMBEDDING_MODEL, SYSTEM_PROMPT_PATH,
};
use dialoguer::{console::Term, theme::ColorfulTheme, Confirm, Input, Select};

pub fn interactive_cli(rt: &tokio::runtime::Runtime) -> Result<()> {
    let theme = ColorfulTheme::default();
    // @TODO refactor this to cli::Commands enum have to move this file to cli lib.
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
            let https_client =
                configs::get_https_client().context("Failed to initialize https client")?;
            let llm_provider = fetch_llm_config(&theme, "Embed Provider:")?;
            let model = Input::with_theme(&theme)
                .with_prompt("Embedding model")
                .default(EMBEDDING_MODEL.to_string())
                .interact_text()?;

            let embedding_provider = EmbeddingProvider::new(llm_provider, model);
            let agent = EmbedAgent::new(https_client, embedding_provider);

            let path: String = Input::with_theme(&theme)
                .with_prompt("Enter file path")
                .interact_text()?;
            let chunk_size: usize = Input::with_theme(&theme)
                .with_prompt("Enter chunk size")
                .default("1024".to_string())
                .interact_text()?
                .parse::<usize>()?;

            let embedding_store = rt.block_on(agent.load_embeddings(path.as_str(), chunk_size))?;
            println!("Embedding store: {:?}", embedding_store);
        }

        "LanceQuery" => {
            let https_client =
                configs::get_https_client().context("Failed to initialize https client")?;
            let llm_provider = fetch_llm_config(&theme, "Embed Provider:")?;
            let model = Input::with_theme(&theme)
                .with_prompt("Embedding model")
                .default(EMBEDDING_MODEL.to_string())
                .interact_text()?;

            let embedding_provider = EmbeddingProvider::new(llm_provider, model);

            let database = Input::with_theme(&theme)
                .with_prompt("Database URI")
                .default("rag_db".to_string())
                .interact_text()?;
            let table = Input::with_theme(&theme)
                .with_prompt("Table name")
                .default("rag_table".to_string())
                .interact_text()?;

            let agent = EmbedAgent::new(https_client, embedding_provider);
            let embedding_store = vectordb::EmbeddingStore::new(&database, &table);

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
            let https_client =
                configs::get_https_client().context("Failed to initialize https client")?;

            let embed_provider = fetch_llm_config(&theme, "Embed provider:")?;
            let embeding_model = Input::with_theme(&theme)
                .with_prompt("Embedding model")
                .default(EMBEDDING_MODEL.to_string())
                .interact_text()?;

            let embedding_provider = EmbeddingProvider::new(embed_provider, embeding_model);
            let embed_agent = EmbedAgent::new(https_client.clone(), embedding_provider);

            let llm_provider = fetch_llm_config(&theme, "LLM Provider:")?;
            let ai_model = Input::with_theme(&theme)
                .with_prompt("AI Model")
                .default(AI_MODEL.to_string())
                .interact_text()?;

            let ai_model = LLMAgent::new(https_client.clone(), llm_provider, ai_model);
            let agent = RagAgent::new(https_client, embed_agent, ai_model);

            let path: String = Input::with_theme(&theme)
                .with_prompt("Enter file path")
                .interact_text()?;
            let system_prompt: String = Input::with_theme(&theme)
                .with_prompt("System prompt file path")
                .default(SYSTEM_PROMPT_PATH.into())
                .interact_text()?;
            agent.rag_query(
                rt,
                &path,
                Input::with_theme(&theme)
                    .with_prompt("Enter chunk size")
                    .default("1024".to_string())
                    .interact_text()?
                    .parse::<usize>()?,
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
                &system_prompt,
                Confirm::with_theme(&theme)
                    .with_prompt("Continue chat?")
                    .default(true)
                    .interact()?,
            )?;
        }

        "Generate" => {
            let https_client =
                configs::get_https_client().context("Failed to initialize https client")?;
            let llm_provider = fetch_llm_config(&theme, "LLM Provider:")?;

            let ai_agent = LLMAgent {
                https_client,
                llm_provider,
                model: Input::with_theme(&theme)
                    .with_prompt("AI Model")
                    .default(AI_MODEL.to_string())
                    .interact_text()?,
            };

            ai_agent.generate(
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
            println!("Available commands:");
            println!("  load         - Load a directory of files into the lance vector database");
            println!("  lance-query  - Query the Lance Vector Database");
            println!("  rag-query    - Query the Lance Vector Database and chat with the AI");
            println!("  generate     - Chat with the AI");
            println!("Use 'Help' to see this message again.");
        }

        _ => return Err(anyhow!("Selected invalid command")),
    }

    Ok(())
}

fn fetch_llm_config(theme: &ColorfulTheme, prompt: &str) -> Result<ModelAPIProvider> {
    let provider = Input::with_theme(theme)
        .with_prompt(prompt)
        .validate_with(|input: &String| -> core::result::Result<(), &str> {
            match LLMProvider::get_provider(&input.to_lowercase()) {
                std::result::Result::Ok(_) => core::result::Result::Ok(()),
                _ => Err("Please input one of the supported providers: ollama, openai"),
            }
        })
        .default("ollama".to_string())
        .interact_text()?;

    let api_url = Input::with_theme(theme)
        .with_prompt("Chat API Url")
        // .default(configs::get_chat_api_url(&provider)?.to_string())
        .default(configs::LLMProvider::get_api_url(&provider)?.to_string())
        .interact_text()?;

    let api_key = Input::with_theme(theme)
        .with_prompt("API Key")
        .default(CHAT_API_KEY.to_string())
        .interact_text()?;

    let llm_provider = ModelAPIProvider {
        provider,
        api_url,
        api_key,
    };

    Ok(llm_provider)
}

