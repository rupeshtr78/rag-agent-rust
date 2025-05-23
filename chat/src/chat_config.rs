use crate::model_options::Options;
use crate::prompt_template::Prompt;
use anyhow::Context;
use anyhow::Result;
use configs::constants::{self, OPEN_AI_CHAT_API, OPEN_AI_URL};
pub use configs::LLMProvider;
use log::debug;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;

/// ChatRole different role of the chat message
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

/// ChatMessage contains the role of the sender and the content of the message
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

/// ChatRequest
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

/// ChatBody body of a chat request
#[derive(Serialize, Deserialize)]
struct ChatBody {
    model: String,
    pub messages: Vec<ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<Options>,
}

#[allow(clippy::too_many_arguments)]
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
        let format = if self.format.is_empty() {
            Some("text".to_string())
        } else {
            None
        };
        let chat_body = ChatBody {
            model: self.model.to_string(),
            messages: self.messages.clone(),
            stream: self.stream,
            format,
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

/// ChatResponse body of chat response
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

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct OpenAiResponse {
    pub id: Option<String>,
    pub object: Option<String>,
    pub created: Option<i64>,
    pub model: Option<String>,
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
    pub service_tier: Option<String>,
    pub system_fingerprint: Option<String>,
}

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Choice {
    pub text: Option<String>,
    pub index: Option<i32>,
    pub message: ChatMessage,
    pub logprobs: Option<String>,
    pub finish_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Usage {}

impl OpenAiResponse {
    pub fn get_message(&self) -> Option<&ChatMessage> {
        if self.choices.is_empty() {
            return None;
        }
        Some(&self.choices[0].message)
    }
}

/// ChatResponseTrait methods for chat responses
pub trait ChatResponseTrait {
    fn get_message(&self) -> Option<&ChatMessage>;
}

impl ChatResponseTrait for ChatResponse {
    fn get_message(&self) -> Option<&ChatMessage> {
        Some(&self.message)
    }
}

impl ChatResponseTrait for OpenAiResponse {
    fn get_message(&self) -> Option<&ChatMessage> {
        if self.choices.is_empty() {
            return None;
        }
        Some(&self.choices[0].message)
    }
}
