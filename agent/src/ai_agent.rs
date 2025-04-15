use anyhow::{Context, Result};
pub struct Agent {
    pub llm_provider: String,
    pub api_url: String,
    pub api_key: String,
    pub embed_model: String,
    pub ai_model: String,
    pub table: String,
    pub database: String,
    pub mcp_client: Option<FnMut(&str) -> Result<Vec<String>>>,
}

pub trait Agents {
    fn load(&self, rt: tokio::runtime::Runtime, path: &str, chunk_size: usize) -> Result<()>;
    fn lance_query(&self, rt: tokio::runtime::Runtime, input: Vec<String>) -> Result<()>;
    fn rag_query(&self, rt: tokio::runtime::Runtime, input: Vec<String>) -> Result<()>;
}

impl Agents for Agent {
    async fn load(&self, rt: tokio::runtime::Runtime, path: &str, chunk_size: usize) -> Result<()> {
        // Load the embedding
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;
        rt.block_on(check_connection(
            &https_client,
            &format!("{}/{}", self.api_url, "api/version"),
        ))
        .context("Failed to check connection")?;

        rt.block_on(vectordb::run_embedding_pipeline(
            path,
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
    async fn lance_query(&self, rt: tokio::runtime::Runtime, input: Vec<String>) -> Result<()> {
        // Query the Lance Vector Database
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;
        rt.block_on(check_connection(
            &https_client,
            &format!("{}/{}", self.api_url, "api/version"),
        ))
        .context("Failed to check connection")?;

        rt.block_on(vectordb::run_lance_query(
            input,
            self.llm_provider.as_str(),
            &self.api_url,
            &self.api_key,
            self.embed_model.as_str(),
            self.table.as_str(),
            &https_client,
        ))
        .context("Failed to run lance query")?;

        println!("Finished Lance Query");
        Ok(())
    }
    async fn rag_query(&self, rt: tokio::runtime::Runtime, input: Vec<String>) -> Result<()> {
        // Query the Lance Vector Database
        let https_client = configs::get_https_client().context("Failed to create HTTPS client")?;
        rt.block_on(check_connection(
            &https_client,
            &format!("{}/{}", self.api_url, "api/version"),
        ))
        .context("Failed to check connection")?;

        rt.block_on(vectordb::run_rag_query(
            input,
            self.llm_provider.as_str(),
            &self.api_url,
            &self.api_key,
            self.embed_model.as_str(),
            self.table.as_str(),
            &https_client,
        ))
        .context("Failed to run lance query")?;

        println!("Finished Lance Query");
        Ok(())
    }
}
