use anyhow::{Context, Result};
use app::commands;
use app::commands::Args;
use clap::Parser;
fn main() -> Result<()> {
    println!("Starting Application");

    // let args = Args::parse();

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("chatbot")
        .enable_all()
        .build()
        .context("Failed to build runtime")?;

    // commands::run_app(args, rt).context("Failed to run Command")?;

    app::ui::launch_app();

    println!("Exiting application");

    Ok(())
}
