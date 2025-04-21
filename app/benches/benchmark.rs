use app::{
    cli::cli,
    commands::{build_args, run_app, Args},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tokio::runtime::Runtime;

pub fn bench_command_parsing(c: &mut Criterion) {
    c.bench_function("command parsing", |b| b.iter(|| black_box(build_args())));
}

pub fn bench_command_execution(c: &mut Criterion) {
    c.bench_function("command execution", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            let cmd = build_args();
            let args = Args {
                cmd: Some(cmd),
                log_level: None,
                interactive: None,
                // Add other Args fields if they exist
            };
            run_app(args, rt).unwrap();
            black_box(())
        })
    });
}

use app::commands::Commands;

pub fn bench_rag_query(c: &mut Criterion) {
    c.bench_function("RAG query", |b| {
        b.iter(|| {
            let rt = Runtime::new().unwrap();
            let commands = Commands::RagQuery {
                input: vec!["what is temperature".to_string()],
                llm_provider: "ollama".to_string(),
                embed_model: "all-minilm-l6-v2".to_string(),
                api_url: "http://localhost:11434".to_string(),
                api_key: "".to_string(),
                ai_model: "llama2".to_string(),
                table: "sample_table".to_string(),
                database: "sample_db".to_string(),
                whole_query: "false".to_string(),
                file_context: "false".to_string(),
                system_prompt: "tests/resources/rag_prompt.txt".to_string(),
                continue_chat: "false".to_string(),
            };
            cli(commands, rt).unwrap();
            black_box(())
        })
    });
}

criterion_group!(
    benches,
    bench_command_parsing,
    bench_command_execution,
    bench_rag_query
);
criterion_main!(benches);
