use crate::model_options::Options;
use crate::prompt_template::Prompt;
use anyhow::Result;
use anyhow::{anyhow, Context};
use configs::constants::{self, OPEN_AI_CHAT_API, OPEN_AI_URL};
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// ChatRole is an enum that represents the role of the chat message
#[derive(Serialize, Deserialize, Debug, Clone)]

pub enum ChatRole {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
    #[serde(rename = "system")]
    System,
    #[serde(rename = "tool")]
    Tool,
}

/// ChatMessage is a struct that represents a chat message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatMessage {
    role: ChatRole,
    content: String,
}

impl fmt::Display for ChatMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {}", self.role, self.content)
    }
}

impl ChatMessage {
    pub fn new(role: ChatRole, content: String) -> ChatMessage {
        ChatMessage { role, content }
    }

    pub fn get_content(&self) -> &String {
        &self.content
    }

    #[allow(dead_code)]
    pub fn pretty_print_chat(&self) {
        let content = self.get_content();
        if let Ok(parsed_json) = serde_json::from_str::<Value>(content) {
            if let Ok(pretty_json) = serde_json::to_string_pretty(&parsed_json) {
                println!("AI Response {:#}", pretty_json);
            } else {
                eprintln!("Failed to pretty print the JSON.");
            }
        } else {
            eprintln!("Failed to parse JSON string.");
        }
    }
}
/// LLMProvider is an enum that represents the provider of the LLM
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LLMProvider {
    OpenAI,
    Ollama,
    // Add other providers
}

impl LLMProvider {
    pub fn get_provider(provider: &str) -> Result<LLMProvider> {
        match provider.to_lowercase().as_str() {
            "ollama" => Ok(LLMProvider::Ollama),
            "openai" => Ok(LLMProvider::OpenAI),
            _ => Err(anyhow!("Unsupported provider: {}", provider)),
        }
    }
}

/// ChatRequest is a struct that represents a chat request
// @TODO: Add provider to choose between OpenAI and other providers like ollama
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatRequest {
    pub provider: LLMProvider,
    pub model: String,
    pub api_url: String,
    pub api_key: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
    pub format: String,
    pub options: Option<Options>,
}

/// ChatBody is a struct that represents the body of a chat request
#[derive(Serialize, Deserialize)]
struct ChatBody {
    model: String,
    pub messages: Vec<ChatMessage>,
    stream: bool,
    format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Options>,
}

impl ChatRequest {
    pub(crate) fn new(
        provider: &str,
        model: &str,
        api_url: String,
        api_key: String,
        stream: bool,
        format: String,
        options: Option<Options>,
        prompt: Prompt,
    ) -> ChatRequest {
        let mut messages = Vec::new();
        let system_message = ChatMessage::new(ChatRole::System, prompt.system_message);
        messages.push(system_message);

        for content in prompt.content.clone().into_iter().flatten() {
            let user_content = ChatMessage::new(ChatRole::User, content.get_content().to_string());
            messages.push(user_content);
        }

        let user_prompt = ChatMessage::new(ChatRole::User, prompt.prompt);
        messages.push(user_prompt);

        let provider = LLMProvider::get_provider(provider).unwrap_or(LLMProvider::Ollama); // Default to Ollama if not specified

        let model = model.to_string();
        ChatRequest {
            provider,
            model,
            api_url,
            api_key,
            messages,
            stream,
            format,
            options,
        }
    }

    pub(crate) fn create_chat_body(&self) -> Result<String> {
        let chat_body = ChatBody {
            model: self.model.to_string(),
            messages: self.messages.clone(),
            stream: self.stream,
            format: self.format.to_string(),
            options: self.options.clone(),
        };

        let body = serde_json::to_string(&chat_body).context("Failed to serialize ChatBody")?;
        debug!("Chat Body: {:?}", body);

        Ok(body)
    }

    pub fn get_chat_api_url(&self) -> Result<String> {
        match self.provider {
            LLMProvider::OpenAI => Ok(format!("{}/{}", OPEN_AI_URL, OPEN_AI_CHAT_API)),
            LLMProvider::Ollama => Ok(format!("{}/{}", self.api_url, constants::OLLAMA_CHAT_API)),
        }
    }

    #[allow(dead_code)]
    pub fn get_embed_api_url(&self) -> Result<String> {
        match self.provider {
            LLMProvider::OpenAI => Ok(format!("{}/{}", OPEN_AI_URL, constants::OPEN_AI_EMBED_API)),
            LLMProvider::Ollama => Ok(format!("{}/{}", self.api_url, constants::OLLAMA_EMBED_API)),
        }
    }
}

/// ChatResponse is a struct that represents a chat response
#[derive(Serialize, Deserialize, Debug)]
pub struct ChatResponse {
    pub model: Option<String>,
    created_at: Option<String>,
    pub message: ChatMessage,
    done_reason: Option<String>,
    done: Option<bool>,
    pub context: Option<Vec<i32>>,
    total_duration: Option<i64>,
    load_duration: Option<i64>,
    prompt_eval_count: Option<i32>,
    prompt_eval_duration: Option<i64>,
    eval_count: Option<i32>,
    eval_duration: Option<i64>,
}

impl ChatResponse {
    pub fn get_message(&self) -> Option<&ChatMessage> {
        Some(&self.message)
    }
}
