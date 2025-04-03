#[cfg(test)]
mod integration_tests {
    use anyhow::Result;
    use app::cli::cli;
    use app::commands::Commands;
    use configs::constants::{AI_MODEL, CHAT_API_KEY, CHAT_API_URL, EMBEDDING_MODEL};
    use tokio::runtime::Runtime;

    #[test]
    fn test_load_embedding_pipeline() -> Result<()> {
        let rt = Runtime::new()?;

        // Adjust paths and parameters according to your actual setup
        let commands = Commands::Load {
            path: "tests/resources/sample/".to_string(),
            chunk_size: 512, // provide realistic test value
            llm_provider: "ollama".to_string(),
            embed_model: EMBEDDING_MODEL.to_string(),
            api_url: CHAT_API_URL.to_string(),
            api_key: CHAT_API_KEY.to_string(),
        };

        // Execute the load command
        cli(commands, rt)?;

        Ok(())
    }

    #[test]
    fn test_rag_query() -> Result<()> {
        let rt = Runtime::new()?;

        let commands = Commands::RagQuery {
            input: vec!["what is temperature".to_string()],
            llm_provider: "ollama".to_string(),
            embed_model: EMBEDDING_MODEL.to_string(),
            api_url: CHAT_API_URL.to_string(),
            api_key: CHAT_API_KEY.to_string(),
            ai_model: AI_MODEL.to_string(), // adjust based on running model
            table: "sample_table".to_string(),
            database: "sample_db".to_string(),
            whole_query: "false".to_string(),
            file_context: "false".to_string(),
            system_prompt: "tests/resources/rag_prompt.txt".to_string(), // your actual prompt
            continue_chat: "false".to_string(),
        };

        // Execute rag-query
        cli(commands, rt)?;
        // Add assertions for no errors or expected output
        assert!(true, "RAG query executed successfully");

        Ok(())
    }
}
