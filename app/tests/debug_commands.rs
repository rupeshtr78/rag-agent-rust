#[cfg(test)]
mod tests {
    use app::commands::{Args, Commands};
    use clap::Parser;

    /// quick and dirty way to test the command line arguments
    #[test]
    fn dbg_cmd() {
        let args = Args::parse();
        println!("Parsed args: {:?}", args);
        let commands = match args.cmd {
            Some(command) => command,
            None => {
                println!("No subcommand provided. Use --help for more information.");
                return;
            }
        };

        match &commands {
            Commands::Version { version } => {
                println!("Version command");
                println!("Version: {:?}", version);
            }
            Commands::Load {
                path,
                chunk_size,
                llm_provider,
                embed_model,
                api_url,
                api_key,
            } => {
                println!("Load command");
                println!("Path: {:?}", path);
                println!("Chunk size: {:?}", chunk_size);
                println!("LLM Provider: {:?}", llm_provider);
                println!("Embed Model: {:?}", embed_model);
                println!("API URL: {:?}", api_url);
                println!("API Key: {:?}", api_key);
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
                println!("Lance Query command");
                println!("Query: {:?}", input);
                println!("LLM Provider: {:?}", llm_provider);
                println!("API URL: {:?}", api_url);
                println!("API Key: {:?}", api_key);
                println!("Model: {:?}", model);
                println!("Table: {:?}", table);
                println!("Database: {:?}", database);
                println!("Whole Query: {:?}", whole_query);
                println!("File Context: {:?}", file_context);
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
                file_context: file_query,
                system_prompt,
                continue_chat,
            } => {
                println!("Lance Query command");
                let cli_input = Commands::fetch_prompt_from_cli(input.clone(), "Enter query: ");
                println!("Query: {:?}", cli_input);
                println!("LLM Provider: {:?}", llm_provider);
                println!("Model: {:?}", api_url);
                println!("API Key: {:?}", api_key);
                println!("Model: {:?}", embed_model);
                println!("AI Model: {:?}", ai_model);
                println!("Table: {:?}", table);
                println!("Database: {:?}", database);
                println!("Whole Query: {:?}", whole_query);
                println!("File Query: {:?}", file_query);
                println!("System Prompt: {:?}", system_prompt);
                println!("Continue Chat: {:?}", continue_chat);
            }
            Commands::Generate {
                prompt,
                llm_provider,
                api_url,
                api_key,
                ai_model,
            } => {
                println!("Chat command");
                println!("Prompt: {:?}", prompt);
                println!("LLM Provider: {:?}", llm_provider);
                println!("API URL: {:?}", api_url);
                println!("API Key: {:?}", api_key);
                println!("AI Model: {:?}", ai_model);
            }
            Commands::Exit => {
                println!("Exit command");
            }
            Commands::Man => {
                println!("Help command");
            }
        }
    }
}
