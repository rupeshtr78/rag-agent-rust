use crate::commands::Commands;
use anyhow::{Context, Result};
use configs::constants::SYSTEM_PROMPT_PATH;
use log::{debug, info};

use agent::ai_agent::{AIModel, Agent, EmbeddingModel, LLMProvider};

pub fn cli(commands: Commands, rt: tokio::runtime::Runtime) -> Result<()> {
    match commands {
        Commands::Load {
            path,
            chunk_size,
            llm_provider,
            embed_model,
            api_url,
            api_key,
        } => {
            info!("Using the Load arguments below:");
            info!(" Path: {:?}", path);
            let chunk_size = chunk_size
                .parse::<usize>()
                .context("Failed to parse chunk size")?;

            let llm_provider = LLMProvider {
                provider: llm_provider,
                api_url,
                api_key,
            };
            let embedding_model = EmbeddingModel { model: embed_model };
            let ai_model = AIModel {
                model: String::new(),
            };
            let table = String::new();
            let database = String::new();

            let agent = Agent::new(llm_provider, embedding_model, ai_model, table, database);
            agent.load_embeddings(rt, &path, chunk_size)?;

            // shutdown the runtime after the embedding is done
            println!("Finished Loading the embedding");
            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::LanceQuery {
            input,
            llm_provider,
            api_url,
            api_key,
            model,
            table,
            database,
            whole_query,
            file_context,
        } => {
            let input_list = Commands::fetch_prompt_from_cli(input.clone(), "Enter query: ");
            let whole_query: bool = whole_query
                .parse()
                .context("Failed to parse whole_query flag")?;
            let file_context: bool = file_context
                .parse()
                .context("Failed to parse file_query flag")?;

            info!(" Query: {:?}", input_list);
            info!(" LLM Provider: {:?}", llm_provider);
            info!(" API URL: {:?}", api_url);
            info!(" Model: {:?}", model);
            info!(" Table: {:?}", table);
            info!(" Whole Query: {:?}", whole_query);
            info!(" File Query: {:?}", file_context);

            let llm_provider = LLMProvider {
                provider: llm_provider,
                api_url,
                api_key,
            };
            let embedding_model = EmbeddingModel { model };
            let ai_model = AIModel {
                model: String::new(),
            };

            let agent = Agent::new(llm_provider, embedding_model, ai_model, table, database);
            agent.query_embeddings(rt, input_list, whole_query, file_context)?;
        }
        Commands::RagQuery {
            input,
            llm_provider,
            embed_model,
            api_url,
            api_key,
            ai_model,
            table,
            database,
            whole_query,
            file_context,
            system_prompt,
            continue_chat,
        } => {
            let input_list = Commands::fetch_prompt_from_cli(input.clone(), "Enter query: ");
            let whole_query: bool = whole_query
                .parse()
                .context("Failed to parse whole_query flag")?;
            let file_context: bool = file_context
                .parse()
                .context("Failed to parse file_query flag")?;
            let continue_chat: bool = continue_chat
                .parse()
                .context("Failed to parse continue_chat flag")?;

            println!("Query command is run with below arguments:");
            println!(" Query: {:?}", input_list);
            println!(" LLM Provider: {:?}", llm_provider);
            println!(" API URL: {:?}", api_url);
            println!(" Embedding Model: {:?}", embed_model);
            println!(" AI Model: {:?}", ai_model);
            println!(" Table: {:?}", table);
            println!(" Continous Chat: {:?}", continue_chat);

            let llm_provider = LLMProvider {
                provider: llm_provider,
                api_url,
                api_key,
            };
            let embedding_model = EmbeddingModel { model: embed_model };
            let ai_model = AIModel { model: ai_model };

            let agent = Agent::new(llm_provider, embedding_model, ai_model, table, database);
            agent.rag_query(
                rt,
                input_list,
                whole_query,
                file_context,
                system_prompt.as_str(),
                continue_chat,
            )?;

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Generate {
            prompt,
            llm_provider,
            api_url,
            api_key,
            ai_model,
        } => {
            println!("Chat command is run with below arguments:");
            println!(" Prompt: {:?}", prompt);
            println!(" LLM Provider: {:?}", llm_provider);
            println!(" API URL: {:?}", api_url);
            println!(" API Key: {:?}", api_key);
            println!(" AI Model: {:?}", ai_model);

            let llm_provider = LLMProvider {
                provider: llm_provider,
                api_url,
                api_key,
            };
            let embedding_model = EmbeddingModel {
                model: String::new(),
            };
            let ai_model = AIModel { model: ai_model };
            let table = String::new();
            let database = String::new();

            let agent = Agent::new(llm_provider, embedding_model, ai_model, table, database);
            let system_prompt = SYSTEM_PROMPT_PATH;
            agent.generate(rt, &prompt, system_prompt)?;

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Version { version } => {
            println!("Version: {}", version);
            std::process::exit(0);
        }
        Commands::Exit => {
            println!("Exiting application");
            std::process::exit(0);
        }
        Commands::Man => {
            println!("Help command is not implemented yet");
            std::process::exit(0);
        }
    }

    Ok(())
}
