use anyhow::{Context, Result};
pub struct Agent {
    pub llm_provider: String,
    pub api_url: String,
    pub api_key: String,
    pub embed_model: String,
    pub ai_model: String,
    pub table: String,
    pub database: String,
    pub system_prompt: String,
    // pub mcp_client: Option<FnMut() -> Vec<String>>,
}

impl Agent {
    pub async fn Load(
        &self,
        rt: tokio::runtime::Runtime,
        path: String,
        chunk_size: usize,
    ) -> Result<()> {
        // Load the embedding
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;

        rt.block_on(vectordb::run_embedding_pipeline(
            &path,
            chunk_size,
            self.llm_provider.as_str(),
            &self.api_url,
            &self.api_key,
            self.embed_model.as_str(),
            &https_client,
        ))
        .context("Failed to run lance vectordb")?;

        println!("Finished Loading the embedding");
        Ok(())
    }
    pub async fn LanceQuery(
        &self,
        rt: tokio::runtime::Runtime,
        input: Vec<String>,
        whole_query: String,
        file_context: String,
    ) -> Result<()> {
        // Query the Lance Vector Database
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;

        // Initialize the database
        let mut db = rt
            .block_on(lancedb::connect(&self.database).execute())
            .context("Failed to connect to the database")?;

        let whole_query: bool = whole_query
            .parse()
            .context("Failed to parse whole_query flag")?;
        let file_context: bool = file_context
            .parse()
            .context("Failed to parse file_query flag")?;

        // Query the database
        let content = rt
            .block_on(vectordb::query::run_query(
                &mut db,
                &self.llm_provider,
                &self.api_url,
                &self.api_key,
                &self.embed_model,
                &input,
                &self.table,
                &https_client,
                whole_query,
                file_context,
            ))
            .context("Failed to run query")?;

        println!("Query Response: {:?}", content);
        Ok(())
    }
    pub async fn RagQuery(
        &self,
        rt: tokio::runtime::Runtime,
        input: Vec<String>,
        continue_chat: String,
    ) -> Result<()> {
        // Query the Lance Vector Database
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;
        // do a check to see if client is up

        // Initialize the database
        let mut db = rt
            .block_on(lancedb::connect(&self.database).execute())
            .context("Failed to connect to the database")?;

        let continue_chat: bool = continue_chat
            .parse()
            .context("Failed to parse continue_chat flag")?;

        // Query the database
        let content = rt
            .block_on(vectordb::query::run_query(
                &mut db,
                &self.llm_provider,
                &self.api_url,
                &self.api_key,
                &self.embed_model,
                &input,
                &self.table,
                &https_client,
                false,
                false,
            ))
            .context("Failed to run query")?;

        let context = content.join(" ");

        rt.block_on(chat::run_chat_with_history(
            &self.system_prompt,
            input.first().unwrap(),
            Some(&context),
            &https_client,
            &self.llm_provider,
            &self.api_url,
            &self.api_key,
            &self.ai_model,
            chat::get_chat_input,
            continue_chat,
        ))
        .context("Failed to run chat")?;

        println!("Finished Lance Query");
        Ok(())
    }
}
