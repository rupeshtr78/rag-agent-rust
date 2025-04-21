use anyhow::{Context, Result};
use log::debug;
use vectordb::EmbeddingStore;

pub struct ModelAPIProvider {
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
    pub llm_provider: ModelAPIProvider,
    pub embedding_model: EmbeddingModel,
    pub ai_model: AIModel,
    // pub agent: Option<FnMut(&str> -> Result<Vec<String>>>,
}

impl Agent {
    pub fn new(
        llm_provider: ModelAPIProvider,
        embedding_model: EmbeddingModel,
        ai_model: AIModel,
    ) -> Self {
        Agent {
            llm_provider,
            embedding_model,
            ai_model,
        }
    }

    pub async fn load_embeddings(&self, path: &str, chunk_size: usize) -> Result<EmbeddingStore> {
        // Load the embedding
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;

        let embedding_store = vectordb::run_embedding_pipeline(
            path,
            chunk_size,
            self.llm_provider.provider.as_str(),
            &self.llm_provider.api_url,
            &self.llm_provider.api_key,
            self.embedding_model.model.as_str(),
            &https_client,
        )
        .await
        .context("Failed to run lance vectordb")?;

        println!("Finished Loading the embedding");
        Ok(embedding_store)
    }

    pub async fn query_embeddings(
        &self,
        input: Vec<String>,
        whole_query: bool,
        file_context: bool,
        embedding_store: &EmbeddingStore,
    ) -> Result<Vec<String>> {
        // Query the Lance Vector Database
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;

        // let db = embedding_store.db;
        // let table = embedding_store.table;
        // Initialize the database
        let mut db = lancedb::connect(&embedding_store.db)
            .execute()
            .await
            .context("Failed to connect to the database")?;

        // Query the database
        let content = vectordb::query::run_query(
            &mut db,
            self.llm_provider.provider.as_str(),
            self.llm_provider.api_url.as_str(),
            self.llm_provider.api_key.as_str(),
            self.embedding_model.model.as_str(),
            &input,
            &embedding_store.table,
            &https_client,
            whole_query,
            file_context,
        )
        .await
        .context("Failed to run lance query")?;

        println!("Query Response: {:?}", content);
        Ok(content)
    }

    pub fn rag_query(
        &self,
        path: &str,
        chunk_size: usize,
        rt: tokio::runtime::Runtime,
        input: Vec<String>,
        whole_query: bool,
        file_context: bool,
        system_prompt: &str,
        continue_chat: bool,
    ) -> Result<()> {
        // Query the Lance Vector Database
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;

        // Load the embedding
        let embedding_store = rt
            .block_on(self.load_embeddings(path, chunk_size))
            .context("Failed to load embeddings")?;

        // query the Lance Vector Database
        let content = rt
            .block_on(self.query_embeddings(
                input.clone(),
                whole_query,
                file_context,
                &embedding_store,
            ))
            .with_context(|| "Failed to query embeddings")?;

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
