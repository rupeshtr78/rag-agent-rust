use anyhow::{Context, Result};
use log::debug;

pub struct LLMProvider {
    pub provider: String,
    pub api_url: String,
    pub api_key: String,
}

pub struct EmbeddingModel {
    pub model: String,
}

pub struct AIModel {
    pub model: String,
}

pub struct Agent {
    pub llm_provider: LLMProvider,
    pub embedding_model: EmbeddingModel,
    pub ai_model: AIModel,
    pub table: String,
    pub database: String,
}

impl Agent {
    pub fn new(
        llm_provider: LLMProvider,
        embedding_model: EmbeddingModel,
        ai_model: AIModel,
        table: String,
        database: String,
    ) -> Self {
        Agent {
            llm_provider,
            embedding_model,
            ai_model,
            table,
            database,
        }
    }

    pub fn load_embeddings(
        &self,
        rt: tokio::runtime::Runtime,
        path: &str,
        chunk_size: usize,
    ) -> Result<()> {
        // Load the embedding
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;

        rt.block_on(vectordb::run_embedding_pipeline(
            &path.to_string(),
            chunk_size,
            self.llm_provider.provider.as_str(),
            &self.llm_provider.api_url,
            &self.llm_provider.api_key,
            self.embedding_model.model.as_str(),
            &https_client,
        ))
        .context("Failed to run lance vectordb")?;

        println!("Finished Loading the embedding");
        Ok(())
    }

    pub fn query_embeddings(
        &self,
        rt: tokio::runtime::Runtime,
        input: Vec<String>,
        whole_query: bool,
        file_context: bool,
    ) -> Result<()> {
        // Query the Lance Vector Database
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;

        // Initialize the database
        let mut db = rt
            .block_on(lancedb::connect(&self.database).execute())
            .context("Failed to connect to the database")?;

        // Query the database
        let content = rt
            .block_on(vectordb::query::run_query(
                &mut db,
                self.llm_provider.provider.as_str(),
                self.llm_provider.api_url.as_str(),
                self.llm_provider.api_key.as_str(),
                self.embedding_model.model.as_str(),
                &input,
                &self.table,
                &https_client,
                whole_query,
                file_context,
            ))
            .context("Failed to run lance query")?;

        println!("Query Response: {:?}", content);
        Ok(())
    }

    pub fn rag_query(
        &self,
        rt: tokio::runtime::Runtime,
        input: Vec<String>,
        whole_query: bool,
        file_context: bool,
        system_prompt: &str,
        continue_chat: bool,
    ) -> Result<()> {
        // Query the Lance Vector Database
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;

        // Initialize the database
        let mut db = rt
            .block_on(lancedb::connect(&self.database).execute())
            .context("Failed to connect to the database")?;

        // Query the database
        let content = rt
            .block_on(vectordb::query::run_query(
                &mut db,
                self.llm_provider.provider.as_str(),
                self.llm_provider.api_url.as_str(),
                self.llm_provider.api_key.as_str(),
                self.embedding_model.model.as_str(),
                &input,
                &self.table,
                &https_client,
                whole_query,
                file_context,
            ))
            .context("Failed to run query")?;

        debug!("Query Response: {:?}", content);

        let context = content.join(" ");
        rt.block_on(chat::run_chat_with_history(
            system_prompt,
            input.first().unwrap(),
            Some(&context),
            &https_client,
            self.llm_provider.provider.as_str(),
            &self.llm_provider.api_url,
            &self.llm_provider.api_key,
            &self.ai_model.model,
            chat::get_chat_input,
            continue_chat,
        ))
        .context("Failed to run chat")?;

        Ok(())
    }

    pub fn generate(
        &self,
        rt: tokio::runtime::Runtime,
        prompt: &str,
        system_prompt: &str,
    ) -> Result<()> {
        let context: Option<&str> = None;
        let client = configs::get_https_client().context("Failed to create HTTPS client")?;

        rt.block_on(chat::run_chat(
            system_prompt,
            prompt,
            context,
            &client,
            self.llm_provider.provider.as_str(),
            &self.llm_provider.api_url,
            &self.llm_provider.api_key,
            &self.ai_model.model,
        ))
        .context("Failed to run chat")?;

        Ok(())
    }
}
