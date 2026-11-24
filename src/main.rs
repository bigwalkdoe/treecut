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
        #[arg(long)]
        stats: bool,
    },
    Slice {
        #[arg(required = true, num_args = 1..)]
        symbols: Vec<String>,
        #[arg(long)]
        path: Option<String>,
        #[arg(long)]
        max_chars: Option<usize>,
        #[arg(long)]
        context_lines: Option<usize>,
        #[arg(long)]
        stats: bool,
    },
    DiffSlice {
        path: Option<String>,
        #[arg(long)]
        max_chars: Option<usize>,
        #[arg(long)]
        base: Option<String>,
        #[arg(long)]
        stats: bool,
    },
    Ask { prompt: String },
}

#[tokio::main]
async fn main() {
    let _ = fmt().with_writer(std::io::stderr).with_ansi(false).try_init();
    let cli = Cli::parse();
    match cli.command {
        Commands::Mcp => treecut::mcp::run().await,
        Commands::Map { path, depth, max_chars, stats } => {
            let args = core::RepoSkeletonArgs { root: path, max_depth: depth, max_chars, stats: Some(stats), ..Default::default() };
            let r = core::repo_skeleton(&args).unwrap();
            println!("{}", r.text);
            if stats {
                if let Some(s) = r.stats {
                    print_stats(&s);
                }
            }
        }
        Commands::Slice { symbols, path, max_chars, context_lines, stats } => {
            if symbols.len() == 1 {
                let args = core::SliceSymbolArgs { root: path, symbol: symbols[0].clone(), max_chars, context_lines, stats: Some(stats) };
                let r = core::slice_symbol(&args).unwrap();
                println!("{}", r.text);
                if stats {
                    if let Some(s) = r.stats {
                        print_stats(&s);
                    }
                }
            } else {
                let args = core::SliceSymbolsArgs { root: path, symbols, max_chars, context_lines, stats: Some(stats) };
                let r = core::slice_symbols(&args).unwrap();
                println!("{}", r.text);
                if stats {
                    if let Some(s) = r.stats {
                        print_stats(&s);
                    }
                }
            }
        }
        Commands::DiffSlice { path, max_chars, base, stats } => {
            let args = core::SliceChangedArgs { root: path, max_chars, base_ref: base, stats: Some(stats) };
            let r = core::slice_changed_symbols(&args).unwrap();
            println!("{}", r.text);
            if stats {
                if let Some(s) = r.stats {
                    print_stats(&s);
                }
            }
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

fn print_stats(stats: &core::SliceStats) {
    eprintln!("\n=== Slice Statistics ===");
    eprintln!("Files scanned:       {}", stats.files_scanned);
    eprintln!("Files matched:       {}", stats.files_matched);
    eprintln!("Full files (bytes):  {}", stats.full_files_bytes);
    eprintln!("Sliced output (bytes): {}", stats.sliced_bytes);
    eprintln!("Est. tokens saved:   {}", stats.estimated_tokens_saved);
    eprintln!("Reduction:           {:.1}%", stats.reduction_percent);
}
