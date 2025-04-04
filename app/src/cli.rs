use crate::commands::Commands;
use anyhow::Result;
use anyhow::{Context, Ok};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::client::connect::HttpInfo;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client as LegacyClient;
use log::{debug, info};

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
            info!(" Chunk Size: {:?}", chunk_size);
            info!(" LLM Provider: {:?}", llm_provider);
            info!(" Embedding Model: {:?}", embed_model);
            info!(" API URL: {:?}", api_url);

            let chunk_size = chunk_size
                .parse::<usize>()
                .context("Failed to parse chunk size")?;
            let https_client =
                configs::get_https_client().context("Failed to create HTTPS client")?;
            // let embed_url = format!("{}/{}", constants::CHAT_API_URL, "api/embed");

            rt.block_on(check_connection(
                &https_client,
                &format!("{}/{}", api_url, "api/version"),
            ))
            .context("Failed to check connection")?;

            // rt.block_on(check_client(
            //     &http_client,
            //     &format!("{}/{}", url, "api/version"),
            // ))
            // .context("Failed to check client")?;

            rt.block_on(vectordb::run_embedding_pipeline(
                &path,
                chunk_size,
                llm_provider.as_str(),
                &api_url,
                &api_key,
                embed_model.as_str(),
                &https_client,
            ))
            .context("Failed to run lance vectordb")?;

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
            // let embed_model = model.to_string();
            // let vector_table = table.to_string();
            // let db_uri = database.to_string();
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

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let https_client =
                configs::get_https_client().context("Failed to create HTTPS client")?;

            // Initialize the database
            let mut db = rt
                .block_on(lancedb::connect(&database).execute())
                .context("Failed to connect to the database")?;

            // Query the database
            let content = rt
                .block_on(vectordb::query::run_query(
                    &mut db,
                    llm_provider.as_str(),
                    api_url.as_str(),
                    api_key.as_str(),
                    model.as_str(),
                    &input_list,
                    &table,
                    &https_client,
                    whole_query,
                    file_context,
                ))
                .context("Failed to run query")?;

            println!("Query Response: {:?}", content);
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
            // let embed_model = embed_model.to_string();
            // let vector_table = table.to_string();
            // let db_uri = database.to_string();
            let whole_query: bool = whole_query
                .parse()
                .context("Failed to parse whole_query flag")?;
            let file_context: bool = file_context
                .parse()
                .context("Failed to parse file_query flag")?;
            // let system_prompt = system_prompt.as_str();
            // let provider = llm_provider.as_str();
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

            // Initialize the http client outside the thread // TODO wrap in Arc<Mutex>
            let https_client =
                configs::get_https_client().context("Failed to create HTTPS client")?;
            // do a check to see if client is up

            // Initialize the database
            let mut db = rt
                .block_on(lancedb::connect(&database).execute())
                .context("Failed to connect to the database")?;

            // Query the database
            let content = rt
                .block_on(vectordb::query::run_query(
                    &mut db,
                    llm_provider.as_str(),
                    api_url.as_str(),
                    api_key.as_str(),
                    embed_model.as_str(),
                    &input_list,
                    &table,
                    &https_client,
                    whole_query,
                    file_context,
                ))
                .context("Failed to run query")?;

            debug!("Query Response: {:?}", content);

            let context = content.join(" ");
            // @ TODO: make this a command line argument
            // let system_prompt = "template/rag_prompt.txt";
            // let system_prompt = "template/software-engineer.txt";
            // let system_prompt = "template/spark_prompt.txt";
            // let system_prompt = "template/spark-engineer.txt";
            rt.block_on(chat::run_chat_with_history(
                system_prompt.as_str(),
                input_list.first().unwrap(),
                Some(&context),
                &https_client,
                llm_provider.as_str(),
                &api_url,
                &api_key,
                &ai_model,
                chat::get_chat_input,
                continue_chat,
            ))
            .context("Failed to run chat")?;

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Generate {
            prompt,
            llm_provider,
            api_url,
            api_key,
            ai_model,
        } => {
            // let prompt = Commands::fetch_prompt_from_cli(Vec::new(), "Enter prompt: ");
            println!("Chat command is run with below arguments:");
            println!(" Prompt: {:?}", prompt);
            println!(" LLM Provider: {:?}", llm_provider);
            println!(" API URL: {:?}", api_url);
            println!(" API Key: {:?}", api_key);
            println!(" AI Model: {:?}", ai_model);

            let context: Option<&str> = None;
            let client = configs::get_https_client().context("Failed to create HTTPS client")?;

            let system_prompt = "template/general_prompt.txt";
            rt.block_on(chat::run_chat(
                system_prompt,
                &prompt,
                context,
                &client,
                llm_provider.as_str(),
                &api_url,
                &api_key,
                &ai_model,
            ))
            .context("Failed to run chat")?;

            rt.shutdown_timeout(std::time::Duration::from_secs(1));
        }
        Commands::Version { version } => {
            println!("Version: {}", version);
            std::process::exit(0);
        }

        Commands::Exit {} => {
            println!("Exiting application");
            std::process::exit(0);
        }

        Commands::Man {} => {
            println!("Help command is not implemented yet");
            std::process::exit(0);
        }
    }

    Ok(())
}

async fn check_connection(client: &HttpsClient, url: &str) -> Result<()> {
    // let uri = hyper::Uri::from_static(&url);
    let uri = url.parse::<http::Uri>()?;

    let res = client.get(uri).await?;
    if res.status().is_success() {
        if let Some(info) = res.extensions().get::<HttpInfo>() {
            info!("remote addr = {}", info.remote_addr())
        }
    } else {
        anyhow::bail!(anyhow::anyhow!("Failed to connect to the server {}", url));
    }

    Ok(())
}

type HttpsClient = LegacyClient<HttpsConnector<HttpConnector>, Full<Bytes>>;
