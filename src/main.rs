use clap::{Parser, Subcommand};
use tracing_subscriber;
use treecut::core;

#[derive(Parser)]
#[command(name = "treecut", version, about = "AST & Dependency Context Slicer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Mcp,
    Map { path: Option<String> },
    Slice { symbol: String, path: Option<String> },
    DiffSlice { path: Option<String> },
    Ask { prompt: String },
}

#[tokio::main]
async fn main() {
    let _ = tracing_subscriber::fmt().with_writer(std::io::stderr).with_ansi(false).try_init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Mcp => treecut::mcp::run().await,
        Commands::Map { path } => {
            let args = core::RepoSkeletonArgs { root: path, ..Default::default() };
            let r = core::repo_skeleton(&args).unwrap();
            println!("{}", r.text);
        }
        Commands::Slice { symbol, path } => {
            let args = core::SliceSymbolArgs { root: path, symbol, ..Default::default() };
            let r = core::slice_symbol(&args).unwrap();
            println!("{}", r.text);
        }
        Commands::DiffSlice { path } => {
            let args = core::SliceChangedArgs { root: path, ..Default::default() };
            let r = core::slice_changed_symbols(&args).unwrap();
            println!("{}", r.text);
        }
        Commands::Ask { prompt } => {
            let model = std::env::var("TREECT_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
            let base_url = std::env::var("TREECT_BASE_URL").unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
            treecut::agent::run_ask(&prompt, ".", &model, &base_url).await;
        }
    }
}
