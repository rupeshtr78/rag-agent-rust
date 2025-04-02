pub mod cli;
pub mod commands;
use anyhow::{Context, Result};
fn main() -> Result<()> {
    println!("Starting Application");

    // app::commands::dbg_cmd(); // Debugging

    let commands = commands::build_args();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("chatbot")
        .enable_all()
        .build()
        .context("Failed to build runtime")?;

    cli::cli(commands, rt).context("Failed to run Command")?;

    println!("Exiting application");

    Ok(())
}
