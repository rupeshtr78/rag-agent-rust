use anyhow::Context;
use app::cli::cli;
use app::commands::{build_args, run_app, Args, Commands};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

// pub fn bench_command_parsing(c: &mut Criterion) {
//     c.bench_function("command parsing", |b| b.iter(|| black_box(build_args())));
// }

pub fn bench_command_execution(c: &mut Criterion) {
    c.bench_function("command execution", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            let cmd = build_args();
            let args = Args {
                cmd: Some(cmd),
                log_level: None,
                interactive: None,
                agent_mode: None,
            };
            run_app(args, rt).unwrap();
            black_box(())
        })
    });
}

pub fn bench_load_embeddings(c: &mut Criterion) {
    c.bench_function("Load Embeddings", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            let commands = Commands::Load {
                path: "tests/resources/sample".to_string(),
                chunk_size: "1000".to_string(),
                llm_provider: "ollama".to_string(),
                embed_model: "nomic-embed-text".to_string(),
                api_url: "http://localhost:11434".to_string(),
                api_key: "".to_string(),
            };

            match cli(commands, rt).context("Failed to run load command") {
                Ok(_) => println!("Load command executed successfully"),
                Err(err) => eprintln!("Error executing load command: {}", err),
            }
            black_box(());
        });
    });
}

pub fn bench_rag_query(c: &mut Criterion) {
    c.bench_function("RAG query", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            let commands = Commands::RagQuery {
                input: vec!["what is temperature".to_string()],
                llm_provider: "ollama".to_string(),
                embed_model: "nomic-embed-text".to_string(),
                api_url: "http://localhost:11434".to_string(),
                api_key: "".to_string(),
                ai_model: "qwen2:7b".to_string(),
                table: "sample_table".to_string(),
                database: "sample_db".to_string(),
                whole_query: "false".to_string(),
                file_context: "false".to_string(),
                system_prompt: "tests/resources/rag_prompt.txt".to_string(),
                continue_chat: "false".to_string(),
            };
            match cli(commands, rt).context("Failed to run rag query") {
                Ok(_) => println!("RAG query executed successfully"),
                Err(err) => eprintln!("Error executing RAG query: {}", err),
            }
            black_box(())
        })
    });
}

criterion_group!(benches, bench_load_embeddings, bench_rag_query);
criterion_main!(benches);
