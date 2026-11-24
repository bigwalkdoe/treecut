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
        Commands::Ask { prompt: _ } => {}
    }
}
