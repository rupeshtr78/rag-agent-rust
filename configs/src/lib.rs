pub mod constants;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client as LegacyClient;
use http_body_util::Full;
use hyper::body::Bytes;
use rustls::crypto::ring::default_provider;
use hyper_util::rt::TokioExecutor;
use log::debug;
use serde::{Deserialize, Serialize};
use anyhow::anyhow;
use crate::constants::{CHAT_API_URL, OPEN_AI_URL};

pub type HttpsClient = LegacyClient<HttpsConnector<HttpConnector>, Full<Bytes>>;

pub fn get_https_client() -> anyhow::Result<HttpsClient> {
    // Install the crypto provider required by rustls
    match default_provider().install_default() {
        anyhow::Result::Ok(_) => debug!("Crypto provider installed successfully"),
        Err(e) => {
            return Err(anyhow::anyhow!(
                "Failed to install crypto provider: {:?}",
                e
            ));
        }
    }

    // Create an HTTPS connector with native roots
    let https = hyper_rustls::HttpsConnectorBuilder::new()
        .with_native_roots()?
        .https_or_http()
        .enable_http1()
        .build();

    // Build the hyper client from the HTTPS connector
    let client: HttpsClient = LegacyClient::builder(TokioExecutor::new()).build(https);
    anyhow::Ok(client)
}

/// LLMProvider is an enum that represents the provider of the LLM
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LLMProvider {
    OpenAI,
    Ollama,
    // Add other providers
}

impl LLMProvider {
    pub fn get_provider(provider: &str) -> anyhow::Result<LLMProvider> {
        match provider.to_lowercase().as_str() {
            "ollama" => Ok(LLMProvider::Ollama),
            "openai" => Ok(LLMProvider::OpenAI),
            _ => Err(anyhow!("Unsupported provider: {}", provider)),
        }
    }

    // @TODO merge this with get_chat_api_url
    #[allow(dead_code)]
    pub fn get_api_url(provider: &str) -> anyhow::Result<String> {
        let provider =
            LLMProvider::get_provider(provider).map_err(|_| anyhow!("Unsupported provider"))?;
        match provider {
            LLMProvider::OpenAI => anyhow::Ok(OPEN_AI_URL.to_string()),
            LLMProvider::Ollama => anyhow::Ok(CHAT_API_URL.to_string()),
        }
    }
}

// pub fn get_chat_api_url(provider: &str) -> anyhow::Result<String> {
//     let provider =
//         LLMProvider::get_provider(provider).map_err(|_| anyhow!("Unsupported provider"))?;
//     match provider {
//         LLMProvider::OpenAI => anyhow::Ok(OPEN_AI_URL.to_string()),
//         LLMProvider::Ollama => anyhow::Ok(CHAT_API_URL.to_string()),
//     }
// }
