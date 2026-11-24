use clap::{Parser, Subcommand};
use tracing_subscriber::fmt;
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
    Map {
        path: Option<String>,
        #[arg(long)]
        depth: Option<usize>,
        #[arg(long)]
        max_chars: Option<usize>,
    },
    Slice {
        symbol: String,
        path: Option<String>,
        #[arg(long)]
        max_chars: Option<usize>,
        #[arg(long)]
        context_lines: Option<usize>,
    },
    DiffSlice {
        path: Option<String>,
        #[arg(long)]
        max_chars: Option<usize>,
        #[arg(long)]
        base: Option<String>,
    },
    Ask { prompt: String },
}

#[tokio::main]
async fn main() {
    let _ = fmt().with_writer(std::io::stderr).with_ansi(false).try_init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Mcp => treecut::mcp::run().await,
        Commands::Map { path, depth, max_chars } => {
            let args = core::RepoSkeletonArgs { root: path, max_depth: depth, max_chars, ..Default::default() };
            let r = core::repo_skeleton(&args).unwrap();
            println!("{}", r.text);
        }
        Commands::Slice { symbol, path, max_chars, context_lines } => {
            let args = core::SliceSymbolArgs { root: path, symbol, max_chars, context_lines, ..Default::default() };
            let r = core::slice_symbol(&args).unwrap();
            println!("{}", r.text);
        }
        Commands::DiffSlice { path, max_chars, base } => {
            let args = core::SliceChangedArgs { root: path, max_chars, base_ref: base, ..Default::default() };
            let r = core::slice_changed_symbols(&args).unwrap();
            println!("{}", r.text);
        }
        Commands::Ask { prompt } => {
            #[cfg(feature = "cli")]
            {
                let model = std::env::var("TREECT_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
                let base_url = std::env::var("TREECT_BASE_URL").unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
                treecut::agent::run_ask(&prompt, ".", &model, &base_url).await;
            }
            #[cfg(not(feature = "cli"))]
            { let _ = prompt; eprintln!("ask requires cli feature"); }
        }
    }
}
