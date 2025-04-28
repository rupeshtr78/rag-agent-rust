use ::std::io::{self, Write};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use configs::constants::{AI_MODEL, EMBEDDING_MODEL, SYSTEM_PROMPT_PATH, VERSION};
use configs::constants::{CHAT_API_KEY, CHAT_API_URL};
use log::info;
use tokio::runtime::Runtime;

use crate::{cli, cli_interactive};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[clap(name = "rag-agent-rust")]
#[clap(about = "A CLI application for LLM Interactions", long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    pub cmd: Option<Commands>,
    /// Select Log level
    #[clap(short, long, global = true)]
    pub log_level: Option<LogLevel>,
    /// Select Cli is to be used
    #[clap(short = 'o', long, global = false, hide = true)]
    pub interactive: Option<bool>,
    /// Select to run in agent mode
    #[clap(short = 'g', long, global = true)]
    pub agent_mode: Option<bool>,
}

#[derive(Subcommand, Debug)]

pub enum Commands {
    /// Get the version of the application
    Version {
        /// The version of the application
        #[clap(short, long)]
        #[clap(default_value = VERSION)]
        version: String,
    },

    /// Load a directory of files into the lance vector database
    Load {
        /// The path to the directory to load
        #[clap(short, long)]
        path: String,
        // chunk size
        #[clap(short, long)]
        #[clap(default_value = "2048")]
        chunk_size: String,
        /// Provide the model to use for query embedding
        #[clap(short = 'm', long)]
        #[clap(default_value = "ollama")]
        llm_provider: String,
        /// Provide the model to use for query embedding
        #[clap(short, long)]
        #[clap(default_value = EMBEDDING_MODEL)]
        embed_model: String,
        /// Provide the API endpoint to use
        #[clap(short = 'u', long)]
        #[clap(default_value = CHAT_API_URL)]
        api_url: String,
        /// Provide the API key to use
        #[clap(short = 'k', long)]
        #[clap(default_value = CHAT_API_KEY)]
        api_key: String,
    },
    /// Query the Lance Vector Database
    LanceQuery {
        /// The query string to use
        #[clap(short, long)]
        input: Vec<String>,
        /// Provide the provider to use for query embedding
        #[clap(short = 'p', long)]
        #[clap(default_value = "ollama")]
        llm_provider: String,
        /// Provide the API endpoint to use
        #[clap(short = 'u', long)]
        #[clap(default_value = CHAT_API_URL)]
        api_url: String,
        /// Provide the API key to use
        #[clap(short = 'k', long)]
        #[clap(default_value = CHAT_API_KEY)]
        api_key: String,
        /// Provide the model to use for query embedding
        #[clap(short, long)]
        #[clap(default_value = EMBEDDING_MODEL)]
        model: String,
        /// Provide the table to use to query
        #[clap(short, long)]
        table: String,
        /// Provide the database to use
        #[clap(short, long)]
        database: String,
        /// specify if the whole table query is to be used default is false
        #[clap(short, long)]
        #[clap(default_value = "false")]
        whole_query: String,
        /// specify if the additional file context default is false
        #[clap(short, long)]
        #[clap(default_value = "false")]
        file_context: String,
    },
    /// Query the Lance Vector Database and chat with the AI
    RagQuery {
        /// The query string to use
        #[clap(short, long)]
        input: Vec<String>,
        /// Provide the model to use for query embedding
        #[clap(short = 'p', long)]
        #[clap(default_value = "ollama")]
        llm_provider: String,
        /// Provide the model to use for query embedding
        #[clap(short, long)]
        #[clap(default_value = EMBEDDING_MODEL)]
        embed_model: String,
        /// Provide the API endpoint to use
        #[clap(short = 'u', long)]
        #[clap(default_value = CHAT_API_URL)]
        api_url: String,
        /// Provide the API key to use
        #[clap(short = 'k', long)]
        #[clap(default_value = CHAT_API_KEY)]
        api_key: String,
        /// Provide the AI model to use for generation
        #[clap(short, long)]
        #[clap(default_value = AI_MODEL)]
        ai_model: String,
        /// Provide the table to use to query
        #[clap(short, long)]
        table: String,
        /// Provide the database to use
        #[clap(short, long)]
        database: String,
        /// specify if the whole table query is to be used default is false
        #[clap(short, long)]
        #[clap(default_value = "false")]
        whole_query: String,
        /// specify if the additional context from same file has to be added default is false
        #[clap(short, long)]
        #[clap(default_value = "false")]
        file_context: String,
        /// specify if the system prompt is to be used default is false
        #[clap(short, long)]
        #[clap(default_value = SYSTEM_PROMPT_PATH)]
        system_prompt: String,
        /// continue flag to continue the conversation
        #[clap(short, long)]
        #[clap(default_value = "true")]
        continue_chat: String,
    },
    /// Chat with the AI
    Generate {
        /// Prompt for AI
        #[clap(short, long)]
        prompt: String,
        /// Provide the model to use for query embedding
        #[clap(short = 'p', long)]
        #[clap(default_value = "ollama")]
        llm_provider: String,
        /// Provide the API endpoint to use
        #[clap(short, long)]
        #[clap(default_value = CHAT_API_URL)]
        api_url: String,
        /// Provide the API key to use
        #[clap(short, long)]
        #[clap(default_value = CHAT_API_KEY)]
        api_key: String,
        /// Provide the AI model to use for generation
        #[clap(short, long)]
        #[clap(default_value = AI_MODEL)]
        ai_model: String,
    },

    /// Exit the application
    Exit,
    Man,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    OFF,
}

impl Commands {
    pub fn fetch_prompt_from_cli(input: Vec<String>, prompt_message: &str) -> Vec<String> {
        if input.is_empty() {
            let user_input = fetch_value(prompt_message);
            vec![user_input]
        } else {
            input
        }
    }
}

impl LogLevel {
    /// map loglevel enum to log::LevelFilter
    pub fn get_log_level_filter(&self) -> log::LevelFilter {
        match self {
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::OFF => log::LevelFilter::Off,
        }
    }
}

/// Initialize the logger with the specified log level.
fn colog_init(log_level: LogLevel) {
    // Initialize env_logger with module path formatting
    let mut builder = env_logger::Builder::new();
    builder
        .filter_level(log_level.get_log_level_filter())
        .format_module_path(true)
        .init();

    println!("Log level set to: {:?}", log_level);
}

/// Run the application with the provided arguments and runtime.
pub fn run_app(args: Args, rt: Runtime) -> Result<()> {
    // Handle log level (if provided)
    let log_level = args.log_level.unwrap_or(LogLevel::Info);
    colog_init(log_level);

    // Run the command in agent mode
    if let Some(true) = args.agent_mode {
        info!("Running in agent mode");
        crate::agent_interactive::interactive_cli(&rt)
            .context("Failed to run agent interactive CLI")?;
    }

    // Run the command interactive for testing
    if let Some(true) = args.interactive {
        let commands =
            cli_interactive::interactive_cli().context("Failed to run interactive CLI")?;
        cli::cli(commands, rt).context("Failed to run interactive Command")?;
    }

    Ok(())
}

#[allow(dead_code)]
/// Initiates the log builds the command line arguments and return the command to run.
pub fn build_args() -> Commands {
    let args = Args::parse();

    // Handle log level (if provided)
    if let Some(log_level) = args.log_level {
        match log_level {
            LogLevel::Error => colog_init(LogLevel::Error),
            LogLevel::Warn => colog_init(LogLevel::Warn),
            LogLevel::Debug => colog_init(LogLevel::Debug),
            _ => colog_init(LogLevel::Info),
        }
    }

    args.cmd.map_or_else(
        || {
            info!("No subcommand provided. Use --help for more information.");
            Commands::Version {
                version: configs::constants::VERSION.to_string(),
            }
        },
        |cmd: Commands| cmd,
    )
}

/// Generic function to fetch a value from the command line if not provided as an argument.
///
/// # Arguments.
/// * `prompt_message` - The message to display when prompting the user for input.
/// # Returns
/// A `String` containing the value provided by the user or from the argument.
fn fetch_value(prompt_message: &str) -> String {
    print!("{}", prompt_message);
    io::stdout()
        .flush()
        .map_err(|e| e.to_string())
        .unwrap_or_else(|e| {
            panic!("Failed to flush stdout: {}", e);
        });
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input.trim().to_string()
}
